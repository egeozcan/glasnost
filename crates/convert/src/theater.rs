//! A theater's tile catalogue (S-0010 claims 1 to 5): the theater data file,
//! the declared tilesets and tile numbers, the tile image files and their
//! variants, and the role starts; with S-0009's fill base tiles (claim 6)
//! and tile number translation (claim 8, Formulas).
//!
//! [`Catalogue::read`] reads the theater data file. [`Catalogue::images`]
//! finds each tile's image files through a file-existence test (the game's
//! search order, S-0005), and [`Catalogue::tile_catalogue`] reads the tile
//! files into the simulation's [`TileCatalogue`] (the terrain code and slope
//! code of every tile cell).

use std::collections::BTreeMap;

use glasnost_ini::Document;
use glasnost_ini::value;
use glasnost_map::terrain::{
    BridgePositions, Roles, TileAnim, TileCatalogue, TileCell, TileInfo, UNSET,
    UNSET_DESTROYABLE_CLIFF, UNSET_POSITION,
};
use glasnost_vfs::Vfs;
use glasnost_vfs::theater::Theater;

/// The theater whose seven roles are forced unset (S-0010 claim 4).
const FORCED_UNSET_THEATER: Theater = Theater::Lunar; // spec: S-0010 claim 4 (the theater with index 5)

/// The unset base tile (S-0009 claim 6).
pub const UNSET_BASE_TILE: i32 = UNSET; // spec: S-0009 Constants (unset base tile), S-0010 claim 4

/// Tile count of a tileset section without the count key (S-0010 Import
/// mapping): it ends the list.
const END_OF_LIST: i32 = -1; // spec: S-0010 claim 2, Import mapping (TilesInSet absent: −1)
/// Longest file-name stem and set name (S-0010 Import mapping).
const NAME_MAX: usize = 63; // spec: S-0010 Import mapping (FileName and SetName, limit 63 bytes)
/// File-name stem of a tileset without the key (S-0010 Import mapping).
const DEFAULT_FILE_NAME: &[u8] = b"TILE"; // spec: S-0010 Import mapping (FileName absent: TILE)
/// Set name of a tileset without the key (S-0010 Import mapping).
const DEFAULT_SET_NAME: &[u8] = b"No Name"; // spec: S-0010 Import mapping (SetName absent: No Name)
/// Non-height-view field without the key (S-0010 Import mapping).
const DEFAULT_NON_HEIGHT_VIEW: i32 = 65535; // spec: S-0010 Import mapping (NonMarbleMadness absent: 65535)
/// The suffixes tried after the theater's fallback suffix: those of
/// theaters 0 and 2 (S-0010 claim 3, table T).
const LAST_RESORT_SUFFIXES: [&str; 2] = ["TEM", "URB"]; // spec: S-0010 Constants table T (TEM, then URB)

/// Highest tileset section number tried (four digits, S-0010 claim 2).
const MAX_TILESETS: u32 = 10_000; // spec: S-0010 Constants (tileset section number width: 4 digits)
/// Variant letters tried after the plain image (`a` to `z`; a bound of our
/// own: claim 3 stops at the first letter without a file).
const MAX_VARIANTS: u8 = 26; // nocite: bound of our own design (the letters a to z)

/// The theater data file and the tile suffixes (S-0010 table T).
fn table_t(theater: Theater) -> (&'static str, &'static str, &'static str) {
    // spec: S-0010 Constants table T (theater data file, tile suffix, fallback suffix)
    match theater {
        Theater::Temperate => ("TEMPERATMD.INI", "TEM", "MMT"),
        Theater::Snow => ("SNOWMD.INI", "SNO", "MMS"),
        Theater::Urban => ("URBANMD.INI", "URB", "MMU"),
        Theater::Desert => ("DESERTMD.INI", "DES", "MMD"),
        Theater::NewUrban => ("URBANNMD.INI", "UBN", "MMT"),
        Theater::Lunar => ("LUNARMD.INI", "LUN", "MML"),
    }
}

/// Quirk switches of S-0010 that the catalogue implements. Each defaults to
/// the original's behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TheaterQuirks {
    /// `quirks.role_one_past_end` (S-0010 claim 4): a role naming the first
    /// missing tileset number starts at the tile count. Off: it is unset.
    pub role_one_past_end: bool,
    /// `quirks.theater_five_forced_unset_roles` (S-0010 claim 4): the theater
    /// with index 5 forces seven roles unset. Off: they use their declared
    /// values.
    pub theater_five_forced_unset_roles: bool,
}

impl Default for TheaterQuirks {
    fn default() -> Self {
        // spec: S-0010 Edge cases (every switch defaults to the original)
        Self {
            role_one_past_end: true,
            theater_five_forced_unset_roles: true,
        }
    }
}

/// The fixed 4 × 4 variant pattern P4 of S-0010 claim 13, by row *Y* mod 4
/// and column *X* mod 4.
// spec: S-0010 Constants (P4, fixed variant pattern)
pub const P4: [[u8; 4]; 4] = [[0, 1, 2, 3], [3, 2, 1, 0], [2, 3, 0, 1], [1, 0, 3, 2]];

/// Image indices of the 8 × 8 pattern before the reduction of claim 13.
const PATTERN_VALUES: u32 = 8; // spec: S-0010 Formulas (8 × 8 pattern: c_i = D_i mod 8)

/// Draws tried per entry of the 8 × 8 pattern (S-0010 claim 14).
const PATTERN_ATTEMPTS: usize = 64; // spec: S-0010 Constants (pattern draw attempts: 64)

/// The neighbour offsets of an entry of the 8 × 8 pattern, along the index
/// (S-0010 Formulas, *8 × 8 pattern*).
// spec: S-0010 Formulas (neighbours (k + d) mod 64 for d in −9, −8, −7, −1, 1, 7, 8, 9)
const PATTERN_NEIGHBOURS: [i32; 8] = [-9, -8, -7, -1, 1, 7, 8, 9];

/// Builds the 8 × 8 variant pattern from consecutive draws of the
/// unsynchronised random stream (S-0010 claim 14, Formulas). Returns the
/// 64 entries, indexed by *X* + 8*Y*, and the number of draws taken.
pub fn variant_pattern(mut draw: impl FnMut() -> u32) -> ([u8; 64], u32) {
    let mut p = [0u8; 64];
    let mut taken = 0u32;
    for k in 0..64i32 {
        let mut c = 0u8;
        for _ in 0..PATTERN_ATTEMPTS {
            // spec: S-0010 Formulas (c_i = D_i mod 8)
            c = (draw() % PATTERN_VALUES) as u8;
            taken += 1;
            // spec: S-0010 Formulas (the first c differing from every neighbour k′ < k; else the 64th)
            let clash = PATTERN_NEIGHBOURS.iter().any(|d| {
                let n = (k + d).rem_euclid(64);
                n < k && p[n as usize] == c
            });
            if !clash {
                break;
            }
        }
        p[k as usize] = c;
    }
    (p, taken)
}

/// The alternate-image mark of a tile cell's flag value (S-0010 Terms and
/// Import mapping): its bit of value 4, whatever its other bits hold.
pub const ALTERNATE_IMAGE_MARK: u32 = 4; // spec: S-0010 Constants (alternate-image mark: bit of value 4 of the flag value)

/// Whether a tile cell's flag value carries the alternate-image mark
/// (S-0010 claim 13, Terms).
pub fn alternate_image_mark(flags: u32) -> bool {
    flags & ALTERNATE_IMAGE_MARK != 0
}

/// The image a cell shows (S-0010 claim 13, Formulas *Variant*): for a cell
/// (x, y) of sub-tile `t` of a tile whose plain image is `w` × `h` cells
/// and has `n` variants, with the 8 × 8 pattern `p8`. `marked` is the
/// alternate-image mark of the plain image's tile cell at index
/// t mod (w × h) (read from the plain image's file only, claim 13): a
/// marked cell uses no pattern and shows image 1 while its alternate-image
/// state is set and image 0 while it is clear. The state is clear at map
/// load and no spec sets it, so a marked cell shows image 0.
pub fn variant_image(
    (x, y): (i32, i32),
    t: i32,
    (w, h): (i32, i32),
    n: usize,
    p8: &[u8; 64],
    marked: bool,
) -> usize {
    if marked {
        // spec: S-0010 claim 13 and Formulas (a marked cell: image 1 with the state set, 0 with it clear; the state is clear at map load)
        return 0;
    }
    let (w, h) = (w.max(1), h.max(1));
    // spec: S-0010 claim 13 and Formulas ((X, Y) = the tile's origin cell divided by the size, truncated, for every sub-tile: our engine's correction, no switch)
    let (cx, cy) = ((x - t.rem_euclid(w)) / w, (y - t.div_euclid(w)) / h);
    // spec: S-0010 claim 13 and Constants (the 4 × 4 pattern with at most 4 images, the plain image included, else the 8 × 8 pattern)
    let p = if n < 4 {
        P4[cy.rem_euclid(4) as usize][cx.rem_euclid(4) as usize]
    } else {
        p8[(cx.rem_euclid(8) + 8 * cy.rem_euclid(8)) as usize]
    };
    // spec: S-0010 claim 13 (image = P mod (n + 1))
    usize::from(p) % (n + 1)
}

/// One declared tileset of a theater (S-0010 claim 2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tileset {
    /// Section number (`TileSetNNNN`).
    pub number: u32,
    /// The file-name stem (S-0010 claim 3).
    pub file_name: Vec<u8>,
    /// The set name.
    pub set_name: Vec<u8>,
    /// Tile count C as given (a negative count contributes no tile).
    pub count: i32,
    /// Authored tile count A, if given (`LastTilesInSet`, S-0009 claim 8).
    pub authored: Option<i32>,
    /// First tile number of the set in the theater's order.
    pub first: i32,
    /// The non-height-view field: 0 disables the file fallback (S-0010
    /// claim 3).
    pub non_height_view: i32,
    /// Each tile's animation, tiles 1 … C in order (S-0010 claim 19).
    pub anims: Vec<Option<TileAnim>>,
    /// The allow-burrowing flag (S-0010 claim 17, `AllowBurrowing`).
    pub allow_burrowing: bool,
    /// The allow-tiberium flag (S-0010 claim 16, S-0401 claim 10).
    pub allow_tiberium: bool,
    /// The shadow-caster flag (S-0010 claim 18, `ShadowCaster`).
    pub shadow_caster: bool,
    /// The shadow-tile count (S-0010 claim 18, `ShadowTiles`).
    pub shadow_tiles: i32,
}

impl Tileset {
    /// Whether the tileset casts shadows (S-0010 claim 18; S-0104 claims 13
    /// and 19).
    pub fn casts_shadows(&self) -> bool {
        // spec: S-0010 claim 18 (only when both the shadow-caster flag is set and the shadow-tile count is not 0)
        self.shadow_caster && self.shadow_tiles != 0
    }
}

/// A theater's catalogue.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Catalogue {
    /// Declared tilesets in declaration order.
    pub tilesets: Vec<Tileset>,
    /// Total tile count of the theater.
    pub tile_count: i32,
    /// Clear base tile (S-0009 claim 6): the clear role's start.
    pub clear_base: i32,
    /// Water base tile (S-0009 claim 6): the water role's start.
    pub water_base: i32,
    /// Tileset number the clear role names, if given.
    pub clear_set: Option<i32>,
    /// Translation pairs (t_i, δ_i) of S-0009 Formulas.
    pub translation: Vec<(i32, i32)>,
    /// The theater's tile suffix (S-0010 table T).
    pub extension: String,
    /// The suffixes of the file fallback in order of preference (S-0010
    /// claim 3).
    pub fallback: Vec<String>,
    /// Role starts (S-0010 claim 4, table R).
    pub roles: Roles,
    /// The bridge tile positions (S-0511 Import mapping).
    pub bridge_positions: BridgePositions,
    /// Importer warnings about the theater data file.
    pub warnings: Vec<String>,
}

/// The theater data file's name (S-0010 claim 1, table T), resolved like
/// any data file.
pub fn data_file_names(theater: Theater) -> [String; 1] {
    [table_t(theater).0.to_owned()]
}

/// A tile's image files as found (S-0010 claim 3).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TileImages {
    /// The plain image's file, if found.
    pub plain: Option<Vec<u8>>,
    /// The variants' files, letters `a`, `b`, … in order.
    pub variants: Vec<Vec<u8>>,
}

impl Catalogue {
    /// Reads the catalogue from the theater's data document, with the
    /// original's behaviour.
    pub fn read(doc: &Document, theater: Theater) -> Self {
        Self::read_with(doc, theater, &TheaterQuirks::default())
    }

    /// Reads the catalogue with the given switches.
    pub fn read_with(doc: &Document, theater: Theater, quirks: &TheaterQuirks) -> Self {
        let mut warnings = Vec::new();
        let mut tilesets = Vec::new();
        let mut first = 0i32;
        let mut authored_first = 0i32;
        let mut translation = Vec::new();
        for number in 0..MAX_TILESETS {
            // spec: S-0010 claim 2 (fixed prefix and a four-digit zero-padded number from 0)
            let name = format!("TileSet{number:04}");
            let section = doc.section(name.as_bytes());
            let count = section
                .and_then(|s| s.value(b"TilesInSet"))
                .and_then(value::integer)
                .unwrap_or(END_OF_LIST);
            // spec: S-0010 claim 2 (the first section without a count, or with −1, ends the list)
            let Some(section) = section.filter(|_| count != END_OF_LIST) else {
                break;
            };
            if count < 0 {
                // spec: S-0010 claim 2 (a negative count other than −1 contributes none; the importer warns)
                warnings.push(format!(
                    "[{name}] TilesInSet is {count}: the tileset contributes no tile (S-0010 claim 2)"
                ));
            }
            let authored = section.value(b"LastTilesInSet").and_then(value::integer);
            // spec: S-0009 Formulas (tile translation: pairs where A differs from C)
            if let Some(a) = authored
                && a != count
            {
                translation.push((authored_first.saturating_add(a), count.saturating_sub(a)));
            }
            authored_first = authored_first.saturating_add(authored.unwrap_or(count));
            let text = |key: &[u8], default: &[u8]| {
                section
                    .value(key)
                    .map_or_else(|| default.to_vec(), |v| value::text(v, NAME_MAX).to_vec())
            };
            let set_name = text(b"SetName", DEFAULT_SET_NAME);
            let anims = tile_anims(doc, &set_name, count);
            tilesets.push(Tileset {
                number,
                file_name: text(b"FileName", DEFAULT_FILE_NAME),
                set_name,
                anims,
                count,
                authored,
                first,
                // spec: S-0010 claim 16 and S-0401 Import mapping (AllowTiberium absent: no)
                allow_tiberium: section
                    .value(b"AllowTiberium")
                    .and_then(value::boolean)
                    .unwrap_or(false),
                non_height_view: section
                    .value(b"NonMarbleMadness")
                    .and_then(value::integer)
                    .unwrap_or(DEFAULT_NON_HEIGHT_VIEW),
                // spec: S-0010 Import mapping (`AllowBurrowing`, boolean, absent: yes)
                allow_burrowing: section
                    .value(b"AllowBurrowing")
                    .and_then(value::boolean)
                    .unwrap_or(true),
                shadow_caster: false,
                shadow_tiles: 0,
            });
            first = first.saturating_add(count.max(0));
        }
        let tile_count = first;
        let general = |key: &str| {
            doc.value(b"General", key.as_bytes())
                .and_then(value::integer)
        };
        let mut start = |key: &str, unset: i32| -> i32 {
            let Some(n) = general(key) else {
                // spec: S-0010 claim 4 (an absent key leaves the role unset)
                return unset;
            };
            if n < 0 {
                // spec: S-0010 claim 4 (a negative number names no tileset; the importer warns)
                warnings.push(format!(
                    "[General] {key}={n} names no tileset: the role is unset (S-0010 claim 4)"
                ));
                return unset;
            }
            match usize::try_from(n).ok().and_then(|i| tilesets.get(i)) {
                // spec: S-0010 claim 4 (a role's start is the first tile of the tileset it names)
                Some(t) => t.first,
                // spec: S-0010 claim 4 (`quirks.role_one_past_end`: the first missing number gives the tile count)
                None if quirks.role_one_past_end && n as usize == tilesets.len() => tile_count,
                // spec: S-0010 claim 4 (a number beyond the declaration leaves the role unset)
                None => unset,
            }
        };
        // spec: S-0010 Import mapping (the general section's role keys)
        let mut roles = Roles {
            clear: start("ClearTile", UNSET),
            water: start("WaterSet", UNSET),
            ramp: start("RampBase", UNSET),
            smooth_ramp: start("RampSmooth", UNSET),
            cliff: start("CliffSet", UNSET),
            shore: start("ShorePieces", UNSET),
            water_bridge: start("WaterBridge", UNSET),
            misc_pave: start("MiscPaveTile", UNSET),
            pave_ground: start("PaveTile", UNSET),
            paved_road: start("PavedRoads", UNSET),
            paved_road_end: start("PavedRoadEnds", UNSET),
            paved_road_slope: start("PavedRoadSlopes", UNSET),
            median: start("Medians", UNSET),
            dirt_road_junction: start("DirtRoadJunction", UNSET),
            dirt_road_curve: start("DirtRoadCurve", UNSET),
            dirt_road_straight: start("DirtRoadStraight", UNSET),
            bridge: start("BridgeSet", UNSET),
            wooden_bridge: start("WoodBridgeSet", UNSET),
            // spec: S-0010 claim 4 (the destroyable-cliff role's unset start is −2)
            destroyable_cliff: start("DestroyableCliffs", UNSET_DESTROYABLE_CLIFF),
            tunnels: [
                start("Tunnels", UNSET),
                start("TrackTunnels", UNSET),
                start("DirtTunnels", UNSET),
                start("DirtTrackTunnels", UNSET),
            ],
            rough_ground: start("RoughTile", UNSET),
            rough_transition: start("ClearToRoughLat", UNSET),
            sand_ground: start("SandTile", UNSET),
            sand_transition: start("ClearToSandLat", UNSET),
            green_ground: start("GreenTile", UNSET),
            green_transition: start("ClearToGreenLat", UNSET),
            pave_transition: start("ClearToPaveLat", UNSET),
        };
        if theater == FORCED_UNSET_THEATER && quirks.theater_five_forced_unset_roles {
            // spec: S-0010 claim 4 (quirks.theater_five_forced_unset_roles: theater 5's bridge, cliff, shore,
            // water, water bridge and wooden bridge roles are unset;
            // the water cliff role has no start in `Roles`)
            for r in [
                &mut roles.bridge,
                &mut roles.cliff,
                &mut roles.shore,
                &mut roles.water,
                &mut roles.water_bridge,
                &mut roles.wooden_bridge,
            ] {
                *r = UNSET;
            }
        }
        // spec: S-0511 Import mapping (the ten bridge positions: 1-based positions q, absent −1)
        let position = |key: &str| general(key).unwrap_or(UNSET_POSITION);
        let bridge_positions = BridgePositions {
            top_left: [position("BridgeTopLeft1"), position("BridgeTopLeft2")],
            top_right: [position("BridgeTopRight1"), position("BridgeTopRight2")],
            bottom_right: [
                position("BridgeBottomRight1"),
                position("BridgeBottomRight2"),
            ],
            bottom_left: [position("BridgeBottomLeft1"), position("BridgeBottomLeft2")],
            middle: [position("BridgeMiddle1"), position("BridgeMiddle2")],
        };
        let (_, suffix, fallback) = table_t(theater);
        Self {
            tile_count,
            // spec: S-0010 claim 4, S-0009 claim 6 (the clear and water roles give the fill base tiles)
            clear_base: roles.clear,
            water_base: roles.water,
            clear_set: general("ClearTile"),
            translation,
            extension: suffix.to_owned(),
            // spec: S-0010 claim 3 (the fallback suffix, then TEM, then URB)
            fallback: std::iter::once(fallback)
                .chain(LAST_RESORT_SUFFIXES)
                .map(str::to_owned)
                .collect(),
            roles,
            bridge_positions,
            warnings,
            tilesets,
        }
    }

    /// Translates a map tile number into the theater's order (S-0009 claim 8,
    /// Formulas). 65535 is kept.
    pub fn translate(&self, m: i32) -> i32 {
        if m == glasnost_map::NO_TILE {
            return m;
        }
        // spec: S-0009 Formulas (m' = m + δ_1 + … + δ_j over the leading pairs with t_i ≤ m)
        let mut out = m;
        for &(t, delta) in &self.translation {
            if t > m {
                break;
            }
            out = out.saturating_add(delta);
        }
        out
    }

    /// The tile number whose artwork a cell's stored tile number shows
    /// (S-0009 claim 8): the marker 65535 and numbers at or beyond the tile
    /// count show the clear tile; a negative number is treated like the
    /// marker (our engine, a departure). `None` when the theater has no clear
    /// tile either.
    pub fn art_tile(&self, stored: i32) -> Option<i32> {
        // spec: S-0009 claim 8 (65535, numbers ≥ tile count and, in our engine, negatives: the clear tile)
        if stored < 0 || stored >= self.tile_count {
            (self.clear_base >= 0 && self.clear_base < self.tile_count).then_some(self.clear_base)
        } else {
            Some(stored)
        }
    }

    /// The tileset holding tile `t` and the tile's 1-based position in it.
    pub fn locate(&self, t: i32) -> Option<(&Tileset, i32)> {
        let i = self
            .tilesets
            .partition_point(|s| s.first <= t)
            .checked_sub(1)?;
        let set = self.tilesets.get(i)?;
        let pos = t - set.first;
        (pos >= 0 && pos < set.count).then_some((set, pos + 1))
    }

    /// The file name of tile `t`'s image `variant` (0 for the plain image)
    /// with `suffix` (S-0010 claim 3): the file-name stem, the position in
    /// the set with at least two digits, the variant letter, a dot and the
    /// suffix.
    pub fn image_name(&self, t: i32, variant: u8, suffix: &str) -> Option<Vec<u8>> {
        let (set, ordinal) = self.locate(t)?;
        let mut name = set.file_name.clone();
        // spec: S-0010 claim 3 (k in decimal, zero-padded to at least two digits)
        name.extend_from_slice(format!("{ordinal:02}").as_bytes());
        if variant > 0 {
            // spec: S-0010 claim 3 (variant 1 is `a`, variant 2 `b`, …)
            name.push(b'a' + (variant - 1));
        }
        name.push(b'.');
        name.extend_from_slice(suffix.as_bytes());
        Some(name)
    }

    /// The plain image's file name of tile `t` with the theater's own suffix.
    pub fn file_of(&self, t: i32) -> Option<Vec<u8>> {
        self.image_name(t, 0, &self.extension)
    }

    /// The file that holds tile `t`'s image `variant`: the name with the
    /// theater's suffix, or else, when the tileset's non-height-view field is
    /// not 0, the first found with the fallback suffixes (S-0010 claim 3).
    pub fn find_image(
        &self,
        t: i32,
        variant: u8,
        exists: &dyn Fn(&[u8]) -> bool,
    ) -> Option<Vec<u8>> {
        let (set, _) = self.locate(t)?;
        let own = self.image_name(t, variant, &self.extension)?;
        if exists(&own) {
            return Some(own);
        }
        // spec: S-0010 claim 3 (the fallback applies when the non-height-view field is not 0)
        if set.non_height_view == 0 {
            return None;
        }
        self.fallback
            .iter()
            .filter_map(|s| self.image_name(t, variant, s))
            .find(|n| exists(n))
    }

    /// Each tile's image files, by tile number (S-0010 claim 3): the plain
    /// image and the variants up to the first letter without a file.
    pub fn images(&self, exists: &dyn Fn(&[u8]) -> bool) -> Vec<TileImages> {
        (0..self.tile_count.max(0))
            .map(|t| self.tile_images(t, exists))
            .collect()
    }

    /// Tile `t`'s image files (S-0010 claim 3): the plain image and the
    /// variants up to the first letter without a file, each found with the
    /// suffix fallback.
    pub fn tile_images(&self, t: i32, exists: &dyn Fn(&[u8]) -> bool) -> TileImages {
        let plain = self.find_image(t, 0, exists);
        let mut variants = Vec::new();
        if plain.is_some() {
            for v in 1..=MAX_VARIANTS {
                // spec: S-0010 claim 3 (variants stop at the first letter without a file)
                match self.find_image(t, v, exists) {
                    Some(f) => variants.push(f),
                    None => break,
                }
            }
        }
        TileImages { plain, variants }
    }

    /// The simulation's tile catalogue from the tiles' images (from
    /// [`Catalogue::images`]) and a reader of their files. Each tile cell
    /// takes the terrain code and slope code of its tile file (S-0010 Terms,
    /// *tile cell*); a tile whose plain image is missing or unreadable has no
    /// image and no tile cells (S-0010 claims 3 and 7). Returns importer
    /// warnings too.
    pub fn tile_catalogue(
        &self,
        images: &[TileImages],
        read: &dyn Fn(&[u8]) -> Option<Vec<u8>>,
    ) -> (TileCatalogue, Vec<String>) {
        let mut warnings = Vec::new();
        let mut tiles = Vec::with_capacity(images.len());
        let mut allow_tiberium = Vec::with_capacity(images.len());
        for (t, img) in images.iter().enumerate() {
            let mut info = TileInfo::default();
            if let Some(name) = &img.plain {
                let show = String::from_utf8_lossy(name).into_owned();
                match read(name).map(|b| tile_info(&b)) {
                    Some(Ok(mut i)) => {
                        i.variants = u8::try_from(img.variants.len()).unwrap_or(u8::MAX);
                        info = i;
                    }
                    Some(Err(e)) => warnings.push(format!(
                        "tile {t}: {show} cannot be read ({e}); the tile has no image (S-0010 claim 7)"
                    )),
                    None => warnings.push(format!(
                        "tile {t}: {show} cannot be read; the tile has no image (S-0010 claim 7)"
                    )),
                }
            }
            let set = self
                .tilesets
                .iter()
                .find(|s| (s.first..s.first.saturating_add(s.count.max(0))).contains(&(t as i32)));
            // spec: S-0010 claim 17 (the tile's tileset's allow-burrowing flag)
            info.no_burrowing = set.is_some_and(|s| !s.allow_burrowing);
            // spec: S-0010 claim 16 (the tile's tileset's allow-tiberium flag)
            allow_tiberium.push(set.is_some_and(|s| s.allow_tiberium));
            // spec: S-0010 claim 19 (a tile's animation, from its tileset's animation section)
            info.anim = set.and_then(|s| {
                s.anims
                    .get((t as i32 - s.first) as usize)
                    .cloned()
                    .flatten()
            });
            tiles.push(info);
        }
        (
            TileCatalogue {
                tiles,
                roles: self.roles,
                allow_tiberium,
                bridge_positions: self.bridge_positions,
            },
            warnings,
        )
    }

    /// [`Catalogue::images`] and [`Catalogue::tile_catalogue`] through the
    /// game's search order (S-0005).
    pub async fn tile_catalogue_from(&self, vfs: &Vfs) -> (TileCatalogue, Vec<String>) {
        let images = self.images(&|n| vfs.resolve_bytes(n).is_some());
        let mut bytes: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        for name in images.iter().filter_map(|i| i.plain.as_ref()) {
            if bytes.contains_key(name) {
                continue;
            }
            if let Some(r) = vfs.resolve_bytes(name)
                && let Ok(b) = vfs.read_resolved(&r).await
            {
                bytes.insert(name.clone(), b);
            }
        }
        self.tile_catalogue(&images, &|n| bytes.get(n).cloned())
    }
}

/// The bytes of a tile animation's name that are read (S-0010 Import
/// mapping).
const ANIM_NAME_MAX: usize = 127; // spec: S-0010 Import mapping (the animation name: text, limit 127 bytes)

/// The tile animations of a tileset (S-0010 claim 19): read from the section
/// named by its set name, keys `TileNNAnim` and `TileNNAttachesTo` with NN
/// the tile's number within the set from 01, at least two digits, with the
/// displacement keys `TileNNXOffset` and `TileNNYOffset`. An empty or absent
/// animation name gives none; an absent attaches-to key −1, an absent
/// displacement 0.
fn tile_anims(doc: &Document, set_name: &[u8], count: i32) -> Vec<Option<TileAnim>> {
    let section = doc.section(set_name);
    (1..=count.max(0))
        .map(|k| {
            let section = section?;
            // spec: S-0010 Import mapping (TileNNAnim, NN from 01 with at least two digits)
            let name = section.value(format!("Tile{k:02}Anim").as_bytes())?;
            let name = value::text(name, ANIM_NAME_MAX);
            if name.is_empty() {
                return None;
            }
            // spec: S-0010 Import mapping (TileNNAttachesTo absent: −1, the animation never starts)
            let attaches_to = section
                .value(format!("Tile{k:02}AttachesTo").as_bytes())
                .and_then(value::integer)
                .unwrap_or(-1);
            // spec: S-0010 Import mapping (TileNNXOffset, TileNNYOffset: the displacement in pixels, absent 0)
            let offset = |axis: &str| {
                section
                    .value(format!("Tile{k:02}{axis}Offset").as_bytes())
                    .and_then(value::integer)
                    .unwrap_or(0)
            };
            Some(TileAnim {
                name: crate::names::encode(name),
                attaches_to,
                dx: offset("X"),
                dy: offset("Y"),
            })
        })
        .collect()
}

/// A tile's size and cells from its tile file.
fn tile_info(bytes: &[u8]) -> Result<TileInfo, String> {
    let tmp = glasnost_fmt::tmp::Tmp::parse(bytes).map_err(|e| e.to_string())?;
    let size = |v: u32| {
        u8::try_from(v).map_err(|_| format!("a template of {v} cells across is too large"))
    };
    Ok(TileInfo {
        width: size(tmp.cells_x)?,
        height: size(tmp.cells_y)?,
        has_image: true,
        cells: tmp
            .tiles
            .iter()
            .map(|c| {
                c.map(|c| TileCell {
                    terrain_code: c.terrain_type,
                    slope: c.ramp_type,
                })
            })
            .collect(),
        variants: 0,
        anim: None,
        no_burrowing: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_fmt::tmp::{TmpTile, TmpWriter, WriteTile};
    use glasnost_ini::Quirks;

    fn doc(text: &str) -> Document {
        Document::parse(text.as_bytes(), Quirks::default(), &mut Vec::new())
    }

    /// spec: S-0010 claim 13 (image P mod (n + 1); P4 for at most 4 images; origin divided by the size, the origin cell included)
    #[test]
    fn claim_13_variant_choice() {
        let p8 = [7u8; 64];
        // One-cell tile with 2 images: P4 at (x mod 4, y mod 4), mod 2.
        assert_eq!(variant_image((5, 2), 0, (1, 1), 1, &p8, false), 3 % 2);
        assert_eq!(variant_image((6, 7), 0, (1, 1), 3, &p8, false), 3);
        // More than 4 images: the 8 × 8 pattern.
        assert_eq!(variant_image((6, 7), 0, (1, 1), 4, &p8, false), 7 % 5);
        // A 2 × 2 tile placed with its origin at (6, 9): sub-tile 3 is cell (7, 10), X = 3, Y = 4.
        assert_eq!(
            variant_image((7, 10), 3, (2, 2), 3, &p8, false),
            usize::from(P4[0][3])
        );
        // The origin cell uses the divided coordinates (3, 4) like the rest of its tile.
        assert_eq!(
            variant_image((6, 9), 0, (2, 2), 3, &p8, false),
            usize::from(P4[0][3])
        );
        // An origin beyond the low edge: truncation toward zero, non-negative remainders.
        assert_eq!(
            variant_image((0, 0), 1, (2, 1), 3, &p8, false),
            usize::from(P4[0][0])
        );
    }

    /// spec: S-0010 claim 13, Terms and Import mapping (the alternate-image
    /// mark is the bit of value 4 of a tile cell's flag value, whatever its
    /// other bits; a marked cell uses no pattern and, its state being clear
    /// at map load, shows the plain image; image count up to 4 uses P4)
    #[test]
    fn claim_13_alternate_image_mark() {
        let p8 = [7u8; 64];
        assert!(alternate_image_mark(4));
        assert!(alternate_image_mark(4 | 1 | 2 | 8));
        assert!(!alternate_image_mark(8));
        assert!(!alternate_image_mark(1 | 2));
        // An 8 × 8 block of a one-cell tile with 4 images: some cells would show image 1 by P4.
        let mut patterned = Vec::new();
        for y in 0..8 {
            for x in 0..8 {
                patterned.push(variant_image((x, y), 0, (1, 1), 3, &p8, false));
                assert_eq!(variant_image((x, y), 0, (1, 1), 3, &p8, true), 0);
            }
        }
        assert!(patterned.contains(&1));
        // Four variants (five images) use the 8 × 8 pattern, never table P4.
        assert_eq!(variant_image((0, 0), 0, (1, 1), 4, &p8, false), 7 % 5);
        assert_eq!(variant_image((0, 0), 0, (1, 1), 4, &p8, true), 0);
    }

    /// spec: S-0010 claim 14 (entries 0 … 7, no clash with earlier index neighbours, 64 … 4096 draws)
    #[test]
    fn claim_14_pattern() {
        let mut x = 12345u32;
        let (p, taken) = variant_pattern(|| {
            x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
            x >> 8
        });
        assert!((64..=4096).contains(&taken));
        for k in 0..64i32 {
            assert!(p[k as usize] < 8);
            for d in PATTERN_NEIGHBOURS {
                let n = (k + d).rem_euclid(64);
                if n < k {
                    assert_ne!(p[k as usize], p[n as usize], "entry {k}");
                }
            }
        }
        // A stream of one value: every entry after the first clashes 64 times and takes the 64th.
        let (p, taken) = variant_pattern(|| 3);
        assert_eq!(p, [3; 64]);
        assert_eq!(taken, 1 + 63 * 64);
    }

    /// spec: S-0010 claim 3 (variants each found with the suffix fallback, stopping at the first gap)
    #[test]
    fn claim_3_variants() {
        let d = doc("[General]\n[TileSet0000]\nFileName=aa\nTilesInSet=1\n");
        let c = Catalogue::read(&d, Theater::Temperate);
        let have = [&b"aa01.TEM"[..], b"aa01a.MMT", b"aa01b.TEM", b"aa01d.TEM"];
        let i = c.tile_images(0, &|n| have.contains(&n));
        assert_eq!(i.plain.as_deref(), Some(&b"aa01.TEM"[..]));
        assert_eq!(
            i.variants,
            vec![b"aa01a.MMT".to_vec(), b"aa01b.TEM".to_vec()]
        );
    }

    /// spec: S-0009 claim 6 (base tiles), claim 8 (identity translation, clear-tile rule); S-0010 claim 4
    #[test]
    fn base_tiles_and_clear_rule() {
        let d = doc("[General]\nClearTile=1\nWaterSet=2\n\
             [TileSet0000]\nFileName=aa\nTilesInSet=3\n\
             [TileSet0001]\nFileName=bb\nTilesInSet=2\n\
             [TileSet0002]\nFileName=cc\nTilesInSet=4\n");
        let c = Catalogue::read(&d, Theater::Temperate);
        assert_eq!(c.tile_count, 9);
        assert_eq!((c.clear_base, c.water_base), (3, 5));
        assert_eq!(c.translate(7), 7);
        assert_eq!(c.art_tile(65535), Some(3));
        assert_eq!(c.art_tile(9), Some(3));
        assert_eq!(c.art_tile(-1), Some(3));
        assert_eq!(c.art_tile(8), Some(8));
        assert_eq!(c.file_of(4), Some(b"bb02.TEM".to_vec()));
        // spec: S-0010 claim 4 (theater 5: the water role is forced unset)
        assert_eq!(Catalogue::read(&d, Theater::Lunar).water_base, -1);
        // spec: S-0010 claim 4 (quirks.theater_five_forced_unset_roles off: the declared value)
        let off = TheaterQuirks {
            theater_five_forced_unset_roles: false,
            ..TheaterQuirks::default()
        };
        assert_eq!(Catalogue::read_with(&d, Theater::Lunar, &off).water_base, 5);
        let none = Catalogue::read(
            &doc("[TileSet0000]\nFileName=aa\nTilesInSet=1\n"),
            Theater::Snow,
        );
        assert_eq!((none.clear_base, none.water_base), (-1, -1));
    }

    /// spec: S-0010 claim 16 and S-0401 Import mapping (the tileset's AllowTiberium flag; absent: no)
    #[test]
    fn claim_16_allow_tiberium() {
        let d =
            doc("[TileSet0000]\nTilesInSet=2\nAllowTiberium=yes\n[TileSet0001]\nTilesInSet=1\n");
        let c = Catalogue::read(&d, Theater::Temperate);
        let images = c.images(&|_| false);
        let (cat, _) = c.tile_catalogue(&images, &|_| None);
        assert_eq!(cat.allow_tiberium, vec![true, true, false]);
    }

    /// spec: S-0010 claim 19 and Import mapping (tile animations from the section named by the set name: TileNNAnim, TileNNAttachesTo, absent −1; an empty name gives none)
    #[test]
    fn claim_19_tile_animations() {
        let d = doc(
            "[TileSet0000]\nTilesInSet=1\n[TileSet0001]\nTilesInSet=3\nSetName=Foo\n\
             [Foo]\nTile01Anim=Spark\nTile02Anim=Glow\nTile02AttachesTo=2\nTile02XOffset=-7\n\
             Tile02YOffset=12\nTile03Anim=\n",
        );
        let c = Catalogue::read(&d, Theater::Temperate);
        let a = &c.tilesets[1].anims;
        assert_eq!(a.len(), 3);
        assert_eq!(
            a[0].as_ref().map(|x| (x.name.as_str(), x.attaches_to)),
            Some(("Spark", -1))
        );
        assert_eq!(
            a[1].as_ref().map(|x| (x.name.as_str(), x.attaches_to)),
            Some(("Glow", 2))
        );
        assert_eq!(a[2], None);
        // spec: S-0010 Import mapping (absent displacements 0)
        assert_eq!(a[0].as_ref().map(|x| (x.dx, x.dy)), Some((0, 0)));
        assert_eq!(a[1].as_ref().map(|x| (x.dx, x.dy)), Some((-7, 12)));
        let images = c.images(&|_| false);
        let (cat, _) = c.tile_catalogue(&images, &|_| None);
        assert_eq!(cat.tiles[0].anim, None);
        assert_eq!(cat.tiles[2].anim.as_ref().map(|x| x.attaches_to), Some(2));
    }

    /// spec: S-0010 claim 17 and Import mapping (`AllowBurrowing`, boolean, absent: yes; carried to each tile of the set)
    #[test]
    fn claim_17_allow_burrowing() {
        let d = doc(
            "[TileSet0000]\nTilesInSet=1\n[TileSet0001]\nTilesInSet=2\nAllowBurrowing=no\n\
             [TileSet0002]\nTilesInSet=1\nAllowBurrowing=yes\n",
        );
        let c = Catalogue::read(&d, Theater::Temperate);
        let flags: Vec<bool> = c.tilesets.iter().map(|s| s.allow_burrowing).collect();
        assert_eq!(flags, [true, false, true]);
        let images = c.images(&|_| false);
        let (cat, _) = c.tile_catalogue(&images, &|_| None);
        let refused: Vec<bool> = cat.tiles.iter().map(|t| t.no_burrowing).collect();
        assert_eq!(refused, [false, true, true, false]);
    }

    /// spec: S-0010 claim 18 and Import mapping (`ShadowCaster`, boolean, absent: no;
    /// `ShadowTiles`, integer, absent: 0; a tileset casts shadows only with the flag set and a
    /// count other than 0)
    #[test]
    fn claim_18_shadow_casting_tilesets() {
        let d = doc(
            "[TileSet0000]\nTilesInSet=1\n\
             [TileSet0001]\nTilesInSet=1\nShadowCaster=yes\nShadowTiles=3\n\
             [TileSet0002]\nTilesInSet=1\nShadowCaster=yes\n\
             [TileSet0003]\nTilesInSet=1\nShadowCaster=yes\nShadowTiles=0\n\
             [TileSet0004]\nTilesInSet=1\nShadowCaster=no\nShadowTiles=3\n\
             [TileSet0005]\nTilesInSet=1\nShadowTiles=2\n\
             [TileSet0006]\nTilesInSet=1\nShadowCaster=yes\nShadowTiles=-1\n",
        );
        let c = Catalogue::read(&d, Theater::Temperate);
        let read: Vec<(bool, i32)> = c
            .tilesets
            .iter()
            .map(|s| (s.shadow_caster, s.shadow_tiles))
            .collect();
        assert_eq!(
            read,
            [
                (false, 0),
                (true, 3),
                (true, 0),
                (true, 0),
                (false, 3),
                (false, 2),
                (true, -1)
            ]
        );
        let casts: Vec<bool> = c.tilesets.iter().map(Tileset::casts_shadows).collect();
        assert_eq!(casts, [false, true, false, false, false, false, true]);
    }

    /// spec: S-0009 Formulas (tile number translation)
    #[test]
    fn translation() {
        // Set 0: C = 3; set 1: C = 5 but authored A = 2; set 2: C = 4.
        let d = doc(
            "[TileSet0000]\nTilesInSet=3\n[TileSet0001]\nTilesInSet=5\nLastTilesInSet=2\n\
             [TileSet0002]\nTilesInSet=4\n",
        );
        let c = Catalogue::read(&d, Theater::Temperate);
        // Authoring space: set 1 starts at 3, set 2 at 5; t_1 = 3 + 2 = 5, δ_1 = 3.
        assert_eq!(c.translation, vec![(5, 3)]);
        assert_eq!(c.translate(4), 4);
        assert_eq!(c.translate(5), 8);
        assert_eq!(c.translate(65535), 65535);
    }

    /// spec: S-0010 claim 1, table T
    #[test]
    fn data_files() {
        assert_eq!(data_file_names(Theater::Temperate), ["TEMPERATMD.INI"]);
        assert_eq!(data_file_names(Theater::NewUrban), ["URBANNMD.INI"]);
        let c = Catalogue::read(&doc(""), Theater::NewUrban);
        assert_eq!(c.extension, "UBN");
        assert_eq!(c.fallback, ["MMT", "TEM", "URB"]);
    }

    /// spec: S-0010 claim 2 (declaration ends at a missing count or −1; negative counts contribute none)
    #[test]
    fn tileset_declaration() {
        let d = doc(
            "[TileSet0000]\nTilesInSet=2\nFileName=abcd\n[TileSet0001]\nTilesInSet=0\n\
             [TileSet0002]\nTilesInSet=-3\n[TileSet0003]\nTilesInSet=4\n\
             [TileSet0004]\nFileName=x\n[TileSet0005]\nTilesInSet=9\n",
        );
        let c = Catalogue::read(&d, Theater::Temperate);
        assert_eq!(c.tilesets.len(), 4);
        assert_eq!(c.tile_count, 6);
        assert_eq!(c.tilesets[3].first, 2);
        assert!(c.warnings.iter().any(|w| w.contains("TileSet0002")));
        // Defaults of the name keys.
        assert_eq!(c.tilesets[1].file_name, b"TILE");
        assert_eq!(c.tilesets[1].set_name, b"No Name");
        assert_eq!(c.tilesets[0].non_height_view, 65535);
        // An explicit −1 ends the list too; a higher section has no effect.
        let d = doc("[TileSet0000]\nTilesInSet=-1\n[TileSet0001]\nTilesInSet=5\n");
        assert_eq!(Catalogue::read(&d, Theater::Temperate).tile_count, 0);
    }

    /// spec: S-0511 Import mapping (the ten bridge positions; absent: −1)
    #[test]
    fn bridge_positions() {
        let d = doc(
            "[General]\nBridgeTopLeft1=3\nBridgeTopLeft2=4\nBridgeTopRight1=5\n\
             BridgeBottomRight1=7\nBridgeBottomLeft2=10\nBridgeMiddle1=11\nBridgeMiddle2=12\n",
        );
        let p = Catalogue::read(&d, Theater::Temperate).bridge_positions;
        assert_eq!(p.top_left, [3, 4]);
        assert_eq!(p.top_right, [5, -1]);
        assert_eq!(p.bottom_right, [7, -1]);
        assert_eq!(p.bottom_left, [-1, 10]);
        assert_eq!(p.middle, [11, 12]);
        let none = Catalogue::read(&doc("[General]\n"), Theater::Temperate).bridge_positions;
        assert_eq!(none, BridgePositions::default());
        assert_eq!(none.middle, [-1, -1]);
    }

    /// spec: S-0010 claim 4 (role starts, unset values, one past the end, theater 5)
    #[test]
    fn roles() {
        let d = doc(
            "[General]\nClearTile=0\nRampBase=1\nRampSmooth=2\nCliffSet=3\nShorePieces=1\n\
             WaterBridge=1\nBridgeSet=1\nWoodBridgeSet=1\nWaterSet=1\nTunnels=1\nDirtTrackTunnels=-4\n\
             PavedRoads=9\nClearToRoughLat=2\n\
             [TileSet0000]\nTilesInSet=2\n[TileSet0001]\nTilesInSet=5\n[TileSet0002]\nTilesInSet=1\n",
        );
        let c = Catalogue::read(&d, Theater::Temperate);
        let r = c.roles;
        assert_eq!((r.clear, r.ramp, r.smooth_ramp), (0, 2, 7));
        // One past the end: the tile count.
        assert_eq!(r.cliff, 8);
        assert_eq!(r.rough_transition, 7);
        // Beyond: unset; negative: unset with a warning; absent: unset.
        assert_eq!(r.paved_road, -1);
        assert_eq!(r.tunnels, [2, -1, -1, -1]);
        assert!(c.warnings.iter().any(|w| w.contains("DirtTrackTunnels=-4")));
        assert_eq!(r.median, -1);
        assert_eq!(r.destroyable_cliff, -2);
        assert_eq!((r.shore, r.water, r.bridge), (2, 2, 2));
        // Switch off: one past the end is unset.
        let off = Catalogue::read_with(
            &d,
            Theater::Temperate,
            &TheaterQuirks {
                role_one_past_end: false,
                ..Default::default()
            },
        );
        assert_eq!(off.roles.cliff, -1);
        // Theater 5 forces six roles unset.
        let lunar = Catalogue::read(&d, Theater::Lunar).roles;
        assert_eq!(
            [
                lunar.bridge,
                lunar.cliff,
                lunar.shore,
                lunar.water,
                lunar.water_bridge,
                lunar.wooden_bridge
            ],
            [-1; 6]
        );
        assert_eq!((lunar.ramp, lunar.tunnels[0]), (2, 2));
    }

    fn header(terrain: u8, ramp: u8) -> TmpTile {
        TmpTile {
            offset: 0,
            x: 0,
            y: 0,
            extra_offset: 0,
            depth_offset: 0,
            extra_depth_offset: 0,
            extra_x: 0,
            extra_y: 0,
            extra_width: 0,
            extra_height: 0,
            flags: 0,
            height: 0,
            terrain_type: terrain,
            ramp_type: ramp,
            radar_left: [0; 3],
            radar_right: [0; 3],
        }
    }

    /// A synthetic tile file of `w` × `h` cells; `None` marks an empty cell.
    fn tmp(w: u32, h: u32, cells: &[Option<(u8, u8)>]) -> Vec<u8> {
        TmpWriter {
            cells_x: w,
            cells_y: h,
            tile_width: 4,
            tile_height: 2,
            tiles: cells
                .iter()
                .map(|c| {
                    c.map(|(t, r)| WriteTile {
                        header: header(t, r),
                        diamond: vec![1; 4],
                        depth: None,
                        extra: None,
                    })
                })
                .collect(),
        }
        .write()
    }

    /// spec: S-0010 claim 3 (file names, variants, the gap rule and the fallback), Terms (tile cells)
    #[test]
    fn tile_files_and_cells() {
        let d = doc("[General]\nClearTile=0\n\
             [TileSet0000]\nTilesInSet=3\nFileName=gr\n\
             [TileSet0001]\nTilesInSet=101\nFileName=big\nNonMarbleMadness=0\n");
        let c = Catalogue::read(&d, Theater::Snow);
        assert_eq!(c.image_name(102, 0, "SNO"), Some(b"big100.SNO".to_vec()));
        assert_eq!(c.image_name(1, 2, "SNO"), Some(b"gr02b.SNO".to_vec()));
        let files: BTreeMap<Vec<u8>, Vec<u8>> = [
            ("GR01.SNO", tmp(2, 1, &[Some((9, 0)), None])),
            ("gr01a.sno", tmp(1, 1, &[Some((0, 0))])),
            ("gr01c.sno", tmp(1, 1, &[Some((0, 0))])),
            // Tile 1 only in the fallback suffix's file; tile 2 only in the last resort.
            ("gr02.mms", tmp(1, 1, &[Some((11, 3))])),
            ("gr03.urb", tmp(1, 1, &[Some((6, 0))])),
            // Tile 4 of the set without the fallback exists only with another suffix.
            ("big02.mms", tmp(1, 1, &[Some((1, 0))])),
            ("big03.sno", b"not a tile file".to_vec()),
        ]
        .into_iter()
        .map(|(n, b)| (n.to_ascii_lowercase().into_bytes(), b))
        .collect();
        let exists = |n: &[u8]| files.contains_key(&n.to_ascii_lowercase());
        let images = c.images(&exists);
        assert_eq!(images.len(), 104);
        assert_eq!(images[0].plain, Some(b"gr01.SNO".to_vec()));
        // The gap rule: `c` without `b` is never used.
        assert_eq!(images[0].variants, [b"gr01a.SNO".to_vec()]);
        assert_eq!(images[1].plain, Some(b"gr02.MMS".to_vec()));
        assert_eq!(images[2].plain, Some(b"gr03.URB".to_vec()));
        assert_eq!(images[4].plain, None);
        let read = |n: &[u8]| files.get(&n.to_ascii_lowercase()).cloned();
        let (cat, warnings) = c.tile_catalogue(&images, &read);
        assert_eq!(cat.tile_count(), 104);
        let t0 = &cat.tiles[0];
        assert_eq!(
            (t0.width, t0.height, t0.has_image, t0.variants),
            (2, 1, true, 1)
        );
        assert_eq!(
            t0.cells,
            vec![
                Some(TileCell {
                    terrain_code: 9,
                    slope: 0
                }),
                None
            ]
        );
        assert_eq!(
            cat.tile_cell(1, 0).map(|c| (c.terrain_code, c.slope)),
            Some((11, 3))
        );
        assert_eq!(cat.tile_cell(0, 1), None);
        // No image: a tile number with no cells.
        assert!(!cat.tiles[4].has_image && cat.tiles[4].cells.is_empty());
        // An unreadable file: no image, with a warning.
        assert!(!cat.tiles[5].has_image);
        assert!(warnings.iter().any(|w| w.contains("big03.SNO")));
        assert_eq!(cat.roles.clear, 0);
    }
}
