//! Palettes and house colour (S-0103).
//!
//! - Palette files become palettes by multiplying each component by 4 in
//!   8-bit arithmetic ([`palette_from_file`], claim 1).
//! - [`PaletteState`] holds the unit, tile, theater and anim palettes in
//!   force and reloads them when a game starts in a different theater, with
//!   the original's rules for missing files (claims 2 and 3).
//! - A colour definition (hue, saturation, value) becomes the 16-entry ramp
//!   that replaces the remap range ([`ramp`], [`house_palette`], claim 6),
//!   and a lobby colour slot picks a definition position ([`slot_position`],
//!   claim 10).
//! - [`palette_for`] says which palette each kind of drawn object uses
//!   (claim 8).
//!
//! Shading a palette by light into rows of hi-colour pixels is S-0104 and
//! is not done here: palettes here are the unshaded 8-bit colours.

use std::ops::RangeInclusive;

use glasnost_fmt::image::{IndexedImage, Palette, Rgb8};
use glasnost_fmt::pal::Palette6;
use glasnost_vfs::theater::Theater;

/// Palette indices replaced by a house colour (S-0103 Constants).
pub const REMAP_RANGE: RangeInclusive<u8> = 16..=31; // spec: S-0103 Constants (remap range)

/// Indices of house-colour palettes that ignore the light's colour
/// (S-0103 claim 7). Tile palettes have none.
pub const TINT_EXEMPT: RangeInclusive<u8> = 240..=254; // spec: S-0103 claim 7

/// Shade rows of the two house-colour palettes of a colour definition
/// (S-0103 claim 5; the rows themselves are S-0104).
pub const SHADE_COUNTS: [usize; 2] = [1, 53]; // spec: S-0103 Constants (shade counts)

/// Definition position shown by each lobby colour slot 0 to 7 (S-0103
/// claim 10).
pub const SLOT_POSITIONS: [usize; 8] = [1, 5, 10, 14, 6, 12, 8, 7]; // spec: S-0103 claim 10

/// The anim palette's file (S-0103 Constants).
pub const ANIM_PALETTE_FILE: &str = "ANIM.PAL"; // spec: S-0103 Constants

/// Quirk switches of S-0103 implemented here. Each defaults to the
/// original's behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteQuirks {
    /// `quirks.palette_component_wrap`: a file component above 63 wraps
    /// after the multiplication by 4; off, it becomes 252.
    pub palette_component_wrap: bool,
    /// `quirks.missing_unit_palette_remultiplied`: a missing unit palette
    /// file multiplies the unit palette in force by 4 again; off, it leaves
    /// it unchanged.
    pub missing_unit_palette_remultiplied: bool,
    /// `quirks.alt_palette_first_definition`: animations flagged for the
    /// alternative palette use the first colour definition; off, their
    /// owner's.
    pub alt_palette_first_definition: bool,
    /// `quirks.veins_local_player_colour`: vein animations use the local
    /// player's house colour; off, the anim palette.
    pub veins_local_player_colour: bool,
}

impl Default for PaletteQuirks {
    fn default() -> Self {
        // spec: S-0103 Edge cases (every switch defaults to on)
        Self {
            palette_component_wrap: true,
            missing_unit_palette_remultiplied: true,
            alt_palette_first_definition: true,
            veins_local_player_colour: true,
        }
    }
}

impl PaletteQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 4] {
        [
            ("palette_component_wrap", self.palette_component_wrap),
            (
                "missing_unit_palette_remultiplied",
                self.missing_unit_palette_remultiplied,
            ),
            (
                "alt_palette_first_definition",
                self.alt_palette_first_definition,
            ),
            ("veins_local_player_colour", self.veins_local_player_colour),
        ]
    }
}

/// One file component as a palette component (S-0103 claim 1).
pub fn load_component(c: u8, quirks: &PaletteQuirks) -> u8 {
    if quirks.palette_component_wrap || c <= 63 {
        // spec: S-0103 claim 1, Formulas (component = (4 · c) mod 256)
        c.wrapping_mul(4)
    } else {
        // spec: S-0103 Edge cases (switch off: a component above 63 becomes 252)
        252
    }
}

/// A palette loaded from a palette file (S-0103 claim 1).
pub fn palette_from_file(file: &Palette6, quirks: &PaletteQuirks) -> Palette {
    let mut pal = Palette::default();
    for (dst, src) in pal.colors.iter_mut().zip(file.colors.iter()) {
        let [r, g, b] = src.map(|c| load_component(c, quirks));
        *dst = Rgb8::new(r, g, b);
    }
    pal
}

/// Every component multiplied by 4 again, modulo 256 (S-0103 claim 2:
/// the unit palette when its file is missing).
fn remultiplied(pal: &Palette) -> Palette {
    let mut out = pal.clone();
    for c in &mut out.colors {
        // spec: S-0103 claim 2 (each component c replaced by (4 · c) mod 256)
        *c = Rgb8::new(
            c.r.wrapping_mul(4),
            c.g.wrapping_mul(4),
            c.b.wrapping_mul(4),
        );
    }
    out
}

/// The theater palette in force when its file is missing (S-0103 claim 2):
/// entry *i* = (*i*, 255 − *i*, (4·*i*) mod 256).
fn missing_theater_palette() -> Palette {
    let mut pal = Palette::default();
    for (i, c) in pal.colors.iter_mut().enumerate() {
        let i = i as u8;
        // spec: S-0103 claim 2 (missing theater palette)
        *c = Rgb8::new(i, 255 - i, i.wrapping_mul(4));
    }
    pal
}

/// The palettes in force during a session (S-0103 claims 1 to 3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteState {
    /// Unit palette: units, infantry, aircraft and buildings, and the base
    /// of every house-colour palette.
    pub unit: Palette,
    /// Tile palette: terrain tiles.
    pub tile: Palette,
    /// Theater palette of the loaded theater. No drawing is established to
    /// use it directly (S-0103 Open questions); the ore and gem overlays use
    /// [`PaletteState::resource`], the temperate theater palette read at
    /// program start (S-0103 Terms and claim 8).
    pub theater: Palette,
    /// Anim palette, read once per session.
    pub anim: Palette,
    /// The palette of the start-up resource table: the temperate theater
    /// palette as read once at program start (S-0112 Terms, claim 2;
    /// S-0103 Terms: later theater loads do not change it); all
    /// black until [`PaletteState::set_startup_resource`].
    pub resource: Palette,
    /// The palette of the start-up mine table: the snow unit palette file as
    /// read once at program start (S-0112 Terms, claim 2).
    pub mine: Palette,
    loaded: Option<Theater>,
    quirks: PaletteQuirks,
}

impl PaletteState {
    /// The state at program start (S-0103 State and ordering): the unit
    /// palette is the snow theater's unit palette file, the tile palette is
    /// all black, and the anim palette is read from its file.
    ///
    /// What the original does when the snow unit palette or the anim
    /// palette file is missing at program start is not established (S-0103
    /// Open questions); a missing file gives an all-black palette here, and
    /// the caller should say so. The theater palette before the first load
    /// is not stated either and starts all black.
    pub fn at_program_start(
        snow_unit: Option<&Palette6>,
        anim: Option<&Palette6>,
        quirks: PaletteQuirks,
    ) -> Self {
        let load =
            |f: Option<&Palette6>| f.map(|f| palette_from_file(f, &quirks)).unwrap_or_default();
        Self {
            unit: load(snow_unit),
            // spec: S-0103 claim 2 (the tile palette in force before the first load is all black)
            tile: Palette::default(),
            theater: Palette::default(),
            anim: load(anim),
            resource: Palette::default(),
            // spec: S-0112 Terms (the start-up mine table: the snow unit palette file × 4, once)
            mine: load(snow_unit),
            loaded: None,
            quirks,
        }
    }

    /// Reads the start-up resource table's palette, the temperate theater
    /// palette file × 4, once at program start (S-0112 Terms, claim 2).
    pub fn set_startup_resource(&mut self, temperate: Option<&Palette6>) {
        // spec: S-0112 Terms (start-up resource table: the temperate theater palette × 4 by
        // S-0103 claim 1)
        self.resource = temperate
            .map(|f| palette_from_file(f, &self.quirks))
            .unwrap_or_default();
    }

    /// The theater whose palettes are loaded, if any.
    pub fn loaded_theater(&self) -> Option<Theater> {
        self.loaded
    }

    /// A game starts in `theater` (S-0103 claim 3): the unit, tile and
    /// theater palettes are reloaded only when the theater differs from the
    /// previous game's (or none is loaded yet). `file` returns the palette
    /// file of a name, or `None` when it cannot be resolved.
    pub fn start_game(&mut self, theater: Theater, file: impl Fn(&str) -> Option<Palette6>) {
        // spec: S-0103 claim 3 (same theater as the previous game: no palette file is read)
        if self.loaded == Some(theater) {
            return;
        }
        let q = self.quirks;
        // spec: S-0103 claim 2 (missing unit palette: the one in force, multiplied by 4 again)
        self.unit = match file(&theater.unit_palette()) {
            Some(f) => palette_from_file(&f, &q),
            None if q.missing_unit_palette_remultiplied => remultiplied(&self.unit),
            None => self.unit.clone(),
        };
        // spec: S-0103 claim 2 (missing tile palette: the one in force, unchanged)
        if let Some(f) = file(&theater.tile_palette()) {
            self.tile = palette_from_file(&f, &q);
        }
        // spec: S-0103 claim 2 (missing theater palette: the fixed formula)
        self.theater = match file(&theater.theater_palette()) {
            Some(f) => palette_from_file(&f, &q),
            None => missing_theater_palette(),
        };
        self.loaded = Some(theater);
    }

    /// A named palette file loaded as a palette (S-0103 claim 17: its
    /// components receive claim 1's multiplication and byte wrap).
    pub fn load_named(&self, file: &Palette6) -> Palette {
        // spec: S-0103 claim 17 (the named file's components receive claim 1's multiplication and byte wrap)
        palette_from_file(file, &self.quirks)
    }
}

/// The longest art `Palette` value that selects a file: the bytes after it
/// are ignored (S-0103 Import mapping).
pub const NAMED_PALETTE_MAX: usize = 31; // spec: S-0103 Import mapping (bytes after the 31st are ignored)

/// The file a type's art `Palette` value `value` names in `theater`
/// (S-0103 claim 17 and Import mapping), upper-case: its first 31 bytes,
/// the theater's suffix *X* and `.PAL`, appended even to a value that
/// already ends in a file extension. `None` for an empty value, which
/// names no palette.
pub fn named_palette_file(value: &[u8], theater: Theater) -> Option<Vec<u8>> {
    // spec: S-0103 Import mapping (absent or empty: no named palette)
    if value.is_empty() {
        return None;
    }
    // spec: S-0103 Import mapping (bytes after the 31st are ignored)
    let mut name = value[..value.len().min(NAMED_PALETTE_MAX)].to_ascii_uppercase();
    // spec: S-0103 claim 17 (the value followed by the current theater's suffix X and .PAL; always appended)
    name.extend_from_slice(theater.extension().as_bytes());
    name.extend_from_slice(b".PAL");
    Some(name)
}

/// Single-precision σ_0 … σ_15 of S-0103 Constants, as bit patterns.
// spec: S-0103 Constants (σ, the original's single-precision values)
const SIGMA_BITS: [u32; 16] = [
    0x3F44_22AA,
    0x3F4B_7F08,
    0x3F52_89B4,
    0x3F59_1E6A,
    0x3F5F_397A,
    0x3F64_C0EE,
    0x3F69_E0D7,
    0x3F6E_7DB7,
    0x3F72_84E4,
    0x3F76_167A,
    0x3F79_1E30,
    0x3F7B_9107,
    0x3F7D_8284,
    0x3F7E_E5F9,
    0x3F7F_B848,
    0x3F80_0000,
];

/// Single-precision γ_0 … γ_15 of S-0103 Constants, as bit patterns.
// spec: S-0103 Constants (γ, the original's single-precision values)
const GAMMA_BITS: [u32; 16] = [
    0x3F7B_14BE,
    0x3F68_AA48,
    0x3F5F_397A,
    0x3F54_330F,
    0x3F47_DE65,
    0x3F3A_37B6,
    0x3F2B_561A,
    0x3F1B_52BB,
    0x3F0A_1E5D,
    0x3EF0_50C4,
    0x3ECA_CE62,
    0x3EA3_F505,
    0x3E78_0CBD,
    0x3E25_C58B,
    0x3DA6_522F,
    0x250D_3000,
];

/// ⌊σ · x⌋ for a positive normal single σ (given by its bits) and a byte
/// `x`, in exact integer arithmetic (S-0103 Formulas: the product is exact
/// in double precision and truncated).
fn floor_scaled(bits: u32, x: u8) -> u8 {
    let mantissa = u64::from(bits & 0x007F_FFFF) | 0x0080_0000;
    let exponent = i64::from((bits >> 23) & 0xFF) - 127 - 23;
    let product = mantissa * u64::from(x);
    let v = if exponent >= 0 {
        product << exponent
    } else if exponent <= -64 {
        0
    } else {
        product >> (-exponent)
    };
    // σ and γ are at most 1, so the result is at most `x`.
    v.min(255) as u8
}

/// The integer HSV-to-RGB conversion of S-0103 Formulas.
pub fn hsv_to_rgb(h: u8, s: u8, v: u8) -> Rgb8 {
    let (h, s, v) = (u32::from(h), u32::from(s), u32::from(v));
    // spec: S-0103 Formulas (integer HSV to RGB; hue scale 255 per circle, six sectors)
    let f = (6 * h) % 255;
    let k = (6 * h) / 255;
    let p = ((255 - s) * v) / 255;
    let q = ((255 - (f * s) / 255) * v) / 255;
    let t = ((255 - ((255 - f) * s) / 255) * v) / 255;
    let (r, g, b) = match k {
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        // k = 0, or 6 for H = 255, which is sector 0.
        _ => (v, t, p),
    };
    Rgb8::new(r as u8, g as u8, b as u8)
}

/// The 16-entry house-colour ramp of a colour definition (S-0103 claim 6):
/// entry *i* has saturation ⌊σ_i·S⌋ and value ⌊γ_i·V⌋; entry 0 is the
/// brightest and entry 15 is always black.
pub fn ramp(definition: [u8; 3]) -> [Rgb8; 16] {
    let [h, s, v] = definition;
    std::array::from_fn(|i| {
        // spec: S-0103 claim 6, Formulas (S_i = ⌊σ_i · S⌋, V_i = ⌊γ_i · V⌋)
        let si = floor_scaled(SIGMA_BITS[i], s);
        let vi = floor_scaled(GAMMA_BITS[i], v);
        hsv_to_rgb(h, si, vi)
    })
}

/// A house-colour palette: `unit` with the remap range replaced by the
/// ramp of `definition` (S-0103 Terms and claim 6).
pub fn house_palette(unit: &Palette, definition: [u8; 3]) -> Palette {
    let mut pal = unit.clone();
    for (i, c) in REMAP_RANGE.zip(ramp(definition)) {
        pal.colors[usize::from(i)] = c;
    }
    pal
}

/// The definition position a player house in lobby colour slot `slot`
/// shows (S-0103 claim 10), or `None` for a slot above 7.
pub fn slot_position(slot: u8) -> Option<usize> {
    SLOT_POSITIONS.get(usize::from(slot)).copied()
}

/// The fixed RGB colour of each setup-screen colour slot 0 … 7 (S-0103
/// claim 11 and Constants, "Lobby swatch colours"): engine constants,
/// independent of every colour definition.
// spec: S-0103 Constants (lobby swatch colours, claim 11)
pub const LOBBY_SWATCHES: [Rgb8; 8] = [
    Rgb8::new(221, 226, 13),
    Rgb8::new(255, 25, 25),
    Rgb8::new(42, 116, 226),
    Rgb8::new(62, 209, 46),
    Rgb8::new(255, 160, 25),
    Rgb8::new(50, 215, 230),
    Rgb8::new(149, 40, 189),
    Rgb8::new(255, 154, 235),
];

/// The setup-screen swatch of colour slot `slot` as a 5-6-5 pixel (S-0103
/// claim 11: the slot's fixed RGB reduced directly by the 16-bit step, with
/// no HSV conversion, ramp, palette multiplication, lighting or subtraction
/// of 1), or `None` for a slot above 7.
pub fn lobby_swatch_565(slot: u8) -> Option<u16> {
    LOBBY_SWATCHES
        .get(usize::from(slot))
        .map(|&c| hicolor_565(c))
}

/// A colour in the 16-bit 5-6-5 pixel layout (S-0103 claim 9).
pub fn hicolor_565(c: Rgb8) -> u16 {
    // spec: S-0103 Formulas (red5 = ⌊R/8⌋, green6 = ⌊G/4⌋, blue5 = ⌊B/8⌋)
    (u16::from(c.r >> 3) << 11) | (u16::from(c.g >> 2) << 5) | u16::from(c.b >> 3)
}

/// A colour in the 16-bit 5-5-5 pixel layout (S-0103 claim 9).
pub fn hicolor_555(c: Rgb8) -> u16 {
    // spec: S-0103 Formulas (5-5-5 layout: green5 = ⌊G/8⌋)
    (u16::from(c.r >> 3) << 10) | (u16::from(c.g >> 3) << 5) | u16::from(c.b >> 3)
}

/// The sidebar icon palette's fixed file (S-0103 Import mapping, claim 13).
pub const ICON_PALETTE_FILE: &[u8] = b"CAMEO.PAL"; // spec: S-0103 Import mapping (fixed asset)

/// Rows of the icon palette's untinted shade table (S-0103 Constants).
pub const ICON_SHADE_ROWS: usize = 53; // spec: S-0103 Constants (icon shade count 53)

/// The row an ordinary icon pixel shows (S-0103 claim 13: the middle row).
pub const ICON_ROW: usize = 26; // spec: S-0103 claim 13, Constants (middle row 26)

/// One component of the icon shade table at row `j` (S-0103 Formulas, "Icon
/// shade table"): min(255, ⌊u·F_j / 65536⌋), F_j = ⌊131072·j / 52⌋.
pub fn icon_shade_component(u: u8, j: usize) -> u8 {
    // spec: S-0103 Formulas, "Icon shade table" (F_j = ⌊131072·j / 52⌋)
    let f = 131_072 * j as u64 / 52;
    // spec: S-0103 Formulas, "Icon shade table" (min(255, ⌊u·F_j / 65536⌋))
    (u64::from(u) * f / 65_536).min(255) as u8
}

/// The icon palette as 5-6-5 pixels (S-0103 claim 13): each file
/// component through claim 1's load rule, at the middle shade row (the
/// converted colour exactly, with no subtraction of 1, no house colour, no
/// battlefield light or tint), packed by the 16-bit step. Entry 0 is
/// transparent; its packed value is meaningless. With no file (the
/// original crashes; our engine's departure) every entry is black.
pub fn icon_palette_565(file: Option<&Palette6>, quirks: &PaletteQuirks) -> [u16; 256] {
    let Some(file) = file else {
        // spec: S-0103 claim 13 (our engine's departure: an all-black icon palette)
        return [0; 256];
    };
    let pal = palette_from_file(file, quirks);
    let mut out = [0u16; 256];
    for (dst, c) in out.iter_mut().zip(pal.colors.iter()) {
        // spec: S-0103 claim 13 (the middle row reproduces the converted colour; claim 9's 16-bit step)
        *dst = hicolor_565(Rgb8::new(
            icon_shade_component(c.r, ICON_ROW),
            icon_shade_component(c.g, ICON_ROW),
            icon_shade_component(c.b, ICON_ROW),
        ));
    }
    out
}

/// What is drawn, as far as the choice of palette goes (S-0103 claim 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawnObject {
    /// A terrain tile.
    TerrainTile,
    /// A unit, infantry or aircraft.
    Mobile,
    /// A building; `terrain_palette` is its art flag to use the tile
    /// palette.
    Building {
        /// The building's art flag for the terrain palette.
        terrain_palette: bool,
    },
    /// An animation and the facts claim 8's table looks at.
    Animation {
        /// A vein animation (art flag).
        veins: bool,
        /// Created as part of a cell's terrain.
        terrain: bool,
        /// An object's own animation drawn with its object (claim 8, row 3).
        drawn_with_object: bool,
        /// Flagged for the alternative palette (art flag).
        alt_palette: bool,
    },
}

/// The palette an object is drawn with (S-0103 claim 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteChoice {
    /// The tile palette.
    Tile,
    /// The owner's 53-shade house-colour palette.
    OwnerHouse,
    /// The local player's 53-shade house-colour palette.
    LocalPlayerHouse,
    /// The single-shade house-colour palette of the first colour definition.
    FirstDefinitionSingleShade,
    /// The owner's single-shade house-colour palette (switch off).
    OwnerSingleShade,
    /// The anim palette.
    Anim,
}

/// Which palette `object` uses (S-0103 claim 8, the first row that applies).
pub fn palette_for(object: DrawnObject, quirks: &PaletteQuirks) -> PaletteChoice {
    match object {
        // spec: S-0103 claim 8 (terrain tiles use the tile palette)
        DrawnObject::TerrainTile => PaletteChoice::Tile,
        // spec: S-0103 claim 8 (units, infantry, aircraft and buildings use the owner's palette)
        DrawnObject::Mobile => PaletteChoice::OwnerHouse,
        DrawnObject::Building { terrain_palette } => {
            if terrain_palette {
                PaletteChoice::Tile
            } else {
                PaletteChoice::OwnerHouse
            }
        }
        DrawnObject::Animation {
            veins,
            terrain,
            drawn_with_object,
            alt_palette,
        } => {
            // spec: S-0103 claim 8, rows 1 to 5 in precedence order
            if veins && quirks.veins_local_player_colour {
                PaletteChoice::LocalPlayerHouse
            } else if terrain {
                PaletteChoice::Tile
            } else if drawn_with_object {
                PaletteChoice::OwnerHouse
            } else if alt_palette {
                if quirks.alt_palette_first_definition {
                    PaletteChoice::FirstDefinitionSingleShade
                } else {
                    PaletteChoice::OwnerSingleShade
                }
            } else {
                PaletteChoice::Anim
            }
        }
    }
}

/// Resolves an indexed image to RGBA bytes through `palette`. With
/// `transparent_zero`, index 0 becomes fully transparent (S-0104: index 0
/// is transparent in every shade row; also the sprite and tile formats'
/// convention for "no pixel").
pub fn resolve_rgba(image: &IndexedImage, palette: &Palette, transparent_zero: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(image.pixels().len() * 4);
    for &i in image.pixels() {
        let c = palette.colors[usize::from(i)];
        let a = if transparent_zero && i == 0 { 0 } else { 255 };
        out.extend_from_slice(&[c.r, c.g, c.b, a]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_fmt::image::PALETTE_LEN;

    fn rgb(t: [(u8, u8, u8); 16]) -> [Rgb8; 16] {
        t.map(|(r, g, b)| Rgb8::new(r, g, b))
    }

    /// spec: S-0103 claim 13, Formulas "Icon shade table" and scenario PL (the icon palette:
    /// claim 1's ×4 with byte wrap, the middle row exactly, no subtraction of 1, no remap at
    /// 16–31, the 16-bit step; a missing file gives black in our engine)
    #[test]
    fn icon_palette_conversion() {
        let mut file = Palette6 {
            colors: [[0; 3]; 256],
        };
        file.colors[5] = [1, 2, 3];
        file.colors[20] = [63, 32, 16];
        file.colors[200] = [64, 65, 100];
        let p = icon_palette_565(Some(&file), &PaletteQuirks::default());
        let split = |c: u16| (c >> 11, (c >> 5) & 0x3F, c & 0x1F);
        assert_eq!(split(p[5]), (0, 2, 1));
        assert_eq!(split(p[20]), (31, 32, 8));
        assert_eq!(split(p[200]), (0, 1, 18));
        // Row 26 reproduces the converted colour; row 52 doubles it, capped.
        assert_eq!(icon_shade_component(252, ICON_ROW), 252);
        assert_eq!(icon_shade_component(100, ICON_SHADE_ROWS - 1), 200);
        assert_eq!(icon_shade_component(200, ICON_SHADE_ROWS - 1), 255);
        assert_eq!(icon_shade_component(200, 0), 0);
        // No file: all black.
        assert!(
            icon_palette_565(None, &PaletteQuirks::default())
                .iter()
                .all(|&c| c == 0)
        );
    }

    /// spec: S-0103 claim 11 and Constants (lobby swatches: the slot's RGB reduced directly by the 16-bit step, no subtraction of 1)
    #[test]
    fn lobby_swatch_is_the_stepped_constant() {
        // Slot 0 is (221, 226, 13): ⌊221/8⌋ = 27, ⌊226/4⌋ = 56, ⌊13/8⌋ = 1.
        assert_eq!(lobby_swatch_565(0), Some((27 << 11) | (56 << 5) | 1));
        for s in 0..8u8 {
            assert_eq!(
                lobby_swatch_565(s),
                Some(hicolor_565(LOBBY_SWATCHES[usize::from(s)]))
            );
        }
        assert_eq!(lobby_swatch_565(8), None);
    }

    /// spec: S-0103 Constants (the bit patterns are the tabulated singles)
    #[test]
    fn constants_match_the_table() {
        let sigma = "0.7661539316177368, 0.7949070930480957, 0.8224136829376221, 0.8481203317642212, \
            0.8719707727432251, 0.8935688734054565, 0.9135870337486267, 0.9316057562828064, \
            0.9473402500152588, 0.9612804651260376, 0.9731168746948242, 0.9826816916465759, \
            0.9902727603912354, 0.9956966042518616, 0.9989056587219238, 1.0";
        let gamma = "0.9807852506637573, 0.9088482856750488, 0.8719707727432251, 0.8289040923118591, \
            0.7807372212409973, 0.7274125814437866, 0.6692825555801392, 0.606731116771698, \
            0.5395258069038391, 0.469366192817688, 0.3961058259010315, 0.3202287256717682, \
            0.24223609268665314, 0.16188637912273407, 0.08121144026517868, 1.2246063538223773e-16";
        for (text, bits) in [(sigma, SIGMA_BITS), (gamma, GAMMA_BITS)] {
            let parsed: Vec<u32> = text
                .split(',')
                .map(|t| t.trim().parse::<f64>().unwrap() as f32)
                .map(f32::to_bits)
                .collect();
            assert_eq!(parsed, bits);
            // Each is exactly a single (widening to double is lossless).
            for (t, b) in text.split(',').zip(bits) {
                assert_eq!(
                    f64::from(f32::from_bits(b)),
                    t.trim().parse::<f64>().unwrap()
                );
            }
        }
    }

    /// spec: S-0103 Formulas (worked examples, entries 16 to 31)
    #[test]
    fn worked_ramps() {
        assert_eq!(
            ramp([0, 255, 255]),
            rgb([
                (250, 58, 58),
                (231, 48, 48),
                (222, 40, 40),
                (211, 32, 32),
                (199, 25, 25),
                (185, 20, 20),
                (170, 15, 15),
                (154, 10, 10),
                (137, 7, 7),
                (119, 4, 4),
                (101, 2, 2),
                (81, 1, 1),
                (61, 0, 0),
                (41, 0, 0),
                (20, 0, 0),
                (0, 0, 0),
            ])
        );
        assert_eq!(
            ramp([170, 128, 200]),
            rgb([
                (120, 120, 196),
                (109, 109, 181),
                (102, 102, 174),
                (95, 95, 165),
                (88, 88, 156),
                (80, 80, 145),
                (72, 72, 133),
                (64, 64, 121),
                (56, 56, 107),
                (48, 48, 93),
                (40, 40, 79),
                (32, 32, 64),
                (24, 24, 48),
                (16, 16, 32),
                (8, 8, 16),
                (0, 0, 0),
            ])
        );
        // H = 255 and H = 0 give the same ramp; entry 31 is black for every definition.
        assert_eq!(ramp([255, 200, 100]), ramp([0, 200, 100]));
        for h in [0u8, 40, 99, 200] {
            assert_eq!(ramp([h, 255, 255])[15], Rgb8::new(0, 0, 0));
        }
    }

    /// spec: S-0103 claim 6 (only entries 16 to 31 change)
    #[test]
    fn house_palette_replaces_the_remap_range() {
        let mut unit = Palette::default();
        for (i, c) in unit.colors.iter_mut().enumerate() {
            *c = Rgb8::new(i as u8, 1, 2);
        }
        let p = house_palette(&unit, [0, 255, 255]);
        assert_eq!(p.colors[15], unit.colors[15]);
        assert_eq!(p.colors[16], Rgb8::new(250, 58, 58));
        assert_eq!(p.colors[31], Rgb8::new(0, 0, 0));
        assert_eq!(p.colors[32], unit.colors[32]);
    }

    /// spec: S-0103 claim 1 and its switch
    #[test]
    fn file_components() {
        let on = PaletteQuirks::default();
        let off = PaletteQuirks {
            palette_component_wrap: false,
            ..on
        };
        assert_eq!(load_component(63, &on), 252);
        assert_eq!(load_component(64, &on), 0);
        assert_eq!(load_component(65, &on), 4);
        assert_eq!(load_component(64, &off), 252);
        assert_eq!(load_component(10, &off), 40);
    }

    fn file_of(c: u8) -> Palette6 {
        Palette6 {
            colors: [[c, c, c]; PALETTE_LEN],
        }
    }

    /// spec: S-0103 claims 2 and 3, State and ordering
    #[test]
    fn palette_state_machine() {
        let q = PaletteQuirks::default();
        let mut s = PaletteState::at_program_start(Some(&file_of(3)), Some(&file_of(1)), q);
        assert_eq!(s.unit.colors[0], Rgb8::new(12, 12, 12));
        assert_eq!(s.tile.colors[9], Rgb8::new(0, 0, 0));
        assert_eq!(s.anim.colors[0], Rgb8::new(4, 4, 4));
        // Temperate with every file missing: unit ×4 again, tile unchanged, theater formula.
        s.start_game(Theater::Temperate, |_| None);
        assert_eq!(s.unit.colors[0], Rgb8::new(48, 48, 48));
        assert_eq!(s.tile.colors[0], Rgb8::new(0, 0, 0));
        assert_eq!(s.theater.colors[10], Rgb8::new(10, 245, 40));
        // Same theater again: nothing is read, nothing changes.
        s.start_game(Theater::Temperate, |_| Some(file_of(9)));
        assert_eq!(s.unit.colors[0], Rgb8::new(48, 48, 48));
        // Another theater, still missing: 4^(n+1) · c mod 256, zero from n = 3.
        s.start_game(Theater::Snow, |_| None);
        assert_eq!(s.unit.colors[0], Rgb8::new(192, 192, 192));
        s.start_game(Theater::Urban, |_| None);
        assert_eq!(s.unit.colors[0], Rgb8::new(0, 0, 0));
        // Files present: loaded ×4.
        s.start_game(Theater::Desert, |n| {
            (n == "UNITDES.PAL").then(|| file_of(5))
        });
        assert_eq!(s.unit.colors[0], Rgb8::new(20, 20, 20));
        // Switch off: a missing unit palette leaves it unchanged.
        let off = PaletteQuirks {
            missing_unit_palette_remultiplied: false,
            ..q
        };
        let mut s = PaletteState::at_program_start(Some(&file_of(3)), None, off);
        s.start_game(Theater::Lunar, |_| None);
        assert_eq!(s.unit.colors[0], Rgb8::new(12, 12, 12));
    }

    /// spec: S-0103 Terms (theater palette) and claim 8 (the ore and gem palette is the temperate
    /// theater palette read at program start; later theater loads do not change it); S-0112 Terms
    #[test]
    fn startup_resource_survives_theater_loads() {
        let q = PaletteQuirks::default();
        let mut s = PaletteState::at_program_start(Some(&file_of(3)), Some(&file_of(1)), q);
        s.set_startup_resource(Some(&file_of(7)));
        assert_eq!(s.resource.colors[0], Rgb8::new(28, 28, 28));
        // Two theater loads, the second reading a temperate theater palette of other values.
        s.start_game(Theater::Snow, |_| Some(file_of(2)));
        s.start_game(Theater::Temperate, |_| Some(file_of(9)));
        assert_eq!(s.theater.colors[0], Rgb8::new(36, 36, 36));
        assert_eq!(s.resource.colors[0], Rgb8::new(28, 28, 28));
    }

    /// spec: S-0103 claim 17 and Import mapping (the art value's first 31 bytes, the current
    /// theater's suffix and `.PAL`, appended even after an extension; empty names none)
    #[test]
    fn named_palette_file_names() {
        let n = |v: &[u8], t| named_palette_file(v, t).map(|f| String::from_utf8(f).unwrap());
        assert_eq!(
            n(b"test", Theater::Temperate).as_deref(),
            Some("TESTTEM.PAL")
        );
        assert_eq!(n(b"Test", Theater::Lunar).as_deref(), Some("TESTLUN.PAL"));
        assert_eq!(n(b"x.pal", Theater::Snow).as_deref(), Some("X.PALSNO.PAL"));
        assert_eq!(n(b"", Theater::Urban), None);
        let long = [b'a'; 40];
        let f = n(&long, Theater::Desert).unwrap();
        assert_eq!(f.len(), NAMED_PALETTE_MAX + "DES.PAL".len());
        assert!(f.ends_with("ADES.PAL"));
    }

    /// spec: S-0103 claim 17 (a named file receives claim 1's multiplication and byte wrap)
    #[test]
    fn named_palette_is_loaded_like_a_palette_file() {
        let q = PaletteQuirks::default();
        let s = PaletteState::at_program_start(Some(&file_of(3)), None, q);
        let mut f = file_of(5);
        f.colors[7] = [64, 65, 63];
        let p = s.load_named(&f);
        assert_eq!(p.colors[0], Rgb8::new(20, 20, 20));
        assert_eq!(p.colors[7], Rgb8::new(0, 4, 252));
    }

    /// spec: S-0103 claim 10
    #[test]
    fn slots() {
        let positions: Vec<usize> = (0..8).map(|s| slot_position(s).unwrap()).collect();
        assert_eq!(positions, [1, 5, 10, 14, 6, 12, 8, 7]);
        assert_eq!(slot_position(8), None);
    }

    /// spec: S-0103 claim 9
    #[test]
    fn hicolor() {
        assert_eq!(hicolor_565(Rgb8::new(248, 252, 248)), 0xFFFF);
        assert_eq!(hicolor_565(Rgb8::new(7, 3, 7)), 0);
        assert_eq!(hicolor_555(Rgb8::new(255, 255, 255)), 0x7FFF);
    }

    /// spec: S-0103 claim 8 (rows and precedence) and the two switches
    #[test]
    fn palette_choice() {
        let q = PaletteQuirks::default();
        let anim = |veins, terrain, drawn_with_object, alt_palette| DrawnObject::Animation {
            veins,
            terrain,
            drawn_with_object,
            alt_palette,
        };
        assert_eq!(
            palette_for(DrawnObject::TerrainTile, &q),
            PaletteChoice::Tile
        );
        assert_eq!(
            palette_for(DrawnObject::Mobile, &q),
            PaletteChoice::OwnerHouse
        );
        assert_eq!(
            palette_for(
                DrawnObject::Building {
                    terrain_palette: true
                },
                &q
            ),
            PaletteChoice::Tile
        );
        assert_eq!(
            palette_for(anim(true, true, true, true), &q),
            PaletteChoice::LocalPlayerHouse
        );
        assert_eq!(
            palette_for(anim(false, true, true, true), &q),
            PaletteChoice::Tile
        );
        assert_eq!(
            palette_for(anim(false, false, true, true), &q),
            PaletteChoice::OwnerHouse
        );
        assert_eq!(
            palette_for(anim(false, false, false, true), &q),
            PaletteChoice::FirstDefinitionSingleShade
        );
        assert_eq!(
            palette_for(anim(false, false, false, false), &q),
            PaletteChoice::Anim
        );
        let off = PaletteQuirks {
            alt_palette_first_definition: false,
            veins_local_player_colour: false,
            ..q
        };
        assert_eq!(
            palette_for(anim(true, false, false, false), &off),
            PaletteChoice::Anim
        );
        assert_eq!(
            palette_for(anim(false, false, false, true), &off),
            PaletteChoice::OwnerSingleShade
        );
    }
}
