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
use glasnost_convert::art::ProjectileColour;
use glasnost_convert::kinds::Kind;
use glasnost_convert::theater::{Catalogue, alternate_image_mark, variant_image, variant_pattern};
use glasnost_fmt::hva::Hva;
use glasnost_fmt::image::{IndexedImage, Palette, Rgb8};
use glasnost_fmt::shp::Shp;
use glasnost_fmt::tmp::Tmp;
use glasnost_fmt::vpl::Vpl;
use glasnost_fmt::vxl::Vxl;
use glasnost_ini::Document;
use glasnost_ini::value;
use glasnost_map::objects::{HealthKind, ListKind, facing_from_value, starting_health};
use glasnost_map::{CellGrid, CellXY, LightingSet, Map, NO_OVERLAY};
use glasnost_num::geom::n_step;
use glasnost_num::{F64, Rounding};
use glasnost_render::compose::{
    CompanionBody, DIAMOND_WIDTH, DepthTest, Paint, Profile, Row565, Shade, SignedImage,
    Translucency, companion_origin, row565,
};

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
use glasnost_render::scene::{Group, Item, ItemDepth, Scene, Stack};
use glasnost_render::voxel::{self, VoxelQuirks};
use glasnost_sim::rules::{Kind as SimKind, SimRules};

use super::quirks::{
    ViewQuirks, cliff_factor, drawn_body_frame, overlay_depth_lift, sprite_vehicle_facing_part,
};
use super::slots;
use super::unload::{self, Unloading};

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

    /// Puts `bytes` in place of the file `name`, or removes it with `None`.
    pub fn set(&mut self, name: &[u8], bytes: Option<Vec<u8>>) {
        match bytes {
            Some(b) => self.insert(name, b),
            None => {
                self.files.remove(&name.to_ascii_uppercase());
            }
        }
    }

    /// A file by name, ignoring case.
    pub fn get(&self, name: &[u8]) -> Option<&[u8]> {
        with_upper(name, |k| self.files.get(k)).map(Vec::as_slice)
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
    /// Player houses show our colour-blind colours
    /// ([`COLOUR_BLIND_SLOTS`]) instead of their slots' definitions: the
    /// person's presentation choice (M8 step C3), never game data.
    pub colour_blind: bool,
    /// The house a client game is played or watched from, as the session
    /// asks for it; `None` outside the client's games. The loader resolves
    /// it to the local house and gives the placeholder human house that
    /// house's lobby colour slot, which the walls take (S-0112 claim 1 row
    /// 4; #94).
    pub local: Option<LocalChoice>,
}

/// How a client game chooses its local house (`Session::start`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LocalChoice {
    /// The house asked for (a network game's slot), if any.
    pub house: Option<glasnost_sim::world::HouseId>,
    /// Whether the game is watched rather than played.
    pub watch: bool,
}

/// Our colour-blind house colours (M8 step C3): the colour definition (H,
/// S, V on S-0103's 0 … 255 scales) a player house in lobby colour slot
/// 0 … 7 shows when the person asks for them, in place of the slot's own
/// definition, shaded by the S-0103 ramp like any definition. They are the
/// Okabe–Ito colours (chosen to stay apart under the common colour-vision
/// deficiencies) in our own order, with white for the last.
// nocite: our own client setting (colour-blind house colours; Okabe and Ito, "Color Universal Design", 2008)
pub const COLOUR_BLIND_SLOTS: [[u8; 3]; 8] = [
    [40, 185, 240],  // yellow
    [19, 255, 213],  // vermillion
    [143, 255, 178], // blue
    [116, 255, 158], // bluish green
    [29, 255, 230],  // orange
    [143, 161, 233], // sky blue
    [231, 104, 204], // reddish purple
    [0, 0, 240],     // white
];

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
            colour_blind: false,
            local: None,
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
/// A sprite particle's brightness (S-0616 claim 24), in ‰.
const SPRITE_PARTICLE_BRIGHTNESS: i32 = 1000; // spec: S-0616 claim 24 and Constants (sprite particle brightness 1000 per mille)
/// A sprite particle's depth bias before −*L(z)* (S-0616 claim 27).
const SPRITE_PARTICLE_BIAS: i32 = -15; // spec: S-0616 claim 27 and Constants (sprite particle depth bias −15 − L(z))
/// A projectile body's brightness (S-0306 claims 30 and 31), in ‰.
const PROJECTILE_BRIGHTNESS: i32 = 1000; // spec: S-0306 claims 30 and 31, Constants (projectile body brightness 1000)
/// A projectile sprite's depth bias before −*L(z)* (S-0102 claim 13).
const PROJECTILE_BIAS: i32 = -30; // spec: S-0102 claim 13 and Constants (projectile depth bias −30 − L(z))
/// A projectile shadow's depth bias before −*L* of the ground under it
/// (S-0102 claim 13).
const PROJECTILE_SHADOW_BIAS: i32 = -10; // spec: S-0102 claim 13 and Constants (projectile shadow bias −10)
/// The brightness of a piece with no house whose type has no resource
/// flag, and the off-map cell's brightness (S-0317 claims 20 and 25).
const PIECE_BRIGHTNESS: i32 = 1000; // spec: S-0317 claims 20 and 25 (brightness 1000: a piece with no house and no resource flag; the off-map cell's U)
/// A flat voxel projectile's final depth bias (S-0102 claim 13, S-0306
/// claim 30): no −*L(z)* term.
const FLAT_VOXEL_PROJECTILE_BIAS: i32 = -65; // spec: S-0306 claim 30 and S-0102 claim 13 (final bias −65, no height-lift term)
/// The general palette's index of the target line (S-0301 claim 24 g).
pub const TARGET_LINE_INDEX: u8 = 8; // spec: S-0301 claim 24 g (palette index 8 of the general palette)
/// The general palette's shade row of the target line (S-0301 claim 24 g).
const TARGET_LINE_ROW: usize = 26; // spec: S-0301 claim 24 g (the neutral shade row 26 of 53)
/// The side of a target line's end square (S-0301 claim 24 g).
const TARGET_LINE_SQUARE: i32 = 3; // spec: S-0301 claim 24 g (a solid 3 × 3 square whose bottom-right pixel is the end point)
/// A pixel particle's depth bias before −*L(z)* (S-0616 claim 28).
const PIXEL_PARTICLE_BIAS: i32 = -50; // spec: S-0616 claim 28 and Constants (pixel particle depth bias −50 − L(z))
/// The pixels of a dotted line from `a` to `b` (S-0301 claim 24 g): one
/// step per pixel along the longer screen axis, the other axis rounded to
/// the nearest pixel, every second step drawn, `phase` choosing whether the
/// even or the odd steps are. Which steps a given tick draws is our interim
/// (the phase of the dots is low confidence there).
pub fn dotted_line(a: (i32, i32), b: (i32, i32), phase: bool) -> Vec<(i32, i32)> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    // spec: S-0301 claim 24 g (walking the line one pixel per step along its longer screen axis)
    let n = dx.abs().max(dy.abs());
    let mut out = Vec::new();
    for k in 0..=n {
        // spec: S-0301 claim 24 g (every second pixel drawn; the drawn and undrawn pixels swap each tick)
        if (k % 2 == 1) != phase {
            continue;
        }
        let step = |d: i32| {
            if n == 0 {
                0
            } else {
                // Our rounding of the shorter axis: to the nearest pixel, halves away from zero.
                let t = 2 * d * k;
                let q = (t.abs() + n) / (2 * n);
                if t < 0 { -q } else { q }
            }
        };
        out.push((a.0 + step(dx), a.1 + step(dy)));
    }
    out
}

/// The image-cache key of the target line's 1 × 1 image (ours).
const TARGET_LINE_IMAGE_KEY: &[u8] = b"\0target line pixel";
/// The image-cache key of the pixel particles' shared 1 × 1 image (ours).
const PIXEL_IMAGE_KEY: &[u8] = b"\0particle pixel";
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

/// The shared building companion depth image file; its image frame 0 is the
/// companion of every eligible building body.
pub const COMPANION_FILE: &[u8] = b"BUILDNGZ.SHA"; // spec: S-0102 Import mapping, claim 31

/// The notes a scene can carry: gaps in the specs (with their request) and
/// parts our viewer does not draw yet.
mod notes {
    pub const NO_GENERAL_PALETTE: &str = "no general palette (PALETTE.PAL missing or unreadable): projectile sprites in it and target lines are not drawn (S-0306 Terms, S-0301 claim 24 g)";
    /// The warning of S-0105 claim 22 and Import mapping for a model whose
    /// `role` image (a part name or "shadow") has a copy rectangle side
    /// above 4096 pixels, at the pose `pose`.
    pub fn voxel_too_large(file: &[u8], role: &str, pose: &str, copy: [i64; 2]) -> String {
        format!(
            "voxel model {}: its {role} image at {pose} has a {} × {} pixel copy rectangle; only the centred 4096 × 4096 pixel window is drawn (S-0105 claim 22). Poses not yet drawn are not checked",
            String::from_utf8_lossy(file),
            copy[0],
            copy[1]
        )
    }
    pub const TILES_UNLOADED: &str = "no game was set up: tiles are drawn as the map stores them, before the load's terrain recalculation and smoothing (S-0010 claim 12)";
    pub const RUBBLE: &str = "rubble overlays outside a rubble cell draw nothing (S-0112 claim 5); rubble cells arise only in play";
    pub const SMUDGE_CLIP: &str = "a smudge is painted whole once per covered cell, not clipped to that cell (S-0102 claim 25): it differs only where covered cells differ in level or light";
    pub const TERRAIN_STRENGTH: &str = "the rules did not resolve: a terrain object type without a strength key is drawn with image frame 0 (its strength from the game-wide tree strength, S-0102 claim 23, needs the typed rules)";
    pub const ORE_MINE_STAGE: &str = "an ore mine's animation (S-0401 claim 15) is not played: it shows image frame 0, its stage at placement (S-0102 claim 23)";
    pub const VOXEL_SHADOW_LIFTED: &str = "a vehicle lifted by a carrier aircraft would have its voxel shadow raised 14 rows, and 60 more over a qualifying bridge (S-0105 claim 44); the simulation has no lifted state yet (the transport specs set it up), so no vehicle shadow is raised";
    pub const SLOT_ANIMS_STILL: &str = "the static scene's building slot animations (S-0108 claim 11) are drawn at their first image frame: they do not advance and power (S-0108 claim 13) does not change them";
    /// See [`SLOT_ANIMS_STILL`].
    pub const SLOT_ANIMS_LIVE: &str = "live buildings' slot animations play (S-0109) and follow S-0108 claims 11, 12, 13 (power, disabling and re-enabling), 15 (empty storage), 17, 20 and 26 to 28 and S-0606's walls; the special outage and low-power slot (S-0108 claim 14), silo fill, the superweapon slots and super-low-power, and the turret and upgrade slots are not modelled; a repair depot's production and special slots are the simulation's (S-0108 claim 20)";
    /// The importer warning of S-0102 claim 33: the shared building depth
    /// image file and why it is unavailable.
    pub fn no_companion(failure: &str) -> String {
        format!(
            "the building depth image file BUILDNGZ.SHA {failure}: building bodies on foundations under 8 cells wide use no companion and are drawn with the plain upright profile (S-0102 claim 33, F2)"
        )
    }
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

/// Calls `f` with `name` in upper case, without allocating for names of up
/// to 64 bytes (a cache lookup on every drawing).
fn with_upper<R>(name: &[u8], f: impl FnOnce(&[u8]) -> R) -> R {
    let mut buf = [0u8; 64]; // nocite: our own stack buffer size
    match buf.get_mut(..name.len()) {
        Some(b) => {
            b.copy_from_slice(name);
            b.make_ascii_uppercase();
            f(b)
        }
        None => f(&upper(name)),
    }
}

/// A sprite file's canvas size and frame headers, read once, with each
/// frame's scene image once it is decoded (`None` before; `Some(None)` for
/// an empty or undecodable frame).
#[derive(Clone, Debug)]
struct ShpHeads {
    canvas: (i32, i32),
    frames: Vec<(i32, i32)>,
    images: Vec<Option<Option<u32>>>,
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

/// An animation type's own art section: the section named by its type name
/// (S-0007 Terms and claim 34). Its playback and drawing properties come
/// from this section alone, whatever section its image name names.
pub fn anim_own_section(import: &Import, idx: usize) -> &[u8] {
    // spec: S-0007 claim 34 (own art section = section named by the animation's type name)
    import
        .outcome
        .registries
        .get(&Kind::Animations)
        .and_then(|r| r.get(idx))
        .map_or(&[][..], |t| &t.name)
}

/// A rules key of a type, as layered.
pub(crate) fn rule<'a>(import: &'a Import, kind: Kind, idx: usize, key: &[u8]) -> Option<&'a [u8]> {
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

pub(crate) fn rule_bool(
    import: &Import,
    kind: Kind,
    idx: usize,
    key: &[u8],
    default: bool,
) -> bool {
    rule(import, kind, idx, key)
        .and_then(value::boolean)
        .unwrap_or(default)
}

pub(crate) fn rule_int(import: &Import, kind: Kind, idx: usize, key: &[u8], default: i32) -> i32 {
    rule(import, kind, idx, key)
        .and_then(value::integer)
        .unwrap_or(default)
}

/// A real-valued key of a type (S-0003 claim 6: a decimal is a single, widened
/// exactly), `None` when absent or without a number.
fn rule_real(import: &Import, kind: Kind, idx: usize, key: &[u8]) -> Option<f64> {
    match value::decimal(rule(import, kind, idx, key)?, value::DecimalField::Double) {
        value::Decimal::Value { value, .. } => value.to_exact_decimal().parse().ok(),
        value::Decimal::NoNumber => None,
    }
}

/// The drawing pitch angle of a flying type when the rules give none,
/// radians (S-0105 Import mapping: `PitchAngle` 20 degrees).
// spec: S-0105 Import mapping (PitchAngle: 20 degrees, engine initial value 0.3490658503988659 radians)
#[allow(clippy::approx_constant)] // the spec's decimal
const PITCH_ANGLE_DEFAULT: f64 = 0.3490658503988659;
/// The bank angle of a flying type when the rules give none, radians.
// spec: S-0105 Import mapping (RollAngle: 30 degrees, engine initial value 0.5235987755982988 radians)
#[allow(clippy::approx_constant)] // the spec's decimal
const ROLL_ANGLE_DEFAULT: f64 = 0.5235987755982988;
/// The speed fraction a flying body must exceed to pitch and bank.
// spec: S-0105 Import mapping (PitchSpeed: 0.25)
const PITCH_SPEED_DEFAULT: f64 = 0.25;
/// The pose a group of transient voxel images is drawn under: a cache key
/// of ours, not a behaviour (design: render (#104)). The images are fresh
/// draws (S-0105 claims 26 and 41, S-0106 claim 13, S-0113 claim 20), kept
/// apart from the settled ones and released once no redraw has used them
/// for [`TRANSIENT_REDRAWS`] redraws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Pose {
    /// A rocking or sinking body's exact drawn pitch and roll bits.
    Rocked(u32, u32),
    /// A crashing body's F1 table steps (0 … 8191) of ψ′ and φ.
    Tumble(u32, u32),
    /// A rocket's fresh pitch: the table step of −p (S-0105 claim 41).
    Rocket(u32),
}

/// Redraws a transient image group or vehicle shadow cutout may go unused
/// before its images are released; our own choice (design: render (#104)):
/// a few, so that a vehicle turning back to a recent facing finds its
/// cutout, while the images held follow the objects in play.
const TRANSIENT_REDRAWS: u64 = 4;

/// The scene images released by the painter, for its next transient images
/// to take (design: render (#104)), and the redraws ended so far.
#[derive(Clone, Debug, Default)]
struct TransientPool {
    free: Vec<u32>,
    now: u64,
}

/// Adds a transient voxel part or shadow image to `scene`, in a released
/// image of `pool` when there is one (design: render (#104)): its index and
/// offset from the draw point (S-0105 claim 25, F6). An empty image adds
/// nothing.
fn add_transient_image(
    scene: &mut Scene,
    pool: &mut TransientPool,
    img: &voxel::PartImage,
) -> Option<PlacedPart> {
    let image = IndexedImage::new(img.width, img.height, img.pixels.clone()).ok()?;
    if image.width() == 0 || image.height() == 0 {
        return None;
    }
    let id = match pool.free.pop() {
        Some(id) => {
            scene.resize_image(id, image);
            id
        }
        None => scene.add_image(image),
    };
    Some((id, [img.offset[0] as i32, img.offset[1] as i32]))
}

/// A type's drawing angle in radians (S-0105 Import mapping): the degrees
/// multiplied by the binary64 π/180, rounded toward zero; −1 keeps the value
/// in force (the default here).
fn rule_angle(import: &Import, kind: Kind, idx: usize, key: &[u8], default: f64) -> f64 {
    match rule_real(import, kind, idx, key) {
        // spec: S-0105 Import mapping (PitchAngle, RollAngle: −1 keeps the value in force)
        Some(d) if d != -1.0 => {
            glasnost_render::rz::mul_rz(d, std::f64::consts::PI / 180.0) // spec: S-0105 Import mapping (degrees times binary64 π/180, toward zero)
        }
        _ => default,
    }
}

/// How a body is posed (S-0105 claims 3, 41, 42), formed from its
/// [`Attitude`] and the type's drawing angles.
#[derive(Clone, Copy)]
enum Tilt {
    /// The body's heading alone.
    Plain,
    /// A rocket missile (claim 41).
    Rocket {
        /// The current pitch.
        pitch: f32,
        /// Its reuse class.
        class: voxel::RocketPitch,
    },
    /// A flying-locomotor body (claim 42).
    Flying(voxel::FlightAttitude),
    /// A crashing flying-locomotor body at strictly positive signed height,
    /// with its tumble (S-0113 claims 16, 18 and 20).
    Tumbling {
        /// The drawn pitch *ψ′*, radians: the tumble pitch, plus the
        /// drawing-pitch angle when the speed test passes (claim 20).
        pitch: f32,
        /// The tumble roll *φ*, radians.
        roll: f32,
    },
    /// A sinking or rocking vehicle (S-0504 claims 15 and 16; S-0303 claim
    /// 20): its pitch and roll, whether its locomotor draws rocking, and
    /// whether it sinks (then it draws no shadow).
    Rocked {
        /// The pitch, radians (positive lowers the bow).
        pitch: f32,
        /// The roll, radians.
        roll: f32,
        /// The locomotor draws rocking (S-0106 claims 5 and 9).
        drawn: bool,
        /// The vehicle sinks (S-0504 Terms).
        sinking: bool,
    },
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
        if options.colour_blind {
            // Our colour-blind colours stand in for the slot's definition
            // (presentation only).
            return COLOUR_BLIND_SLOTS.get(usize::from(s)).copied();
        }
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

/// The neutral radar fallback colour (S-0410 claim 8, S-0516 claim 38 and
/// Import mapping): the radar display RGB of the `LightGrey` colour
/// definition, as for an ownerless radar object; a disguise whose disguise
/// house is none uses it too. Without the definition, the grey our owner
/// colours fall back on.
pub fn neutral_radar_colour(import: &Import, bits: [u8; 3]) -> [u8; 3] {
    // spec: S-0516 Import mapping (LightGrey: the fixed lookup name of the fallback radar colour; its values come from user data)
    let hsv = import
        .outcome
        .colours
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(LIGHT_GREY))
        .map_or([0, 0, 200], |c| c.hsv); // nocite: the same placeholder grey as owner_definition
    display_rgb(hsv, bits)
}

/// The radar display RGB of colour definition `hsv` (S-0103 claim 15 and
/// Formulas).
fn display_rgb(hsv: [u8; 3], bits: [u8; 3]) -> [u8; 3] {
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
    display_rgb(hsv, bits)
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
    // spec: S-0112 claim 4 (an ore or gem cell draws the body of its tiberium's run at its cell
    // position, not its stored type: a map that stores one type for all its ore needs the run's
    // other bodies); S-0401 claims 9, 11 and 15 (resource spreads to new cells and mines make it in
    // play)
    tiberium_run_files(import, &mut out);
    out.insert(upper(VPL_FILE));
    // spec: S-0102 claim 31 (the shared building companion depth image)
    out.insert(upper(COMPANION_FILE));
    // spec: S-0111 Import mapping (the overlay sprites and palette of the status extras)
    for f in [
        super::combat::PIP_SPRITE,
        super::combat::PIP2_SPRITE,
        super::combat::BRACKET_SPRITE,
        super::combat::OVERLAY_PALETTE,
        // spec: S-0519 claim 24 and S-0111 Import mapping (the bomb icon and its cursor palette)
        super::combat::BOMB_SPRITE,
        super::combat::CURSOR_PALETTE,
        // spec: S-0111 claim 19 and Import mapping (the wrench sprite, through the cursor palette)
        super::combat::WRENCH_SPRITE,
    ] {
        out.insert(upper(f));
    }
    // spec: S-0402 Import mapping (the fixed harvesting sprite of claim 5a)
    out.insert(upper(super::harvesting::SPRITE_FILE));
    // spec: S-0107 Import mapping (the generic infantry-shadow sprite of claim 19, a fixed file)
    out.insert(upper(GENERIC_SHADOW_FILE));
    // spec: S-0103 claim 13 (the sidebar icon palette)
    out.insert(upper(glasnost_render::palette::ICON_PALETTE_FILE));
    out
}

/// Each tiberium's image run, in tiberium list order, from the image
/// numbers its rules layers gave (S-0112 claim 10).
fn tiberium_runs(import: &Import) -> Vec<Option<Run>> {
    let q = OverlayQuirks::default();
    let Some(list) = import.outcome.registries.get(&Kind::Tiberiums) else {
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

/// The sprites of every overlay type in every tiberium's run, flat and
/// sloped (S-0112 claim 4: an ore or gem cell's body is the run's type at
/// its cell position, whatever type the cell stores; S-0112 Terms).
fn tiberium_run_files(import: &Import, out: &mut BTreeSet<Vec<u8>>) {
    for run in tiberium_runs(import).into_iter().flatten() {
        // spec: S-0112 Terms (a run: F flat positions from P, then S sloped ones)
        for p in run.p..run.p + run.f + run.s {
            if let Ok(p) = usize::try_from(p) {
                art_files(import, Kind::Overlays, p, out);
            }
        }
    }
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

/// The art files of every building, vehicle, infantry and aircraft type: a
/// live game can create types the map does not hold (deploys, production,
/// spawned aircraft and missiles).
pub fn game_files(import: &Import) -> BTreeSet<Vec<u8>> {
    let mut out = BTreeSet::new();
    let art = art_document(import);
    for &(kind, idx) in import.arts.keys() {
        // spec: S-0306 claim 28 and S-0007 claim 27 (a projectile's sprite or voxel body)
        if matches!(
            kind,
            Kind::Buildings | Kind::Vehicles | Kind::Infantry | Kind::Aircraft | Kind::Projectiles
        ) {
            art_files(import, kind, idx, &mut out);
        }
        // spec: S-0007 claim 30 (a voxel animation's own model; no motion file is loaded for it)
        if kind == Kind::VoxelAnimations
            && let Some(a) = import.arts.get(&(kind, idx))
        {
            out.extend(a.voxels.values().map(|v| upper(v)));
        }
        if kind == Kind::Buildings {
            slot_files(import, &art, idx, &mut out);
        }
        // spec: S-0520 claim 3 (a crate placed during play is an overlay of its cell and is drawn:
        // the crate types' sprites, which a skirmish map's own crates never bring, S-0009 claim 12)
        if kind == Kind::Overlays && rule_bool(import, kind, idx, b"Crate", false) {
            art_files(import, kind, idx, &mut out);
        }
    }
    // spec: S-0102 claim 31 (the shared building companion depth image)
    out.insert(upper(COMPANION_FILE));
    out
}

/// The sprites of every animation and particle type, for the animations
/// and particles a game creates in play (S-0109, the move-feedback
/// animation of S-0214 claim 30 among them; S-0007 claim 28).
pub fn anim_files(import: &Import) -> BTreeSet<Vec<u8>> {
    import
        .arts
        .iter()
        .filter(|((kind, _), _)| matches!(kind, Kind::Animations | Kind::Particles))
        .filter_map(|(_, a)| a.sprite.as_ref().map(|s| upper(s)))
        .collect()
}

/// The named palette files of every infantry, vehicle, aircraft and
/// building type in `theater`, upper-case (S-0103 claims 16 and 17: each
/// file named for the current theater is read when the game's palette
/// files are; the game's types then find theirs among the held files).
pub fn named_palette_files(
    import: &Import,
    theater: glasnost_vfs::theater::Theater,
) -> BTreeSet<Vec<u8>> {
    let art = art_document(import);
    import
        .arts
        .iter()
        // spec: S-0103 claim 16 (only these four kinds name a palette of their own)
        .filter(|((kind, _), _)| {
            matches!(
                kind,
                Kind::Infantry | Kind::Vehicles | Kind::Aircraft | Kind::Buildings
            )
        })
        // spec: S-0103 Import mapping (Palette, from the type's art section)
        .filter_map(|(_, a)| art.value(&a.image, b"Palette"))
        .filter_map(|v| glasnost_render::palette::named_palette_file(v, theater))
        .collect()
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

/// What a house-coloured drawing's 53-shade table is built from: its
/// owner's colour definition and, for a type with a named palette, that
/// palette (its index in [`Painter`]'s named palettes) in place of the unit
/// palette (S-0103 claims 6, 7, 16 and 17).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Remap {
    hsv: [u8; 3],
    named: Option<u16>,
}

impl From<[u8; 3]> for Remap {
    fn from(hsv: [u8; 3]) -> Self {
        Self { hsv, named: None }
    }
}

/// Palette rows added to the scene, by table.
#[derive(Default)]
struct Rows {
    tiles: BTreeMap<TileTable, (usize, usize)>,
    /// The 53-shade house-colour and named-palette tables by colour
    /// definition and source palette, and the tint (‰) they are shaded
    /// with (S-0104 claim 3; S-0103 claim 17).
    houses: BTreeMap<(Remap, [i32; 3]), (usize, usize)>,
    /// The start-up tables: `false` the resource table, `true` the mine
    /// table (S-0112 claim 2).
    startup: BTreeMap<bool, (usize, usize)>,
    /// The anim palette's untinted table (S-0103 claim 8 row 5).
    anim: Option<(usize, usize)>,
    /// The single-shade table of the first colour definition (S-0103
    /// claim 8 row 4).
    alt: Option<(usize, usize)>,
    /// The single-shade tables of colour definitions by colour (S-0103
    /// claim 5), for voxel projectiles (S-0306 claim 30).
    single: BTreeMap<[u8; 3], (usize, usize)>,
    /// The general palette's untinted table (S-0306 Terms); `None` inside
    /// when its file is missing or unreadable.
    general: Option<Option<(usize, usize)>>,
    /// The first rows of the tile and house-colour tables given back by
    /// [`Painter::free_unused_rows`], by row count, for the next table of
    /// that size (design: render (#107)).
    free: BTreeMap<usize, Vec<usize>>,
}

impl Rows {
    /// Puts the table `rows` into `scene`: into the rows of a freed table
    /// of its size, or after the last row (design: render (#107)). Returns
    /// its first row.
    fn place(&mut self, scene: &mut Scene, rows: &[Row565]) -> usize {
        if let Some(base) = self.free.get_mut(&rows.len()).and_then(Vec::pop)
            && scene.replace_rows(base, rows)
        {
            return base;
        }
        let base = scene.rows.len();
        for &row in rows {
            scene.add_row(row);
        }
        base
    }
}

/// The shared building companion depth image as read (S-0102 claims 31
/// and 33).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Companion {
    /// Read: its index in `Scene::signed` and its stored rectangle's canvas
    /// column and row. A valid empty rectangle is read too: it covers no
    /// body pixel.
    Image(u32, (i32, i32)),
    /// Unavailable: eligible bodies use no companion.
    Unavailable,
}

/// Reads image frame 0 of the shared building companion depth image file
/// `bytes` (S-0102 claims 31 and 33): its signed adjustments and its stored
/// rectangle's canvas column and row, or the warning saying why it is
/// unavailable.
fn read_companion(bytes: Option<&[u8]>) -> Result<(SignedImage, (i32, i32)), String> {
    // spec: S-0102 claim 33 (a missing or unreadable file, zero image frames or an undecodable image frame 0: warn, no companion)
    let fail = |why: &str| Err(notes::no_companion(why));
    let Some(bytes) = bytes else {
        return fail("is missing");
    };
    let shp = match Shp::parse(bytes) {
        Ok(s) => s,
        Err(e) => return fail(&format!("cannot be read as a shape file ({e})")),
    };
    let Some(f) = shp.frames.first().copied() else {
        return fail("has no image frames");
    };
    if f.offset == 0 && f.width > 0 && f.height > 0 {
        // spec: S-0102 claim 33 and Import mapping (for this shared companion only, a non-empty image frame 0 with pixel-data position 0 is failed input, whatever bytes follow: warn and use no companion, not empty coverage; Q-0520)
        return fail("has an image frame 0 with a non-empty rectangle and pixel-data position 0");
    }
    // spec: S-0102 claim 33 (a valid, decodable empty rectangle retains its empty coverage, with no warning)
    let img = match shp.decode_frame(0) {
        Ok(i) => i,
        Err(e) => {
            return fail(&format!(
                "has an image frame 0 that cannot be decoded ({e})"
            ));
        }
    };
    // spec: S-0102 claim 31 (image frame 0 of the shared file; its bytes are signed depth adjustments decoded by F5, neither colours nor transparency)
    Ok((
        SignedImage::from_companion_bytes(img.width(), img.height(), img.pixels()),
        (i32::from(f.x), i32::from(f.y)),
    ))
}

/// What a scene's builder keeps for drawing more items later: the art file,
/// cell lights, palette rows and the images already added to the scene.
pub struct Painter {
    art: Document,
    lights: Lights,
    rows: Rows,
    images: BTreeMap<(Vec<u8>, usize, u8), Option<u32>>,
    /// Sprite files' headers and decoded frames, by upper-case name (`None`
    /// for a missing or unreadable file). A file's bytes do not change
    /// during a game ([`Assets`] changes only when it is loaded, and for the
    /// cursor's palette, which is no sprite).
    shps: BTreeMap<Vec<u8>, Option<ShpHeads>>,
    /// Animation types' drawing fields, read from their own art sections
    /// (S-0007 claim 34) once, by import registration index: the art file
    /// does not change during a game.
    anim_types: BTreeMap<usize, std::rc::Rc<slots::AnimType>>,
    /// Each type's image cache of voxel part and shadow images (S-0105
    /// claims 26, 31), as scene images with their offsets.
    voxels: BTreeMap<(Kind, usize), voxel::ImageCache<Option<PlacedPart>>>,
    /// The images of rocking bodies (a sinking vehicle's lean, S-0504
    /// claims 15 and 16), crashing bodies and rockets' fresh pitches, by
    /// type and [`Pose`]. S-0105 claims 26 and 41, S-0106 claim 13 and
    /// S-0113 claim 20 draw them fresh; a fresh draw is a function of its
    /// key and its angles alone, so these keep it under them, apart from
    /// the settled images, while redraws use it (design: render (#104)):
    /// each group with the last redraw that used it.
    rocked: BTreeMap<(Kind, usize, Pose), PoseImages>,
    /// The unmasked vehicle shadow footprints (S-0105 claims 31, 39), by
    /// type, pose (`None` for the settled images) and shadow key: kept here,
    /// not in the scene, as no item draws them (design: render (#106)). A
    /// pose's go with its images (design: render (#104)).
    footprints: BTreeMap<(Kind, usize, Option<Pose>, voxel::ImageKey), Option<Footprint>>,
    /// The number the next footprint takes, for the cutouts' keys.
    next_footprint: u32,
    /// Vehicle shadows with the current part image excluded, by footprint,
    /// part images and the parts' screen shift from the shadow's draw point
    /// (S-0105 claim 39): a memo of the per-appearance exclusion, each with
    /// the last redraw that used it, released when unused (design: render
    /// (#104)).
    cutouts: Cutouts,
    /// The released scene images and the redraw count (design: render
    /// (#104)).
    pool: TransientPool,
    /// The composed images of sprite-body vehicles with the turret flag
    /// (S-0105 claim 12, *One image*), by their parts (scene image and
    /// offset from the draw point, in painting order): bounded by the
    /// cached images they combine.
    composed: BTreeMap<Vec<(u32, i32, i32)>, Option<PlacedPart>>,
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
    /// The shared building companion depth image (S-0102 claims 31 and
    /// 33), once read.
    companion: Option<Companion>,
    /// Voxel models (upper-case file names) already warned about for an
    /// image larger than the retained limit (S-0105 claim 22: once per
    /// model).
    oversized: BTreeSet<Vec<u8>>,
    /// The building light sources the cells were lit with (S-0104 claim
    /// 12), kept for a relight.
    sources: Vec<LightSource>,
    /// The radiation fields' light sources the cells were last lit with
    /// (S-0525 claim 13), after the building lights.
    field_sources: Vec<LightSource>,
    /// The cell-pass items of terrain tiles (diamonds and extra images)
    /// with their cells, for a relight.
    tile_items: Vec<(usize, CellXY)>,
    /// The look of the building occupying each cell and that cell's shroud
    /// answer, for the live game's building slot animations (S-0102 claim
    /// 34, animation rule; S-0513 claim 29b); empty for the static scene.
    protected_cells: BTreeMap<CellXY, super::protection::SlotCell>,
    /// The live game's debris pieces' image slots (S-0317 claims 22, 24
    /// and 27).
    pieces: super::piece_pool::PiecePool<PieceKey>,
    /// The level factors of the object height terms (S-0104 claim 13),
    /// thousandths per height level: λ, chosen by claim 2's per-cell
    /// ranking, and λ<sub>air</sub>, the ion storm's or the normal one.
    object_levels: (i32, i32),
    /// The shadowed cells among those the live game's mobile objects stand
    /// on (S-0104 Terms, claim 13); empty for the static scene.
    shadowed_cells: BTreeSet<CellXY>,
    /// The effect tint the tintable tables show, hundredths (S-0104
    /// claims 3 and 5); `None` for every table's own tint.
    effect_tint: Option<[i32; 3]>,
    /// The rendering quirk switches the drawing reads (`docs/development.md`,
    /// *Rendering quirks*).
    view_quirks: ViewQuirks,
    /// Each type's named palette (S-0103 claims 16 and 17), by import kind
    /// and registration index: its index in `named`, or `None` for a type
    /// with none in the current theater. Settled at a type's first use:
    /// the art and the game's files do not change during a game.
    named_of: BTreeMap<(Kind, usize), Option<u16>>,
    /// The named palette files read, by upper-case name: their index in
    /// `named`, or `None` for a missing or unreadable file.
    named_files: BTreeMap<Vec<u8>, Option<u16>>,
    /// The named palettes, loaded (S-0103 claim 17).
    named: Vec<Palette>,
}

/// A voxel part image added to the scene: its index and offset from the
/// draw point (S-0105 claim 25, F6).
pub(super) type PlacedPart = (u32, [i32; 2]);

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
    /// The running slot animations of the live building being drawn
    /// (S-0108), in place of the set-up slots at their first image frame.
    live_slots: Option<Vec<LiveSlot>>,
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
/// The vehicle-shadow cutout memo of [`Painter`]: by footprint number,
/// part images ([`NO_PART`] after the last) and the parts' shift from the
/// shadow's draw point, with the last redraw that used each.
type Cutouts = BTreeMap<(u32, [u32; MAX_PARTS], (i32, i32)), (Cut, u64)>;

/// The most part images a voxel vehicle draws: body, turret and barrel
/// (S-0105 claims 4, 8, 12).
const MAX_PARTS: usize = 3;
/// Fills a cutout key's part images after the last (design: render (#106)).
const NO_PART: u32 = u32::MAX;

/// A vehicle shadow's unmasked footprint, kept in the painter (design:
/// render (#106)): its number, its image, and the scene image showing it
/// as it is, once an appearance needs it.
#[derive(Clone, Debug)]
struct Footprint {
    id: u32,
    image: voxel::PartImage,
    as_is: Option<Option<PlacedPart>>,
}

/// A cutout memo entry (design: render (#106)): the footprint as it is,
/// when the exclusion changes no pixel, or an image of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cut {
    AsIs,
    Own(Option<PlacedPart>),
}
/// A [`Pose`]'s images in [`Painter`], with the last redraw that used them.
type PoseImages = (voxel::ImageCache<Option<PlacedPart>>, u64);

/// One animation's draw: its layer, ground-layer sort key and items.
type SlotItems = (slots::Layer, i64, Vec<Item>);

/// A playing animation of the live game to draw (S-0109).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimDraw {
    /// Its current type's import registration index.
    pub idx: usize,
    /// Image frame *a* + *k* (claim 1).
    pub frame: i32,
    /// Stage *k*.
    pub stage: i32,
    /// End field *E* in play (claim 20's fade).
    pub end: i32,
    /// World position in leptons.
    pub pos: (i32, i32, i32),
    /// The house name of the object it is attached to and the unscaled
    /// ground light of that object's cell sampled at creation: a
    /// paratrooper's parachute is drawn with its carrier's house-colour unit
    /// palette (S-0522 claim 18, S-0103 claim 8 row 3).
    pub carrier: Option<(Vec<u8>, i32)>,
    /// The import type whose named palette it takes with its carrier's
    /// colour, when it is that object's own animation: a vehicle's deploy
    /// or undeploy animation, a parachute's carried object (S-0103 claim
    /// 16).
    pub named: Option<(Kind, usize)>,
    /// The depth adjustment its creator gave in place of its type's: a
    /// building's shot animation (S-0301 claim 22, F8).
    pub z_adjust: Option<i32>,
}

/// A particle of the live game to draw (S-0616 claims 23 to 28).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParticleDraw {
    /// A particle of a sprite kind (gas, smoke, fire, no kind).
    Sprite {
        /// Its type's import registration index.
        idx: usize,
        /// Its image frame ([`super::particles::frame`]).
        frame: i32,
        /// Its translucency code ([`super::particles::translucency`]).
        code: u8,
        /// World position in leptons.
        pos: (i32, i32, i32),
    },
    /// A particle of a pixel kind (spark, railgun): one pixel of its
    /// current colour (S-0616 F16).
    Pixel {
        /// Its current colour.
        rgb: [u8; 3],
        /// World position in leptons.
        pos: (i32, i32, i32),
    },
}

/// The palette of a projectile's sprite body (S-0306 claim 31).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectilePalette {
    /// The anim palette (the image's anim-palette flag).
    Anim,
    /// The 53-shade house-colour palette of the house of this name (the
    /// firer's-palette flag).
    House(Vec<u8>),
    /// The general palette.
    General,
}

/// A projectile in flight to draw (S-0306 claims 28 to 32).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectileDraw {
    /// Its type's import registration index.
    pub idx: usize,
    /// World position in leptons.
    pub pos: (i32, i32, i32),
    /// The 32-step drawn facing ψ and pitch facing *e* (F14).
    pub facings: (u8, u8),
    /// A sprite body's image frame (claim 29).
    pub frame: i32,
    /// A sprite body's palette (claim 31).
    pub palette: ProjectilePalette,
    /// A sprite body's shadow (claim 32): the shadow height η and the world
    /// height of the ground the shadow lies on (terrain or deck); `None`
    /// without one.
    pub shadow: Option<(i32, i32)>,
}

/// A debris piece in play to draw (S-0317 claims 15 and 22 to 28).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PieceDraw {
    /// Its number in the simulation, which keys its image slots.
    pub id: u32,
    /// Its type's import registration index (a voxel animation).
    pub idx: usize,
    /// World position in leptons.
    pub pos: (i32, i32, i32),
    /// The spin axis (*a*₃, *a*₂, *a*₁) before normalisation (S-0317 F5).
    pub axis: [i32; 3],
    /// The spin rate in radians per tick, a double's bits (S-0317 F5).
    pub spin: u64,
    /// The net count of its flying turns (S-0317 claim 22, F10).
    pub turns: i32,
    /// The name of the piece's house; empty without one.
    pub owner: Vec<u8>,
    /// The ground height *f* at its position (S-0317 claim 27; 0 off the
    /// legal map, claim 20).
    pub ground: i32,
    /// Its cell is a high-bridge cell (S-0317 claim 27).
    pub bridge: bool,
    /// Its cell is on the legal map (S-0317 claims 20 and 25).
    pub on_map: bool,
}

/// What a piece's images are drawn from (S-0317 claims 22 to 24): its type,
/// spin axis, spin rate's bits and net turn count.
pub(super) type PieceKey = (usize, [i32; 3], u64, i32);

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
        live_slots: None,
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

impl<'a> Builder<'a> {
    /// Animation type `idx`'s drawing fields, from its own art section.
    fn anim_fields(&mut self, idx: usize) -> std::rc::Rc<slots::AnimType> {
        let import = self.import;
        let p = &mut self.p;
        p.anim_types
            .entry(idx)
            .or_insert_with(|| {
                // spec: S-0007 claim 34 (the playback and drawing fields come from the animation's own art section)
                std::rc::Rc::new(slots::read_anim(&p.art, anim_own_section(import, idx)))
            })
            .clone()
    }

    /// A type's art, borrowed from the import rather than from the builder.
    fn type_art(&self, key: (Kind, usize)) -> Option<&'a glasnost_convert::art::TypeArt> {
        self.import.arts.get(&key)
    }

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
        self.p.sources = sources;
        // spec: S-0104 claim 1 (the map's normal set; current, base and target ambient start equal; tint in hundredths)
        self.p
            .compute_lights(self.map, self.rules, l.ambient, l, self.options.detail);
        // spec: S-0104 claims 2 and 13 (with no effect in force, λ and λ_air are the normal level factor)
        self.p.object_levels = (l.level, l.level);
    }

    fn tile_row(&mut self, table: TileTable, brightness: i32) -> Shade {
        self.p.tile_row_in(
            &mut self.built.scene,
            &self.palettes.tile,
            table,
            brightness,
        )
    }

    fn house_row(&mut self, remap: impl Into<Remap>, brightness: i32) -> Shade {
        let tint = self.p.house_tint();
        let (base, n) =
            self.p
                .house_table_in(&mut self.built.scene, self.palettes, remap.into(), tint);
        // spec: S-0104 claim 15, S-0110 claim 5 (each pixel takes the row of its light-alpha value)
        Shade::new(base, n, brightness)
    }

    /// The named palette of type `idx` of kind `kind` in the current
    /// theater (S-0103 claims 16 and 17): its index among the painter's
    /// named palettes, or `None` when the type has none.
    fn named_palette(&mut self, kind: Kind, idx: usize) -> Option<u16> {
        // spec: S-0103 claim 16 (only an infantry, vehicle, aircraft or building type names a palette of its own)
        if !matches!(
            kind,
            Kind::Infantry | Kind::Vehicles | Kind::Aircraft | Kind::Buildings
        ) {
            return None;
        }
        if let Some(&n) = self.p.named_of.get(&(kind, idx)) {
            return n;
        }
        // spec: S-0103 Import mapping (Palette, from the type's art section) and claim 17 (the
        // value with the current theater's suffix and .PAL)
        let theater = glasnost_convert::map::to_vfs_theater(self.map.theater);
        let file = self
            .art_value(kind, idx, b"Palette")
            .and_then(|v| glasnost_render::palette::named_palette_file(v, theater));
        let n = file.and_then(|f| self.p.named_file(&f, self.assets, self.palettes));
        self.p.named_of.insert((kind, idx), n);
        n
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
        let assets = self.assets;
        let scene = &mut self.built.scene;
        let heads = self.p.shp_heads(assets, file)?;
        let offset = *heads.frames.get(frame)?;
        let canvas = heads.canvas;
        let id = match heads.images[frame] {
            Some(id) => id,
            None => {
                let id = assets
                    .get(file)
                    .and_then(|b| Shp::parse(b).ok())
                    .and_then(|shp| shp.decode_frame(frame).ok())
                    .filter(|i| i.width() > 0 && i.height() > 0)
                    .map(|i| scene.add_image(i));
                heads.images[frame] = Some(id);
                id
            }
        };
        Some((id?, offset, canvas))
    }

    fn sprite_frames(&mut self, file: &[u8]) -> Option<usize> {
        let assets = self.assets;
        self.p.shp_heads(assets, file).map(|h| h.frames.len())
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
            stack: None,
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
                // spec: S-0010 claim 24 (missing tile cells: the plain image's tile cell at t mod (w × h) instead, our engine's correction with no switch; none there either: nothing painted)
                None => match plain
                    .cells
                    .get(s.rem_euclid((plain.width * plain.height).max(1)) as usize)
                    .copied()
                    .flatten()
                {
                    Some(i) => i,
                    None => {
                        self.drop_object(
                            "cell paints nothing: neither its shown image nor the plain image has a tile cell there",
                        );
                        continue;
                    }
                },
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
            let first = self.built.scene.items.len();
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
                    stack: None,
                });
            };
            // spec: S-0010 claim 22 (every pixel of the diamond is painted, index 0 included)
            push(img, dep, left, top, Paint::LitDiamond(row));
            if let Some((ei, ed, dx, dy)) = extra {
                // spec: S-0010 claim 23 (the extra image at the diamond box's corner plus the position difference;
                // index 0 transparent; drawn right after its own cell's diamond)
                push(ei, ed, left + dx, top + dy, Paint::Lit(row));
            }
            let pushed = first..self.built.scene.items.len();
            self.p.tile_items.extend(pushed.map(|i| (i, c)));
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
        tiberium_runs(self.import)
    }

    /// Overlays and their shadows (S-0112 claims 1 to 9; S-0102 claims 12,
    /// 13 and 24; S-0009 claim 12).
    fn overlays(&mut self) {
        let grid = self.cells_loaded.unwrap_or(&self.map.cells);
        let cells: Vec<(CellXY, glasnost_map::Cell)> =
            grid.iter().map(|(c, cell)| (c, *cell)).collect();
        self.overlays_in(cells, false);
    }

    /// The overlays and shadows of `cells`, as [`Builder::overlays`] draws
    /// them; `live` cells (redrawn during play) are not added to the
    /// object list, so their items name no object.
    fn overlays_in(&mut self, cells: Vec<(CellXY, glasnost_map::Cell)>, live: bool) {
        let q = OverlayQuirks::default();
        let k = Kind::Overlays;
        let list_len = self.import.outcome.registries.get(&k).map_or(0, Vec::len);
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
            // spec: S-0009 claim 12 (the map's crate overlays never appear in skirmish or multiplayer,
            // quirks.skirmish_drops_map_crates); S-0520 claim 3 (a crate placed during play is an overlay
            // of the cell and is drawn)
            let drops_crates = self
                .rules
                .is_none_or(|r| r.quirks.skirmish_drops_map_crates);
            if !live
                && self.cells_loaded.is_none()
                && self.options.skirmish
                && flags.crate_
                && drops_crates
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
                // spec: S-0112 claim 1 row 6 (a crate: the data value, or 0 with quirks.crate_data_value_ignored on)
                OverlayRow::Crate => own_sprite.clone().map(|s| (s, q.crate_body_frame(d))),
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
            // spec: S-0102 claim 13 (overlay biases: −2 − 15ℓ, or −17 − 15ℓ for other upright types)
            let lift = overlay_depth_lift(depth_level);
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
            let object = (!live).then(|| {
                self.add_object(ObjectInfo {
                    kind: "overlay",
                    type_name: String::from_utf8_lossy(&name).into_owned(),
                    owner: String::new(),
                    cell: c,
                    entry: format!("type index {o}, data {d}"),
                    notes: Vec::new(),
                })
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
                    object,
                )
            {
                self.built.scene.items.push(item);
            }
            // spec: S-0112 claim 8 (the shadow: crate N/2; rubble none; otherwise the stored type, d + N/2)
            let shadow: Option<(Vec<u8>, usize)> = if flags.crate_ {
                own_sprite.map(|s| {
                    let n = self.sprite_frames(&s).unwrap_or(0);
                    // spec: S-0112 claim 8 precedence 1 (N/2, or the data value + N/2 with quirks.crate_data_value_ignored off)
                    (s, q.crate_shadow_frame(d, n))
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
            // (every overlay kind)
            let shadow_bias = -2 - overlay_depth_lift(level);
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
                    object,
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
            // design: render (#111): only the covered cells inside the map grid
            // (0 ≤ x, y < the coordinate side) are walked, in i64, so a huge Width or
            // Height ends at the grid's edge instead of looping over cells that do not exist
            let side = i64::from(self.map.cells.footprint().coordinate_side());
            let (x0, y0) = (i64::from(s.cell.x), i64::from(s.cell.y));
            let xs = x0.max(0)..(x0 + i64::from(w.max(0))).min(side);
            let ys = y0.max(0)..(y0 + i64::from(h.max(0))).min(side);
            // spec: S-0009 claim 42 (the w × h cells (x + i, y + j)), S-0102 claim 25 (each covered cell paints the image again)
            for y in ys {
                for x in xs.clone() {
                    let (Ok(x), Ok(y)) = (i32::try_from(x), i32::try_from(y)) else {
                        continue;
                    };
                    let c = CellXY::new(x, y);
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
            if self.map.cells.get(t.cell).is_none() {
                continue;
            }
            // spec: S-0102 claim 23 (the centre of its cell at the terrain height: the surface height there, S-0106 claim 2, so a slope raises it)
            let (x, y) = cell_centre(t.cell.x, t.cell.y);
            let z = self.terrain_height(x, y);
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

    /// The shared building companion depth image, read once into
    /// [`Scene::signed`] (S-0102 claims 31 and 33, F5). An unavailable
    /// image is warned about once, when it is first needed.
    fn companion_image(&mut self) -> Companion {
        if let Some(c) = self.p.companion {
            return c;
        }
        let c = match read_companion(self.assets.get(COMPANION_FILE)) {
            Ok((image, stored)) => {
                self.built.scene.signed.push(image);
                Companion::Image((self.built.scene.signed.len() - 1) as u32, stored)
            }
            Err(warning) => {
                // spec: S-0102 claim 33 (an importer warning naming the shared file and the failure; loading continues)
                self.note(&warning);
                Companion::Unavailable
            }
        };
        self.p.companion = Some(c);
        c
    }

    /// The companion of the body of building type `i` drawn at draw point
    /// `p` (S-0102 claim 31, F5: live constants, the art displacement
    /// applied unless the building is in the construction state).
    fn body_companion(
        &mut self,
        i: usize,
        p: (i32, i32),
        construction: bool,
    ) -> Option<(u32, i32, i32)> {
        let foundation = self.foundation(i);
        // spec: S-0102 Import mapping (ZShapePointMove: two integers, default 0, 0)
        let m = self
            .art_value(Kind::Buildings, i, b"ZShapePointMove")
            .map(|v| value::int_tuple_or::<2>(v, [0, 0]).0)
            .unwrap_or([0, 0]);
        // spec: S-0102 F5 (live construction and selling omit the art displacement)
        let m = if construction { [0, 0] } else { m };
        match self.companion_image() {
            Companion::Image(id, stored) => {
                let origin =
                    companion_origin(p, foundation, (m[0], m[1]), CompanionBody::Live, stored);
                origin.map(|(x, y)| (id, x, y))
            }
            // spec: S-0102 claim 33 (an unavailable companion: the body keeps its draw point and bias, uses the plain upright profile of F2 and writes depth); F5 (widths of 8 cells or more use no companion anyway)
            Companion::Unavailable => None,
        }
    }

    /// Buildings in the cell pass (S-0108, S-0102 claims 13 and 16).
    fn structures(&mut self, ground: &mut Vec<GroundEntry>) {
        let list = self.map.objects.structures.clone();
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
            // spec: S-0112 claim 7 (a wall building of the map's building list is placed as its wall
            // overlay, which the game's cells hold); S-0406 claim 10 (no building remains)
            if self.cells_loaded.is_some()
                && self.rules.is_some_and(|r| {
                    r.index_of(
                        glasnost_sim::rules::Kind::Building,
                        &String::from_utf8_lossy(&p.type_name),
                    )
                    .and_then(|t| r.buildings.get(t as usize))
                    .is_some_and(|b| {
                        b.raw.econ.wall.unwrap_or(false)
                            && b.raw.econ.to_overlay.as_deref().is_some_and(|o| {
                                r.overlays.iter().any(|v| v.name.eq_ignore_ascii_case(o))
                            })
                    })
                })
            {
                self.drop_object("wall building placed as its wall overlay");
                continue;
            }
            let Some(art) = self.type_art((Kind::Buildings, i)) else {
                self.drop_object("building without art");
                continue;
            };
            let Some(cell) = self.map.cells.get(p.cell).copied() else {
                self.drop_object("building on a non-legal cell");
                continue;
            };
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
            // A map-placed building is not unloading when the scene is built.
            let items = self.building_items(
                // spec: S-0105 claim 16 and S-0609 claim 14 (a map-placed building has not spun: counter 0)
                i,
                art,
                p.cell,
                cell.level,
                &p.owner,
                condition,
                false,
                facing,
                0,
                object,
                None,
                None,
                Default::default(),
                // spec: S-0009 claim 40 (a map-placed building is complete: not in the construction state)
                None,
                ground,
            );
            self.built.scene.items.extend(items);
        }
    }

    /// The items of one building of type `idx` with its foundation's
    /// top-left cell at `cell` (height level `level`): body, shadow and bib
    /// (returned, for the cell pass) and its voxel turret or barrel (pushed
    /// to `ground` as a ground-layer entry). `condition` is (damaged, badly
    /// damaged); `facing` the turret's facing; `unloading` the unloading
    /// state of a live building, which adds the images of S-0108 claims 22
    /// and 23 (the door image goes to `ground` with the turret); `buildup`
    /// the build-up stage of a live building in the construction state
    /// (S-0108 claims 1, 2 and 4).
    #[allow(clippy::too_many_arguments)]
    fn building_items(
        &mut self,
        i: usize,
        art: &glasnost_convert::art::TypeArt,
        cell: CellXY,
        level: i8,
        owner: &[u8],
        condition: (bool, bool),
        occupied: bool,
        facing: u16,
        anim: u32,
        object: u32,
        unloading: Option<Unloading>,
        gate_door: Option<unload::DoorView>,
        look: super::protection::BuildingLook,
        buildup: Option<Buildup>,
        ground: &mut Vec<GroundEntry>,
    ) -> Vec<Item> {
        let mut items = Vec::new();
        let gate = rule_bool(self.import, Kind::Buildings, i, b"Gate", false);
        // spec: S-0108 Import mapping (GateStages, an art key: integer, absent 9)
        let gate_stages = self
            .art_value(Kind::Buildings, i, b"GateStages")
            .and_then(value::integer)
            .unwrap_or(9);
        // spec: S-0108 claim 4 (a gate opening or closing draws frame p, or p + G + 1 damaged, from
        // the body sprite; F8's rounding mode as the rules hold it); claim 2 (construction takes
        // precedence)
        let moving_gate = gate_door
            .filter(|_| gate && buildup.is_none())
            .and_then(|d| {
                let mode = self.rules.map_or(Rounding::TowardZero, |r| r.mode);
                unload::gate_frame(d, gate_stages, condition.0, mode)
            });
        // spec: S-0108 claim 4 (while the gate moves no unloading or under-door image is drawn);
        // claim 2 (construction takes precedence over the unloading sprites)
        let unloading = unloading.filter(|_| moving_gate.is_none() && buildup.is_none());
        let (hsv, _) = self.owner_colour(owner);
        // spec: S-0103 claim 16 (a building type's named palette: its body and sprite parts unless
        // it uses the terrain palette, its voxel parts and its own animations drawn with it)
        let remap = Remap {
            hsv,
            named: self.named_palette(Kind::Buildings, i),
        };
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
        // The building's row at its ordinary brightness adjusted by `look`.
        let lit = |b: &mut Self, look: super::protection::BuildingLook| {
            if terrain_palette {
                // spec: S-0103 claim 8, S-0104 claim 13 (terrain-palette building: the centre cell's table and scaled ground)
                let (w, h) = b.foundation(i);
                let (cx, cy) = light::building_centre_cell((cell.x, cell.y), (w, h));
                let centre = CellXY::new(cx, cy);
                let l = b.p.lights.get(centre);
                let table = l.map_or(TileTable::Neutral, |l| l.table);
                let ground = l.map_or(NEUTRAL_BRIGHTNESS, |l| l.scaled_ground);
                b.tile_row(table, look.brightness(ground))
            } else {
                // spec: S-0104 claim 13 (an ordinary building: U of its top-left foundation cell)
                b.house_row(remap, look.brightness(u))
            }
        };
        let row = lit(self, super::protection::BuildingLook::default());
        // spec: S-0102 claim 34 (a protected building body drawn as a sprite: S-0513 F9's brightness
        // before the shade-row selection; the force-shield selector's mask when force-shielded, none
        // under the curtain); S-0519 claims 25 and 26, F7 (the flash and the mark's factor reach the
        // body however it is drawn; a marked body takes the laser-target mask, united with the
        // force-shield mask)
        let shield_mask = self.protection_masks().building_look(look);
        let body_row = if look.active() {
            lit(self, look).with_mask(shield_mask)
        } else {
            row
        };
        // spec: S-0513 claim 29b (the bib, under-door and under-roof-door images, the image drawn
        // in place of the body and the door image take the body's envelope and mask: the
        // force-shield mask under the force shield, none under the curtain; the door's mask is
        // the drawing correction); S-0519 claims 25 and 26 (the flash and the mark reach the body,
        // its voxel parts and its animations only: accessory drawings stay outside)
        let protected = super::protection::BuildingLook {
            protection: look.protection,
            ..Default::default()
        };
        let accessory_row = match look.protection {
            Some(_) => {
                lit(self, protected).with_mask(self.protection_masks().building(look.protection))
            }
            None => row,
        };
        // spec: S-0108 claim 1 (in the construction state the build-up sprite; when it does not exist
        // nothing of the body is drawn: no body, bib or shadow, the slot animations still are)
        let body_sprite = match buildup {
            Some(_) => art
                .building_sprites
                .get("buildup")
                .cloned()
                .unwrap_or_default(),
            None => {
                let Some(s) = art.sprite.clone() else {
                    return items;
                };
                s
            }
        };
        let quirks = self.p.view_quirks;
        // spec: S-0108 claim 22 (while unloading, the deploying or roof-deploying sprite replaces the
        // body sprite)
        let unload_sprites = unloading.map(|u| {
            // spec: S-0108 claim 22 (quirks.roof_unload_no_fallback)
            let no_fallback = quirks.roof_unload_no_fallback;
            unload::unload_sprites(&art.building_sprites, u.roof, no_fallback)
        });
        let replaced = unload_sprites.and_then(|u| u.body);
        // spec: S-0108 claims 3 and 9 (N, the clamp and the shadow are the sprite in use's)
        let sprite = replaced.map_or(body_sprite, |(s, _)| s.to_vec());
        // spec: S-0108 claim 1 (no build-up sprite: nothing of the body, as a sprite of no frames)
        let n = if sprite.is_empty() {
            0
        } else {
            self.sprite_frames(&sprite).unwrap_or(0)
        };
        // spec: S-0009 claim 40 (a map-placed building is complete and idle at once);
        // S-0108 claim 6 (a change during the game's set-up is immediate: stage = a of the range)
        let idle = self
            .art_value(Kind::Buildings, i, b"AnimIdle")
            .map(|v| value::int_tuple_or::<3>(v, [0, 1, 0]).0)
            .unwrap_or([0, 1, 0]);
        // spec: S-0108 claim 3 (a negative start is kept: a negative index draws nothing)
        let k = idle[0];
        let occupiable = rule_bool(self.import, Kind::Buildings, i, b"CanBeOccupied", false);
        let (damaged, badly) = condition;
        // spec: S-0108 claim 2 (outside construction the occupiable rule comes before the stationary
        // gate rule: occupiable 2·[occupied] + [hurt], hurt badly damaged at tech level 0 or below,
        // damaged above; a gate 0, damaged G + 1; others k, damaged k + 1 in the idle state)
        let f: i32 = if let Some(b) = buildup {
            // spec: S-0108 claims 2 and 4 (construction: k; selling, and a gate both ways:
            // a + c − k − 1 with a = 0, c = n)
            if b.selling || gate {
                b.count.saturating_sub(1).saturating_sub(b.stage)
            } else {
                b.stage
            }
        } else if let Some(m) = moving_gate {
            // spec: S-0108 claim 2 (a moving gate uses claim 4, before the occupiable rule)
            i32::try_from(m).unwrap_or(-1)
        } else if occupiable {
            // spec: S-0108 Import mapping (TechLevel absent: 255)
            let tech = rule_int(self.import, Kind::Buildings, i, b"TechLevel", 255);
            // spec: S-0108 claim 2 (quirks.occupiable_hurt_rule; 2·[occupied] + [hurt])
            super::quirks::occupiable_frame(tech, damaged, badly, occupied, &quirks)
        } else if gate {
            if damaged {
                gate_stages.saturating_add(1)
            } else {
                0
            }
        } else {
            k.saturating_add(i32::from(damaged))
        };
        let n_i = i32::try_from(n).unwrap_or(i32::MAX);
        // spec: S-0108 claim 3 (drawn frame min(f, ⌊N/2⌋ − 1); with no body frames nothing is drawn)
        let body_frame = if moving_gate.is_some() {
            // spec: S-0108 claim 4 (the clamp of claim 3 does not apply to a moving gate)
            f
        } else {
            drawn_body_frame(f, n_i)
        };
        // spec: S-0108 claim 3, F2 (an index below 0 or at or past N draws nothing)
        let frame_of = |x: i32| usize::try_from(x).ok().filter(|&x| x < n);
        let normal_z = self
            .art_value(Kind::Buildings, i, b"NormalZAdjust")
            .and_then(value::integer)
            .unwrap_or(0);
        // spec: S-0102 claim 31, F5 (the body's companion depth image, placed from its draw point;
        // the art displacement applies outside the construction state)
        let companion = self.body_companion(i, anchor, buildup.is_some());
        // spec: S-0102 claim 13 (body: −L(z) − 2 + normal depth adjustment, upright, writes; drawn
        // as a deploying or roof-deploying sprite: −L(z) − 22 or −L(z) − 42 replacing the normal
        // depth adjustment, the companion depth image as for the normal body)
        let body_bias = replaced.map_or(-lz - 2 + normal_z, |(_, b)| -lz + b);
        // spec: S-0102 claim 16 (each building's group: body, then shadow, then bib, then under-door)
        if let Some(frame) = frame_of(body_frame)
            && let Some(item) = self.place_sprite(
                &sprite,
                frame,
                anchor,
                DepthTest::Writer,
                ItemDepth::Profile {
                    profile: Profile::Upright,
                    bias: body_bias,
                    companion,
                },
                Paint::Lit(body_row),
                Group::Buildings,
                false,
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
                false,
                Some(object),
            )
        {
            items.push(item);
        }
        // spec: S-0108 claim 7 (bib: same frame f, no clamp, outside the construction state)
        // spec: S-0108 claim 3 (a negative bib index draws nothing; one past the bib sprite neither)
        // spec: S-0108 claim 4 (no bib while a gate opens or closes)
        // spec: S-0108 claim 7 (no bib in the construction state, so never during build-up or selling)
        if let Some(bib) = art.building_sprites.get("bib").cloned()
            && moving_gate.is_none()
            && buildup.is_none()
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
                Paint::Lit(accessory_row),
                Group::Bibs,
                false,
                Some(object),
            )
        {
            items.push(item);
        }
        // spec: S-0108 claim 22 (the under-door sprite: image frame 1 when damaged, 0 otherwise;
        // none past the sprite, S-0108 claim 3), drawn after the bib (S-0102 claim 16). Its anchor
        // (claim 8's) and paint (the body's lit row) are our placeholders: no spec gives them;
        // its protection appearance is S-0513 claim 29b's
        if let Some(under) = unload_sprites.and_then(|u| u.under_door) {
            let frame = unload::under_door_frame(damaged);
            if self.sprite_frames(under).is_some_and(|c| frame < c)
                && let Some(item) = self.place_sprite(
                    under,
                    frame,
                    anchor,
                    DepthTest::Writer,
                    // spec: S-0102 claim 13 (under-door sprites: −L(z) − 2, flat, writes)
                    ItemDepth::Profile {
                        profile: Profile::Flat,
                        bias: -lz - 2,
                        companion: None,
                    },
                    Paint::Lit(accessory_row),
                    Group::Buildings,
                    false,
                    Some(object),
                )
            {
                items.push(item);
            }
        }
        // spec: S-0102 claim 34 (a building slot animation drawn with its building's palette and
        // stored brightness: the protected building's brightness, refreshed on each building turn);
        // S-0519 F7 (the same b₃ is the stored brightness: flash, protection and mark)
        let slot_u = look.brightness(u);
        self.slot_animations(i, cell, z, slot_u, remap, damaged, object, ground);
        // The building's parts in the object pass: the door image, then its voxel turret or barrel
        // (our placeholder order between the two).
        let mut parts = Vec::new();
        if let Some(un) = unloading
            && let Some(door) = art.building_sprites.get("door")
        {
            // spec: S-0108 claim 24 (`DoorStages` and `DamagedDoor` from the art section named by the image name)
            let (stages, damaged_door) = unload::door_inputs(
                self.art_value(Kind::Buildings, i, b"DoorStages"),
                self.art_value(Kind::Buildings, i, b"DamagedDoor"),
            );
            // spec: S-0108 F8, Edge cases (round-toward-zero mode: S-0205's switch, as the rules hold it)
            let mode = self.rules.map_or(Rounding::TowardZero, |r| r.mode);
            let frame = unload::door_frame(un.door, stages, damaged, damaged_door, mode);
            let count = self.sprite_frames(door).unwrap_or(0);
            if let Some(frame) = unload::door_image_frame(frame, count)
                // spec: S-0108 claim 23 (at the image anchor of claim 8); its paint, the body's lit row,
                // is our placeholder: S-0103 and S-0104 do not name the door image; its protection
                // appearance is S-0513 claim 29b's (as the bib, the drawing correction)
                && let Some(item) = self.place_sprite(
                    door,
                    frame,
                    anchor,
                    DepthTest::Reader,
                    // spec: S-0102 claim 13 (door sprite, object pass: −L(z) − 7, flat, no depth write)
                    ItemDepth::Profile {
                        profile: Profile::Flat,
                        bias: -lz - 7,
                        companion: None,
                    },
                    Paint::Lit(accessory_row),
                    Group::Buildings,
                    false,
                    Some(object),
                )
            {
                parts.push(item);
            }
        }
        // Voxel turret or barrel in the object pass (S-0105 claims 5, 9, 37; S-0102 claims 5, 13).
        let turret_voxel = rule_bool(self.import, Kind::Buildings, i, b"TurretAnimIsVoxel", false);
        if turret_voxel || art.voxels.contains_key("barrel") {
            // spec: S-0102 claim 34 (a building voxel turret/barrel: S-0513 F9's brightness; the
            // force-shield selector's mask for the force shield, none for the curtain); S-0519
            // claims 25 and 26, F7 (flash and mark reach the voxel parts; the marked body's mask)
            let b = look.brightness(u);
            parts.extend(self.building_voxels(
                i,
                art,
                facing,
                anim,
                anchor,
                lz,
                b,
                shield_mask,
                remap,
                object,
            ));
        }
        if !parts.is_empty() {
            // spec: S-0102 F4 (key = 256·(c_x + c_y) + 32 for a voxel turret − 16 for a gate)
            let key = i64::from(256 * (cell.x + cell.y)) + if turret_voxel { 32 } else { 0 }
                - if gate { 16 } else { 0 };
            ground.push(GroundEntry {
                key,
                items: parts,
                object: Some(object),
            });
        }
        items
    }

    /// A building's slot animations (S-0108 claims 10 and 11), each drawn
    /// as an animation of its own in its type's layer (S-0102 claim 17):
    /// ground layer entries go to `ground`, the others to
    /// [`Built::layered`]. A live building draws the ones running
    /// ([`LiveSlot`]), a building of the static scene those the set-up
    /// starts, at their first image frame.
    #[allow(clippy::too_many_arguments)]
    fn slot_animations(
        &mut self,
        i: usize,
        cell: CellXY,
        z: i32,
        u: i32,
        remap: Remap,
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
        // A live building draws its running slot animations at their current image frames
        // (S-0108 Terms; S-0109 claim 1); a building of the static scene the slots the set-up
        // starts, at their first image frame.
        let running = self.live_slots.take();
        let still = running.is_none();
        #[allow(clippy::type_complexity)]
        let entries: Vec<(slots::SlotDef, usize, Option<(i32, i32, i32)>)> =
            if let Some(list) = running {
                list.into_iter()
                    .map(|s| (s.def, s.idx, Some((s.frame, s.stage, s.end))))
                    .collect()
            } else {
                // spec: S-0108 claim 11 (placed during the set-up: idle and active-1 … active-4,
                // occupied variants allowed)
                slots::SETUP_SLOTS
                    .iter()
                    .map(|base| slots::read_slot(&self.p.art, &image, base))
                    .filter_map(|def| {
                        // spec: S-0108 claim 11 (no occupants at placement: the damaged or the normal variant)
                        let name = slots::variant(&def, damaged, false, true).to_vec();
                        if name.is_empty() {
                            return None;
                        }
                        // Unknown names start nothing here (S-0108 claim 11 leaves them unmeasured).
                        let a = find_type(self.import, Kind::Animations, &name)?;
                        Some((def, a, None))
                    })
                    .collect()
            };
        let mut any = false;
        for (def, a, at) in entries {
            let Some(art) = self.type_art((Kind::Animations, a)) else {
                continue;
            };
            let Some(sprite) = art.sprite.clone() else {
                continue;
            };
            any = true;
            // spec: S-0007 claim 34 (the playback and drawing fields come from the animation's own art section, whatever section its image name names)
            let t = self.anim_fields(a);
            // spec: S-0109 claim 19 (detail field above the player's level: not drawn)
            if !slots::shown_at(&t, self.options.detail.number()) {
                continue;
            }
            let n = i32::try_from(self.sprite_frames(&sprite).unwrap_or(0)).unwrap_or(i32::MAX);
            // spec: S-0108 claim 10, S-0101 F6 (the anchor moved by the displacement, converted to
            // world units)
            let (dx, dy) = screen_to_ground(def.x, def.y);
            let pos = (256 * cell.x + dx, 256 * cell.y + dy, z);
            let row = self.anim_row(&t, remap, u, CellXY::new(pos.0 >> 8, pos.1 >> 8));
            // spec: S-0513 claim 29b (the cell holding the animation's world position: its horizontal coordinates divided by 256, truncated toward zero, height ignored)
            let anim_cell = CellXY::new(pos.0 / 256, pos.1 / 256);
            let detail = self.options.detail.number();
            let (frame, code) = match at {
                // spec: S-0109 claims 1 and 20 (image frame a + k; the code from stage k and end E,
                // translucency level 0)
                Some((frame, stage, end)) => (frame, slots::translucency(&t, detail, stage, end)),
                None => (
                    slots::first_frame(&t, n),
                    slots::first_translucency(&t, detail, n),
                ),
            };
            // spec: S-0102 claim 34, animation rule (a building slot animation is marked as a
            // building animation: the building occupying its cell supplies the force-shield mask
            // while it is force-shielded; curtain protection supplies none; the laser-target mask
            // while it is marked, united with the force-shield mask, S-0519 claim 26). S-0513 claim
            // 29b (slot animations, Q-1529): the animation tests its own cell, not its building's
            // centre cell; while that cell is shrouded for the local player it takes no colour mask
            // at all, and its brightness is unchanged. The rule is for opaque drawings; one whose
            // own art draws it translucent takes no mask (our interim reading: S-0513 claim 29b and
            // Open questions leave it open, and F8's blend takes no mask).
            let cell_look = self.p.protected_cells.get(&anim_cell).copied();
            let row = if code == 0 {
                row.with_mask(self.protection_masks().slot_animation(cell_look))
            } else {
                row
            };
            // spec: S-0102 claim 17 (the slot's depth adjustment replaces the type's)
            let items = self.anim_items(
                &sprite,
                &t,
                n,
                frame,
                pos,
                def.z_adjust,
                (row, code),
                Group::Buildings,
                Some(object),
            );
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
            self.note(if still {
                notes::SLOT_ANIMS_STILL
            } else {
                notes::SLOT_ANIMS_LIVE
            });
        }
    }

    /// The items of one animation of type `t` showing image frame `frame`
    /// of `sprite` (which holds `n` image frames) at world position `pos`,
    /// with animation depth adjustment `z_adjust`, shade row and
    /// translucency code `shade`: the image (or its tiled stack), then its
    /// shadow (S-0109 claims 1, 10, 26 and 27; S-0102 claims 13 and 19).
    #[allow(clippy::too_many_arguments)]
    fn anim_items(
        &mut self,
        sprite: &[u8],
        t: &slots::AnimType,
        n: i32,
        frame: i32,
        pos: (i32, i32, i32),
        z_adjust: i32,
        shade: (Shade, u8),
        group: Group,
        object: Option<u32>,
    ) -> Vec<Item> {
        let lz = height_lift(pos.2);
        // spec: S-0109 claim 26 (the screen pixel of the position with the height lift)
        let p = project_lifted(pos.0, pos.1, pos.2);
        let v = t.y_draw_offset;
        // spec: S-0109 claim 26 (the image is moved down by the type's vertical draw shift v)
        let point = (p.0, p.1 + v);
        let (row, code) = shade;
        // spec: S-0109 claim 20, S-0102 claim 32 and F8 (a nonzero translucency code blends the
        // image's pixels with the colour already drawn; code 0 draws opaque)
        let paint = match Translucency::from_code(i32::from(code)) {
            Some(code) => Paint::Translucent(row, code),
            None => Paint::Lit(row),
        };
        // spec: S-0109 claim 1 (an image frame outside 0 … N − 1 draws nothing, nor its shadow)
        let frame_of = |x: i32| {
            usize::try_from(x)
                .ok()
                .filter(|&x| (x as i64) < i64::from(n))
        };
        // spec: S-0109 claim 27 (a tiled type draws a stack of copies of its current image frame)
        let (image_point, stack) = if t.tiled {
            self.tiled_placement(sprite, p, v)
        } else {
            (point, None)
        };
        let mut items = Vec::new();
        // spec: S-0102 claim 13 (animation rows: ordinary, flat, tiled)
        let (profile, bias) = if t.tiled {
            (Profile::Upright, v + z_adjust - lz - 50)
        } else if t.flat {
            (Profile::Flat, v + z_adjust - lz - 3)
        } else {
            (Profile::Upright, v + z_adjust - lz - 2)
        };
        let in_range = frame_of(frame);
        if let Some(f) = in_range
            && let Some(mut item) = self.place_sprite(
                sprite,
                f,
                image_point,
                // spec: S-0102 claims 13 and 17 (animations are depth readers)
                DepthTest::Reader,
                ItemDepth::Profile {
                    profile,
                    bias,
                    companion: None,
                },
                paint,
                group,
                false,
                object,
            )
        {
            item.stack = stack;
            items.push(item);
        }
        // spec: S-0109 claim 10 (a shadow type also draws f + ⌊N/2⌋ as its shadow, unless flat or
        // tiled or f itself is out of range: after the image, with nothing between, at the
        // unshifted point p)
        if t.shadow
            && !t.flat
            && !t.tiled
            && in_range.is_some()
            && let Some(sf) = frame_of(frame.saturating_add(n / 2))
            && let Some(item) = self.place_sprite(
                sprite,
                sf,
                p,
                DepthTest::Reader,
                // spec: S-0102 claim 13 (animation shadow: −2 − L(z), flat)
                ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias: -2 - lz,
                    companion: None,
                },
                // spec: S-0102 claim 32 (the shadow is never blended: plain halving, even
                // under a translucent image)
                Paint::Shadow,
                group,
                false,
                object,
            )
        {
            items.push(item);
        }
        items
    }

    /// One playing animation of the live game (S-0109): its layer, its
    /// ground-layer sort key and its items, or `None` when it draws
    /// nothing (no art, no sprite, or above the player's detail level).
    fn live_anim(&mut self, a: &AnimDraw) -> Option<(slots::Layer, i64, Vec<Item>)> {
        let art = self.type_art((Kind::Animations, a.idx))?;
        let sprite = art.sprite.clone()?;
        // spec: S-0007 claim 34 (the animation's own art section)
        let t = self.anim_fields(a.idx);
        // spec: S-0109 claim 19 (detail field above the player's level: not drawn, still updated)
        if !slots::shown_at(&t, self.options.detail.number()) {
            return None;
        }
        let n = i32::try_from(self.sprite_frames(&sprite).unwrap_or(0)).unwrap_or(i32::MAX);
        // spec: S-0103 claim 8 (an animation that belongs to no object: row 4 or 5, by its own
        // art flags; row 3 is only for an object's own animation)
        let unowned = slots::AnimType {
            cell_drawer: false,
            ..(*t).clone()
        };
        let cell = CellXY::new(a.pos.0 >> 8, a.pos.1 >> 8);
        let row = if let Some((house, light)) = &a.carrier {
            // spec: S-0522 claim 18 and S-0103 claim 8 (an ordinary untiled parachute uses the carried infantry's house-colour unit palette, row 3, at the unscaled ground light of its cell sampled at creation, without extra unit light or an airborne-height addition; the normal-light override, special palettes and the tiled-copy exception stay S-0103 claim 8's)
            let (hsv, _) = self.owner_colour(house);
            // spec: S-0103 claim 16 (an object's deploy, undeploy or parachute animation takes its
            // named palette, also for a vehicle that ignores it)
            let named = a.named.and_then(|(k, i)| self.named_palette(k, i));
            let carried = slots::AnimType {
                cell_drawer: true,
                ..(*t).clone()
            };
            self.anim_row(&carried, Remap { hsv, named }, *light, cell)
        } else {
            self.anim_row(&unowned, [0, 0, 0].into(), NEUTRAL_BRIGHTNESS, cell)
        };
        // spec: S-0109 claim 20 (an animation other than a building's slot animations has t = 0)
        let code = slots::translucency(&t, self.options.detail.number(), a.stage, a.end);
        let items = self.anim_items(
            &sprite,
            &t,
            n,
            a.frame,
            a.pos,
            // spec: S-0102 claim 13, Import mapping (ZAdjust: the type's depth adjustment, unless its creator replaced it, S-0301 F8)
            a.z_adjust.unwrap_or(t.z_adjust),
            (row, code),
            Group::Effects,
            None,
        );
        // spec: S-0102 claim 5 (X + Y of the position plus the animation's sort adjustment)
        let key = i64::from(a.pos.0) + i64::from(a.pos.1) + i64::from(t.y_sort_adjust);
        // spec: S-0102 claim 3 (an unattached animation is in its type's layer)
        Some((t.layer, key, items))
    }

    /// One particle of the live game: a sprite kind's image frame or a
    /// pixel kind's pixel (S-0616 claims 27 and 28); no items without a
    /// sprite or for a frame outside the sprite.
    /// One projectile in flight (S-0306 claims 28 to 32): its sprite body
    /// with its shadow first, or its voxel body; no items when nothing is
    /// drawn for it (no art, a frame the sprite does not hold, or no
    /// general palette for a body that needs it).
    fn live_projectile(&mut self, d: &ProjectileDraw) -> Vec<Item> {
        let Some(art) = self.type_art((Kind::Projectiles, d.idx)) else {
            return Vec::new();
        };
        let Some(look) = art.projectile.clone() else {
            return Vec::new();
        };
        // spec: S-0306 claim 28 (an invisible projectile draws nothing at all)
        if look.inviso {
            return Vec::new();
        }
        // spec: S-0306 claim 28 and S-0101 F1 (the draw point: the screen point of its position, height lift included, no further shift)
        let point = project_lifted(d.pos.0, d.pos.1, d.pos.2);
        if look.voxel {
            return self.voxel_projectile(d, art, &look, point);
        }
        // spec: S-0306 claim 28 (a sprite projectile without a sprite draws nothing)
        let Some(sprite) = art.sprite.clone() else {
            return Vec::new();
        };
        let n = self.sprite_frames(&sprite).unwrap_or(0);
        // spec: S-0306 claim 28 (a frame number at or above the frame count draws nothing: neither body nor shadow)
        let Some(f) = usize::try_from(d.frame).ok().filter(|&f| f < n) else {
            return Vec::new();
        };
        let row = match &d.palette {
            // spec: S-0306 claim 31 case 1 (the anim palette, at brightness 1000)
            ProjectilePalette::Anim => self.anim_palette_row(PROJECTILE_BRIGHTNESS),
            // spec: S-0306 claim 31 case 2 (the 53-shade house-colour palette of that house, at brightness 1000)
            ProjectilePalette::House(owner) => {
                let (hsv, _) = self.owner_colour(owner);
                self.house_row(hsv, PROJECTILE_BRIGHTNESS)
            }
            // spec: S-0306 claim 31 case 3 (the general palette, at brightness 1000)
            ProjectilePalette::General => match self.general_row() {
                Some((base, rows)) => Shade::new(base, rows, PROJECTILE_BRIGHTNESS),
                None => {
                    self.note(notes::NO_GENERAL_PALETTE);
                    return Vec::new();
                }
            },
        };
        let mut items = Vec::new();
        if let Some((eta, ground)) = d.shadow {
            // spec: S-0306 claim 32 (the body's own frame as a shadow, centred L(η) rows below the draw point, just before the body)
            let at = (point.0, point.1 + height_lift(eta));
            items.extend(self.place_sprite(
                &sprite,
                f,
                at,
                // spec: S-0102 claims 10, 12 and 13 (a flat reader, bias −10 − L of the ground or deck world height under it)
                DepthTest::Reader,
                ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias: PROJECTILE_SHADOW_BIAS - height_lift(ground),
                    companion: None,
                },
                // spec: S-0102 claim 26 (a plain darkening that ignores palette, brightness and light-alpha values)
                Paint::Shadow,
                Group::Shadows,
                false,
                None,
            ));
        }
        items.extend(self.place_sprite(
            &sprite,
            f,
            point,
            // spec: S-0102 claims 10, 12 and 13 (a projectile sprite: a flat reader, bias −30 − L(z))
            DepthTest::Reader,
            ItemDepth::Profile {
                profile: Profile::Flat,
                bias: PROJECTILE_BIAS - height_lift(d.pos.2),
                companion: None,
            },
            // spec: S-0306 claim 31 (through the light-alpha values; never translucent)
            Paint::Lit(row),
            Group::Effects,
            false,
            None,
        ));
        items
    }

    /// A projectile's voxel body (S-0306 claim 30): the body model of its
    /// voxel art in motion-file frame 0, posed by its drawn facing and
    /// pitch, shaded by F16, in the palette of its colour definition; no
    /// shadow.
    fn voxel_projectile(
        &mut self,
        d: &ProjectileDraw,
        art: &glasnost_convert::art::TypeArt,
        look: &glasnost_convert::art::ProjectileLook,
        point: (i32, i32),
    ) -> Vec<Item> {
        let q = VoxelQuirks::default();
        // spec: S-0306 claims 28 and 30 (only the body model; absent or discarded voxel art draws nothing)
        let Some(file) = art.voxels.get("body").cloned() else {
            return Vec::new();
        };
        let (psi, e) = d.facings;
        // Our cache key: a voxel projectile is drawn afresh in every drawn frame (claim 30), and
        // its image depends only on the model, ψ and e (frame 0, a fixed light, a stateless
        // shade table), so a kept image equals a fresh draw.
        let key = voxel::ImageKey::Projectile {
            heading: psi,
            pitch: e,
        };
        let part = self.cached_part(Kind::Projectiles, d.idx, key, |b| {
            let model = b.voxel_model(&file)?;
            let mode = model.sections.first()?.normal_mode;
            // spec: S-0306 claim 30 (Rz(−(π/16)(ψ − 8)) · Ry(+(π/16)(e − 8)) under the fixed view)
            let draw = voxel::projectile_body(psi, e, &q);
            // spec: S-0306 claim 30 and F16 (one diffuse term, the table of the first section's normal mode)
            let shades = voxel::projectile_shade_table(&draw, mode, &q);
            let vpl = b.vpl();
            // spec: S-0306 claim 30 (motion-file frame 0)
            let img = voxel::draw_model_shaded(&model, 0, &draw, vpl.as_ref(), &shades, &q);
            b.limit_checked(&file, "projectile", key, img)
        });
        let Some(part) = part else {
            return Vec::new();
        };
        let colours = &self.import.outcome.colours;
        let row = match look.colour {
            // spec: S-0306 claim 30 (the 53-shade palette of the selected colour definition, at brightness 1000)
            ProjectileColour::Shades(i) => {
                let hsv = colours.get(i).map_or([0, 0, 0], |c| c.hsv);
                self.house_row(hsv, PROJECTILE_BRIGHTNESS)
            }
            // spec: S-0306 claim 30 (the single-shade palette of that definition: one shade row)
            ProjectileColour::Single(i) => {
                let hsv = colours.get(i).map_or([0, 0, 0], |c| c.hsv);
                let (base, n) = self.single_row(hsv);
                Shade::new(base, n, PROJECTILE_BRIGHTNESS)
            }
        };
        let (test, bias) = if look.flat {
            // spec: S-0306 claim 30 and S-0102 claim 13 (a flat voxel projectile: a flat reader, final bias −65, no −L(z))
            (DepthTest::Reader, FLAT_VOXEL_PROJECTILE_BIAS)
        } else {
            // spec: S-0306 claim 30 and S-0102 claim 10 (a non-flat voxel projectile neither tests nor writes depth)
            (DepthTest::Ignore, 0)
        };
        vec![Item {
            image: part.0,
            // spec: S-0306 claim 28 and S-0105 claim 25 (the copy rectangle around the draw point, as a part)
            x: point.0 + part.1[0],
            y: point.1 + part.1[1],
            test,
            depth: ItemDepth::Profile {
                profile: Profile::Flat,
                bias,
                companion: None,
            },
            // spec: S-0306 claim 30 (through the light-alpha values)
            paint: Paint::Lit(row),
            group: Group::Effects,
            provisional: false,
            object: None,
            stack: None,
        }]
    }

    /// A debris piece's shadow and body (S-0317 claims 15 and 22 to 28,
    /// S-0007 claim 30): one section of its own model, with no motion file,
    /// turned by its orientation alone; no items when its type has no
    /// model, the model does not parse or the section is not in it.
    fn live_piece(&mut self, d: &PieceDraw) -> Vec<Item> {
        let Some(file) = self
            .import
            .arts
            .get(&(Kind::VoxelAnimations, d.idx))
            .and_then(|a| a.voxels.get("body").cloned())
        else {
            return Vec::new();
        };
        let q = VoxelQuirks::default();
        // spec: S-0007 claim 30 (the own model loads no motion file)
        let Some(model) = self.bare_voxel_model(&file) else {
            return Vec::new();
        };
        // spec: S-0317 claim 23 and Import mapping (the section of the type's VoxelIndex, absent 0; one outside the model draws nothing)
        let index = rule_int(self.import, Kind::VoxelAnimations, d.idx, b"VoxelIndex", 0);
        let Some(section) = usize::try_from(index)
            .ok()
            .and_then(|i| model.sections.get(i))
            .cloned()
        else {
            return Vec::new();
        };
        let key: PieceKey = (d.idx, d.axis, d.spin, d.turns);
        let images = match self.p.pieces.current(d.id, &key) {
            Some(i) => i.clone(),
            None => {
                // spec: S-0317 claim 22 and F10 (the turn about the axis by the spin rate times the net turn count; no view transform; a zero axis: no spin, F5)
                let angle = f64::from_bits(d.spin) * f64::from(d.turns);
                let r = voxel::piece_rotation(d.axis, angle).unwrap_or(voxel::Mat34::IDENTITY);
                // spec: S-0317 claim 24 (the normal table of the first section's normal mode, S-0105 claim 18's quirks.voxel_light_first_section_mode; off, the drawn section's)
                let mode = if q.voxel_light_first_section_mode {
                    model.sections.first().map_or(0, |s| s.normal_mode)
                } else {
                    section.normal_mode
                };
                // spec: S-0317 claim 24 (S-0306 F16's one diffuse term with L′ = Rᵀ L, no specular term)
                let shades = voxel::projectile_shade_table(&r, mode, &q);
                // spec: S-0317 claim 23 (the section drawn with claim 22's transform only, no motion-file transform)
                let one = voxel::VoxelModel::from_sections(vec![section.clone()], None, &q);
                let vpl = self.vpl();
                let body = voxel::draw_model_shaded(&one, 0, &r, vpl.as_ref(), &shades, &q);
                let body = self.limit_checked(&file, "piece", "its orientation", body);
                // spec: S-0317 claim 27 and F11 (the footprint of the drawn section under the rotation alone)
                let shadow = voxel::draw_piece_shadow(&section, &r, &q);
                self.p
                    .pieces
                    .store(
                        &mut self.built.scene,
                        d.id,
                        key,
                        body.as_ref(),
                        shadow.as_ref(),
                    )
                    .clone()
            }
        };
        let (x, y, z) = d.pos;
        // spec: S-0317 Terms (the cell of a position: (trunc(x / 256), trunc(y / 256)))
        let cell = CellXY::new(x / 256, y / 256);
        let mut items = Vec::new();
        // spec: S-0317 claim 27 (the shadow before the body)
        if let Some(part) = images.shadow.placed {
            let f = d.ground;
            // spec: S-0317 claim 27 (in a high-bridge cell with the piece at least f + 416 high, lifted by L(f + 417), the depth from f + 416)
            let (lift, hs) = if d.bridge && z >= f + 416 {
                (f + 417, f + 416)
            } else {
                (f, f)
            };
            // spec: S-0317 claim 27 (the ground projection of (x, y) lifted once by L(f))
            let point = project_lifted(x, y, lift);
            items.push(Item {
                image: part.0,
                x: point.0 + part.1[0],
                y: point.1 + part.1[1],
                // spec: S-0317 claim 27 (reads depth with the flat profile, final bias −4 − L(hₛ); never writes depth)
                test: DepthTest::Reader,
                depth: ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias: -4 - height_lift(hs),
                    companion: None,
                },
                // spec: S-0317 claim 27 (the ordinary shadow treatment, not through the light-alpha values)
                paint: Paint::Shadow,
                group: Group::Shadows,
                provisional: false,
                object: None,
                stack: None,
            });
        }
        if let Some(part) = images.body.placed {
            // spec: S-0317 claim 25 and claim 20 (U: the unscaled ground level of the position's cell; off the legal map the off-map cell's 1000)
            let u = if d.on_map {
                self.p
                    .lights
                    .get(cell)
                    .map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground)
            } else {
                PIECE_BRIGHTNESS
            };
            let row = if !d.owner.is_empty() {
                // spec: S-0317 claim 25 (a piece that holds a house: its house-colour palette at U, no extra light, height or deck term)
                let hsv = self.owner_colour(&d.owner).0;
                self.house_row(hsv, u)
            } else if rule_bool(
                self.import,
                Kind::VoxelAnimations,
                d.idx,
                b"IsTiberium",
                false,
            ) {
                // spec: S-0317 claim 25 (no house and the resource flag: the start-up mine table at U)
                self.startup_row(true, u)
            } else {
                // spec: S-0317 claim 25 (any other piece: the start-up mine table at brightness 1000)
                self.startup_row(true, PIECE_BRIGHTNESS)
            };
            // spec: S-0317 claim 25 and Import mapping (Translucent: 50 % by S-0102 F8's row; absent, opaque; through the light-alpha values either way)
            let paint = if rule_bool(
                self.import,
                Kind::VoxelAnimations,
                d.idx,
                b"Translucent",
                false,
            ) {
                Paint::Translucent(row, Translucency::Half)
            } else {
                Paint::Lit(row)
            };
            // spec: S-0317 claim 26 (around the ground projection of the position lifted once by L(z))
            let point = project_lifted(x, y, z);
            items.push(Item {
                image: part.0,
                x: point.0 + part.1[0],
                y: point.1 + part.1[1],
                // spec: S-0317 claim 26 (reads depth with the flat profile and the strict test, final bias −2 − L(z); never writes depth)
                test: DepthTest::Reader,
                depth: ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias: -2 - height_lift(z),
                    companion: None,
                },
                paint,
                group: Group::Effects,
                provisional: false,
                object: None,
                stack: None,
            });
        }
        items
    }

    /// The single-shade table of a colour definition (S-0103 claim 5): one
    /// untinted row (S-0104 claim 3), added to the scene once.
    fn single_row(&mut self, hsv: [u8; 3]) -> (usize, usize) {
        if let Some(r) = self.p.rows.single.get(&hsv) {
            return *r;
        }
        let pal = house_palette(&self.palettes.unit, hsv);
        // spec: S-0103 claim 5 (a single-shade palette has one shade row)
        let t = light::shade_table(&pal, WHITE, 1, None);
        let base = self.built.scene.rows.len();
        for row in &t {
            self.built.scene.add_row(row565(row));
        }
        self.p.rows.single.insert(hsv, (base, t.len()));
        (base, t.len())
    }

    /// The general palette's shade table (S-0306 Terms): the overlay
    /// palette's file, components ×4, a 53-row untinted table; `None` when
    /// the file is missing or unreadable.
    fn general_row(&mut self) -> Option<(usize, usize)> {
        if let Some(r) = self.p.rows.general {
            return r;
        }
        let r = self
            .assets
            .get(super::combat::OVERLAY_PALETTE)
            .and_then(|b| glasnost_fmt::pal::Palette6::parse(b).ok())
            .map(|p| {
                // spec: S-0306 Terms and S-0103 claim 1 (the components multiplied by 4)
                let pal = glasnost_render::palette::palette_from_file(
                    &p,
                    &glasnost_render::palette::PaletteQuirks::default(),
                );
                // spec: S-0306 Terms and S-0104 claim 6 (a 53-row table built like the anim palette's, never tinted)
                let t = light::untinted_table(&pal);
                let base = self.built.scene.rows.len();
                for row in &t {
                    self.built.scene.add_row(row565(row));
                }
                (base, t.len())
            });
        self.p.rows.general = Some(r);
        r
    }

    /// A target line's items (S-0301 claim 24 f to h); none without the
    /// general palette.
    fn target_line_items(&mut self, a: (i32, i32), b: (i32, i32), phase: bool) -> Vec<Item> {
        let Some((base, rows)) = self.general_row() else {
            self.note(notes::NO_GENERAL_PALETTE);
            return Vec::new();
        };
        // spec: S-0301 claim 24 g (shade row 26 of 53: never lit, tinted or house-coloured)
        let row = base + TARGET_LINE_ROW.min(rows.saturating_sub(1));
        let key = (TARGET_LINE_IMAGE_KEY.to_vec(), 0, 0);
        let image = match self.p.images.get(&key).copied().flatten() {
            Some(i) => i,
            None => {
                let img = IndexedImage::new(1, 1, vec![TARGET_LINE_INDEX]).expect("1 × 1");
                let i = self.built.scene.add_image(img);
                self.p.images.insert(key, Some(i));
                i
            }
        };
        let pixel = |x: i32, y: i32| Item {
            image,
            x,
            y,
            // spec: S-0301 claim 24 h (no depth test and no depth write)
            test: DepthTest::Ignore,
            depth: ItemDepth::Profile {
                profile: Profile::Flat,
                bias: 0,
                companion: None,
            },
            paint: Paint::Row(row),
            group: Group::Effects,
            provisional: false,
            object: None,
            stack: None,
        };
        let mut out: Vec<Item> = dotted_line(a, b, phase)
            .into_iter()
            .map(|(x, y)| pixel(x, y))
            .collect();
        for (ex, ey) in [a, b] {
            // spec: S-0301 claim 24 g (a solid 3 × 3 square whose bottom-right pixel is the end point)
            for dy in 0..TARGET_LINE_SQUARE {
                for dx in 0..TARGET_LINE_SQUARE {
                    out.push(pixel(ex - dx, ey - dy));
                }
            }
        }
        out
    }

    fn live_particle(&mut self, d: &ParticleDraw) -> Vec<Item> {
        match *d {
            ParticleDraw::Sprite {
                idx,
                frame,
                code,
                pos,
            } => self.sprite_particle(idx, frame, code, pos),
            ParticleDraw::Pixel { rgb, pos } => self.pixel_particle(rgb, pos),
        }
    }

    /// A sprite-kind particle (S-0616 claims 24 and 27).
    fn sprite_particle(
        &mut self,
        idx: usize,
        frame: i32,
        code: u8,
        pos: (i32, i32, i32),
    ) -> Vec<Item> {
        // spec: S-0616 claim 23 and S-0007 claim 28 (the particle type's sprite; a sprite kind without one draws nothing)
        let Some(sprite) = self
            .import
            .arts
            .get(&(Kind::Particles, idx))
            .and_then(|a| a.sprite.clone())
        else {
            return Vec::new();
        };
        let n = self.sprite_frames(&sprite).unwrap_or(0);
        // spec: S-0616 claim 25 (our engine draws nothing for a frame number the sprite file does not hold, a negative one included)
        let Some(f) = usize::try_from(frame).ok().filter(|&f| f < n) else {
            return Vec::new();
        };
        // spec: S-0616 claim 24 (the anim palette at brightness 1000 per mille, whatever the cell's light, height or owner; light-alpha still applies)
        let row = self.anim_palette_row(SPRITE_PARTICLE_BRIGHTNESS);
        // spec: S-0616 claim 26, S-0102 claim 32 and F8 (a nonzero code blends; 0 draws opaque)
        let paint = match Translucency::from_code(i32::from(code)) {
            Some(code) => Paint::Translucent(row, code),
            None => Paint::Lit(row),
        };
        // spec: S-0616 claim 27, S-0101 F1 (the screen pixel of its position with its height lift; centred, S-0102 claim 19)
        let p = project_lifted(pos.0, pos.1, pos.2);
        let lz = height_lift(pos.2);
        self.place_sprite(
            &sprite,
            f,
            p,
            // spec: S-0616 claim 27 (reads depth, never writes it)
            DepthTest::Reader,
            // spec: S-0616 claim 27 and Constants (upright profile, bias −15 − L(z), nothing further; no shadow)
            ItemDepth::Profile {
                profile: Profile::Upright,
                bias: SPRITE_PARTICLE_BIAS - lz,
                companion: None,
            },
            paint,
            Group::Effects,
            false,
            None,
        )
        .into_iter()
        .collect()
    }

    /// A pixel-kind particle (S-0616 claim 28): one opaque pixel of its
    /// colour at the screen pixel of its position, a flat depth reader with
    /// bias −50 − L(z).
    fn pixel_particle(&mut self, rgb: [u8; 3], pos: (i32, i32, i32)) -> Vec<Item> {
        let key = (PIXEL_IMAGE_KEY.to_vec(), 0, 0);
        let image = match self.p.images.get(&key).copied().flatten() {
            Some(i) => i,
            None => {
                // One shared 1 × 1 image whose single pixel is not index 0 (S-0104 claim 6: entry 0 is transparent).
                let img = IndexedImage::new(1, 1, vec![1]).expect("1 × 1");
                let i = self.built.scene.add_image(img);
                self.p.images.insert(key, Some(i));
                i
            }
        };
        // spec: S-0616 claim 28, S-0101 F1 (the screen pixel of its position with its height lift)
        let (x, y) = project_lifted(pos.0, pos.1, pos.2);
        let lz = height_lift(pos.2);
        vec![Item {
            image,
            x,
            y,
            // spec: S-0616 claim 28 (drawn only where d − L(z) − 50 is strictly less than the stored depth; never writes depth)
            test: DepthTest::Reader,
            // spec: S-0616 claim 28 and Constants (S-0102 F2's flat profile on one row, bias −50 − L(z), no rounding)
            depth: ItemDepth::Profile {
                profile: Profile::Flat,
                bias: PIXEL_PARTICLE_BIAS - lz,
                companion: None,
            },
            // spec: S-0616 claim 28 (its colour as it is, no palette, tint or cell light; light-alpha scaling; opaque)
            paint: Paint::Pixel(Rgb8 {
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
            }),
            group: Group::Effects,
            provisional: false,
            object: None,
            stack: None,
        }]
    }

    /// The lowest copy's draw point and the stack of a tiled animation
    /// whose sprite is `sprite`, projected at `p` with vertical draw shift
    /// `v` (S-0109 claim 27). A zero-height first image frame gives one copy
    /// at (pₓ, pᵧ + v) and a note naming the sprite.
    fn tiled_placement(
        &mut self,
        sprite: &[u8],
        p: (i32, i32),
        v: i32,
    ) -> ((i32, i32), Option<Stack>) {
        // spec: S-0109 claim 27 (h: the stored rectangle height of image frame 0, not the canvas
        // and not the current frame)
        let h = self
            .assets
            .get(sprite)
            .and_then(|b| Shp::parse(b).ok())
            .and_then(|s| s.frames.first().map(|f| i32::from(f.height)))
            .unwrap_or(0);
        if h <= 0 {
            // spec: S-0109 claim 27 (zero height: at most one copy at (pₓ, pᵧ + v), with a warning)
            self.note(&format!(
                "tiled animation sprite {} has a zero-height first image frame: drawn as one copy (S-0109 claim 27)",
                String::from_utf8_lossy(sprite)
            ));
            return ((p.0, p.1 + v), None);
        }
        let half = h / 2; // spec: S-0109 claim 27 (⌊h/2⌋ for h > 0)
        // spec: S-0109 claim 27 (copy j centred at (pₓ, pᵧ − ⌊h/2⌋ − j·h + v); the count uses the
        // unshifted pᵧ − ⌊h/2⌋)
        (
            (p.0, p.1 - half + v),
            Some(Stack {
                spacing: h,
                base_row: p.1 - half,
            }),
        )
    }

    /// The shade row of a building's slot animation of type `t` (S-0103
    /// claim 8 rows 3 to 5 and its tiled copies): `remap` and `u` are its
    /// object's colour definition with its named palette (S-0103 claim 16)
    /// and its brightness, `cell` the animation's cell.
    fn anim_row(&mut self, t: &slots::AnimType, remap: Remap, u: i32, cell: CellXY) -> Shade {
        // The brightness of the selected row.
        let b = if t.normal_light {
            // spec: S-0103 claim 8 (the normal-light flag: brightness 1000, tiled copies included)
            NEUTRAL_BRIGHTNESS
        } else if t.cell_drawer {
            // spec: S-0103 claim 8 row 3 (a building's own animation: the building's brightness U)
            u
        } else {
            // spec: S-0103 claim 8 rows 4 and 5, S-0104 claim 13 (its cell's unscaled ground level)
            self.p
                .lights
                .get(cell)
                .map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground)
        };
        if t.cell_drawer && !t.tiled {
            // spec: S-0103 claim 8 row 3 (the owner's 53-shade palette); claim 16 (the object's named
            // palette when it has one)
            return self.house_row(remap, b);
        }
        if t.alt_palette && !t.tiled {
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
        // spec: S-0103 claim 8 row 5, and its tiled copies (the anim palette whatever row the type
        // would select, at that row's brightness)
        self.anim_palette_row(b)
    }

    /// The anim palette's untinted shade table at brightness `b` (S-0103
    /// claim 8 row 5, S-0104 claim 6), added to the scene once.
    fn anim_palette_row(&mut self, b: i32) -> Shade {
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
        anim: u32,
        anchor: (i32, i32),
        lz: i32,
        brightness: i32,
        mask: u16,
        remap: Remap,
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
        // spec: S-0104 claim 13 (every part of an ordinary building: U of its top-left cell);
        // S-0102 claim 34 (a protected building's parts: the adjusted brightness and its mask);
        // S-0103 claim 16 (a building's voxel parts take its named palette)
        let row = self.house_row(remap, brightness).with_mask(mask);
        let model_of = |b: &mut Self, part: &'static str| {
            let file = art.voxels.get(part)?.clone();
            b.voxel_model(&file).map(|m| (m, file, part))
        };
        let turret = model_of(self, "turret");
        let barrel = model_of(self, "barrel");
        let mut items = Vec::new();
        // spec: S-0105 claim 16 (the animation-frame counter: 0 before any attack); S-0609 claim 14 (a gattling building's counter advances while it spins)
        let counter = anim;
        let parts = match turret {
            Some(t) => {
                let pose = voxel::building_turret_parts(facing, offset, elevation, &q);
                let turret_frame = voxel::building_turret_frame(counter, t.0.frame_count() as u32);
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
                    voxel::barrel_only_frame(counter, b.0.frame_count() as u32)
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
            let Some((m, file, role)) = model else {
                continue;
            };
            if let Some(part) = self.cached_part(k, idx, key, |b| {
                let img =
                    b.p.canvas
                        .draw_part(&m, frame, &draw, vpl.as_ref(), depth_buffered, &q);
                b.limit_checked(&file, role, key, img)
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
            // spec: S-0105 claim 16 (no attack before the scene: counter 0)
            anim: 0,
            infantry,
            object,
            // spec: S-0402 claim 5a (shown only while a harvest is loading; no tick has run)
            harvesting: None,
            shadow: None,
            // spec: S-0105 claim 12 (no tick has run: the spawned aircraft are docked, the main body shows)
            alternative: false,
            attitude: Attitude::None,
            disabled: false,
            // spec: S-0107 claim 19 (a map unit stands on its surface)
            above: 0,
            // A map unit is not protected before any tick (S-0513 claim 25).
            protection: None,
            // spec: S-0107 claim 19 (a map unit stands on its surface: h = 0, no generic shadow)
            generic_shadow: None,
            // A map unit is not berserk before any tick (S-0610 claim 4).
            berserk: None,
            // spec: S-0105 claim 43 and S3 (the index a vehicle is created with: no tick has run)
            turret_index: creation_turret_index(self.rules, self.import, kind, i),
            walk: Default::default(),
            // A map unit stands on its surface (S-0107 claim 19), on no deck.
            deck: false,
        };
        Some(self.mobile(&m))
    }

    /// The terrain surface height at a position, from the map's cells
    /// (S-0106 claim 2, F1 with its default switch).
    fn terrain_height(&self, x: i32, y: i32) -> i32 {
        let c = CellXY::new(x.div_euclid(256), y.div_euclid(256)); // spec: S-0204 claim 2 (the cell holding a position)
        self.map.cells.get(c).map_or(0, |cell| {
            glasnost_sim::surface::surface_height(
                i32::from(cell.level),
                cell.slope,
                x.rem_euclid(256), // spec: S-0204 claim 2 (the position within the cell)
                y.rem_euclid(256),
                true, // spec: S-0106 F1 (quirks.negative_level_floor_raised, default on)
            )
        })
    }

    /// The unit depth adjustment *U* = −L(z) + s + F (S-0102 claim 14) for a
    /// unit of `kind`/`idx` standing on cell `c` at world height `z`, on a
    /// high bridge's deck when `deck`. Only the cliff fudge can be judged:
    /// bridges, columns and tunnels need specs not written yet, so their
    /// factors are 0.
    fn unit_depth(&self, kind: Kind, idx: usize, c: CellXY, z: i32, deck: bool) -> i32 {
        let level = |dx: i32| {
            self.map
                .cells
                .get(CellXY::new(c.x + dx, c.y + dx))
                .map(|cell| i32::from(cell.level))
        };
        let own = level(0).unwrap_or(0);
        // spec: S-0102 claim 14 (a cell is high when more than 3 levels above the unit's cell)
        let high = |dx: i32| level(dx).is_some_and(|l| l - own > CLIFF_LEVELS);
        // spec: S-0102 claim 14 (cliff factor, quirks.cliff_fudge_far_cliff_weaker; 0 only for a unit on a bridge deck; a unit on a low bridge or under a high bridge takes the other rows, as on open ground)
        let factor = if deck {
            0
        } else {
            cliff_factor(high(1), high(2), &self.p.view_quirks)
        };
        let fudge = rule_int(self.import, kind, idx, b"ZFudgeCliff", DEFAULT_CLIFF_FUDGE);
        // spec: S-0102 claim 14 (U = −L(z) + s + F, F the largest fudge)
        -height_lift(z) + SITUATIONAL + (factor * fudge).max(0)
    }

    /// The protection colour masks of the import's selectors (S-0102 claim
    /// 34, F9; S-0513 claim 29a).
    fn protection_masks(&self) -> super::protection::Masks {
        super::protection::Masks::read(self.import)
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
        let u = self.unit_depth(m.kind, m.idx, cell, z, m.deck);
        let (hsv, _) = self.owner_colour(&m.owner);
        let named = self.named_palette(m.kind, m.idx);
        let levels = self.p.lights.get(cell).map_or(
            light::CellLevels {
                unscaled_ground: NEUTRAL_BRIGHTNESS,
                ..Default::default()
            },
            light::CellLevels::from,
        );
        let extra = light::ExtraLight {
            unit: self.extra_light(b"ExtraUnitLight"),
            infantry: self.extra_light(b"ExtraInfantryLight"),
            aircraft: self.extra_light(b"ExtraAircraftLight"),
        };
        // spec: S-0104 claim 13 (the height above the surface, from the deck on a bridge deck)
        let height = m.above;
        // spec: S-0104 Terms and claim 13 (the shadowed-cell rows: the cell holding the position, as the live game marks it)
        let shadowed_cell = self.p.shadowed_cells.contains(&cell);
        let object = match m.kind {
            Kind::Infantry => light::LitObject::Infantry {
                on_bridge_deck: m.deck,
                shadowed_cell,
                height,
            },
            Kind::Aircraft => light::LitObject::Aircraft { height },
            _ => light::LitObject::Vehicle {
                on_bridge_deck: m.deck,
                shadowed_cell,
                height,
            },
        };
        let (lambda, lambda_air) = self.p.object_levels;
        // spec: S-0104 claim 13, Formulas "Object brightness" (vehicles, infantry and aircraft)
        let lit = light::object_brightness(
            object,
            levels,
            lambda,
            lambda_air,
            extra,
            &LightQuirks::default(),
        );
        let items = if m.kind == Kind::Infantry {
            // spec: S-0610 claim 7 (a berserk infantry sprite body takes the berserk mask, unless the
            // centre of its cell is shrouded for the viewer; S-0102 F9's union after the shade row)
            let mask = self.protection_masks().berserk_of(
                m.berserk.is_some(),
                super::protection::MobileKind::Infantry,
                m.berserk == Some(true),
            );
            // spec: S-0103 claim 16 (an infantry type's named palette, at its owner's colour definition)
            let row = self.house_row(Remap { hsv, named }, lit).with_mask(mask);
            self.infantry_items(m, point, u, row)
        } else {
            let voxel = self
                .import
                .arts
                .get(&(m.kind, m.idx))
                .is_some_and(|a| a.voxels.contains_key("body"));
            let b = if m.disabled && m.kind == Kind::Vehicles && voxel {
                // spec: S-0104 claim 13 (Deactivated voxel vehicles: trunc(b / 2) ‰, before claim 15's shade-row mapping)
                lit / 2
            } else {
                lit
            };
            // spec: S-0513 F9, S-0102 claim 34 (protection changes the ordinary brightness before the shade-row selection)
            let b = super::protection::brightness(m.protection, b);
            // spec: S-0102 claim 34 (the vehicle's body and voxel turret and barrel share the mask of its row)
            let masks = self.protection_masks();
            let curtain = match m.protection {
                Some(_) => masks.mobile(m.protection, m.kind == Kind::Vehicles, voxel),
                None => 0,
            };
            let kind = match (m.kind, voxel) {
                (Kind::Vehicles, true) => super::protection::MobileKind::VoxelVehicle,
                (Kind::Vehicles, false) => super::protection::MobileKind::SpriteVehicle,
                _ => super::protection::MobileKind::Aircraft,
            };
            // spec: S-0610 claim 7 (a berserk vehicle drawn as voxels: body and voxel turret/barrel
            // take the berserk mask, united with the curtain mask when both apply; a sprite vehicle
            // and an aircraft take none)
            let mask = curtain | masks.berserk_of(m.berserk.is_some(), kind, false);
            let named = match m.kind {
                // spec: S-0103 claim 16 (a vehicle drawn as a voxel model or as a sprite with a
                // turret keeps its owner's ordinary palette; one drawn as a sprite without a turret
                // takes its named palette)
                Kind::Vehicles
                    if voxel || rule_bool(self.import, m.kind, m.idx, b"Turret", false) =>
                {
                    None
                }
                // spec: S-0103 claim 16 (aircraft, their voxel parts included)
                _ => named,
            };
            let row = self.house_row(Remap { hsv, named }, b).with_mask(mask);
            let body = self.vehicle_items(m, point, u, row);
            // spec: S-0402 claim 5a (composited before both the vehicle's shadow and its body)
            let mut items: Vec<Item> = m
                .harvesting
                .and_then(|h| self.harvesting_item(m, h, u, lit))
                .into_iter()
                .collect();
            items.extend(body);
            items
        };
        GroundEntry {
            // spec: S-0102 claim 5 (sort key X + Y of the position; height plays no part)
            key: i64::from(x) + i64::from(y),
            items,
            object: Some(m.object),
        }
    }

    /// A loading vehicle harvester's harvesting image (S-0402 claim 5a,
    /// F8): the fixed harvesting sprite's frame centred on the projected
    /// anchor, drawn with the anim palette and no house-colour remap at the
    /// vehicle's brightness `light`, depth-tested flat at the vehicle's
    /// *U* − 2 without writing depth (S-0102 claim 13). A missing file or
    /// image frame draws nothing.
    fn harvesting_item(
        &mut self,
        m: &Mobile,
        h: super::harvesting::Harvesting,
        u: i32,
        light: i32,
    ) -> Option<Item> {
        // spec: S-0402 claim 5a; S-0102 Import mapping (the fixed game asset OREGATH.SHP, by its file name)
        self.assets.get(super::harvesting::SPRITE_FILE)?;
        // spec: S-0402 F8 (project the anchor by S-0101 claim 1 and centre the image there); S-0102 claim 13 (the canvas centred at the displaced point)
        let at = project_lifted(h.anchor.0, h.anchor.1, h.anchor.2);
        // spec: S-0402 claim 5a (the animation palette, no house-colour remap, at the vehicle's brightness of S-0104 claim 13); S-0103 claim 8 row 5
        let row = self.anim_palette_row(light);
        self.place_sprite(
            super::harvesting::SPRITE_FILE,
            h.frame,
            at,
            // spec: S-0102 claim 13 (vehicle harvesting sprite: passes only where strictly nearer, never writes depth)
            DepthTest::Reader,
            // spec: S-0102 claim 13 (vehicle harvesting sprite: flat, the vehicle's U − 2, no companion)
            ItemDepth::Profile {
                profile: Profile::Flat,
                bias: u - 2,
                companion: None,
            },
            Paint::Lit(row),
            Group::Vehicles,
            false,
            Some(m.object),
        )
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
        // spec: S-0107 claim 19 (c, d) and S-0102 claim 20 (the generic shadow when h is not 0 and
        // hₛ > 0: image frame 1 of the fixed generic infantry-shadow sprite, a darkening, centred
        // L(hₛ) rows below the draw point, right before the body; a file that cannot be found or
        // holds fewer than two image frames draws nothing, without a warning)
        if let Some(hs) = m.generic_shadow
            && self
                .sprite_frames(GENERIC_SHADOW_FILE)
                .is_some_and(|n| n > GENERIC_SHADOW_FRAME)
            && let Some(item) = self.place_sprite(
                GENERIC_SHADOW_FILE,
                GENERIC_SHADOW_FRAME,
                (point.0, point.1 + height_lift(hs)),
                DepthTest::Reader,
                // spec: S-0102 claim 13 (generic shadow of an infantry unit with hₛ > 0: −5 − L(z − hₛ), flat, no depth write)
                ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias: GENERIC_SHADOW_BIAS - height_lift(m.pos.2 - hs),
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
        // spec: S-0102 claims 13 and 14 (infantry body, h = 0: U − 12; h not 0: −L(z) − 12, U does not apply; upright, reader)
        let bias = if m.above == 0 { u - 12 } else { -lz - 12 };
        if let Some(item) = self.place_sprite(
            &sprite,
            frame,
            point,
            DepthTest::Reader,
            ItemDepth::Profile {
                profile: Profile::Upright,
                bias,
                companion: None,
            },
            Paint::Lit(row),
            Group::Infantry,
            false,
            Some(m.object),
        ) {
            items.push(item);
        }
        // spec: S-0107 claim 19 (height above the surface 0: shadow frame + ⌊L/2⌋ at the same position, unless
        // suppressed; any other height: no body-sprite shadow, the generic shadow above instead);
        // S-0102 claims 20 and 28 (drawn right after the body; it darkens the body's own pixels)
        if m.above == 0
            && !rule_bool(self.import, m.kind, m.idx, b"NoShadow", false)
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
        let Some(art) = self.type_art((m.kind, m.idx)) else {
            return Vec::new();
        };
        let group = if m.kind == Kind::Aircraft {
            Group::Aircraft
        } else {
            Group::Vehicles
        };
        if art.voxels.contains_key("body") {
            self.vehicle_voxels(m, art, point, u, row, group)
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
        let d = sprite_vehicle_facing_part(facings, m.body, &self.p.view_quirks);
        // spec: S-0102 Import mapping (IdleRate absent: 0)
        let idle_rate = rule_int(self.import, k, i, b"IdleRate", 0);
        // spec: S-0102 claim 22 (frame table: nothing fires or dies; the standing-pose state is on
        // from creation until a movement step starts and again once it ends, so the row d never
        // applies to a vehicle that is not part-way through a step)
        let frame = sprite_vehicle_frame(
            SpriteVehicleRows {
                walk_start,
                walk_frames,
                stand_start,
                standing_frames,
            },
            d,
            m.walk,
            idle_rate,
        );
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
        // spec: S-0102 claims 13 and 14 (a vehicle drawn as a sprite, h = 0: U − 2; h not 0: −L(z) − 2, U does not apply)
        let body_bias = if m.above == 0 { u - 2 } else { -lz - 2 };
        let body = |b: &mut Self, f: usize| {
            // spec: S-0102 claim 13 (a vehicle drawn as a sprite: upright, reader); S-0105 claim 12
            // (Depth: only without the turret flag)
            b.place_sprite(
                sprite,
                f,
                point,
                DepthTest::Reader,
                ItemDepth::Profile {
                    profile: Profile::Upright,
                    bias: body_bias,
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
            // spec: S-0102 claim 22 (a turret vehicle: its shadow, U − 2, lies under the combined
            // image and never darkens it, so it goes first); S-0105 claim 12 (One image: the
            // vehicle's own shadow is drawn before the composed image and keeps its own row)
            // spec: S-0102 claim 22 (in our engine under claim 21's three conditions: only while the height above the surface is 0, at U − 2 there, never with the no-shadow or either visceroid flag)
            if m.above == 0 {
                items.extend(shadow_item(self, u - 2));
            }
            // A sprite frame as a part placed relative to the draw point (S-0102 F6).
            let sprite_part = |b: &mut Self, f: usize| {
                b.sprite_frame(sprite, f)
                    .map(|(id, (fx, fy), (cw, ch))| (id, [-cw / 2 + fx, -ch / 2 + fy]))
            };
            let body_part = sprite_part(self, frame);
            let barrel = self.sprite_body_barrel(m);
            // spec: S-0102 claim 22 (turret frame ((t₃₂ + 4) mod 32) + 8·F_w)
            let t = (n_step(m.turret, 32) as i32 + 4) % 32 + 8 * walk_frames;
            let turret_part = usize::try_from(t).ok().and_then(|t| sprite_part(self, t));
            let parts = sprite_turret_parts(body_part, barrel, turret_part, m.turret);
            items.extend(self.composed_item(&parts, point, u, row, m.object));
        } else {
            items.extend(body(self, frame));
            // spec: S-0102 claims 21 and 28 (the own shadow right after the body: −L(z) − 4, flat); claim 13 (only while h = 0)
            if m.above == 0 {
                items.extend(shadow_item(self, -lz - 4));
            }
        }
        items
    }

    /// The single voxel barrel of a sprite-body vehicle with the turret
    /// flag (S-0105 claim 12, *Sprite body*; S-0007 claim 19), as a part
    /// placed relative to the draw point, or `None` without a barrel part.
    /// The voxel turret part is never drawn. It is placed by claim 12's
    /// *Barrel pose* and drawn in HVA frame 0 whatever the counters and in
    /// both states of `quirks.barrel_frame_zero` (claim 12, *Frame*); it
    /// joins the vehicle's composed image ([`Self::composed_item`]) and
    /// casts no shadow of its own (claim 12, *No barrel shadow*).
    fn sprite_body_barrel(&mut self, m: &Mobile) -> Option<PlacedPart> {
        let q = VoxelQuirks::default();
        let (kind, idx) = (m.kind, m.idx);
        let art = self.type_art((kind, idx))?;
        let file = art.voxels.get("barrel")?.clone();
        // spec: S-0105 Import mapping (TurretOffset, Art [image], default 0); S-0313 claim 11
        let offset = self.sim_turret_offset(kind, idx).unwrap_or_else(|| {
            self.art_value(kind, idx, b"TurretOffset")
                .and_then(value::integer)
                .unwrap_or(0)
        });
        let name = self.import.outcome.registries[&kind][idx].name.clone();
        // spec: S-0105 Import mapping (UseBuffer is read from the art section of the type's own name)
        let depth_buffered = self
            .p
            .art
            .value(&name, b"UseBuffer")
            .and_then(value::boolean)
            .unwrap_or(false);
        // spec: S-0105 claim 12 (Frame, Q-1885 question 4: always HVA frame 0, in both states of
        // quirks.barrel_frame_zero, since no voxel turret is drawn whose frame it could take)
        let barrel_frame = 0;
        // spec: S-0105 claim 43 (Any body: the barrel key holds the selected turret index of a
        // multi-turret type); claim 26 (barrel key: the turret's key plus the 32-step elevation)
        let index = if multi_turret(self.import, kind, idx) {
            m.turret_index.and_then(|k| u8::try_from(k).ok())
        } else {
            None
        };
        // spec: S-0105 claim 26 (turret key: flat ground, the alternative-model state off, and the
        // frame drawn, which claim 12's Frame fixes at 0)
        let turret_key = voxel::vehicle_turret_key(
            voxel::RAMP_FLAT,
            m.body,
            m.turret,
            offset,
            index,
            false,
            barrel_frame,
        );
        let key = voxel::barrel_key(turret_key, m.elevation);
        let (body_facing, turret_facing, elevation) = (m.body, m.turret, m.elevation);
        self.cached_part(kind, idx, key, |b| {
            let model = b.voxel_model(&file)?;
            let vpl = b.vpl();
            // spec: S-0105 claim 12 (Barrel pose: claim 8's transform with d = trunc(δ / 8), claim
            // 4's relative turn, claim 6's elevation about claim 10's axis; the invisible slot-0
            // projectile's distorted elevation is an original glitch fixed without a switch)
            let pose =
                voxel::sprite_body_barrel_pose(body_facing, turret_facing, offset, elevation, &q);
            let img = b.p.canvas.draw_part(
                &model,
                barrel_frame,
                &pose,
                vpl.as_ref(),
                depth_buffered,
                &q,
            );
            b.limit_checked(&file, "barrel", key, img)
        })
    }

    /// The composed image of a sprite-body vehicle with the turret flag
    /// (S-0105 claim 12, *One image* and *Depth*), as one item at the draw
    /// point `point`: `parts` (scene images placed relative to the draw
    /// point) painted in order into the smallest rectangle holding them
    /// all, later opaque pixels winning, with no depth test between them.
    /// The composed image takes S-0102 claim 13's row for a vehicle drawn
    /// as a voxel model, *U*, upright, a reader, at every height, its
    /// anchor row the composed rectangle's last row (S-0102 F2). Each
    /// distinct list of parts is composed once and kept, as the parts are.
    fn composed_item(
        &mut self,
        parts: &[PlacedPart],
        point: (i32, i32),
        u: i32,
        row: Shade,
        object: u32,
    ) -> Option<Item> {
        let key: Vec<(u32, i32, i32)> = parts.iter().map(|&(i, [x, y])| (i, x, y)).collect();
        let placed = match self.p.composed.get(&key) {
            Some(p) => *p,
            None => {
                let p = compose_parts(&self.built.scene, parts)
                    .and_then(|img| add_part_image(&mut self.built.scene, &img));
                self.p.composed.insert(key, p);
                p
            }
        }?;
        // spec: S-0105 claim 12 (Depth: S-0102 claim 13's voxel-vehicle row, U with no further
        // constant, upright, a reader, at every height; the colour row of the body)
        // design: S-0105 claim 12 (Split of a tall image) is never made: every row keeps U,
        // upright; S-0102 owns the split and its second condition is open (Open questions)
        Some(Self::part_item(
            placed,
            point,
            u,
            Paint::Lit(row),
            Group::Vehicles,
            object,
        ))
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

    /// A parsed voxel model without a motion file, parsed once (a voxel
    /// animation's own model, S-0007 claim 30).
    fn bare_voxel_model(&mut self, file: &[u8]) -> Option<std::rc::Rc<voxel::VoxelModel>> {
        let mut key = upper(file);
        // Our cache key: kept apart from the same file loaded with its motion file.
        key.extend_from_slice(b"|bare");
        if let Some(m) = self.p.models.get(&key) {
            return m.clone();
        }
        let m = Vxl::parse(self.assets.get(file)?).ok().map(|vxl| {
            std::rc::Rc::new(voxel::VoxelModel::new(&vxl, None, &VoxelQuirks::default()))
        });
        self.p.models.insert(key, m.clone());
        m
    }

    /// `img`, drawn from the model `file` in the role `role` at the image
    /// key `key`, after warning once per model when its copy rectangle is
    /// larger than the retained limit (S-0105 claim 22, Import mapping).
    fn limit_checked(
        &mut self,
        file: &[u8],
        role: &str,
        key: impl std::fmt::Debug,
        img: Option<voxel::PartImage>,
    ) -> Option<voxel::PartImage> {
        if let Some(i) = &img
            && i.exceeds_limit()
            // spec: S-0105 claim 22, Import mapping (one warning per model, not per pose, appearance or image role)
            && self.p.oversized.insert(upper(file))
        {
            let note = notes::voxel_too_large(file, role, &format!("{key:?}"), i.copy_size);
            self.note(&note);
        }
        img
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

    /// The turret displacement δ that the simulation's fire points use for
    /// type `idx` of `kind` (S-0313 claim 11, F2), when the typed rules are
    /// present and name the type.
    fn sim_turret_offset(&self, kind: Kind, idx: usize) -> Option<i32> {
        let r = self.rules?;
        let sim = match kind {
            Kind::Vehicles => SimKind::Vehicle,
            Kind::Aircraft => SimKind::Aircraft,
            _ => return None,
        };
        let name = &self.import.outcome.registries.get(&kind)?.get(idx)?.name;
        let t = glasnost_sim::objects::type_index(r, sim, name)?;
        r.unit(sim, t).map(|u| u.combat.turret_offset)
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
            stack: None,
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
        self.cached_part_in(kind, idx, None, key, draw)
    }

    /// [`Self::cached_part`] in the type's cache for `rocked`: the settled
    /// one for `None`, else that pose's own (S-0504 claim 16), whose images
    /// are transient (design: render (#104)).
    fn cached_part_in(
        &mut self,
        kind: Kind,
        idx: usize,
        rocked: Option<Pose>,
        key: voxel::ImageKey,
        draw: impl FnOnce(&mut Self) -> Option<voxel::PartImage>,
    ) -> Option<(u32, [i32; 2])> {
        if let Some(p) = self
            .image_cache(kind, idx, rocked)
            .and_then(|c| c.get(&key))
        {
            return *p;
        }
        let part = draw(self).and_then(|img| match rocked {
            None => self.add_part(&img),
            Some(_) => add_transient_image(&mut self.built.scene, &mut self.p.pool, &img),
        });
        self.image_cache_mut(kind, idx, rocked).insert(key, part);
        part
    }

    /// The image cache of type `(kind, idx)` for `rocked` (`None`: settled).
    fn image_cache(
        &self,
        kind: Kind,
        idx: usize,
        rocked: Option<Pose>,
    ) -> Option<&voxel::ImageCache<Option<PlacedPart>>> {
        match rocked {
            None => self.p.voxels.get(&(kind, idx)),
            Some(pose) => self.p.rocked.get(&(kind, idx, pose)).map(|(c, _)| c),
        }
    }

    /// [`Self::image_cache`], made when missing; a pose's own is marked
    /// used in this redraw (design: render (#104)).
    fn image_cache_mut(
        &mut self,
        kind: Kind,
        idx: usize,
        rocked: Option<Pose>,
    ) -> &mut voxel::ImageCache<Option<PlacedPart>> {
        match rocked {
            None => self.p.voxels.entry((kind, idx)).or_default(),
            Some(pose) => {
                let now = self.p.pool.now;
                let entry = self.p.rocked.entry((kind, idx, pose)).or_default();
                entry.1 = now;
                &mut entry.0
            }
        }
    }

    /// How a body is posed: its heading alone, a rocket's nose pitch, a
    /// flying body's pitch and bank (S-0105 claims 41, 42), or a sinking
    /// vehicle's lean (S-0504 claims 15, 16).
    fn tilt(&self, m: &Mobile, q: &VoxelQuirks) -> Tilt {
        match m.attitude {
            Attitude::None => Tilt::Plain,
            Attitude::Sinking { pitch, roll, drawn } => Tilt::Rocked {
                pitch: f32::from_bits(pitch),
                roll: f32::from_bits(roll),
                drawn,
                sinking: true,
            },
            Attitude::Rocking { pitch, roll, drawn } => Tilt::Rocked {
                pitch: f32::from_bits(pitch),
                roll: f32::from_bits(roll),
                drawn,
                sinking: false,
            },
            Attitude::Rocket {
                pitch,
                initial,
                final_,
            } => {
                let pitch = f32::from_bits(pitch);
                let class = voxel::rocket_pitch_class(
                    pitch,
                    f32::from_bits(initial),
                    f32::from_bits(final_),
                    q,
                );
                Tilt::Rocket { pitch, class }
            }
            Attitude::Flying {
                height,
                speed,
                turn,
                crashing,
                tumble: (pitch, roll),
            } => {
                let (kind, idx) = (m.kind, m.idx);
                let pitch_angle =
                    rule_angle(self.import, kind, idx, b"PitchAngle", PITCH_ANGLE_DEFAULT);
                let bank_angle =
                    rule_angle(self.import, kind, idx, b"RollAngle", ROLL_ANGLE_DEFAULT);
                let pitch_speed =
                    rule_real(self.import, kind, idx, b"PitchSpeed").unwrap_or(PITCH_SPEED_DEFAULT);
                if crashing {
                    // spec: S-0113 claim 20 (at zero or negative signed height a crashing body is turned to its heading only: no tumble, no α and no β)
                    if height <= 0 {
                        return Tilt::Flying(voxel::FlightAttitude::LEVEL);
                    }
                    // spec: S-0113 claims 18 and 20 (the flying locomotor draws the tumble only while the aircraft is crashing at strictly positive signed height; ψ′ takes α under S-0105 claim 42's speed test, the bank never)
                    return Tilt::Tumbling {
                        pitch: voxel::crash_pitch(
                            f32::from_bits(pitch),
                            f64::from_bits(speed),
                            pitch_speed,
                            pitch_angle,
                        ),
                        roll: f32::from_bits(roll),
                    };
                }
                Tilt::Flying(voxel::flight_attitude(
                    height,
                    f64::from_bits(speed),
                    turn,
                    pitch_speed,
                    pitch_angle,
                    bank_angle,
                ))
            }
        }
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
        let Some(main_file) = art.voxels.get("body").cloned() else {
            return Vec::new();
        };
        // spec: S-0105 claim 12 (the alternative body replaces the main body while no spawned
        // aircraft is docked, only for a type with the option and an alternative body file; the
        // main body otherwise, and the shadow is the main body's, claim 29)
        let alt_drawn = m.alternative
            && kind == Kind::Vehicles
            && rule_bool(self.import, kind, idx, b"NoSpawnAlt", false)
            && art.voxels.contains_key("alternate_body");
        let body_file = if alt_drawn {
            art.voxels["alternate_body"].clone()
        } else {
            main_file.clone()
        };
        let art_int = |b: &Self, key: &[u8], d: i32| {
            b.art_value(kind, idx, key)
                .and_then(value::integer)
                .unwrap_or(d)
        };
        // spec: S-0105 Import mapping (TurretOffset, Art [image], default 0); S-0313 claim 11 (the
        // turret is drawn with the same displacement δ that F2 uses for the fire point: the
        // simulation's imported value when the typed rules name the type, else the art read)
        let offset = self
            .sim_turret_offset(kind, idx)
            .unwrap_or_else(|| art_int(self, b"TurretOffset", 0));
        let name = self.import.outcome.registries[&kind][idx].name.clone();
        // spec: S-0105 Import mapping (UseBuffer is read from the art section of the type's own name)
        let depth_buffered = self
            .p
            .art
            .value(&name, b"UseBuffer")
            .and_then(value::boolean)
            .unwrap_or(false);
        let (body_raw, turret_raw, elevation) = (m.body, m.turret, m.elevation);
        let tilt = self.tilt(m, &q);
        // spec: S-0105 Terms and claim 3 (flight drawing heading: a flying-locomotor aircraft's body and shadow take its aiming facing, a rocket and every other body its body facing)
        let heading = if matches!(tilt, Tilt::Flying(_) | Tilt::Tumbling { .. }) {
            turret_raw
        } else {
            body_raw
        };
        // spec: S-0504 claim 16 and F5 (a sinking or rocking body whose locomotor draws rocking is drawn with its pitch and roll; F5's size values from the first section of the body's main model, also while the alternative body is drawn); S-0106 claim 9
        let rocking = match tilt {
            Tilt::Rocked {
                pitch,
                roll,
                drawn: true,
                ..
            } => self
                .voxel_model(&main_file)
                .and_then(|b| b.sections.first().map(|s| s.size))
                .map(|size| voxel::Rocking::new(pitch, roll, size))
                .filter(voxel::Rocking::rocks),
            _ => None,
        };
        // spec: S-0504 claim 16 (a sinking vehicle drawn as a voxel model draws no shadow, from the first drawn frame after sinking begins)
        let sinking = matches!(tilt, Tilt::Rocked { sinking: true, .. });
        let pose = move || match tilt {
            Tilt::Plain => voxel::vehicle_parts(body_raw, turret_raw, offset, elevation, &q),
            // spec: S-0504 claim 16 and F5 (D for any rocking; T and B are built on it, S-0105 claim 8, so the turret and barrel lean with the body); a body that does not rock is drawn settled
            // design: F5 for the hovering and jumpjet locomotors too (S-0504 Open questions leave their order and lift open; theirs is drawn in the driving order, which matters now that rocking rolls a body); no clipping below a fixed row (also open)
            Tilt::Rocked { .. } => match rocking {
                Some(r) => {
                    voxel::vehicle_parts_rocking(body_raw, turret_raw, offset, elevation, &r, &q)
                }
                None => voxel::vehicle_parts(body_raw, turret_raw, offset, elevation, &q),
            },
            // spec: S-0105 claim 41 (V · Rz(θ_body) · Ry(−p), the shadow unpitched); claim 45 (an aircraft draws no turret or barrel, so only the body and shadow are posed)
            Tilt::Rocket { pitch, .. } => voxel::rocket_body_pose(body_raw, pitch, &q),
            // spec: S-0105 claim 42 (V · Rz(θ_flight) · Ry(α) · Rx(β), the shadow the heading alone); claim 45
            Tilt::Flying(a) => voxel::flying_body_pose(heading, &a, &q),
            // spec: S-0113 claim 20 (a crashing flying body at strictly positive signed height: V · Rz(θ_flight) · Rx(φ) · Ry(ψ′), whatever the size of its angles; the shadow on the heading alone); S-0105 claim 45
            Tilt::Tumbling { pitch, roll } => voxel::crashing_body_pose(heading, roll, pitch, &q),
        };
        // spec: S-0105 claim 26 and S-0106 claim 13 (a rocking body, turret and barrel are drawn fresh: kept apart under their exact angles)
        let list = match tilt {
            // spec: S-0113 claim 20 (a crashing body at positive height is drawn fresh, never reused, with the rotations of S-0105 F1): the drawn transform depends on ψ′ and φ only through their F1 table steps, so the images are kept apart under those steps, as a rocket's fresh pitch is (S-0105 claim 41); the pixels are a fresh draw's
            Tilt::Tumbling { pitch, roll } => Some(Pose::Tumble(
                voxel::table_step(f64::from(pitch), &q),
                voxel::table_step(f64::from(roll), &q),
            )),
            // spec: S-0105 claim 41 (a pitch of none of the three classes is drawn fresh and not retained); design: render (#104): its images are transient, kept apart under the table step that fixes them
            Tilt::Rocket {
                class: voxel::RocketPitch::Fresh(step),
                ..
            } => Some(Pose::Rocket(step)),
            _ => rocking.map(|r| Pose::Rocked(r.pitch.to_bits(), r.roll.to_bits())),
        };
        if list.is_some() {
            // design: render (#104): the pose's images are used in this redraw
            self.image_cache_mut(kind, idx, list);
        }
        let object = m.object;
        // spec: S-0105 claim 40 (an aircraft's shadow draw point is the projection of its horizontal
        // position at the terrain surface height there, 60 pixels higher over a qualifying high
        // bridge, whatever its flight height; it is not the body's lifted point).
        // spec: S-0105 claim 44 Locomotors (our engine: claim 40's P itself for a vehicle type with the hunter-seeker property, a corrected presentation glitch without a switch)
        let hunter_seeker =
            kind == Kind::Vehicles && rule_bool(self.import, kind, idx, b"HunterSeeker", false);
        let shadow_point = if kind == Kind::Aircraft || hunter_seeker {
            let anchor = m.shadow.unwrap_or_else(|| ShadowAnchor {
                ground: self.terrain_height(m.pos.0, m.pos.1),
                deck: false,
            });
            let (sx, sy) = project_lifted(m.pos.0, m.pos.1, anchor.ground);
            (sx, sy - if anchor.deck { SHADOW_DECK_RAISE } else { 0 })
        } else {
            // spec: S-0105 claim 44 (B moved down by L(a); the flying locomotor too, our engine's correction)
            vehicle_shadow_point(point, m.above)
        };
        let mut items = Vec::new();
        // spec: S-0105 claims 29 to 31 (the main body's shadow section in frame 0; skipped with the no-shadow option)
        let shadow =
            (!sinking && !rule_bool(self.import, kind, idx, b"NoShadow", false)).then(|| {
                let section = art_int(self, b"ShadowIndex", 0).max(0) as usize;
                // spec: S-0105 Import mapping (ConsideredAircraft absent: yes for an aircraft type, no for every other type)
                let aircraft_like = rule_bool(
                    self.import,
                    kind,
                    idx,
                    b"ConsideredAircraft",
                    kind == Kind::Aircraft,
                );
                (section, aircraft_like)
            });
        // spec: S-0105 claim 31 (shadow key: ramp, 32-step body facing; bodies stand on flat ground), claims 41 and 42 (the 32-step flight drawing heading for a flying-locomotor aircraft, no pitch or bank class)
        let shadow_key = voxel::shadow_key(voxel::RAMP_FLAT, heading);
        let shadow_file = main_file.clone();
        let draw_shadow = move |b: &mut Self, (section, aircraft_like): (usize, bool)| {
            let body = b.voxel_model(&shadow_file)?;
            // spec: S-0105 claim 32 (a shadow never enables depth mode, whatever the type's option)
            let img =
                b.p.canvas
                    .draw_shadow(&body, section, &pose().shadow, 0, aircraft_like, &q);
            b.limit_checked(&shadow_file, "shadow", shadow_key, img)
        };
        // spec: S-0113 claim 20 (a crashing body's shadow follows only the heading and the slope shape, and its image key is S-0105 claim 42's), so it is the level flying body's image, not one kept apart under the angles.
        // spec: S-0105 claim 41 (a rocket's shadow is unpitched): it is the settled image, not one kept apart under the fresh pitch
        let unpitched = match list {
            Some(Pose::Rocket(_)) => None,
            l => l,
        };
        let shadow_list = if matches!(tilt, Tilt::Tumbling { .. }) {
            None
        } else {
            unpitched
        };
        // An aircraft's shadow comes before the body, which covers it.
        if kind != Kind::Vehicles
            && let Some(s) = shadow
            && let Some(part) =
                self.cached_part_in(kind, idx, shadow_list, shadow_key, |b| draw_shadow(b, s))
        {
            // spec: S-0102 claim 18 (the voxel shadow: bias U, upright, a reader, in the object's
            // own drawing; it never paints over its own aircraft)
            items.push(Self::part_item(
                part,
                shadow_point,
                u,
                Paint::Shadow,
                Group::Shadows,
                object,
            ));
        }
        let mut placed = Vec::new();
        // spec: S-0105 claim 16 (f_b = ν mod F_body, ν the walk-animation counter: the view's walk
        // counter of S-0102 claim 22, design Q-2176; a body without a motion file has one frame)
        let body_frame = self.voxel_model(&body_file).map_or(0, |b| {
            voxel::body_frame(m.walk.counter, b.frame_count() as u32)
        });
        // spec: S-0105 claim 26 (body key; flat ground, the alternative-model state as drawn:
        // claim 12's alternative body while no spawned aircraft is docked)
        // spec: S-0105 claims 26, 41, 42 (a rocket body's key: heading, frame and pitch class; a flying body's: flight drawing heading, frame, pitch presence and bank state; neither has a ramp state)
        let key = match tilt {
            Tilt::Plain | Tilt::Rocked { .. } => {
                voxel::body_key(voxel::RAMP_FLAT, body_raw, alt_drawn, body_frame)
            }
            Tilt::Rocket { class, .. } => voxel::rocket_body_key(body_raw, body_frame, class),
            Tilt::Flying(a) => voxel::flying_body_key(heading, body_frame, &a),
            // spec: S-0113 claim 20 (the body at positive height is drawn fresh, never reused, S-0105 claim 26: its images are kept apart under the F1 table steps of ψ′ and φ, `list` above, beside the level key)
            Tilt::Tumbling { .. } => {
                voxel::flying_body_key(heading, body_frame, &voxel::FlightAttitude::LEVEL)
            }
        };
        if let Some(part) = self.cached_part_in(kind, idx, list, key, |b| {
            let body = b.voxel_model(&body_file)?;
            let vpl = b.vpl();
            let img = b.p.canvas.draw_part(
                &body,
                body_frame,
                &pose().body,
                vpl.as_ref(),
                depth_buffered,
                &q,
            );
            b.limit_checked(&body_file, "body", key, img)
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
        // spec: S-0105 claims 4, 8 (a turret part only with the turret flag); claim 45 (an aircraft-kind object draws no turret and no barrel, in any attitude, whatever its turret flag)
        if kind != Kind::Aircraft && rule_bool(self.import, kind, idx, b"Turret", false) {
            // spec: S-0105 claim 12 (a multi-turret type draws the turret and barrel models of its
            // selected turret index k, claim 43, instead of the ordinary ones; S-0007 claim 16 names
            // them turret k and barrel k). With the index none, or k outside 0 … N − 1, neither a
            // turret nor a barrel is drawn; a missing barrel k alone leaves turret k drawn.
            let multi = multi_turret(self.import, kind, idx);
            // spec: S-0007 claim 17 (the turret count, at most 18 numbered parts)
            let count = rule_int(self.import, kind, idx, b"TurretCount", 0).clamp(0, 18);
            let index = if multi {
                m.turret_index
                    .filter(|k| (0..count).contains(k))
                    .and_then(|k| u8::try_from(k).ok())
            } else {
                None
            };
            let (turret_part, barrel_part) = match (multi, index) {
                (true, Some(k)) => (format!("turret{k}"), format!("barrel{k}")),
                (true, None) => (String::new(), String::new()),
                (false, _) => ("turret".to_string(), "barrel".to_string()),
            };
            let turret_frame = if multi {
                // spec: S-0105 claim 16 (a numbered turret model is drawn in frame f_b whatever its value, never from the animation-frame counter; claim 13 reduces it modulo the model's own HVA frame count, and its turret key records the same frame)
                body_frame
            } else {
                // spec: S-0105 claim 16 (body frame 0: the turret frame from the animation-frame counter, which S-0609 claim 14 advances while a gattling vehicle spins)
                let frames = art
                    .voxels
                    .get(turret_part.as_str())
                    .cloned()
                    .and_then(|f| self.voxel_model(&f))
                    .map(|t| t.frame_count() as u32);
                voxel::vehicle_turret_frame(body_frame, m.anim, frames)
            };
            // spec: S-0105 claim 16 (quirks.barrel_frame_zero)
            let barrel_frame = voxel::barrel_frame_with_turret(turret_frame, &q);
            // spec: S-0105 claim 26 (turret key; flat ground, the selected turret index of a multi-turret type, the alternative-model state off)
            let turret_key = voxel::vehicle_turret_key(
                voxel::RAMP_FLAT,
                body_raw,
                turret_raw,
                offset,
                index,
                false,
                turret_frame,
            );
            let mut parts = vec![
                (
                    false,
                    turret_part.as_str(),
                    voxel::ImageKey::Turret(turret_key),
                ),
                // spec: S-0105 claim 26 (barrel key: the turret's key plus the current 32-step elevation)
                (
                    true,
                    barrel_part.as_str(),
                    voxel::barrel_key(turret_key, elevation),
                ),
            ];
            // spec: S-0105 claim 11 (turret first when its 4-step facing is 1 or 2)
            if !voxel::turret_drawn_first(m.turret) {
                parts.reverse();
            }
            for (is_barrel, part_name, key) in parts {
                let Some(file) = art.voxels.get(part_name).cloned() else {
                    continue;
                };
                if let Some(part) = self.cached_part_in(kind, idx, list, key, |b| {
                    let model = b.voxel_model(&file)?;
                    let vpl = b.vpl();
                    let p = pose();
                    let (draw, frame) = if is_barrel {
                        (p.barrel, barrel_frame)
                    } else {
                        (p.turret, turret_frame)
                    };
                    let img = b.p.canvas.draw_part(
                        &model,
                        frame,
                        &draw,
                        vpl.as_ref(),
                        depth_buffered,
                        &q,
                    );
                    b.limit_checked(&file, part_name, key, img)
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
            // design: render (#106): the footprint is kept in the painter (a pose's goes with the
            // pose's images, design: render (#104))
            let fkey = (kind, idx, unpitched, shadow_key);
            if !self.p.footprints.contains_key(&fkey) {
                // spec: S-0105 claims 31, 39 (cache only the unmasked footprint)
                let image = draw_shadow(self, s).filter(|i| i.width > 0 && i.height > 0);
                let id = self.p.next_footprint;
                self.p.next_footprint = id.wrapping_add(1);
                let footprint = image.map(|image| Footprint {
                    id,
                    image,
                    as_is: None,
                });
                self.p.footprints.insert(fkey, footprint);
            }
            let part = match self.p.footprints.get_mut(&fkey) {
                Some(Some(footprint)) => vehicle_shadow(
                    &mut self.built.scene,
                    Shadows {
                        footprint,
                        cutouts: &mut self.p.cutouts,
                        pool: &mut self.p.pool,
                        transient: unpitched.is_some(),
                    },
                    &placed,
                    (point.0 - shadow_point.0, point.1 - shadow_point.1),
                ),
                // spec: S-0105 claims 31, 39 (a reused footprint is not redrawn)
                _ => None,
            };
            if let Some(part) = part {
                // spec: S-0102 claim 18 (the voxel shadow: bias U, upright, a reader, in the object's own drawing)
                items.push(Self::part_item(
                    part,
                    shadow_point,
                    u,
                    Paint::Shadow,
                    Group::Shadows,
                    object,
                ));
                self.note(notes::VOXEL_SHADOW_LIFTED);
            }
        }
        items
    }
}

/// The parts of a sprite-body vehicle with the turret flag in the order
/// they are painted into its composed image (S-0105 claim 12, *One
/// image*): the body frame first; then at 4-step turret facings 0 and 3
/// (the turret's own facing) the barrel and the turret frame, at 1 and 2
/// the turret frame and the barrel. A missing part is left out.
pub(super) fn sprite_turret_parts(
    body: Option<PlacedPart>,
    barrel: Option<PlacedPart>,
    turret: Option<PlacedPart>,
    turret_facing: u16,
) -> Vec<PlacedPart> {
    // spec: S-0105 claim 12 (One image: claim 11's order by the 4-step turret facing alone)
    let rest = if voxel::turret_drawn_first(turret_facing) {
        [turret, barrel]
    } else {
        [barrel, turret]
    };
    std::iter::once(body).chain(rest).flatten().collect()
}

/// Paints `parts` (scene images placed relative to a draw point) in order
/// into one image (S-0105 claim 12, *One image* and *Depth*): the smallest
/// rectangle holding every part's rectangle, exactly, with no row or column
/// added past the lowest or rightmost part (the original's one-row
/// extension is not reproduced); transparent pixels (index 0) paint
/// nothing and a later opaque pixel wins. `None` without a part.
pub(super) fn compose_parts(scene: &Scene, parts: &[PlacedPart]) -> Option<voxel::PartImage> {
    let rect = |&(id, [x, y]): &PlacedPart| {
        let img = &scene.images[id as usize];
        (x, y, x + img.width() as i32, y + img.height() as i32)
    };
    // spec: S-0105 claim 12 (Depth: the smallest rectangle holding the rectangles of all its parts;
    // the exact rectangle, fixed without a switch)
    let (l, t, r, b) = parts
        .iter()
        .map(rect)
        .reduce(|a, c| (a.0.min(c.0), a.1.min(c.1), a.2.max(c.2), a.3.max(c.3)))?;
    let (w, h) = ((r - l) as u32, (b - t) as u32);
    let mut pixels = vec![0u8; w as usize * h as usize];
    for &(id, [x, y]) in parts {
        let img = &scene.images[id as usize];
        for iy in 0..img.height() {
            for ix in 0..img.width() {
                // spec: S-0105 claim 12 (One image: transparent pixels paint nothing; later pixels win)
                match img.get(ix, iy) {
                    Some(0) | None => {}
                    Some(p) => {
                        let (cx, cy) = ((x - l) as u32 + ix, (y - t) as u32 + iy);
                        pixels[(cy * w + cx) as usize] = p;
                    }
                }
            }
        }
    }
    Some(voxel::PartImage {
        width: w,
        height: h,
        pixels,
        offset: [i64::from(l), i64::from(t)],
        copy_size: [i64::from(w), i64::from(h)],
    })
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

/// A voxel vehicle's shadow draw point (S-0105 claim 44): the body's draw
/// point *B* (`point`, with any locomotor draw shift) moved down by the
/// lift of `above`, its signed height above the surface in leptons (the
/// glossary's, less 416 on a bridge deck).
///
/// The same rule holds at every height: at 0 the anchor is *B*, below the
/// surface the signed lift places it at or above *B*. A vehicle lifted by a
/// carrier aircraft would be raised 14 rows, and 60 more over a qualifying
/// bridge; the simulation has no lifted state yet (the transport specs set
/// it up), so no vehicle is raised.
pub fn vehicle_shadow_point(point: (i32, i32), above: i32) -> (i32, i32) {
    // spec: S-0105 claim 44 (B moved down by L(a) rows, S-0101 claim 3's signed lift, the same lift that lifts B; exact signed integer arithmetic)
    (point.0, point.1 + height_lift(above))
}

/// A vehicle shadow's appearance (S-0105 claims 31, 39): the scene image
/// showing `footprint`, the type's unmasked footprint, with this draw's
/// completed body, turret and barrel images `parts` excluded.
///
/// Every appearance, fresh or reused, excludes the current parts (claim
/// 39). `shift` is the parts' draw point less the shadow's, in screen
/// pixels: (0, 0) unless the vehicle's height above its surface has a lift
/// (S-0105 claim 44), so the exclusion compares at the same screen
/// position. An exclusion that changes no pixel, as when no part's
/// rectangle meets the footprint's, shows the footprint's own scene image,
/// one for all such appearances (design: render (#106)).
fn vehicle_shadow(
    scene: &mut Scene,
    s: Shadows<'_>,
    parts: &[PlacedPart],
    shift: (i32, i32),
) -> Option<PlacedPart> {
    let Shadows {
        footprint,
        cutouts,
        pool,
        transient,
    } = s;
    let now = pool.now;
    // design: render (#106): the footprint as it is, made once (transient with a pose's images)
    let mut as_is = |scene: &mut Scene, pool: &mut TransientPool| {
        *footprint.as_is.get_or_insert_with(|| {
            if transient {
                add_transient_image(scene, pool, &footprint.image)
            } else {
                add_part_image(scene, &footprint.image)
            }
        })
    };
    // design: render (#106): no part rectangle meets the footprint's, so the exclusion changes
    // no pixel
    let unmasked = &footprint.image;
    let span = |at: i64, len: u32| at..at + i64::from(len);
    let meets =
        |a: &std::ops::Range<i64>, b: &std::ops::Range<i64>| a.start < b.end && b.start < a.end;
    let (fx, fy) = (
        span(unmasked.offset[0], unmasked.width),
        span(unmasked.offset[1], unmasked.height),
    );
    let met = parts.iter().any(|&(id, [x, y])| {
        let image = &scene.images[id as usize];
        let px = span(i64::from(x) + i64::from(shift.0), image.width());
        let py = span(i64::from(y) + i64::from(shift.1), image.height());
        meets(&px, &fx) && meets(&py, &fy)
    });
    if !met {
        return as_is(scene, pool);
    }
    // spec: S-0105 claim 39 (exclude this object's current completed part image on every
    // appearance, fresh or reused; no old-pose cutouts persist)
    debug_assert!(
        parts.len() <= MAX_PARTS,
        "a vehicle draws at most three parts"
    );
    let mut part_ids = [NO_PART; MAX_PARTS];
    for (slot, p) in part_ids.iter_mut().zip(parts) {
        *slot = p.0;
    }
    let ids = (footprint.id, part_ids, shift);
    if let Some((cut, used)) = cutouts.get_mut(&ids) {
        // design: render (#104): used in this redraw
        *used = now;
        return match *cut {
            Cut::AsIs => as_is(scene, pool),
            Cut::Own(p) => p,
        };
    }
    let mut img = unmasked.clone();
    // spec: S-0105 claim 39 (at the same screen position: the parts placed relative to the shadow's draw point)
    let images: Vec<voxel::PartImage> = parts
        .iter()
        .map(|&p| {
            let mut i = placed_image(scene, p);
            i.offset = [
                i.offset[0] + i64::from(shift.0),
                i.offset[1] + i64::from(shift.1),
            ];
            i
        })
        .collect();
    let refs: Vec<&voxel::PartImage> = images.iter().collect();
    voxel::exclude_own_image(&mut img, &refs);
    // design: render (#106): an exclusion that changes no pixel is the footprint as it is
    if img.pixels == unmasked.pixels {
        cutouts.insert(ids, (Cut::AsIs, now));
        return as_is(scene, pool);
    }
    // design: render (#104): a cutout is transient
    let part = add_transient_image(scene, pool, &img);
    cutouts.insert(ids, (Cut::Own(part), now));
    part
}

/// What [`vehicle_shadow`] works with: the type's footprint for the pose
/// (settled or [`Pose`]), the cutouts, the released images, and whether
/// the pose's images are transient (design: render (#104), (#106)).
struct Shadows<'a> {
    footprint: &'a mut Footprint,
    cutouts: &'a mut Cutouts,
    pool: &'a mut TransientPool,
    transient: bool,
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

/// The generic infantry-shadow sprite (S-0107 claim 19 d), a fixed file.
pub const GENERIC_SHADOW_FILE: &[u8] = b"INFSHDW.SHP"; // spec: S-0107 Import mapping (game data file INFSHDW.SHP, found by the name lookup of S-0005)
/// The image frame of the generic infantry shadow.
pub const GENERIC_SHADOW_FRAME: usize = 1; // spec: S-0107 Constants (generic shadow image frame 1)
/// The generic infantry shadow's depth bias, before −*L*(*z* − *h*ₛ).
pub const GENERIC_SHADOW_BIAS: i32 = -5; // spec: S-0102 Constants (generic infantry shadow bias −5)

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
    /// The animation-frame counter of S-0105 claim 16 (S-0609 claim 14):
    /// 0 without a game state.
    pub anim: u32,
    /// For infantry: the entry and stage to show.
    pub infantry: Option<InfantryFrame>,
    /// The object code its items carry ([`Item::object`]).
    pub object: u32,
    /// For a loading vehicle harvester: its harvesting image (S-0402
    /// claim 5a, F8).
    pub harvesting: Option<super::harvesting::Harvesting>,
    /// For an aircraft, or a vehicle (used for a hunter-seeker type, S-0105
    /// claim 44): the surface anchor of claim 40, when the caller knows the
    /// game state; `None` takes the terrain under it from the map and no
    /// bridge deck.
    pub shadow: Option<ShadowAnchor>,
    /// For a vehicle that carries spawned aircraft: no spawned aircraft is
    /// docked, so a type with the alternative-body option draws its
    /// alternative body (S-0105 claim 12). `false` without a game state.
    pub alternative: bool,
    /// For an aircraft: how its body is tilted (S-0105 claims 41, 42).
    pub attitude: Attitude,
    /// A vehicle disabled under S-0405 claims 13 and 20 (S-0104 claim 13:
    /// a voxel vehicle is then drawn at half its brightness). `false`
    /// without a game state.
    pub disabled: bool,
    /// The height above the surface in leptons (glossary; 0 on a bridge
    /// deck): an infantry unit above 0 draws no body-sprite shadow
    /// (S-0107 claim 19). 0 without a game state.
    pub above: i32,
    /// Its protection appearance in the shown tick (S-0513 claim 29,
    /// S-0102 claim 34); `None` without a game state.
    pub protection: Option<super::protection::Look>,
    /// For an infantry unit whose height above the surface is not 0: the
    /// shadow height *h*ₛ of its generic shadow when one is drawn (S-0107
    /// claim 19 b and c); `None` otherwise and without a game state.
    pub generic_shadow: Option<i32>,
    /// While berserk: whether the centre of its cell is shrouded for the
    /// viewer (S-0610 claim 7); `None` when not berserk or without a game
    /// state.
    pub berserk: Option<bool>,
    /// For a vehicle of a multi-turret type: its selected turret index
    /// (S-0105 claim 43, state table S3), which picks the numbered turret
    /// and barrel models (claim 12) and is a component of the turret image
    /// key (claim 26). [`selected_turret_index`] gives it. `None` is the
    /// index *none*; an index outside 0 … *N* − 1, which only an imported
    /// turret table entry can hold, is kept, and neither draws a numbered
    /// turret or barrel (claim 12).
    pub turret_index: Option<i32>,
    /// For a vehicle: its walk counter and whether it is moving in the
    /// shown tick (S-0102 claim 22, S-0105 claim 16), as the view keeps
    /// them ([`super::walk`]); counter 0 and standing without a game state.
    pub walk: super::walk::Walk,
    /// Whether it is on a bridge deck (S-0104 claim 13, the deck rows);
    /// `false` without a game state.
    pub deck: bool,
}

/// The selected turret index of a vehicle (S-0105 claim 43, state table
/// S3), derived from the game state in the shown tick `shown`:
/// - a charge-turret type (a multi-turret type, so its turret count *N* is
///   above 0 and it has no rotating-barrel weapon stages): S-0301 claim
///   16's turret number *c*, from creation on in our engine;
/// - any other gunner type: the turret table entry `table` holds for its
///   weapon number, which S-0308 claims 5 and 6 set to 0 at creation and
///   at every hand-over, so the entry for the number it holds now is the
///   one its last setting chose;
/// - every other type: *none* for life, as its weapon number is never set.
///
/// `table` is the type's imported turret table (`None`: all *none*).
pub fn selected_turret_index(
    w: &glasnost_sim::world::World,
    id: glasnost_sim::world::ObjectId,
    shown: i32,
    table: Option<&glasnost_convert::turrets::TurretTable>,
) -> Option<i32> {
    let o = w.obj(id)?;
    if o.kind != SimKind::Vehicle {
        return None;
    }
    let tc = glasnost_sim::combat::veterancy::type_combat(w, id)?;
    turret_index_from(
        tc,
        glasnost_sim::combat::weapon::weapon_number(w, id),
        (o.combat.rearm_until, o.combat.rearm_delay, shown),
        table,
    )
}

/// The selected turret index *t* a turret-tip vehicle's name tip reads
/// (S-0111 claim 23): [`selected_turret_index`], except for a type with
/// rotating-barrel weapon stages, which gets *t* by the rules of a gunner
/// type: the turret table entry for its weapon number when it is a gunner
/// type without the charge-turret flag, *none* otherwise.
pub fn tip_turret_index(
    w: &glasnost_sim::world::World,
    id: glasnost_sim::world::ObjectId,
    shown: i32,
    table: Option<&glasnost_convert::turrets::TurretTable>,
) -> Option<i32> {
    let tc = glasnost_sim::combat::veterancy::type_combat(w, id)?;
    if !tc.gattling {
        return selected_turret_index(w, id, shown, table);
    }
    if tc.gunner && !tc.charge_turret {
        // spec: S-0111 claim 23 (a rotating-barrel type gets t by the rules for a gunner type: the turret table entry for the weapon number, set at creation and at every hand-over, S-0308 claim 6; a change of weapon stage never changes t)
        let weapon = glasnost_sim::combat::weapon::weapon_number(w, id);
        return table.and_then(|t| t.get(weapon).copied().flatten());
    }
    // spec: S-0111 claim 23 (none for a rotating-barrel type that is not a gunner type, and for one with the charge-turret flag for its whole life)
    None
}

/// The selected turret index of a vehicle whose type has the combat
/// fields `tc`, with weapon number `weapon` and rearm countdown
/// (`until`, `delay`) in tick `tick` (S-0105 claim 43, S3).
fn turret_index_from(
    tc: &glasnost_sim::combat::data::TypeCombat,
    weapon: usize,
    (until, delay, tick): (i32, i32, i32),
    table: Option<&glasnost_convert::turrets::TurretTable>,
) -> Option<i32> {
    // spec: S-0105 Terms (a multi-turret type: a turret count above 0 and no rotating-barrel weapon stages); S-0301 claim 16 (its scope)
    if tc.turret_count <= 0 || tc.gattling {
        return None;
    }
    if tc.charge_turret {
        // spec: S-0105 claim 43 (a charge-turret type: in each of its turns the index becomes S-0301 claim 16's turret number c; our engine gives c from creation, fixed without a switch)
        return Some(charge_turret_number(tc.turret_count, until, delay, tick));
    }
    if tc.gunner {
        // spec: S-0105 claim 43 and S3 (each time the weapon number is set, the index becomes the turret table entry for the resulting number, in the same tick; a gunner type's number is set to 0 at creation and at every hand-over, S-0308 claims 5 and 6)
        return table.and_then(|t| t.get(weapon).copied().flatten());
    }
    // spec: S-0105 claim 43 (neither a gunner type nor a charge-turret type: its weapon number is never set, so its index stays none for life)
    None
}

/// S-0301 claim 16's turret number *c* of a charge-turret vehicle with
/// turret count `n` in tick `tick`: trunc(*N* × *r* / *D*) clamped to
/// 0 … *N* − 1 while the rearm countdown runs (*r* = the time remaining,
/// `until` − `tick`; *D* = `delay`, the delay last chosen), and 0 once it
/// is over or whenever *D* ≤ 0.
pub(crate) fn charge_turret_number(n: i32, until: i32, delay: i32, tick: i32) -> i32 {
    // spec: S-0301 claim 1 (the countdown is over from its end tick on) and claim 16 (c = 0 with D ≤ 0 or once the countdown is over)
    if delay <= 0 || tick >= until {
        return 0;
    }
    let r = i64::from(until) - i64::from(tick);
    // spec: S-0301 claim 16 (c = trunc(N × r / D) clamped to 0 … N − 1)
    (i64::from(n) * r / i64::from(delay)).clamp(0, i64::from(n.max(1) - 1)) as i32
}

/// The selected turret index a vehicle of type `idx` is created with,
/// for a map vehicle drawn before any tick (S-0105 claim 43, S3): turret
/// 0 for a charge-turret type (our engine's *c* from creation), the turret
/// table entry for weapon number 0 for a gunner type, *none* otherwise.
/// `None` without typed rules.
pub(crate) fn creation_turret_index(
    rules: Option<&SimRules>,
    import: &Import,
    kind: Kind,
    idx: usize,
) -> Option<i32> {
    if kind != Kind::Vehicles {
        return None;
    }
    let r = rules?;
    let name = &import.outcome.registries.get(&kind)?.get(idx)?.name;
    let t = glasnost_sim::objects::type_index(r, SimKind::Vehicle, name)?;
    let tc = &r.unit(SimKind::Vehicle, t)?.combat;
    // spec: S-0105 claim 43 (a gunner type's weapon number is set to 0 when it is created, however it is made; a charge-turret type shows turret 0 from creation)
    turret_index_from(tc, 0, (i32::MIN, 0, 0), import.turret_tables.get(&idx))
}

/// Whether a type draws numbered turret and barrel models (S-0105 claim 12,
/// S-0007 claim 16): a vehicle type with the turret flag, a turret count of
/// 1 or more and no rotating-barrel weapon stages, the importer's test for
/// loading `turret{k}` and `barrel{k}`.
pub(crate) fn multi_turret(import: &Import, kind: Kind, idx: usize) -> bool {
    // spec: S-0007 claim 16 (numbered parts: a vehicle with Turret, TurretCount 1 or more and not IsGattling); S-0105 claim 12 (rotating-barrel types draw the ordinary turret whatever the count)
    kind == Kind::Vehicles
        && rule_bool(import, kind, idx, b"Turret", false)
        && rule_int(import, kind, idx, b"TurretCount", 0) >= 1
        && !rule_bool(import, kind, idx, b"IsGattling", false)
}

/// How an aircraft body is drawn tilted, read from the game state (S-0105
/// claims 41 and 42), or a sinking vehicle's lean (S-0504 claims 15, 16).
/// Single and double precision values travel as bit patterns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Attitude {
    /// Heading only: a vehicle, or an aircraft without a game state.
    #[default]
    None,
    /// A rocket missile: nose pitch and heading (claim 41).
    Rocket {
        /// The current pitch *p* (S-0506), radians, single-precision bits.
        pitch: u32,
        /// The selected set's initial pitch fraction, single-precision bits.
        initial: u32,
        /// The selected set's final pitch fraction, single-precision bits.
        final_: u32,
    },
    /// A flying-locomotor aircraft: the inputs of its pitch and bank
    /// (claim 42). Its heading is its aiming facing, `Mobile::turret`.
    Flying {
        /// The signed height above the current surface (claim 40), leptons.
        height: i32,
        /// The current speed fraction, double-precision bits.
        speed: u64,
        /// The sign of the unfinished aiming-facing turn: positive clockwise.
        turn: i32,
        /// Crashing (S-0312): drawn with its tumble, not claim 42's angles.
        crashing: bool,
        /// While crashing, the tumble's pitch and roll (S-0113 claims 16 and
        /// 18), radians, single-precision bits; the view holds them
        /// (`super::rocking`), the game none.
        tumble: (u32, u32),
    },
    /// A vehicle that rocks (S-0303 claim 20, S-0614 claims 9 and 10) and
    /// does not sink: where its locomotor draws rocking its body is drawn
    /// with its pitch and roll (S-0106 claim 9; S-0504 claim 16 and F5).
    /// The view holds the angles (`super::rocking`); the game holds none.
    Rocking {
        /// The pitch, radians, single-precision bits.
        pitch: u32,
        /// The roll, radians, single-precision bits.
        roll: u32,
        /// Its locomotor draws rocking: driving, ship, hovering or jumpjet
        /// (S-0504 claim 16; S-0106 claims 5 and 9).
        drawn: bool,
    },
    /// A vehicle the simulation marks sinking (S-0504 Terms): it draws no
    /// shadow (claim 16), and where its locomotor draws rocking its body is
    /// drawn with its pitch and roll (claims 15, 16; F5).
    Sinking {
        /// The pitch (S-0504 claim 15, F4), radians, single-precision bits.
        pitch: u32,
        /// The roll, radians, single-precision bits.
        roll: u32,
        /// Its locomotor draws rocking: driving, ship, hovering or jumpjet
        /// (S-0504 claim 16; S-0106 claims 5 and 9).
        drawn: bool,
    },
}

/// The surface anchor of an aircraft's voxel shadow (S-0105 claim 40).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShadowAnchor {
    /// The terrain surface height at the aircraft's exact horizontal
    /// position, in leptons (S-0106 claim 2).
    pub ground: i32,
    /// A qualifying high-bridge deck raises the anchor by 60 screen pixels.
    pub deck: bool,
}

/// Raises the aircraft shadow anchor by this many screen pixels over a
/// qualifying high-bridge deck.
// spec: S-0105 claim 40 (a qualifying high bridge raises the anchor by exactly 60 screen pixels)
pub const SHADOW_DECK_RAISE: i32 = 60;

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
    /// Whether a type's image cache holds an image for `key` (S-0105
    /// claim 26), for tests.
    #[cfg(test)]
    pub(crate) fn voxel_cached(&self, kind: Kind, idx: usize, key: &voxel::ImageKey) -> bool {
        self.voxels
            .get(&(kind, idx))
            .is_some_and(|c| c.get(key).is_some())
    }

    /// The parts (scene image and offset from the draw point, in painting
    /// order) the composed image `image` was made of (S-0105 claim 12, *One
    /// image*), for tests.
    #[cfg(test)]
    pub(crate) fn composed_parts(&self, image: u32) -> Option<Vec<PlacedPart>> {
        self.composed
            .iter()
            .find(|(_, p)| p.is_some_and(|(i, _)| i == image))
            .map(|(k, _)| k.iter().map(|&(i, x, y)| (i, [x, y])).collect())
    }

    /// The unscaled ground light of a cell (S-0104 claim 13), the neutral
    /// brightness off the lit area.
    pub fn ground_light(&self, cell: CellXY) -> i32 {
        self.lights
            .get(cell)
            .map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground)
    }

    /// A sprite file's headers, read from `assets` the first time it is
    /// asked for.
    fn shp_heads(&mut self, assets: &Assets, file: &[u8]) -> Option<&mut ShpHeads> {
        if !with_upper(file, |k| self.shps.contains_key(k)) {
            let heads = assets
                .get(file)
                .and_then(|b| Shp::parse(b).ok())
                .map(|shp| ShpHeads {
                    canvas: (i32::from(shp.width), i32::from(shp.height)),
                    frames: shp
                        .frames
                        .iter()
                        .map(|f| (i32::from(f.x), i32::from(f.y)))
                        .collect(),
                    images: vec![None; shp.frames.len()],
                });
            self.shps.insert(upper(file), heads);
        }
        with_upper(file, |k| self.shps.get_mut(k))?.as_mut()
    }

    fn new(art: Document, canvas: voxel::VoxelCanvas) -> Self {
        Self {
            art,
            lights: Lights {
                side: 0,
                cells: Vec::new(),
            },
            rows: Rows::default(),
            images: BTreeMap::new(),
            shps: BTreeMap::new(),
            anim_types: BTreeMap::new(),
            voxels: BTreeMap::new(),
            rocked: BTreeMap::new(),
            footprints: BTreeMap::new(),
            next_footprint: 0,
            cutouts: BTreeMap::new(),
            pool: TransientPool::default(),
            composed: BTreeMap::new(),
            canvas,
            models: BTreeMap::new(),
            vpl: None,
            tile_files: BTreeMap::new(),
            companion: None,
            oversized: BTreeSet::new(),
            sources: Vec::new(),
            field_sources: Vec::new(),
            tile_items: Vec::new(),
            protected_cells: BTreeMap::new(),
            pieces: Default::default(),
            object_levels: (0, 0),
            shadowed_cells: BTreeSet::new(),
            effect_tint: None,
            view_quirks: ViewQuirks::default(),
            named_of: BTreeMap::new(),
            named_files: BTreeMap::new(),
            named: Vec::new(),
        }
    }

    /// Sets the shadowed cells (S-0104 Terms, claim 13) the next
    /// [`Painter::mobiles`] lights its vehicles and infantry by.
    pub fn set_shadowed_cells(&mut self, cells: BTreeSet<CellXY>) {
        self.shadowed_cells = cells;
    }

    /// The shadowed cells the next [`Painter::mobiles`] lights by (tests).
    #[cfg(test)]
    pub fn shadowed_cells(&self) -> &BTreeSet<CellXY> {
        &self.shadowed_cells
    }

    /// Sets the rendering quirk switches; what is drawn next reads them.
    pub fn set_view_quirks(&mut self, q: ViewQuirks) {
        self.view_quirks = q;
    }

    /// Sets the effect tint the tintable tables show (S-0104 claims 3 and
    /// 5), hundredths: the set the game's last lighting re-evaluation
    /// chose (`glasnost_sim::lighting::tint`), `None` for every table's
    /// own tint. What is drawn next takes it.
    pub fn set_effect_tint(&mut self, tint: Option<[i32; 3]>) {
        self.effect_tint = tint;
    }

    /// Sets the level factors of the object height terms (S-0104 claim
    /// 13): `lambda` by claim 2's per-cell ranking and `lambda_air` for
    /// aircraft, thousandths per height level. The next
    /// [`Painter::mobiles`] draws with them.
    pub fn set_object_levels(&mut self, lambda: i32, lambda_air: i32) {
        self.object_levels = (lambda, lambda_air);
    }

    /// Sets the look of the building occupying each cell and whether the
    /// cell is shrouded for the local player, which the next
    /// [`Painter::buildings`] draws building slot animations by (S-0102
    /// claim 34, animation rule; S-0513 claim 29b).
    pub fn set_protected_cells(&mut self, cells: BTreeMap<CellXY, super::protection::SlotCell>) {
        self.protected_cells = cells;
    }

    /// Lights every cell afresh with ambient `ambient` (hundredths) and
    /// lighting set `set`'s level factor, ground bias and tint (S-0104
    /// claims 5 and 9).
    fn compute_lights(
        &mut self,
        map: &Map,
        rules: Option<&SimRules>,
        ambient: i32,
        set: LightingSet,
        detail: DetailLevel,
    ) {
        let side = i32::from(map.cells.footprint().coordinate_side());
        let mut cells = vec![None; (side * side) as usize];
        let q = light_quirks(rules);
        // spec: S-0525 claim 13 (field lights add up with the other light sources)
        let sources: Vec<LightSource> = self
            .sources
            .iter()
            .chain(&self.field_sources)
            .copied()
            .collect();
        for (c, cell) in map.cells.iter() {
            cells[(c.y * side + c.x) as usize] = Some(light::full_recompute(
                ambient,
                set.level,
                set.ground,
                set.tint,
                (c.x, c.y),
                i32::from(cell.level),
                &sources,
                detail,
                &q,
            ));
        }
        self.lights = Lights { side, cells };
    }

    /// The shade row of tile table `table` at brightness `b`, adding the
    /// table's rows to `scene` the first time.
    fn tile_row_in(
        &mut self,
        scene: &mut Scene,
        palette: &Palette,
        table: TileTable,
        b: i32,
    ) -> Shade {
        let (base, n) = match self.rows.tiles.get(&table) {
            Some(r) => *r,
            None => {
                // spec: S-0103 claim 8 (terrain tiles use the tile palette, shaded by their cell's table)
                let t = light::shade_table(palette, table.tint(), table.rows(), None);
                let rows: Vec<Row565> = t.iter().map(row565).collect();
                let base = self.rows.place(scene, &rows);
                self.rows.tiles.insert(table, (base, t.len()));
                (base, t.len())
            }
        };
        // spec: S-0104 claim 15, S-0110 claim 5 (each pixel takes the row of its light-alpha value)
        Shade::new(base, n, b)
    }

    /// The tint (‰) the house-colour tables are shaded with now.
    fn house_tint(&self) -> [i32; 3] {
        match self.effect_tint {
            // spec: S-0104 claims 3 and 5 (an effect's set replaces the tint of the house-colour
            // palettes); Formulas, "Map value storage" (‰ = hundredths × 10)
            Some(t) => t.map(|c| c * 10),
            // spec: S-0104 claim 3 (otherwise the house-colour palettes are shaded with white light)
            None => WHITE,
        }
    }

    /// The named palette of file `file` (upper-case), read from `assets`
    /// once: its index in [`Painter::named`], or `None` when the file is
    /// missing or unreadable.
    fn named_file(&mut self, file: &[u8], assets: &Assets, palettes: &PaletteState) -> Option<u16> {
        if let Some(&n) = self.named_files.get(file) {
            return n;
        }
        // spec: S-0103 claim 16 (a missing file: no named palette from it; no other theater's file
        // or unsuffixed name is tried). design: render (#75) (a file that does not parse as a
        // palette counts as missing)
        let n = assets
            .get(file)
            .and_then(|b| glasnost_fmt::pal::Palette6::parse(b).ok())
            .and_then(|f| {
                let i = u16::try_from(self.named.len()).ok()?;
                // spec: S-0103 claim 17 (claim 1's multiplication and byte wrap)
                self.named.push(palettes.load_named(&f));
                Some(i)
            });
        self.named_files.insert(file.to_vec(), n);
        n
    }

    /// The first row and row count of `remap`'s 53-shade table shaded with
    /// tint `tint` (‰), adding its rows to `scene` the first time.
    ///
    /// A table is built when it is first used, at the tint in force then.
    /// S-0103 claim 17 has our engine build the named palettes at game
    /// start with white tint and retint them at every lighting
    /// re-evaluation, "normal" choice and trigger-action tint change; the
    /// shading is a function of the palette and the tint alone (S-0104
    /// claim 3), so a table built later at today's tint is the one that
    /// build and retint give.
    fn house_table_in(
        &mut self,
        scene: &mut Scene,
        palettes: &PaletteState,
        remap: Remap,
        tint: [i32; 3],
    ) -> (usize, usize) {
        if let Some(r) = self.rows.houses.get(&(remap, tint)) {
            return *r;
        }
        let source = match remap.named.and_then(|i| self.named.get(usize::from(i))) {
            // spec: S-0103 claim 17 (a named palette: built as the definition's house-colour palette
            // with the named file in place of the unit palette)
            Some(named) => named,
            // spec: S-0103 claim 6 (the house-colour palette: the unit palette with the ramp)
            None => &palettes.unit,
        };
        let pal = house_palette(source, remap.hsv);
        // spec: S-0103 claims 7 and 17 (indices 240 … 254 are tint-exempt in house-colour and named palettes)
        let t = light::shade_table(&pal, tint, HOUSE_ROWS, Some(TINT_EXEMPT));
        let rows: Vec<Row565> = t.iter().map(row565).collect();
        let base = self.rows.place(scene, &rows);
        self.rows.houses.insert((remap, tint), (base, t.len()));
        (base, t.len())
    }

    /// Shades the house-colour drawings of `items` again with the tint the
    /// house-colour tables show now (S-0104 claims 3 and 5; set by
    /// [`Painter::set_effect_tint`]): each drawing shaded by a house table
    /// of another tint takes the same colour definition's table of today's
    /// tint, keeping its brightness and protection mask. A named-palette
    /// drawing takes its named palette's table of today's tint: S-0103
    /// claim 17 has every event retint every named palette like the
    /// house-colour palettes. For the drawings kept from frame to frame,
    /// the static objects of the cell pass among them (#75). Returns
    /// whether any drawing changed.
    pub fn retint_houses(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
        items: &mut [Item],
    ) -> bool {
        let tint = self.house_tint();
        let others: BTreeMap<u32, Remap> = self
            .rows
            .houses
            .iter()
            .filter(|((_, t), _)| *t != tint)
            .map(|((remap, _), &(base, _))| (base as u32, *remap))
            .collect();
        if others.is_empty() {
            return false;
        }
        let mut changed = false;
        for item in items {
            let (Paint::Lit(shade) | Paint::LitDiamond(shade) | Paint::Translucent(shade, _)) =
                &mut item.paint
            else {
                continue;
            };
            let Some(&remap) = others.get(&shade.base) else {
                continue;
            };
            // spec: S-0103 claim 17 (in our engine every event retints every named palette, with the
            // house-colour palettes' tint)
            let (base, n) = self.house_table_in(scene, inputs.palettes, remap, tint);
            shade.base = base as u32;
            shade.rows = n as u16;
            changed = true;
        }
        changed
    }

    /// Relights the game's view to the simulation's global lighting
    /// (S-0104 claims 4, 5 and 9; `glasnost_sim::lighting`): every cell
    /// at current ambient `ambient` with the chosen set `set`'s level
    /// factor, ground bias and tint, and the terrain tiles of the cell pass
    /// `items` shaded again. Objects pick the new cell light up when they
    /// are next drawn.
    ///
    /// Our presentation: a full recompute of every cell with the chosen
    /// set's terms and its tint as the map tint, which stands in for claim
    /// 5's retint of the tile shade tables and claim 9's incremental update
    /// (the house-colour tables take the effect tint by
    /// [`Painter::set_effect_tint`], and the drawings kept from frame to
    /// frame are shaded again by [`Painter::retint_houses`]); claim 10's
    /// and claim 16's height terms and the cell light of the static objects
    /// of the cell pass are not followed. The named palettes' tables take
    /// the effect tint with the house-colour tables (S-0103 claim 17). The
    /// house tables are all built when the game starts (S-0104 Open questions),
    /// so claim 2's third column, the tint of a table built while an effect
    /// is in force, never applies to them.
    pub fn relight(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
        items: &mut [Item],
        ambient: i32,
        set: LightingSet,
    ) {
        self.compute_lights(
            inputs.map,
            inputs.rules,
            ambient,
            set,
            inputs.options.detail,
        );
        let tiles = std::mem::take(&mut self.tile_items);
        for &(i, c) in &tiles {
            let Some(item) = items.get_mut(i) else {
                continue;
            };
            let (table, b) = match self.lights.get(c) {
                // spec: S-0104 claim 13 (a terrain tile, ground and extra image: the cell's scaled ground level)
                Some(l) => (l.table, l.scaled_ground),
                None => (TileTable::Neutral, NEUTRAL_BRIGHTNESS),
            };
            let row = self.tile_row_in(scene, &inputs.palettes.tile, table, b);
            item.paint = match item.paint {
                Paint::LitDiamond(_) => Paint::LitDiamond(row),
                Paint::Lit(_) => Paint::Lit(row),
                other => other,
            };
        }
        self.tile_items = tiles;
    }

    /// Gives back the rows of the tile and house-colour tables that no
    /// drawing can take any more, after a relight or a retint (design:
    /// render (#107)): a tile table no cell chooses now and a house-colour
    /// or named-palette table of another tint than today's, either one
    /// used by none of the drawings kept from frame to frame, `held`.
    /// Tables of the same size take their rows later. The drawings made
    /// each frame are made after this, from today's tables.
    pub fn free_unused_rows<'a>(&mut self, held: impl IntoIterator<Item = &'a Item>) {
        // Every row a kept drawing reads, so that a table is kept whichever
        // of its rows is named.
        let mut used = BTreeSet::new();
        for item in held {
            match item.paint {
                Paint::Lit(s) | Paint::LitDiamond(s) | Paint::Translucent(s, _) => {
                    let base = s.base as usize;
                    used.extend(base..base + usize::from(s.rows));
                }
                Paint::Row(r) | Paint::Diamond(r) => {
                    used.insert(r);
                }
                Paint::Shadow | Paint::Pixel(_) => {}
            }
        }
        let unused = |&(base, n): &(usize, usize)| used.range(base..base + n).next().is_none();
        let chosen: BTreeSet<TileTable> = self
            .lights
            .cells
            .iter()
            .flatten()
            .map(|l| l.table)
            .chain([TileTable::Neutral])
            .collect();
        let tint = self.house_tint();
        let tiles: Vec<TileTable> = self
            .rows
            .tiles
            .iter()
            .filter(|(t, r)| !chosen.contains(t) && unused(r))
            .map(|(t, _)| *t)
            .collect();
        let houses: Vec<(Remap, [i32; 3])> = self
            .rows
            .houses
            .iter()
            .filter(|((_, t), r)| *t != tint && unused(r))
            .map(|(k, _)| *k)
            .collect();
        let mut freed: Vec<(usize, usize)> = Vec::new();
        for t in tiles {
            freed.extend(self.rows.tiles.remove(&t));
        }
        for k in houses {
            freed.extend(self.rows.houses.remove(&k));
        }
        for (base, n) in freed {
            self.rows.free.entry(n).or_default().push(base);
        }
    }

    /// Sets the radiation fields' light sources (S-0525 claim 13); returns
    /// whether they changed, so that the caller relights (claim 13: every
    /// change relights the cells it reaches in the same tick).
    pub fn set_field_lights(&mut self, lights: Vec<LightSource>) -> bool {
        if lights == self.field_sources {
            return false;
        }
        self.field_sources = lights;
        true
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

    /// A boolean art key of a type's image, as [`Painter::art_int`].
    pub fn art_bool(&self, import: &Import, kind: Kind, idx: usize, key: &[u8]) -> Option<bool> {
        let image = match import.arts.get(&(kind, idx)) {
            Some(a) => a.image.clone(),
            None => import.outcome.registries.get(&kind)?.get(idx)?.name.clone(),
        };
        self.art.value(&image, key).and_then(value::boolean)
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
            live_slots: None,
        };
        let entries = mobiles.iter().map(|m| b.mobile(m)).collect();
        *self = b.p;
        *scene = b.built.scene;
        (entries, b.built.notes)
    }
    /// The overlay and overlay-shadow items of `cells` as they stand during
    /// play (S-0112 claims 1 to 9): every body before every shadow (S-0102
    /// claim 16 rank 3). The items name no object.
    pub fn live_overlays(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
        cells: Vec<(CellXY, glasnost_map::Cell)>,
    ) -> (Vec<Item>, BTreeSet<String>) {
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
            live_slots: None,
        };
        let start = b.built.scene.items.len();
        b.overlays_in(cells, true);
        let items = b.built.scene.items.split_off(start);
        *self = b.p;
        *scene = b.built.scene;
        (items, b.built.notes)
    }

    /// Draws the live game's playing animations into `scene` (adding the
    /// images and rows they need): for each, its layer, ground-layer sort
    /// key and items, or `None` when it draws nothing.
    pub fn anims(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
        list: &[AnimDraw],
    ) -> (Vec<Option<SlotItems>>, BTreeSet<String>) {
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
            live_slots: None,
        };
        let out = list.iter().map(|a| b.live_anim(a)).collect();
        *self = b.p;
        *scene = b.built.scene;
        (out, b.built.notes)
    }

    /// Draws the live game's particles into `scene` (adding the images and
    /// rows they need): for each, its items, none when its type has no
    /// sprite or its frame is not in the sprite.
    pub fn particles(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
        list: &[ParticleDraw],
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
            live_slots: None,
        };
        let out = list.iter().map(|d| b.live_particle(d)).collect();
        *self = b.p;
        *scene = b.built.scene;
        (out, b.built.notes)
    }

    /// Draws the live game's projectiles in flight into `scene` (adding the
    /// images and rows they need): for each, its items (shadow, then body),
    /// none when nothing is drawn for it (S-0306 claim 28).
    pub fn projectiles(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
        list: &[ProjectileDraw],
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
            live_slots: None,
        };
        let out = list.iter().map(|d| b.live_projectile(d)).collect();
        *self = b.p;
        *scene = b.built.scene;
        (out, b.built.notes)
    }

    /// Draws the live game's debris pieces into `scene` (adding the images
    /// and rows they need): for each, its items, none when its type has no
    /// model (S-0317 claim 15, S-0007 claim 30).
    pub fn pieces(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
        list: &[PieceDraw],
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
            live_slots: None,
        };
        let out = list.iter().map(|d| b.live_piece(d)).collect();
        // Pieces no longer in play give their image slots back.
        b.p.pieces.release_unseen();
        *self = b.p;
        *scene = b.built.scene;
        (out, b.built.notes)
    }

    /// Ends a redraw of the live game's objects: the transient images no
    /// redraw has used for a while are released (design: render (#104)).
    ///
    /// A pose's group of images (with its shadow footprints, design: render
    /// (#106)) and a cutout go when unused for [`TRANSIENT_REDRAWS`]
    /// redraws; a cutout made from a released footprint or image goes with
    /// it, so no entry names a released image. A released image becomes
    /// 0 × 0 and its index is given to the next transient image. The items
    /// of the redraw just ended name only images it used, none released
    /// here.
    pub fn end_redraw(&mut self, scene: &mut Scene) {
        let Painter {
            rocked,
            footprints,
            cutouts,
            pool,
            ..
        } = self;
        let now = pool.now;
        pool.now += 1;
        let stale = |used: u64| used + TRANSIENT_REDRAWS <= now;
        let mut released = BTreeSet::new();
        let mut gone = BTreeSet::new();
        rocked.retain(|&(kind, idx, pose), (cache, used)| {
            let keep = !stale(*used);
            if !keep {
                released.extend(cache.values().flatten().map(|p| p.0));
                gone.insert((kind, idx, pose));
            }
            keep
        });
        let mut gone_footprints = BTreeSet::new();
        footprints.retain(|&(kind, idx, pose, _), footprint| {
            let keep = pose.is_none_or(|pose| !gone.contains(&(kind, idx, pose)));
            if !keep && let Some(f) = footprint {
                gone_footprints.insert(f.id);
                released.extend(f.as_is.flatten().map(|p| p.0));
            }
            keep
        });
        cutouts.retain(|(footprint, parts, _), (cut, used)| {
            let keep = !stale(*used)
                && !gone_footprints.contains(footprint)
                && parts.iter().all(|p| !released.contains(p));
            if !keep && let Cut::Own(Some(p)) = cut {
                released.insert(p.0);
            }
            keep
        });
        for id in released {
            let empty = IndexedImage::new(0, 0, Vec::new()).expect("an empty image");
            scene.resize_image(id, empty);
            pool.free.push(id);
        }
    }

    /// The number of scene images the debris pieces' slots hold, in use or
    /// free.
    pub fn piece_images(&self) -> usize {
        self.pieces.images()
    }

    /// A vehicle's target line from screen pixel `start` to `end` (S-0301
    /// claim 24 f to h): every second pixel along the longer screen axis,
    /// `phase` choosing which, then a 3 × 3 square whose bottom-right pixel
    /// is each end point; palette index 8 of the general palette in its
    /// shade row 26, with no depth test. Nothing without the general palette.
    pub fn target_line(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
        start: (i32, i32),
        end: (i32, i32),
        phase: bool,
    ) -> Vec<Item> {
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
            live_slots: None,
        };
        let out = b.target_line_items(start, end, phase);
        *self = b.p;
        *scene = b.built.scene;
        out
    }

    /// Building type `idx`'s slot definitions, from the art section its
    /// image name names (S-0108 Import mapping).
    pub fn slot_defs(&self, inputs: &Inputs, idx: usize) -> super::slot_play::Defs {
        inputs
            .import
            .arts
            .get(&(Kind::Buildings, idx))
            .map(|a| super::slot_play::read_defs(&self.art, &a.image))
            .unwrap_or_default()
    }

    /// The import's animation type `idx`'s art fields and the image-frame
    /// count of its sprite (0 without one), for playback (S-0109 F2).
    pub fn anim_type(&self, inputs: &Inputs, idx: usize) -> Option<(slots::AnimType, i32)> {
        let art = inputs.import.arts.get(&(Kind::Animations, idx))?;
        // spec: S-0007 claim 34 (the animation's own art section)
        let t = slots::read_anim(&self.art, anim_own_section(inputs.import, idx));
        let n = art
            .sprite
            .as_ref()
            .and_then(|s| inputs.assets.get(s))
            .and_then(|b| Shp::parse(b).ok())
            .map_or(0, |s| i32::try_from(s.frames.len()).unwrap_or(i32::MAX));
        Some((t, n))
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
            live_slots: None,
        };
        let mut out = Vec::new();
        for d in list {
            let Some(art) = b.type_art((Kind::Buildings, d.idx)) else {
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
            b.live_slots = Some(d.slots.clone());
            let mut items = b.building_items(
                d.idx,
                art,
                d.cell,
                level,
                &d.owner,
                condition,
                d.occupied,
                d.facing,
                d.anim,
                d.object,
                d.unloading,
                d.gate_door,
                super::protection::BuildingLook {
                    protection: d.protection,
                    adjust: d.adjust,
                },
                d.buildup,
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

/// The lighting switches the cell light reads: S-0104's at their defaults
/// (none is set from a ruleset yet), and the falloff distance's root and
/// rounding from the ruleset's `quirks.table_root_distance` and
/// `quirks.chop_rounding` when the rules resolved (S-0104 claim 11).
pub(super) fn light_quirks(rules: Option<&SimRules>) -> LightQuirks {
    let mut q = LightQuirks::default();
    if let Some(r) = rules {
        // spec: S-0104 claim 11, Formulas "Cell light" (S-0204's quirks.table_root_distance)
        q.table_root_distance = r.quirks.table_root_distance;
        // spec: S-0104 Formulas "Cell light" (S-0205's quirks.chop_rounding)
        q.chop_rounding = r.quirks.chop_rounding;
    }
    q
}

/// Whether a building with health `h` of strength `strength` is damaged
/// and badly damaged (S-0108 F3).
pub(super) fn condition_of(r: &SimRules, h: i32, strength: i32) -> (bool, bool) {
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

/// A building the live game draws: one it put on the map (deployed or
/// placed), or a gate of the map's own (whose door moves).
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
    /// The animation-frame counter of S-0105 claim 16 (S-0609 claim 14).
    pub anim: u32,
    /// It holds at least one garrison occupant (S-0108 Terms, claim 2).
    pub occupied: bool,
    /// Its running slot animations (S-0108 Terms), in slot order.
    pub slots: Vec<LiveSlot>,
    /// Item object code.
    pub object: u32,
    /// Its unloading state, while it unloads a unit (S-0417 Terms; S-0108
    /// claims 22 and 23).
    pub unloading: Option<Unloading>,
    /// The door of a gate (S-0215 claim 35; S-0108 claim 4); `None` for
    /// any other building.
    pub gate_door: Option<unload::DoorView>,
    /// Its protection appearance in the shown tick (S-0513 claim 29,
    /// S-0102 claim 34).
    pub protection: Option<super::protection::Look>,
    /// Its charge flash count and air-strike mark in the shown tick
    /// (S-0519 claims 25 and 26, F7).
    pub adjust: glasnost_sim::mark::Adjust,
    /// Its build-up stage while it is in the construction state (S-0108
    /// claims 1 and 2); `None` outside it.
    pub buildup: Option<Buildup>,
}

/// The build-up stage a live building in the construction state shows
/// (S-0108 claims 1, 2, 4 and 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Buildup {
    /// Stage *k* of the construction range.
    pub stage: i32,
    /// The range's count *c* = *n* (its start *a* is 0).
    pub count: i32,
    /// The building is being sold: the build-up plays backwards.
    pub selling: bool,
}

/// A running slot animation of a live building, as the drawing takes it
/// (S-0108 Terms; S-0109 claims 1 and 20).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiveSlot {
    /// Its slot's definition (displacement, depth adjustment, y-sort).
    pub def: slots::SlotDef,
    /// Its current type's import registration index.
    pub idx: usize,
    /// Image frame *a* + *k*.
    pub frame: i32,
    /// Stage *k*.
    pub stage: i32,
    /// End field *E* in play.
    pub end: i32,
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

    /// The footprint of [`footprint`], kept in the painter, number 0.
    fn kept() -> Footprint {
        Footprint {
            id: 0,
            image: footprint(),
            as_is: None,
        }
    }

    /// [`vehicle_shadow`] for a settled pose, with no released images to use again.
    fn shadow(
        scene: &mut Scene,
        footprint: &mut Footprint,
        cutouts: &mut Cutouts,
        parts: &[PlacedPart],
        shift: (i32, i32),
    ) -> Option<PlacedPart> {
        let s = Shadows {
            footprint,
            cutouts,
            pool: &mut TransientPool::default(),
            transient: false,
        };
        vehicle_shadow(scene, s, parts, shift)
    }

    /// spec: S-0105 claims 31, 39 (the cache keeps the unmasked footprint; the current
    /// completed part image is excluded on every appearance, fresh or reused, so no old-pose
    /// cutout persists; lead-confirmed departure), S-0106 claim 11 (every vehicle shadow excludes
    /// the current own-part image)
    #[test]
    fn cutouts_follow_the_current_parts() {
        let mut scene = Scene::default();
        let mut f = kept();
        let mut cutouts = BTreeMap::new();
        let a = add_part_image(&mut scene, &img(1, 1, [1, 0], &[9])).unwrap();
        let b = add_part_image(&mut scene, &img(2, 1, [2, 0], &[0, 9])).unwrap();
        let m = voxel::SHADOW_MARK;
        let start = scene.images.len();
        let first = shadow(&mut scene, &mut f, &mut cutouts, &[a], (0, 0));
        assert_eq!(pixels(&scene, first), [m, 0, m, m]);
        // design: render (#106): the painter keeps the unmasked footprint; the scene holds only
        // the cutout drawn.
        assert_eq!(f.image.pixels, [m; 4]);
        assert_eq!(scene.images.len(), start + 1);
        let second = shadow(&mut scene, &mut f, &mut cutouts, &[b], (0, 0));
        // B's transparent pixel at x = 2 excludes nothing; its pixel at x = 3 does.
        assert_eq!(pixels(&scene, second), [m, m, m, 0]);
        // The same parts again reuse the masked image: the scene does not grow.
        let n = scene.images.len();
        let third = shadow(&mut scene, &mut f, &mut cutouts, &[b], (0, 0));
        // Back to A's pose: A's cutout again, not B's.
        let fourth = shadow(&mut scene, &mut f, &mut cutouts, &[a], (0, 0));
        assert_eq!(fourth, first);
        assert_eq!((third, scene.images.len()), (second, n));
    }

    /// spec: S-0105 claim 39 (a part pixel at the same screen position excludes the shadow
    /// pixel) and claim 44 (a vehicle above its surface: the shadow drawn below the body, so the
    /// parts are shifted by their draw point less the shadow's; our engine compares at the same
    /// screen position, MS-34 part e)
    #[test]
    fn a_lowered_shadow_excludes_the_parts_where_they_land() {
        let mut scene = Scene::default();
        let mut f = kept();
        let mut cutouts = BTreeMap::new();
        let m = voxel::SHADOW_MARK;
        // A part pixel at x = 1, drawn one pixel to the right of the shadow's point.
        let a = add_part_image(&mut scene, &img(1, 1, [1, 0], &[9])).unwrap();
        let shifted = shadow(&mut scene, &mut f, &mut cutouts, &[a], (1, 0));
        assert_eq!(pixels(&scene, shifted), [m, m, 0, m]);
        // Drawn a row above the shadow's: it covers none of the footprint's row.
        let above = shadow(&mut scene, &mut f, &mut cutouts, &[a], (0, -1));
        assert_eq!(pixels(&scene, above), [m; 4]);
        // The same parts with no shift keep their own memo.
        let level = shadow(&mut scene, &mut f, &mut cutouts, &[a], (0, 0));
        assert_eq!(pixels(&scene, level), [m, 0, m, m]);
    }

    /// design: render (#106): an appearance whose parts change no pixel of the footprint, their
    /// rectangles clear of it or their pixels over it transparent, is the footprint's own scene
    /// image, one for all of them (S-0105 claim 39's exclusion, which removes nothing there)
    #[test]
    fn an_exclusion_that_changes_nothing_is_the_footprint_itself() {
        let mut scene = Scene::default();
        let mut f = kept();
        let mut cutouts = BTreeMap::new();
        let m = voxel::SHADOW_MARK;
        let a = add_part_image(&mut scene, &img(1, 1, [1, 0], &[9])).unwrap();
        let clear = add_part_image(&mut scene, &img(2, 1, [1, 0], &[0, 0])).unwrap();
        let start = scene.images.len();
        let shown = [
            shadow(&mut scene, &mut f, &mut cutouts, &[a], (0, -1)),
            shadow(&mut scene, &mut f, &mut cutouts, &[a], (0, 5)),
            shadow(&mut scene, &mut f, &mut cutouts, &[clear], (0, 0)),
            shadow(&mut scene, &mut f, &mut cutouts, &[clear], (0, 0)),
        ];
        assert!(shown.iter().all(|&s| s == shown[0]), "{shown:?}");
        assert_eq!(shown[0], f.as_is.flatten());
        assert_eq!(pixels(&scene, shown[0]), [m; 4]);
        assert_eq!(scene.images[shown[0].unwrap().0 as usize].width(), 4);
        assert_eq!(shown[0].unwrap().1, [0, 0]);
        assert_eq!(scene.images.len(), start + 1);
    }

    /// spec: S-0105 claim 44 and MS-34 part e (B moved down by S-0101 claim 3's signed lift of
    /// the height above the surface: exactly the surface projection on a surface at a whole
    /// level, within one row of it elsewhere, B itself at a = 0, at or above B for a < 0)
    #[test]
    fn a_vehicle_shadow_anchor_is_the_body_point_lowered_by_the_lift() {
        let (x, y) = (3000, 2000);
        // A surface at a whole level h (104 leptons a level): exactly the surface projection.
        for h in 0..=13 {
            for a in [0, 1, 7, 52, 103, 104, 300, 416, 1500] {
                let b = project_lifted(x, y, 104 * h + a);
                assert_eq!(
                    vehicle_shadow_point(b, a),
                    project_lifted(x, y, 104 * h),
                    "level {h}, a = {a}"
                );
            }
        }
        // A slope (any surface height s): within one row of the surface projection.
        for s in [5, 37, 60, 99, 151, 260, 777] {
            for a in [0, 1, 20, 104, 333, 1000] {
                let b = project_lifted(x, y, s + a);
                let at = vehicle_shadow_point(b, a);
                let surface = project_lifted(x, y, s);
                assert_eq!(at.0, surface.0);
                assert!((at.1 - surface.1).abs() <= 1, "s = {s}, a = {a}");
            }
        }
        // a = 0: B itself; a < 0: the signed lift places the anchor at or above B.
        let b = project_lifted(x, y, 400);
        assert_eq!(vehicle_shadow_point(b, 0), b);
        for a in [-1, -50, -104, -416] {
            let at = vehicle_shadow_point(b, a);
            assert_eq!(at, (b.0, b.1 + height_lift(a)));
            assert!(at.1 <= b.1, "a = {a}");
        }
        // Applied to B as drawn: a locomotor draw shift of B moves the anchor with it.
        assert_eq!(
            vehicle_shadow_point((b.0, b.1 - 3), 104),
            (b.0, b.1 - 3 + 15)
        );
    }
}

/// The walking and standing rows of a vehicle drawn as a sprite (S-0102
/// claim 22 and Import mapping): *S*<sub>w</sub>, *F*<sub>w</sub>,
/// *S*<sub>s</sub>, *F*<sub>s</sub>.
#[derive(Clone, Copy, Debug)]
pub struct SpriteVehicleRows {
    /// The walking start frame *S*<sub>w</sub>.
    pub walk_start: i32,
    /// The walking frame count *F*<sub>w</sub>.
    pub walk_frames: i32,
    /// The standing start frame *S*<sub>s</sub>.
    pub stand_start: i32,
    /// The standing frame count *F*<sub>s</sub>.
    pub standing_frames: i32,
}

/// The image frame of a vehicle drawn as a sprite that neither fires nor
/// dies (S-0102 claim 22's frame table): the moving row while it moves or
/// its type's idle rate is not 0, with its walk counter *w*; else the
/// standing row, or the walking start of its facing part `d`.
pub fn sprite_vehicle_frame(
    rows: SpriteVehicleRows,
    d: i32,
    walk: super::walk::Walk,
    idle_rate: i32,
) -> i32 {
    let SpriteVehicleRows {
        walk_start,
        walk_frames,
        stand_start,
        standing_frames,
    } = rows;
    if walk.in_progress || idle_rate != 0 {
        // spec: S-0102 claim 22 (movement in progress, or the idle rate not 0: S_w + d·F_w + (w mod F_w))
        let w = i64::from(walk.counter);
        // design: a walking frame count of 0 (w mod 0 undefined) adds nothing
        let step = if walk_frames > 0 {
            w.rem_euclid(i64::from(walk_frames)) as i32
        } else {
            0
        };
        walk_start + d * walk_frames + step
    } else if standing_frames > 0 {
        // spec: S-0102 claim 22 (F_s > 0: S_s + d·F_s)
        stand_start + d * standing_frames
    } else {
        // spec: S-0102 claim 22 (otherwise: S_w + d·F_w)
        walk_start + d * walk_frames
    }
}
