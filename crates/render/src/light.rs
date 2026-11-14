//! Lighting maths (S-0104), on the CPU, in exact integer arithmetic.
//!
//! - Tint channels become 16.16 shade factors ([`tint_factor`], claim 7),
//!   and a palette, a tint and a shade count become shade rows
//!   ([`shade_table`], [`untinted_table`], claim 6).
//! - A cell's raw tint is normalised and quantised, and picks a tile shade
//!   table ([`normalise`], [`quantise`], [`choose_tile_table`],
//!   [`choose_tile_table_on_relight`], claim 8).
//! - A cell's light levels come from the ambient, the height terms and the
//!   light sources ([`full_recompute`], [`full_recompute_with_terms`],
//!   [`incremental_update`], claims 9 to 11).
//! - A drawn object's brightness comes from its cell's levels
//!   ([`object_brightness`], claim 13) and picks a shade row
//!   ([`brightness_row`], claim 15).
//! - The ambient fade moves one step at a time ([`ambient_step`], claim 4).
//!
//! Not here: which lighting set and which effects are in force (claims 2,
//! 5, 14, 16 and 17), the fade's timing (claim 4), the import of map and
//! rules values (claims 1 and 12) and the shadowed-cell marks. Those need
//! simulation state and the caller passes their results in.
//!
//! Every floating-point step of the original that this module covers is
//! done in exact integer arithmetic that S-0104 states gives the same
//! result. The round-toward-zero results are always used: the
//! round-to-nearest variants belong to `quirks.chop_rounding` of the
//! planned number-domains spec and are not offered here.

use std::cmp::Ordering;
use std::ops::RangeInclusive;

use glasnost_fmt::image::{PALETTE_LEN, Palette, Rgb8};
use glasnost_num::Rounding;
use glasnost_num::geom::root_floor;

use crate::palette::hicolor_565;

/// Neutral light level and neutral tint channel, in ‰.
pub const NEUTRAL: i32 = 1000; // spec: S-0104 Constants (neutral)

/// Largest usable light level and tint channel, in ‰.
pub const LEVEL_MAX: i32 = 2000; // spec: S-0104 Constants (light level range 0 … 2000)

/// Default light-alpha value of a screen pixel (claim 15).
pub const DEFAULT_LIGHT_ALPHA: i32 = 127; // spec: S-0104 Constants (default light-alpha)

/// Rows of the 53-shade tables: the neutral tile table, tile tables whose
/// quantised channels sum to 2000 or more, and the untinted anim-palette
/// table (claims 6 and 8).
pub const FULL_ROWS: usize = 53; // spec: S-0104 Constants (tile shade counts; anim palette rows)

/// Rows of a tile table whose quantised channels sum to less than 2000
/// (claim 8).
pub const REDUCED_ROWS: usize = 27; // spec: S-0104 Constants (tile shade counts)

/// The nuke flash set's level factor, ‰ per height level (fixed; claim 10
/// uses it for the dominator's ground level).
pub const NUKE_LEVEL_FACTOR: i32 = 100; // spec: S-0104 Constants (nuke flash ground / level: 100 ‰ per level)

/// Default radius of a building light, in leptons (claim 12).
pub const BUILDING_LIGHT_RADIUS: i32 = 5000; // spec: S-0104 claim 12, Constants (building light radius default)

/// A 16.16 factor of exactly 1.
const ONE_16_16: i64 = 65536; // spec: S-0104 Terms (16.16 factor F means F/65536)

/// Leptons per cell side; a cell's centre is at coordinate × 256 + 128.
const CELL_LEPTONS: i64 = 256; // spec: S-0104 Formulas, "Cell light" (cell centre = cell coordinate × 256 + 128)

/// Bits of δ, the double nearest to 0.001 (S-0104 Constants, red-largest
/// divisor factor). It is slightly above 0.001 (checked in the tests).
// spec: S-0104 Constants (red-largest divisor factor δ), Formulas "Tint normalisation"
const DELTA_BITS: u64 = 0x3F50_624D_D2F1_A9FC;

/// Quirk switches of S-0104 implemented here. Each defaults to the
/// original's behaviour (on).
///
/// The other switches of S-0104 act on simulation state, the importer or
/// the effect state machine, outside this module
/// (`app::mapview::quirks::NOT_IMPLEMENTED`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightQuirks {
    /// `quirks.tile_table_kept_on_relight` (claim 8): a full recompute
    /// caused by a light change keeps the cell's table when its own tint
    /// equals the new quantised tint; off, the table is chosen afresh.
    pub tile_table_kept_on_relight: bool,
    /// `quirks.near_black_tint_zeroes_scaled_ground` (claim 8): a
    /// near-black raw cell tint zeroes the scaled ground level at a full
    /// recompute; off, it does not.
    pub near_black_tint_zeroes_scaled_ground: bool,
    /// `quirks.red_normalisation_divisor` (claim 8): red-largest tints use
    /// the inexact divisor and can come out 1 ‰ low; off, the exact
    /// quotient.
    pub red_normalisation_divisor: bool,
    /// `quirks.normalisation_truncated_scale` (claim 8): green- and
    /// blue-largest tints divide by the truncated scale *K*; off, the exact
    /// quotient.
    pub normalisation_truncated_scale: bool,
    /// `quirks.dominator_ground_uses_nuke_level` (claim 10): a full
    /// recompute under the dominator uses the nuke level factor for the
    /// ground level; off, the dominator's in both levels.
    pub dominator_ground_uses_nuke_level: bool,
    /// `quirks.incremental_relight_differs` (claim 9, Formulas
    /// "Incremental update"): the incremental update scales the unclamped
    /// ground and does not repeat the near-black zeroing; off, it gives
    /// what a full recompute would.
    pub incremental_relight_differs: bool,
    /// `quirks.aircraft_height_light_ion_only` (claim 13): aircraft use the
    /// ion storm's or the normal level factor; off, the per-cell one.
    pub aircraft_height_light_ion_only: bool,
    /// `quirks.building_light_from_top_left_cell` (claim 13): an ordinary
    /// building takes the light of its top-left foundation cell; off, of
    /// the cell under its centre ([`building_light_cell`]).
    pub building_light_from_top_left_cell: bool,
    /// `quirks.deploy_animation_unlit` (claim 13): deploy and undeploy
    /// animations are drawn at 1000; off, at the unit cell's unscaled
    /// ground level.
    pub deploy_animation_unlit: bool,
    /// `quirks.building_light_tint_default_1000` (claim 12): an absent
    /// building light tint is 1000.0, stored 1 000 000 ‰
    /// ([`building_light_tint_default`]). Off: 1.0, stored 1000 ‰.
    pub building_light_tint_default_1000: bool,
    /// `quirks.infantry_height_light_limited` (claim 13): infantry add their
    /// height term only on a bridge deck (measured from the deck) and in a
    /// shadowed cell above 208 leptons. Off (claim 18): (h ÷ 208) × λ
    /// always, h above the terrain, plus −500 in a shadowed cell off a deck
    /// at h ≤ 208.
    pub infantry_height_light_limited: bool,
    /// The ruleset's `quirks.table_root_distance` (S-0204, read here by
    /// claim 11): the falloff distance is the table square root; off, the
    /// integer square root. S-0204 owns the switch, so [`Self::named`]
    /// leaves it out.
    pub table_root_distance: bool,
    /// The ruleset's `quirks.chop_rounding` (S-0205, read here by claim 11
    /// and Formulas "Cell light"): the squared distance goes to single
    /// precision rounded toward zero; off, to nearest. S-0205 owns it.
    pub chop_rounding: bool,
}

impl Default for LightQuirks {
    fn default() -> Self {
        // spec: S-0104 Edge cases and quirks (every switch defaults to on)
        Self {
            tile_table_kept_on_relight: true,
            near_black_tint_zeroes_scaled_ground: true,
            red_normalisation_divisor: true,
            normalisation_truncated_scale: true,
            dominator_ground_uses_nuke_level: true,
            incremental_relight_differs: true,
            aircraft_height_light_ion_only: true,
            building_light_from_top_left_cell: true,
            deploy_animation_unlit: true,
            building_light_tint_default_1000: true,
            infantry_height_light_limited: true,
            // spec: S-0204 claim 5, S-0104 claim 11 (quirks.table_root_distance, default on)
            table_root_distance: true,
            // spec: S-0205 claim 5, S-0104 Formulas "Cell light" (quirks.chop_rounding, default on)
            chop_rounding: true,
        }
    }
}

impl LightQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 11] {
        [
            (
                "tile_table_kept_on_relight",
                self.tile_table_kept_on_relight,
            ),
            (
                "near_black_tint_zeroes_scaled_ground",
                self.near_black_tint_zeroes_scaled_ground,
            ),
            ("red_normalisation_divisor", self.red_normalisation_divisor),
            (
                "normalisation_truncated_scale",
                self.normalisation_truncated_scale,
            ),
            (
                "dominator_ground_uses_nuke_level",
                self.dominator_ground_uses_nuke_level,
            ),
            (
                "incremental_relight_differs",
                self.incremental_relight_differs,
            ),
            (
                "aircraft_height_light_ion_only",
                self.aircraft_height_light_ion_only,
            ),
            (
                "building_light_from_top_left_cell",
                self.building_light_from_top_left_cell,
            ),
            ("deploy_animation_unlit", self.deploy_animation_unlit),
            (
                "building_light_tint_default_1000",
                self.building_light_tint_default_1000,
            ),
            (
                "infantry_height_light_limited",
                self.infantry_height_light_limited,
            ),
        ]
    }
}

/// The stored value, in ‰, of a building light tint whose key is absent
/// (claim 12).
pub fn building_light_tint_default(q: &LightQuirks) -> i32 {
    if q.building_light_tint_default_1000 {
        // spec: S-0104 claim 12 (an absent tint is 1000.0, stored 1 000 000 ‰)
        1_000_000
    } else {
        // spec: S-0104 Edge cases (quirks.building_light_tint_default_1000 off: 1.0, stored 1000 ‰)
        NEUTRAL
    }
}

/// The player's visual-detail option (S-0104 Terms, detail level).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DetailLevel {
    /// Detail level 0.
    Low,
    /// Detail level 1 (reachable only through the settings file).
    Medium,
    /// Detail level 2.
    High,
}

impl DetailLevel {
    /// The tint quantum *Q* of claim 8: 128, 64 or 32 ‰.
    pub fn quantum(self) -> i32 {
        // spec: S-0104 claim 8, Constants (tint quantum 128 / 64 / 32)
        match self {
            DetailLevel::Low => 128,
            DetailLevel::Medium => 64,
            DetailLevel::High => 32,
        }
    }

    /// The level as a number: 0, 1 or 2 (S-0104 Terms).
    pub fn number(self) -> i32 {
        // spec: S-0104 Terms (detail levels 0, 1 and 2)
        match self {
            DetailLevel::Low => 0,
            DetailLevel::Medium => 1,
            DetailLevel::High => 2,
        }
    }

    /// The level of a settings-file value: above 2 becomes 2, below 0
    /// becomes 0 (S-0104 Import mapping, `DetailLevel`).
    pub fn from_settings(value: i32) -> Self {
        // spec: S-0104 Import mapping (DetailLevel: values above 2 become 2, below 0 become 0)
        match value {
            i32::MIN..=0 => DetailLevel::Low,
            1 => DetailLevel::Medium,
            _ => DetailLevel::High,
        }
    }
}

/// *M*(*t*): the 16.16 shade factor of a tint channel *t* in ‰, clamped to
/// [0, 2000] first (S-0104 claim 7, Formulas "Shade table"). This is the
/// round-toward-zero form, ⌊8192·*t*/125⌋ − ε(*t*) with ε(*t*) = 1 for a
/// positive multiple of 125, so *M*(1000) = 65535, not 65536.
pub fn tint_factor(t_permille: i32) -> i64 {
    // spec: S-0104 claim 6 (each tint channel clamped to [0, 2000])
    let t = i64::from(t_permille.clamp(0, LEVEL_MAX));
    // spec: S-0104 Formulas, "Shade table" (M(t) = ⌊8192·t / 125⌋ − ε(t))
    let epsilon = i64::from(t > 0 && t % 125 == 0);
    8192 * t / 125 - epsilon
}

/// The normalisation scale *K* = ⌊8192·*X*/125⌋ = ⌊*X*·65.536⌋ of a
/// largest channel *X* (S-0104 Formulas, "Tint normalisation"). Unlike
/// [`tint_factor`] it has no ε: *K*(1000) = 65536.
pub fn normalisation_scale(x: i32) -> i64 {
    // spec: S-0104 Formulas, "Tint normalisation" (K = ⌊8192·X / 125⌋)
    8192 * i64::from(x) / 125
}

/// How *D*(*X*) = RZ(*X*·65536·δ) compares with *X*·65.536 (S-0104
/// Formulas, "Tint normalisation"), computed exactly from δ's bits.
fn d_compare(x: i32) -> Ordering {
    // δ = m · 2^e with a 53-bit mantissa m.
    let mantissa = u128::from(DELTA_BITS & ((1 << 52) - 1)) | (1 << 52);
    let e = ((DELTA_BITS >> 52) & 0x7FF) as i32 - 1023 - 52;
    // X·65536·δ = (X·m) · 2^(e + 16). The scaling by powers of two is
    // exact, so the one rounding is of X·m to 53 significant bits, toward
    // zero (spec: S-0104 Formulas, "Precision of decimal conversions").
    let x = u128::from(x.max(0).unsigned_abs());
    let p = x * mantissa;
    let bits = 128 - p.leading_zeros();
    let p = if bits > 53 {
        let drop = bits - 53;
        (p >> drop) << drop
    } else {
        p
    };
    // D = p · 2^(e + 16) against X · 65536 / 1000, both sides times
    // 1000 · 2^-(e + 16) (e + 16 is negative for δ ≈ 2^-10).
    let shift = u32::try_from(-(e + 16)).expect("δ is below 2^-16");
    // spec: S-0104 Formulas, "Tint normalisation" (𝒟: D(X) > X·65.536 = X·65536/1000)
    (p * 1000).cmp(&((x * 65536) << shift))
}

/// Whether *X* is in the set 𝒟 of S-0104 Formulas, "Tint normalisation":
/// *D*(*X*) = RZ(*X*·65536·δ) > *X*·65.536, with δ the double nearest to
/// 0.001, computed exactly in integer arithmetic from δ's bits.
pub fn in_d_set(x: i32) -> bool {
    d_compare(x) == Ordering::Greater
}

/// The red-largest channel *n*(*c*, *X*) of S-0104 Formulas, "Tint
/// normalisation" (the original's inexact divisor): the exact quotient,
/// 1 lower when *c* > 0, 1000·*c* is a multiple of *X* and *X* ∈ 𝒟.
fn red_largest_channel(c: i64, x: i64, q: &LightQuirks) -> i64 {
    // spec: S-0104 Formulas, "Tint normalisation" (exact quotient ⌊1000·c / X⌋)
    let exact = 1000 * c / x;
    // spec: S-0104 claim 8 (quirks.red_normalisation_divisor), Formulas n(c, X)
    if q.red_normalisation_divisor && c > 0 && (1000 * c) % x == 0 && in_d_set(x as i32) {
        exact - 1
    } else {
        exact
    }
}

/// A raw cell tint after normalisation (S-0104 claim 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Normalised {
    /// The normalised tint in ‰: white, or largest channel 1000. Blue can
    /// be 1001 to 1003 when it ties with a small odd largest green
    /// (S-0104 Terms, normalised tint).
    pub tint: [i32; 3],
    /// The scale *K*, a 16.16 factor applied to the cell's light levels.
    pub k: i64,
    /// The raw cell tint was near-black (largest channel at most 1 ‰) and
    /// `quirks.near_black_tint_zeroes_scaled_ground` is on, so a full
    /// recompute zeroes the scaled ground level.
    pub near_black: bool,
}

/// Clamps and normalises a raw cell tint in ‰ (S-0104 claim 8, Formulas
/// "Tint normalisation").
pub fn normalise(raw: [i32; 3], q: &LightQuirks) -> Normalised {
    const WHITE: [i32; 3] = [NEUTRAL; 3];
    // spec: S-0104 Formulas, "Tint normalisation" (each channel clamped to [0, 2000])
    let [r, g, b] = raw.map(|c| i64::from(c.clamp(0, LEVEL_MAX)));
    // spec: S-0104 Formulas, "Tint normalisation" (row (1000, 1000, 1000): K = 65536)
    if [r, g, b] == [1000; 3] {
        return Normalised {
            tint: WHITE,
            k: ONE_16_16,
            near_black: false,
        };
    }
    // spec: S-0104 claim 8 (ties: red counts as largest, then green, then blue)
    let largest = if r >= g && r >= b {
        0
    } else if g >= b {
        1
    } else {
        2
    };
    let x = [r, g, b][largest];
    // spec: S-0104 claim 8 (near-black: X ≤ 1 normalises to white, K = 65536), Constants (near-black threshold 1)
    if x <= 1 {
        return Normalised {
            tint: WHITE,
            k: ONE_16_16,
            // spec: S-0104 Formulas, "Tint normalisation" (quirks.near_black_tint_zeroes_scaled_ground)
            near_black: q.near_black_tint_zeroes_scaled_ground,
        };
    }
    let k = normalisation_scale(x as i32);
    let mut tint = WHITE;
    for (i, c) in [r, g, b].into_iter().enumerate() {
        tint[i] = if i == largest {
            // spec: S-0104 claim 8 (largest channel set to 1000)
            NEUTRAL as i64
        } else if largest == 0 {
            red_largest_channel(c, x, q)
        } else if q.normalisation_truncated_scale {
            // spec: S-0104 Formulas, "Tint normalisation" (green or blue largest: ⌊c·65536 / K⌋)
            c * ONE_16_16 / k
        } else {
            // spec: S-0104 Edge cases (normalisation_truncated_scale off: exact quotient)
            1000 * c / x
        } as i32;
    }
    Normalised {
        tint,
        k,
        near_black: false,
    }
}

/// Quantises a normalised tint by detail level: each channel clamped to
/// [0, 1000] and rounded down to a multiple of the quantum (S-0104 claim 8,
/// Formulas "Quantisation").
pub fn quantise(tint: [i32; 3], detail: DetailLevel) -> [i32; 3] {
    let quantum = detail.quantum();
    tint.map(|c| {
        // spec: S-0104 Formulas, "Quantisation" (c′ = m − (m mod Q), m = min(max(c, 0), 1000))
        let m = c.clamp(0, NEUTRAL);
        m - m % quantum
    })
}

/// A tile shade table, identified by the tint it was built for (S-0104
/// claim 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TileTable {
    /// The neutral tile table: white, 53 rows, no quantisation.
    Neutral,
    /// The table of a quantised tint, shared by every cell with that tint.
    Quantised([i32; 3]),
}

impl TileTable {
    /// The table's row count: 53, or 27 when the quantised channels sum to
    /// less than 2000 (S-0104 claim 8, Formulas "Tile table choice").
    pub fn rows(&self) -> usize {
        match self {
            TileTable::Neutral => FULL_ROWS,
            // spec: S-0104 Formulas, "Tile table choice" (53 rows if r′ + g′ + b′ ≥ 2000, 27 otherwise)
            TileTable::Quantised(t) if t.iter().sum::<i32>() >= 2000 => FULL_ROWS,
            TileTable::Quantised(_) => REDUCED_ROWS,
        }
    }

    /// The tint the table was built for (S-0104 Terms, neutral tile table).
    /// A trigger action can later give a table another tint of its own
    /// (claim 17); that is the caller's state.
    pub fn tint(&self) -> [i32; 3] {
        match self {
            TileTable::Neutral => [NEUTRAL; 3],
            TileTable::Quantised(t) => *t,
        }
    }
}

/// The tile table a cell gets when it is chosen afresh (S-0104 claim 8,
/// Formulas "Tile table choice"): the neutral tile table for an exactly
/// white normalised tint, otherwise the table of the quantised tint.
pub fn choose_tile_table(normalised: &Normalised, detail: DetailLevel) -> TileTable {
    // spec: S-0104 claim 8 (the test is made on the normalised tint, before quantisation)
    if normalised.tint == [NEUTRAL; 3] {
        TileTable::Neutral
    } else {
        TileTable::Quantised(quantise(normalised.tint, detail))
    }
}

/// The tile table after a full recompute caused by a light-source change
/// (S-0104 claim 8, kept table): `current` is kept when `current_own_tint`
/// (the tint it was built for, or the map tint a trigger action last gave
/// it; never an effect tint) equals the new quantised tint. Otherwise, and
/// always with `quirks.tile_table_kept_on_relight` off, it is chosen
/// afresh.
pub fn choose_tile_table_on_relight(
    current: TileTable,
    current_own_tint: [i32; 3],
    normalised: &Normalised,
    detail: DetailLevel,
    q: &LightQuirks,
) -> TileTable {
    // spec: S-0104 claim 8 (kept table at a light change), Formulas "Tile table choice"
    if q.tile_table_kept_on_relight && current_own_tint == quantise(normalised.tint, detail) {
        current
    } else {
        choose_tile_table(normalised, detail)
    }
}

/// One point light (S-0104 Terms, light source; claims 11 and 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LightSource {
    /// Position x, in leptons.
    pub x: i32,
    /// Position y, in leptons.
    pub y: i32,
    /// Radius *R*, in leptons. A radius of 0 reaches no cell (departure).
    pub radius: i32,
    /// Intensity *I*, in ‰ (may be negative).
    pub intensity: i32,
    /// Tint per channel, in ‰ (may be negative).
    pub tint: [i32; 3],
    /// The light counts only at this detail level and above.
    pub required_detail: DetailLevel,
}

impl LightSource {
    /// A building's light source (S-0104 claim 12): at the centre of the
    /// foundation whose top-left cell is `top_left` and whose size is
    /// `size` cells along x and y, counting only at high detail. Only a
    /// building type with a non-zero intensity gets one.
    pub fn building(
        top_left: (i32, i32),
        size: (i32, i32),
        radius: i32,
        intensity: i32,
        tint: [i32; 3],
    ) -> Self {
        // spec: S-0104 claim 12 (one light source at the building's centre)
        let centre = |cell: i32, extent: i32| cell * CELL_LEPTONS as i32 + extent * 128;
        Self {
            x: centre(top_left.0, size.0),
            y: centre(top_left.1, size.1),
            radius,
            intensity,
            tint,
            // spec: S-0104 claim 11, Constants (building light required detail 2)
            required_detail: DetailLevel::High,
        }
    }

    /// A radiation field's light source (S-0525 claim 13): at the centre
    /// point of the field's centre cell `centre`, with the field's reach
    /// `radius` (leptons), counting at every detail level.
    pub fn field(centre: (i32, i32), radius: i32, intensity: i32, tint: [i32; 3]) -> Self {
        // spec: S-0525 claim 13 (one light source at the centre point of the centre cell)
        let point = |cell: i32| cell * CELL_LEPTONS as i32 + 128;
        Self {
            x: point(centre.0),
            y: point(centre.1),
            radius,
            intensity,
            tint,
            // spec: S-0525 claim 13 (unlike a building's light it counts at every detail level)
            required_detail: DetailLevel::Low,
        }
    }

    /// The falloff *f* in ‰ of this light at `cell`, or `None` when it
    /// does not reach the cell's centre (S-0104 claim 11, Formulas "Cell
    /// light"). `q` gives the distance's square root.
    pub fn falloff(&self, cell: (i32, i32), q: &LightQuirks) -> Option<i64> {
        let radius = i64::from(self.radius);
        // spec: S-0104 claim 11 (radius 0 reaches no cell: a departure)
        if radius <= 0 {
            return None;
        }
        // spec: S-0104 Formulas, "Cell light" (cell centre = cell coordinate × 256 + 128)
        let cx = i64::from(cell.0) * CELL_LEPTONS + 128;
        let cy = i64::from(cell.1) * CELL_LEPTONS + 128;
        let dx = cx - i64::from(self.x);
        let dy = cy - i64::from(self.y);
        let n = (dx * dx + dy * dy).unsigned_abs();
        // spec: S-0104 claim 11, Formulas "Cell light" (reaches the cell when n ≤ R², exact integers)
        if n > radius.unsigned_abs().pow(2) {
            return None;
        }
        let mode = if q.chop_rounding {
            Rounding::TowardZero
        } else {
            // spec: S-0104 Formulas "Cell light", S-0205 (quirks.chop_rounding off: σ(n) rounds to nearest)
            Rounding::Nearest
        };
        // spec: S-0104 claim 11, Formulas "Cell light" (d = ⌊√ₜ(n)⌋, the table square root; with
        // quirks.table_root_distance off ⌊√n⌋)
        let d = root_floor(n, q.table_root_distance, mode) as i64;
        // spec: S-0104 claim 11, Formulas "Cell light" (f = ⌊1000·(R − d) / R⌋)
        Some(1000 * (radius - d) / radius)
    }
}

/// The level factors and ground bias a cell update uses (S-0104 claims 9
/// and 10), in ‰ per height level and ‰.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HeightTerms {
    /// Level factor λ of the ground level.
    pub ground_level_factor: i32,
    /// Level factor λ of the raised level.
    pub raised_level_factor: i32,
    /// Ground bias *G*.
    pub ground_bias: i32,
}

impl HeightTerms {
    /// One lighting set's terms, the same λ for both levels (S-0104 claim
    /// 9). Every incremental update and every full recompute outside the
    /// dominator case of claim 10 uses these.
    pub fn uniform(level_factor: i32, ground_bias: i32) -> Self {
        Self {
            ground_level_factor: level_factor,
            raised_level_factor: level_factor,
            ground_bias,
        }
    }

    /// The terms of a full recompute while the per-cell ranking of claim 2
    /// chooses the psychic dominator (S-0104 claim 10): the ground level
    /// uses the nuke flash's level factor with the dominator's ground bias,
    /// the raised level the dominator's level factor. With
    /// `quirks.dominator_ground_uses_nuke_level` off, the dominator's in
    /// both.
    pub fn dominator_full_recompute(
        dominator_level_factor: i32,
        dominator_ground_bias: i32,
        q: &LightQuirks,
    ) -> Self {
        // spec: S-0104 claim 10 (dominator height bug), Edge cases (quirks.dominator_ground_uses_nuke_level)
        let ground_level_factor = if q.dominator_ground_uses_nuke_level {
            NUKE_LEVEL_FACTOR
        } else {
            dominator_level_factor
        };
        Self {
            ground_level_factor,
            raised_level_factor: dominator_level_factor,
            ground_bias: dominator_ground_bias,
        }
    }
}

/// A cell's light after a full recompute or an incremental update
/// (S-0104 claim 9, Formulas "Cell light").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellLight {
    /// Unscaled ground level, ‰, clamped to [0, 2000].
    pub unscaled_ground: i32,
    /// Scaled ground level, ‰, clamped to [0, 2000].
    pub scaled_ground: i32,
    /// Scaled raised level, ‰, clamped to [0, 2000].
    pub scaled_raised: i32,
    /// Raw cell tint, ‰ (unclamped; saturated to the `i32` range).
    pub raw_tint: [i32; 3],
    /// The normalised raw cell tint and its scale *K*.
    pub normalised: Normalised,
    /// The tile table chosen afresh at the last full recompute. At a light
    /// change the caller applies [`choose_tile_table_on_relight`] instead.
    pub table: TileTable,
    /// Light-source sum *S*, ‰ (saturated to the `i32` range).
    pub source_sum: i32,
}

fn saturate(v: i64) -> i32 {
    v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

/// Ground and raised levels before scaling and clamping (S-0104 Formulas,
/// "Cell light").
fn unscaled_levels(ambient: i32, source_sum: i64, terms: HeightTerms, z: i32) -> (i64, i64) {
    let z = i64::from(z);
    // spec: S-0104 Formulas, "Precision of decimal conversions" (‰ = hundredths · 10)
    let base = i64::from(ambient) * 10 + source_sum - i64::from(terms.ground_bias);
    // spec: S-0104 claim 9, Formulas "Cell light" (ground = A·10 + S + λ·z − G)
    let ground = base + i64::from(terms.ground_level_factor) * z;
    // spec: S-0104 claim 9, Constants (raised step 4), Formulas "Cell light" (raised = A·10 + S + λ·(z + 4) − G)
    let raised = base + i64::from(terms.raised_level_factor) * (z + 4);
    (ground, raised)
}

fn clamp_level(v: i64) -> i32 {
    // spec: S-0104 Formulas, "Cell light" (each light value finally clamped to [0, 2000])
    v.clamp(0, i64::from(LEVEL_MAX)) as i32
}

/// Full recompute of one cell (S-0104 claims 9 and 11, Formulas "Cell
/// light") with one level factor for both levels. `ambient` is the current
/// ambient in hundredths, `level_factor` and `ground_bias` the set's in ‰,
/// `map_tint` in hundredths, `cell` the cell's coordinates and
/// `height_level` its height in levels. Only sources whose required detail
/// is at most `detail` count; a radius of 0 reaches no cell (departure).
#[allow(clippy::too_many_arguments)]
pub fn full_recompute(
    ambient: i32,
    level_factor: i32,
    ground_bias: i32,
    map_tint: [i32; 3],
    cell: (i32, i32),
    height_level: i32,
    sources: &[LightSource],
    detail: DetailLevel,
    q: &LightQuirks,
) -> CellLight {
    full_recompute_with_terms(
        ambient,
        HeightTerms::uniform(level_factor, ground_bias),
        map_tint,
        cell,
        height_level,
        sources,
        detail,
        q,
    )
}

/// [`full_recompute`] with separate ground and raised level factors, for
/// the dominator case of claim 10 ([`HeightTerms::dominator_full_recompute`]).
#[allow(clippy::too_many_arguments)]
pub fn full_recompute_with_terms(
    ambient: i32,
    terms: HeightTerms,
    map_tint: [i32; 3],
    cell: (i32, i32),
    height_level: i32,
    sources: &[LightSource],
    detail: DetailLevel,
    q: &LightQuirks,
) -> CellLight {
    let mut sum = 0i64;
    // spec: S-0104 Formulas, "Cell light" (raw tint_c = 10·map tint_c + Σ ⌊t_k,c·f_k / 1000⌋)
    let mut raw = map_tint.map(|t| i64::from(t) * 10);
    for source in sources {
        // spec: S-0104 claim 11 (a light counts only when the detail level is at least its required level)
        if source.required_detail > detail {
            continue;
        }
        let Some(f) = source.falloff(cell, q) else {
            continue;
        };
        // spec: S-0104 claim 11 (⌊I·f / 1000⌋, truncation toward zero)
        sum += i64::from(source.intensity) * f / 1000;
        for (r, t) in raw.iter_mut().zip(source.tint) {
            // spec: S-0104 claim 11 (⌊t_c·f / 1000⌋ per tint channel)
            *r += i64::from(t) * f / 1000;
        }
    }
    let raw_tint = raw.map(saturate);
    let normalised = normalise(raw_tint, q);
    let (ground, raised) = unscaled_levels(ambient, sum, terms, height_level);
    let k = normalised.k;
    // spec: S-0104 Formulas, "Cell light" (scaled ground = min(2000, max(0, ⌊min(ground, 2000)·K / 65536⌋)))
    let scaled_ground = if normalised.near_black {
        // spec: S-0104 claim 8 (near-black: the scaled ground level becomes 0 at a full recompute)
        0
    } else {
        clamp_level(ground.min(i64::from(LEVEL_MAX)) * k / ONE_16_16)
    };
    CellLight {
        // spec: S-0104 Formulas, "Cell light" (unscaled ground = min(2000, max(0, ground)))
        unscaled_ground: clamp_level(ground),
        scaled_ground,
        // spec: S-0104 Formulas, "Cell light" (scaled raised = min(2000, max(0, ⌊raised·K / 65536⌋)), no clamp before scaling)
        scaled_raised: clamp_level(raised * k / ONE_16_16),
        raw_tint,
        normalised,
        table: choose_tile_table(&normalised, detail),
        source_sum: saturate(sum),
    }
}

/// Incremental update of a cell (S-0104 claim 9, Formulas "Incremental
/// update"): the ground and raised levels are evaluated again from the
/// stored light-source sum with the current `ambient` (hundredths) and
/// `terms`; the stored *K*, raw cell tint, normalised tint and table are
/// kept. Claim 10's bug does not apply here: pass
/// [`HeightTerms::uniform`] with the chosen set's own level factor.
pub fn incremental_update(
    previous: &CellLight,
    ambient: i32,
    terms: HeightTerms,
    height_level: i32,
    q: &LightQuirks,
) -> CellLight {
    let (ground, raised) =
        unscaled_levels(ambient, i64::from(previous.source_sum), terms, height_level);
    let k = previous.normalised.k;
    let scaled_ground = if q.incremental_relight_differs {
        // spec: S-0104 Formulas, "Incremental update" (⌊ground·K / 65536⌋ from the unclamped ground; no near-black zeroing)
        clamp_level(ground * k / ONE_16_16)
    } else if previous.normalised.near_black {
        // spec: S-0104 Edge cases (incremental_relight_differs off: exactly what a full recompute would give)
        0
    } else {
        clamp_level(ground.min(i64::from(LEVEL_MAX)) * k / ONE_16_16)
    };
    CellLight {
        unscaled_ground: clamp_level(ground),
        scaled_ground,
        scaled_raised: clamp_level(raised * k / ONE_16_16),
        ..*previous
    }
}

/// One ambient fade step (S-0104 claim 4, Formulas "Ambient fade"), all in
/// hundredths: returns the new current and the new target ambient. A
/// negative target is raised to 0 when the step falls due (the caller
/// stores the returned target), and a step never overshoots. When a step
/// falls due is the caller's (the fade interval).
pub fn ambient_step(current: i32, target: i32, step: i32) -> (i32, i32) {
    // spec: S-0104 claim 4 (a negative target is raised to 0 when a step falls due), Formulas "Ambient fade" (A_t = max(target, 0))
    let target = target.max(0);
    // spec: S-0104 Formulas, "Ambient fade" (A′ = A_t if |A_t − A| ≤ step, else A + step·sgn(A_t − A))
    let current = if (i64::from(target) - i64::from(current)).abs() <= i64::from(step) {
        target
    } else {
        current + step * (target - current).signum()
    };
    (current, target)
}

/// The tint-exempt brightness rows *L* for a shade count *N* (S-0104
/// Formulas, "Shade table"): 6 for 53, 3 for 27.
fn exempt_rows(n: usize) -> usize {
    // spec: S-0104 Formulas, "Shade table" (L = min(max(⌊30N/200⌋ − 1, 0), ⌊(N − 1)/2⌋))
    (30 * n / 200).saturating_sub(1).min((n - 1) / 2)
}

/// One output channel: *u* times the factor with its low 4 bits dropped,
/// capped at 255 (S-0104 claim 6, Formulas "Shade table").
fn shade_channel(u: u8, factor: i64) -> u8 {
    // spec: S-0104 claim 6 (factors take effect as ⌊F/16⌋·16), Constants (shading factor granularity 16)
    let factor = factor / 16 * 16;
    // spec: S-0104 Formulas, "Shade table" (o_c = min(255, ⌊u_c·16·⌊T_c(j)/16⌋ / 65536⌋))
    (i64::from(u) * factor / ONE_16_16).min(255) as u8
}

/// A shade table (S-0104 claims 6 and 7, Formulas "Shade table"): `rows`
/// rows of 256 8-bit colours before 16-bit packing ([`pack_565`]) for
/// `palette` under `tint` (‰ per channel), with the indices in `exempt`
/// tint-exempt (S-0103 claim 7: [`crate::palette::TINT_EXEMPT`] for
/// house-colour palettes, `None` for the tile palette). Entry 0 is
/// transparent and given as black. `rows == 1` is the single-row case.
/// Low 4 bits of every factor are dropped (claim 6; the full-factor form is
/// a departure with no switch).
///
/// # Panics
///
/// With tint-exempt indices and a shade count from 2 to 13, for which
/// *L* = 0 and the tint-exempt factor is undefined (S-0104 Formulas; only
/// 1, 27 and 53 are used). Without tint-exempt indices every count gives a
/// table.
pub fn shade_table(
    palette: &Palette,
    tint: [i32; 3],
    rows: usize,
    exempt: Option<RangeInclusive<u8>>,
) -> Vec<[Rgb8; PALETTE_LEN]> {
    if rows == 0 {
        return Vec::new();
    }
    // spec: S-0104 claim 6 (tint clamped to [0, 2000] and converted to a 16.16 factor)
    let m = tint.map(tint_factor);
    let l = exempt_rows(rows);
    assert!(
        rows == 1 || l > 0 || exempt.is_none(),
        "S-0104 leaves the tint-exempt factor undefined for {rows} rows"
    );
    (0..rows)
        .map(|j| {
            let (t, w) = if rows == 1 {
                // spec: S-0104 Formulas, "Shade table" (N = 1: T_c = M_c, W = 65536)
                (m, ONE_16_16)
            } else {
                let (j64, span) = (j as i64, rows as i64 - 1);
                // spec: S-0104 claim 6, Formulas "Shade table" (T_c(j) = ⌊2j·M_c / (N − 1)⌋)
                let t = m.map(|mc| 2 * j64 * mc / span);
                // spec: S-0104 Formulas, "Shade table" (W(j) = ⌊65536·j / L⌋ for j ≤ L, 65536 above; W is read only for tint-exempt indices, so it is not formed without them)
                let w = if exempt.is_none() {
                    0
                } else if j <= l {
                    ONE_16_16 * j64 / l as i64
                } else {
                    ONE_16_16
                };
                (t, w)
            };
            let mut row = [Rgb8::default(); PALETTE_LEN];
            // spec: S-0104 claim 6 (entry 0 of every row is transparent)
            for (i, (out, u)) in row
                .iter_mut()
                .zip(palette.colors.iter())
                .enumerate()
                .skip(1)
            {
                let exempt_index = exempt.as_ref().is_some_and(|e| e.contains(&(i as u8)));
                let f = if exempt_index { [w; 3] } else { t };
                *out = Rgb8::new(
                    shade_channel(u.r, f[0]),
                    shade_channel(u.g, f[1]),
                    shade_channel(u.b, f[2]),
                );
            }
            row
        })
        .collect()
}

/// The anim palette's untinted table (S-0104 claim 6, Formulas "Untinted
/// table"): 53 rows, row *j* the palette times 2*j*/52 exactly, with no
/// tint, no tint-exempt indices and no dropped low bits. Entry 0 is
/// transparent and given as black.
pub fn untinted_table(palette: &Palette) -> Vec<[Rgb8; PALETTE_LEN]> {
    let span = FULL_ROWS as i64 - 1;
    (0..FULL_ROWS as i64)
        .map(|j| {
            // spec: S-0104 Formulas, "Untinted table" (factor ⌊2j·65536 / (N − 1)⌋, N = 53)
            let factor = 2 * j * ONE_16_16 / span;
            // spec: S-0104 Formulas, "Untinted table" (o_c = min(255, ⌊u_c·factor / 65536⌋))
            let ch = |u: u8| (i64::from(u) * factor / ONE_16_16).min(255) as u8;
            let mut row = [Rgb8::default(); PALETTE_LEN];
            for (out, u) in row.iter_mut().zip(palette.colors.iter()).skip(1) {
                *out = Rgb8::new(ch(u.r), ch(u.g), ch(u.b));
            }
            row
        })
        .collect()
}

/// Shade rows packed into the 5-6-5 hi-colour layout (S-0103 claim 9).
/// Entry 0 stays transparent; its packed value is meaningless.
pub fn pack_565(rows: &[[Rgb8; PALETTE_LEN]]) -> Vec<[u16; PALETTE_LEN]> {
    rows.iter().map(|row| row.map(hicolor_565)).collect()
}

/// The shade row of a pixel drawn at brightness `b` ‰ with a table of
/// `rows` rows and light-alpha `alpha` (S-0104 claim 15, Formulas
/// "Brightness to row"). [`DEFAULT_LIGHT_ALPHA`] gives 1000 → row 26 of
/// 53.
pub fn brightness_row(b: i32, rows: usize, alpha: i32) -> usize {
    alpha_row(alpha, brightness_rho(b), rows)
}

/// ρ of a brightness `b` ‰ (S-0104 claim 15): min(254, ⌊max(b, 0)·261 / 2048⌋).
pub fn brightness_rho(b: i32) -> i32 {
    // spec: S-0104 claim 15, Constants (row mapping: ρ = min(254, ⌊max(b, 0)·261 / 2048⌋))
    (i64::from(b.max(0)) * 261 / 2048).min(254) as i32
}

/// The shade row for light-alpha `alpha` (0 … 255, S-0104 Terms) and ρ
/// `rho` with a table of `rows` rows (S-0104 claim 15).
pub fn alpha_row(alpha: i32, rho: i32, rows: usize) -> usize {
    if rows == 0 {
        return 0;
    }
    let span = rows as i64 - 1;
    // spec: S-0104 claim 15, Constants (row = min(N − 1, ⌊a·ρ·(N − 1) / 32258⌋))
    let row = (i64::from(alpha) * i64::from(rho) * span / 32258).min(span);
    // spec: S-0104 Terms (light-alpha lies in 0 … 255 and ρ ≥ 0, so the row is never negative)
    row.max(0) as usize
}

/// The cell under a building's centre (S-0104 claim 13): for a foundation
/// of `size` cells along x and y with top-left cell `top_left`; on an even
/// side the centre lies on a cell edge and the larger coordinate counts.
pub fn building_centre_cell(top_left: (i32, i32), size: (i32, i32)) -> (i32, i32) {
    // spec: S-0104 claim 13 (the cell under the centre; on an even side, the larger coordinate)
    (top_left.0 + size.0 / 2, top_left.1 + size.1 / 2)
}

/// The cell whose light an ordinary building, the voxel parts of a
/// terrain-palette building and a building's own animations take (S-0104
/// claim 13): the top-left foundation cell, or with
/// `quirks.building_light_from_top_left_cell` off the cell under the
/// centre.
pub fn building_light_cell(top_left: (i32, i32), size: (i32, i32), q: &LightQuirks) -> (i32, i32) {
    // spec: S-0104 claim 13, Edge cases (quirks.building_light_from_top_left_cell)
    if q.building_light_from_top_left_cell {
        top_left
    } else {
        building_centre_cell(top_left, size)
    }
}

/// What is drawn, as far as its brightness goes (S-0104 claim 13, Formulas
/// "Object brightness"). Each variant says whose cell levels to pass to
/// [`object_brightness`]; *h* is the height above the surface in leptons
/// (S-0104 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LitObject {
    /// A vehicle of any kind; its own cell.
    Vehicle {
        /// Standing or driving on a bridge deck.
        on_bridge_deck: bool,
        /// Its cell is a shadowed cell.
        shadowed_cell: bool,
        /// Height *h* above the surface, leptons.
        height: i32,
    },
    /// An infantry unit; its own cell.
    Infantry {
        /// Standing on a bridge deck.
        on_bridge_deck: bool,
        /// Its cell is a shadowed cell.
        shadowed_cell: bool,
        /// Height *h* above the surface (the deck when on one), leptons.
        height: i32,
    },
    /// An aircraft; its own cell.
    Aircraft {
        /// Height *h* above the surface, leptons.
        height: i32,
    },
    /// Any part of an ordinary building; the cell of
    /// [`building_light_cell`].
    OrdinaryBuilding,
    /// The body and sprite parts of a terrain-palette building; the cell of
    /// [`building_centre_cell`].
    TerrainPaletteBuildingBody,
    /// The voxel parts of a terrain-palette building; the cell of
    /// [`building_light_cell`].
    TerrainPaletteBuildingVoxel,
    /// A plain terrain object; its own cell.
    PlainTerrainObject,
    /// An ore-spawning terrain object; its own cell.
    OreSpawningTerrainObject,
    /// An ore or gem overlay.
    OreOrGemOverlay,
    /// A wall overlay; its own cell.
    WallOverlay,
    /// Any overlay in a deck cell; its own cell.
    DeckOverlay,
    /// Any other overlay; its own cell.
    OtherOverlay,
    /// A vein animation or an animation created as part of a cell's
    /// terrain (S-0103 claim 8, rows 1 and 2); its own cell.
    TerrainAnimation,
    /// An anim-palette animation (S-0103 claim 8, row 5); its own cell.
    AnimPaletteAnimation,
    /// A building's own animation; the cell of [`building_light_cell`],
    /// evaluated every tick.
    BuildingOwnAnimation,
    /// A parachute; the carried object's cell on the tick the parachute is
    /// created (the caller keeps the result for the fall).
    Parachute,
    /// A unit's deploy or undeploy animation; the unit's cell.
    DeployAnimation,
    /// An animation flagged to use normal light. The flag wins over every
    /// other case (claim 13: "in every case, row 3 included"), so the
    /// caller maps a flagged animation to this variant before any other,
    /// including [`LitObject::BuildingOwnAnimation`],
    /// [`LitObject::Parachute`] and [`LitObject::DeployAnimation`].
    NormalLightAnimation,
}

/// The three light values of the cell an object reads (S-0104 claim 9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CellLevels {
    /// Unscaled ground level *U*, ‰.
    pub unscaled_ground: i32,
    /// Scaled ground level, ‰.
    pub scaled_ground: i32,
    /// Scaled raised level, ‰.
    pub scaled_raised: i32,
}

impl From<&CellLight> for CellLevels {
    fn from(c: &CellLight) -> Self {
        Self {
            unscaled_ground: c.unscaled_ground,
            scaled_ground: c.scaled_ground,
            scaled_raised: c.scaled_raised,
        }
    }
}

/// The rules' extra-light values, ‰ as stored (S-0104 claim 13).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ExtraLight {
    /// Extra unit light *e*<sub>unit</sub> (vehicles).
    pub unit: i32,
    /// Extra infantry light *e*<sub>inf</sub>.
    pub infantry: i32,
    /// Extra aircraft light *e*<sub>air</sub>.
    pub aircraft: i32,
}

/// Height-term step: 2 height levels of 104 leptons.
const HEIGHT_STEP: i32 = 208; // spec: S-0104 claim 13, Constants (height-term step 208 leptons)

/// Added to vehicles and infantry in shadowed cells.
const SHADOW_PENALTY: i32 = -500; // spec: S-0104 claim 13, Constants (shadowed-cell penalty −500)

/// Height levels of a bridge deck above its cell's terrain.
const DECK_LEVELS: i32 = 4; // spec: S-0104 Constants (bridge deck height 4 height levels)

/// Leptons per height level.
const LEVEL_LEPTONS: i32 = 104; // spec: S-0104 claim 18 (4 height levels of 104 leptons)

/// An object's brightness *b* in ‰ (S-0104 claim 13, Formulas "Object
/// brightness"), unclamped. `cell` holds the levels of the cell
/// [`LitObject`] names. `lambda` is the level factor of the set the
/// per-cell ranking of claim 2 chooses, `lambda_air` the ion storm's while
/// a storm is in force and the normal one otherwise.
pub fn object_brightness(
    object: LitObject,
    cell: CellLevels,
    lambda: i32,
    lambda_air: i32,
    extra: ExtraLight,
    q: &LightQuirks,
) -> i32 {
    let u = cell.unscaled_ground;
    match object {
        LitObject::Vehicle {
            on_bridge_deck,
            shadowed_cell,
            height,
        } => {
            if on_bridge_deck {
                // spec: S-0104 Formulas, "Object brightness" (vehicle on a bridge deck: U + 4λ + e_unit)
                u + DECK_LEVELS * lambda + extra.unit
            } else if shadowed_cell && height < HEIGHT_STEP {
                // spec: S-0104 Formulas, "Object brightness" (shadowed cell, h < 208: U − 500 + e_unit)
                u + SHADOW_PENALTY + extra.unit
            } else {
                // spec: S-0104 Formulas, "Object brightness" (any other vehicle: U + e_unit)
                u + extra.unit
            }
        }
        LitObject::Infantry {
            on_bridge_deck,
            shadowed_cell,
            height,
        } if !q.infantry_height_light_limited => {
            // spec: S-0104 claim 18 (switch off: h above the terrain, so a deck adds its 4 levels
            // of 104 leptons)
            let h = if on_bridge_deck {
                height + DECK_LEVELS * LEVEL_LEPTONS
            } else {
                height
            };
            // spec: S-0104 claim 18 (the −500 penalty is added to the height term, in a shadowed
            // cell, off a bridge deck, at h ≤ 208)
            let penalty = if shadowed_cell && !on_bridge_deck && h <= HEIGHT_STEP {
                SHADOW_PENALTY
            } else {
                0
            };
            // spec: S-0104 claim 18 (b = U + e_inf + (h ÷ 208)·λ + P)
            u + extra.infantry + h / HEIGHT_STEP * lambda + penalty
        }
        LitObject::Infantry {
            on_bridge_deck,
            shadowed_cell,
            height,
        } => {
            let term = if on_bridge_deck || (shadowed_cell && height > HEIGHT_STEP) {
                // spec: S-0104 Formulas, "Object brightness" (infantry: (h ÷ 208)·λ on a deck or shadowed above 208)
                height / HEIGHT_STEP * lambda
            } else if shadowed_cell {
                // spec: S-0104 Formulas, "Object brightness" (infantry, shadowed cell, h ≤ 208: −500)
                SHADOW_PENALTY
            } else {
                // spec: S-0104 claim 13 (anywhere else infantry ignore their height)
                0
            };
            u + term + extra.infantry
        }
        LitObject::Aircraft { height } => {
            // spec: S-0104 claim 13, Edge cases (quirks.aircraft_height_light_ion_only)
            let l = if q.aircraft_height_light_ion_only {
                lambda_air
            } else {
                lambda
            };
            // spec: S-0104 Formulas, "Object brightness" (aircraft: U + (h ÷ 208)·λ_air + e_air)
            u + height / HEIGHT_STEP * l + extra.aircraft
        }
        // spec: S-0104 Formulas, "Object brightness" (buildings, voxel parts, ore-spawning terrain objects, walls, own animations, parachutes: U)
        LitObject::OrdinaryBuilding
        | LitObject::TerrainPaletteBuildingVoxel
        | LitObject::OreSpawningTerrainObject
        | LitObject::WallOverlay
        | LitObject::AnimPaletteAnimation
        | LitObject::BuildingOwnAnimation
        | LitObject::Parachute => u,
        // spec: S-0104 Formulas, "Object brightness" (terrain-palette body, plain terrain objects, other overlays: scaled ground); claim 13 (vein and terrain animations)
        LitObject::TerrainPaletteBuildingBody
        | LitObject::PlainTerrainObject
        | LitObject::OtherOverlay
        | LitObject::TerrainAnimation => cell.scaled_ground,
        // spec: S-0104 Formulas, "Object brightness" (overlay in a deck cell: scaled raised level)
        LitObject::DeckOverlay => cell.scaled_raised,
        // spec: S-0104 Formulas, "Object brightness" (ore and gem overlays: 1000)
        LitObject::OreOrGemOverlay => NEUTRAL,
        LitObject::DeployAnimation => {
            // spec: S-0104 claim 13, Edge cases (quirks.deploy_animation_unlit: 1000; off, U of the unit's cell)
            if q.deploy_animation_unlit { NEUTRAL } else { u }
        }
        // spec: S-0104 claim 13 (an animation flagged to use normal light is drawn at 1000)
        LitObject::NormalLightAnimation => NEUTRAL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::TINT_EXEMPT;

    const Q: LightQuirks = LightQuirks {
        tile_table_kept_on_relight: true,
        near_black_tint_zeroes_scaled_ground: true,
        red_normalisation_divisor: true,
        normalisation_truncated_scale: true,
        dominator_ground_uses_nuke_level: true,
        incremental_relight_differs: true,
        aircraft_height_light_ion_only: true,
        building_light_from_top_left_cell: true,
        deploy_animation_unlit: true,
        building_light_tint_default_1000: true,
        infantry_height_light_limited: true,
        table_root_distance: true,
        chop_rounding: true,
    };

    /// Synthetic palette: entry i = (i, 255 − i, i / 2).
    fn synthetic() -> Palette {
        let mut p = Palette::default();
        for (i, c) in p.colors.iter_mut().enumerate() {
            let i = i as u8;
            *c = Rgb8::new(i, 255 - i, i / 2);
        }
        p
    }

    /// Synthetic palette with every channel of entry i equal to i.
    fn grey() -> Palette {
        let mut p = Palette::default();
        for (i, c) in p.colors.iter_mut().enumerate() {
            *c = Rgb8::new(i as u8, i as u8, i as u8);
        }
        p
    }

    /// spec: S-0104 Edge cases and quirks (every switch defaults to on)
    #[test]
    fn default_quirks_are_on() {
        assert_eq!(LightQuirks::default(), Q);
    }

    /// spec: S-0104 claim 7, Formulas "Shade table"
    #[test]
    fn tint_factor_worked_values() {
        assert_eq!(tint_factor(1000), 65535);
        assert_eq!(tint_factor(500), 32767);
        assert_eq!(tint_factor(2000), 131071);
        assert_eq!(tint_factor(999), 65470);
        assert_eq!(tint_factor(0), 0);
        assert_eq!(tint_factor(125), 8191);
        // Clamped to [0, 2000].
        assert_eq!(tint_factor(-5), 0);
        assert_eq!(tint_factor(1_000_000), 131071);
    }

    /// spec: S-0104 Formulas, "Tint normalisation" (K has no ε)
    #[test]
    fn normalisation_scale_values() {
        assert_eq!(normalisation_scale(2000), 131072);
        assert_eq!(normalisation_scale(1000), 65536);
        assert_eq!(normalisation_scale(800), 52428);
        assert_eq!(normalisation_scale(1010), 66191);
        assert_eq!(normalisation_scale(1030), 67502);
        assert_eq!(normalisation_scale(994), 65142);
    }

    /// spec: S-0104 Constants (δ is the double nearest to 0.001, slightly above it)
    #[test]
    fn delta_bits_are_the_double_nearest_a_thousandth() {
        assert_eq!(f64::from_bits(DELTA_BITS), 0.001);
        // Slightly above 0.001: m · 2^-62 · 1000 > 1, exactly.
        let m = u128::from(DELTA_BITS & ((1 << 52) - 1)) | (1 << 52);
        assert_eq!((DELTA_BITS >> 52) & 0x7FF, 1023 - 10);
        assert!(m * 1000 > 1u128 << 62);
    }

    /// spec: S-0104 Formulas, "Tint normalisation" (𝒟: 284 above, 1699 below, 16 equal)
    #[test]
    fn d_set_counts_and_smallest_members() {
        let mut counts = [0usize; 3];
        let mut members = Vec::new();
        for x in 2..=2000 {
            match d_compare(x) {
                Ordering::Greater => {
                    counts[0] += 1;
                    members.push(x);
                }
                Ordering::Less => counts[1] += 1,
                Ordering::Equal => counts[2] += 1,
            }
        }
        assert_eq!(counts, [284, 1699, 16]);
        assert_eq!(
            members[..15],
            [2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 20, 24, 28, 32]
        );
        assert_eq!(members[15..17], [40, 47]);
    }

    /// spec: S-0104 claim 8 (grey map tints whose ‰ value is in 𝒟)
    #[test]
    fn grey_map_tints_in_d_set() {
        let listed = [
            10, 20, 40, 80, 110, 160, 190, 220, 320, 380, 440, 510, 630, 640, 760, 880, 890, 1010,
            1020, 1260, 1270, 1280, 1510, 1520, 1530, 1760, 1770, 1780, 1790,
        ];
        let found: Vec<i32> = (1..=200).map(|h| h * 10).filter(|&x| in_d_set(x)).collect();
        assert_eq!(found, listed);
        assert!(!in_d_set(1000));
        assert!(!in_d_set(1030));
        assert!(!in_d_set(800));
        assert!(!in_d_set(2000));
    }

    /// spec: S-0104 claim 8, Formulas "Tint normalisation" (red largest: 1435 pairs one lower, and
    /// the integer form equals the floating-point division by D(X))
    #[test]
    fn red_largest_pairs() {
        let mut lower = 0;
        for x in 2..=2000i64 {
            // D(X) = p · 2^-46 with p = X·m truncated to 53 bits.
            let m = u128::from(DELTA_BITS & ((1 << 52) - 1)) | (1 << 52);
            let p = x as u128 * m;
            let drop = (128 - p.leading_zeros()).saturating_sub(53);
            let p = (p >> drop) << drop;
            for c in 0..=x {
                let n = red_largest_channel(c, x, &Q);
                // ⌊RZ(c·65536 / D)⌋ = ⌊c·65536 / D⌋ = ⌊c·2^62 / p⌋.
                let float = ((c as u128) << 62) / p;
                assert_eq!(n as u128, float, "X = {x}, c = {c}");
                if n < 1000 * c / x {
                    lower += 1;
                }
                assert_eq!(
                    red_largest_channel(
                        c,
                        x,
                        &LightQuirks {
                            red_normalisation_divisor: false,
                            ..Q
                        }
                    ),
                    1000 * c / x
                );
            }
        }
        assert_eq!(lower, 1435);
    }

    /// spec: S-0104 claim 8, Formulas "Tint normalisation" (green or blue largest: 4920 pairs +1,
    /// 3 pairs +2, 1 pair +3)
    #[test]
    fn truncated_scale_pairs() {
        let mut above = [0usize; 4];
        for x in 2..=2000i64 {
            let k = normalisation_scale(x as i32);
            for c in 0..=x {
                let diff = c * 65536 / k - 1000 * c / x;
                assert!((0..=3).contains(&diff));
                above[diff as usize] += 1;
            }
        }
        assert_eq!(above[1..], [4920, 3, 1]);
    }

    /// spec: S-0104 claim 8 (X = 2, c = 1 gives 499 when red is largest)
    #[test]
    fn red_largest_x2_c1() {
        assert_eq!(normalise([2, 1, 0], &Q).tint, [1000, 499, 0]);
        let off = LightQuirks {
            red_normalisation_divisor: false,
            ..Q
        };
        assert_eq!(normalise([2, 1, 0], &off).tint, [1000, 500, 0]);
    }

    /// spec: S-0104 Formulas, "Tint normalisation" (blue tying with a largest green)
    #[test]
    fn green_largest_blue_tie_exceeds_1000() {
        for (x, blue) in [
            (3, 1003),
            (5, 1002),
            (7, 1001),
            (9, 1001),
            (11, 1001),
            (13, 1001),
        ] {
            let n = normalise([0, x, x], &Q);
            assert_eq!(n.tint, [0, 1000, blue], "X = {x}");
            // Quantisation clamps the excess.
            assert_eq!(quantise(n.tint, DetailLevel::High), [0, 992, 992]);
        }
        let others: Vec<i32> = (2..=2000)
            .filter(|&x| normalise([0, x, x], &Q).tint[2] > 1000)
            .collect();
        assert_eq!(others, [3, 5, 7, 9, 11, 13]);
        let off = LightQuirks {
            normalisation_truncated_scale: false,
            ..Q
        };
        assert_eq!(normalise([0, 3, 3], &off).tint, [0, 1000, 1000]);
    }

    /// spec: S-0104 claim 8 (ties: red counts as largest; grey tints in 𝒟 miss white)
    #[test]
    fn grey_tints() {
        assert_eq!(normalise([1010; 3], &Q).tint, [1000, 999, 999]);
        assert_eq!(normalise([1010; 3], &Q).k, 66191);
        assert_eq!(normalise([1030; 3], &Q).tint, [1000; 3]);
        assert_eq!(normalise([2000; 3], &Q).tint, [1000; 3]);
        assert_eq!(normalise([2000; 3], &Q).k, 131072);
        for x in 2..=8 {
            assert_eq!(normalise([x; 3], &Q).tint, [1000, 999, 999]);
        }
        let off = LightQuirks {
            red_normalisation_divisor: false,
            ..Q
        };
        assert!((2..=2000).all(|x| normalise([x; 3], &off).tint == [1000; 3]));
    }

    /// spec: S-0104 claim 8 (near-black tint), Formulas "Tint normalisation"
    #[test]
    fn near_black() {
        for raw in [[0, 0, 0], [1, 1, 0], [1, 0, 1], [-50, 1, -3]] {
            let n = normalise(raw, &Q);
            assert_eq!(n.tint, [1000; 3]);
            assert_eq!(n.k, 65536);
            assert!(n.near_black);
        }
        let off = LightQuirks {
            near_black_tint_zeroes_scaled_ground: false,
            ..Q
        };
        assert!(!normalise([0, 0, 0], &off).near_black);
        assert!(!normalise([2, 0, 0], &Q).near_black);
        assert_eq!(
            normalise([1000; 3], &Q),
            Normalised {
                tint: [1000; 3],
                k: 65536,
                near_black: false
            }
        );
    }

    /// spec: S-0104 claim 8, Scenario L6 (quantisation by detail level), Terms (quantised white)
    #[test]
    fn quantisation() {
        let n = normalise([1000, 900, 800], &Q);
        assert_eq!(n.tint, [1000, 900, 800]);
        assert_eq!(quantise(n.tint, DetailLevel::Low), [896, 896, 768]);
        assert_eq!(quantise(n.tint, DetailLevel::Medium), [960, 896, 768]);
        assert_eq!(quantise(n.tint, DetailLevel::High), [992, 896, 800]);
        assert_eq!(quantise([1000; 3], DetailLevel::Low), [896; 3]);
        assert_eq!(quantise([1000; 3], DetailLevel::Medium), [960; 3]);
        assert_eq!(quantise([1000; 3], DetailLevel::High), [992; 3]);
        assert_eq!(quantise([-5, 1003, 31], DetailLevel::High), [0, 992, 0]);
        assert_eq!(
            choose_tile_table(&n, DetailLevel::High),
            TileTable::Quantised([992, 896, 800])
        );
    }

    /// spec: S-0104 Import mapping (DetailLevel clamped to 0 … 2), Constants (tint quantum)
    #[test]
    fn detail_levels() {
        assert_eq!(DetailLevel::from_settings(-3), DetailLevel::Low);
        assert_eq!(DetailLevel::from_settings(1), DetailLevel::Medium);
        assert_eq!(DetailLevel::from_settings(9), DetailLevel::High);
        assert_eq!(DetailLevel::Low.quantum(), 128);
        assert_eq!(DetailLevel::Medium.quantum(), 64);
        assert_eq!(DetailLevel::High.quantum(), 32);
        assert!(DetailLevel::Low < DetailLevel::Medium && DetailLevel::Medium < DetailLevel::High);
    }

    /// spec: S-0104 claim 8, Formulas "Tile table choice" (53 rows, or 27 below a sum of 2000)
    #[test]
    fn table_rows() {
        assert_eq!(TileTable::Neutral.rows(), 53);
        assert_eq!(TileTable::Neutral.tint(), [1000; 3]);
        assert_eq!(TileTable::Quantised([992; 3]).rows(), 53);
        assert_eq!(TileTable::Quantised([1000, 1000, 0]).rows(), 53);
        assert_eq!(TileTable::Quantised([992, 992, 0]).rows(), 27);
        assert_eq!(TileTable::Quantised([480, 992, 992]).rows(), 53);
        assert_eq!(TileTable::Quantised([480, 992, 480]).rows(), 27);
        assert_eq!(
            TileTable::Quantised([480, 992, 480]).tint(),
            [480, 992, 480]
        );
    }

    /// spec: S-0104 claim 8 (table chosen on the normalised tint), Scenarios L18 Part B and L19
    #[test]
    fn table_choice_on_normalised_tint() {
        let black = normalise([0; 3], &Q);
        assert_eq!(
            choose_tile_table(&black, DetailLevel::Low),
            TileTable::Neutral
        );
        let g1010 = normalise([1010; 3], &Q);
        assert_eq!(
            choose_tile_table(&g1010, DetailLevel::Low),
            TileTable::Quantised([896; 3])
        );
        let g1030 = normalise([1030; 3], &Q);
        assert_eq!(
            choose_tile_table(&g1030, DetailLevel::Low),
            TileTable::Neutral
        );
    }

    /// spec: S-0104 claim 8 (kept table at a light change), Scenario L20
    #[test]
    fn kept_table_on_relight() {
        // While the light stands: raw red 1008 … 1002 gives (1000, 992 … 998, …).
        for (n, g) in [(2, 992), (8, 998)] {
            let norm = normalise([1000 + 10 - n, 1000, 1000], &Q);
            assert_eq!(norm.tint, [1000, g, g]);
            assert_eq!(
                choose_tile_table(&norm, DetailLevel::High),
                TileTable::Quantised([992; 3])
            );
        }
        // The light goes out: the normalised tint is exactly white.
        let white = normalise([1000; 3], &Q);
        let current = TileTable::Quantised([992; 3]);
        assert_eq!(
            choose_tile_table_on_relight(current, [992; 3], &white, DetailLevel::High, &Q),
            current
        );
        let off = LightQuirks {
            tile_table_kept_on_relight: false,
            ..Q
        };
        assert_eq!(
            choose_tile_table_on_relight(current, [992; 3], &white, DetailLevel::High, &off),
            TileTable::Neutral
        );
        // The neutral tile table is never kept: a quantised tint is never white.
        let tinted = normalise([1000, 900, 800], &Q);
        assert_eq!(
            choose_tile_table_on_relight(
                TileTable::Neutral,
                [1000; 3],
                &tinted,
                DetailLevel::High,
                &Q
            ),
            TileTable::Quantised([992, 896, 800])
        );
    }

    /// spec: S-0104 claim 8 (green largest normalises high), Scenario L22
    #[test]
    fn scenario_l22() {
        let light = |tint| LightSource {
            x: 5 * 256 + 128,
            y: 5 * 256 + 128,
            radius: 2560,
            intensity: 1,
            tint,
            required_detail: DetailLevel::High,
        };
        let r = full_recompute(
            101,
            0,
            0,
            [99, 82, 82],
            (6, 5),
            0,
            &[light([5, 8, 8])],
            DetailLevel::High,
            &Q,
        );
        assert_eq!(r.raw_tint, [994, 827, 827]);
        assert_eq!(r.normalised.tint, [1000, 831, 831]);
        assert_eq!(r.table, TileTable::Quantised([992, 800, 800]));
        let g = full_recompute(
            101,
            0,
            0,
            [82, 99, 82],
            (5, 6),
            0,
            &[light([8, 5, 8])],
            DetailLevel::High,
            &Q,
        );
        assert_eq!(g.raw_tint, [827, 994, 827]);
        assert_eq!(g.normalised.k, 65142);
        assert_eq!(g.normalised.tint, [832, 1000, 832]);
        assert_eq!(g.table, TileTable::Quantised([832, 992, 832]));
        for c in [r, g] {
            assert_eq!(c.normalised.k, 65142);
            assert_eq!(c.unscaled_ground, 1010);
            assert_eq!(c.scaled_ground, 1003);
            assert_eq!(c.table.rows(), 53);
            assert_eq!(brightness_row(c.scaled_ground, 53, DEFAULT_LIGHT_ALPHA), 26);
        }
        // Blue 248 at row 26 of each table: 198 and 206.
        let p = grey();
        assert_eq!(shade_table(&p, r.table.tint(), 53, None)[26][248].b, 198);
        assert_eq!(shade_table(&p, g.table.tint(), 53, None)[26][248].b, 206);
    }

    /// spec: S-0104 claim 11, Scenario L7 (falloff profile, building lights only at high detail)
    #[test]
    fn falloff_profile_and_detail() {
        let light = LightSource::building((10, 10), (1, 1), 2560, 500, [0; 3]);
        assert_eq!((light.x, light.y), (10 * 256 + 128, 10 * 256 + 128));
        for k in 0..=10 {
            let f = light.falloff((10 + k, 10), &Q).unwrap();
            assert_eq!(f, 1000 - 100 * i64::from(k));
            let hi = full_recompute(
                30,
                0,
                0,
                [100; 3],
                (10, 10 + k),
                0,
                &[light],
                DetailLevel::High,
                &Q,
            );
            assert_eq!(hi.unscaled_ground, 300 + 500 - 50 * k);
            for detail in [DetailLevel::Low, DetailLevel::Medium] {
                let lo = full_recompute(30, 0, 0, [100; 3], (10, 10 + k), 0, &[light], detail, &Q);
                assert_eq!(lo.unscaled_ground, 300);
                assert_eq!(lo.source_sum, 0);
            }
        }
        assert_eq!(light.falloff((21, 10), &Q), None);
        // Diagonal: Δ = (256, 256), d = ⌊√131072⌋ = 362, f = ⌊1000·2198 / 2560⌋ = 858.
        assert_eq!(light.falloff((11, 11), &Q), Some(858));
    }

    /// #80. spec: S-0104 claim 11, Formulas "Cell light" (d = ⌊√ₜ(n)⌋, the table square root
    /// of S-0105 F1 and S-0204 claim 5; for a building light it first differs from the integer
    /// root at 16388 leptons, with the light half a cell off one axis; reach stays n ≤ R²)
    #[test]
    fn falloff_takes_the_table_root() {
        use glasnost_num::Rounding;
        use glasnost_num::geom::{isqrt, root_floor};
        // A 2 × 1 foundation: its light is half a cell off the cell centres along x.
        let radius = 24000;
        let light = LightSource::building((0, 0), (2, 1), radius, 500, [0; 3]);
        assert_eq!((light.x, light.y), (256, 128));
        // quirks.table_root_distance off (S-0204): the integer root.
        let off = LightQuirks {
            table_root_distance: false,
            ..Q
        };
        let mut differing = 0;
        for i in 0..100 {
            for j in 0..100 {
                let (dx, dy) = (i64::from(i) * 256 - 128, i64::from(j) * 256);
                let n = (dx * dx + dy * dy) as u64;
                let f = light.falloff((i, j), &Q);
                if n > (radius as u64).pow(2) {
                    assert_eq!(f, None, "cell ({i}, {j})");
                    assert_eq!(light.falloff((i, j), &off), None, "cell ({i}, {j})");
                    continue;
                }
                let d = root_floor(n, true, Rounding::TowardZero) as i64;
                let exact = isqrt(u128::from(n)) as i64;
                if d != exact {
                    differing += 1;
                    assert!(exact >= 16388, "cell ({i}, {j}) at {exact}");
                }
                let r = i64::from(radius);
                assert_eq!(f, Some(1000 * (r - d) / r), "cell ({i}, {j})");
                assert_eq!(
                    light.falloff((i, j), &off),
                    Some(1000 * (r - exact) / r),
                    "cell ({i}, {j})"
                );
                // A building light's n is exact in single precision: the mode changes nothing.
                let nearest = LightQuirks {
                    chop_rounding: false,
                    ..Q
                };
                assert_eq!(light.falloff((i, j), &nearest), f, "cell ({i}, {j})");
            }
        }
        assert!(differing > 0, "some reached cell takes a lower table root");
        // Reach is n ≤ R² exactly: a cell whose integer distance truncates to R is not reached.
        let edge = LightSource::building((0, 0), (1, 1), 300, 500, [0; 3]);
        // Δ = (256, 256): n = 131072, ⌊√n⌋ = 362 but 362² = 131044 < n; R = 363 reaches it.
        let at = |radius| LightSource { radius, ..edge };
        assert_eq!(at(363).falloff((1, 1), &Q), Some(1000 / 363));
        assert_eq!(at(362).falloff((1, 1), &Q), None);
    }

    /// spec: S-0525 claim 13 (a field's light: at the centre point of its centre cell, radius R,
    /// counting at every detail level, low and medium included)
    #[test]
    fn field_light_counts_at_every_detail_level() {
        let light = LightSource::field((10, 10), 2560, 500, [0, 1000, 0]);
        assert_eq!((light.x, light.y), (10 * 256 + 128, 10 * 256 + 128));
        assert_eq!(light.falloff((10, 10), &Q), Some(1000));
        assert_eq!(light.falloff((21, 10), &Q), None);
        for detail in [DetailLevel::Low, DetailLevel::Medium, DetailLevel::High] {
            let c = full_recompute(30, 0, 0, [100; 3], (10, 10), 0, &[light], detail, &Q);
            assert_eq!(c.unscaled_ground, 300 + 500);
            assert_eq!(c.raw_tint[1], c.raw_tint[0] + 1000);
        }
    }

    /// spec: S-0104 claim 11 (negative lights darken symmetrically; contributions add up)
    #[test]
    fn negative_and_summed_lights() {
        let at = |x: i32, intensity, tint| LightSource {
            x: x * 256 + 128,
            y: 128,
            radius: 1000,
            intensity,
            tint,
            required_detail: DetailLevel::Low,
        };
        // d = 256: f = ⌊1000·744/1000⌋ = 744; ⌊−333·744/1000⌋ = −247 (toward zero).
        let c = full_recompute(
            100,
            0,
            0,
            [100; 3],
            (1, 0),
            0,
            &[at(0, -333, [-333, 0, 333]), at(2, 333, [0; 3])],
            DetailLevel::Low,
            &Q,
        );
        assert_eq!(c.source_sum, 0);
        assert_eq!(c.raw_tint, [1000 - 247, 1000, 1000 + 247]);
    }

    /// spec: S-0104 claim 11 (radius 0 reaches no cell: a departure)
    #[test]
    fn radius_zero_reaches_nothing() {
        let light = LightSource::building((3, 3), (1, 1), 0, 500, [0; 3]);
        assert_eq!(light.falloff((3, 3), &Q), None);
        let c = full_recompute(
            100,
            0,
            0,
            [100; 3],
            (3, 3),
            0,
            &[light],
            DetailLevel::High,
            &Q,
        );
        assert_eq!(c.unscaled_ground, 1000);
    }

    /// spec: S-0104 claim 9, Formulas "Cell light" (ground, raised, ground bias and level factor)
    #[test]
    fn ground_and_raised_levels() {
        // Defaults: ground bias 50 ‰, level factor 8 ‰ per level.
        let c = full_recompute(100, 8, 50, [100; 3], (0, 0), 3, &[], DetailLevel::High, &Q);
        assert_eq!(c.unscaled_ground, 1000 + 24 - 50);
        assert_eq!(c.scaled_ground, 974);
        assert_eq!(c.scaled_raised, 1000 + 56 - 50);
        assert_eq!(c.table, TileTable::Neutral);
        assert_eq!(brightness_row(950, 53, DEFAULT_LIGHT_ALPHA), 24);
        // Clamps.
        let dark = full_recompute(0, 0, 50, [100; 3], (0, 0), 0, &[], DetailLevel::High, &Q);
        assert_eq!((dark.unscaled_ground, dark.scaled_ground), (0, 0));
        let bright = full_recompute(300, 0, 0, [100; 3], (0, 0), 0, &[], DetailLevel::High, &Q);
        assert_eq!(bright.unscaled_ground, 2000);
        assert_eq!(bright.scaled_raised, 2000);
    }

    /// spec: S-0104 Formulas "Cell light", Scenario L23 Part A (scaled ground of a grey 800 tint)
    #[test]
    fn scaled_ground_grey_800() {
        let c = full_recompute(100, 0, 0, [80; 3], (0, 0), 0, &[], DetailLevel::High, &Q);
        assert_eq!(c.normalised.k, 52428);
        assert_eq!(c.table, TileTable::Neutral);
        assert_eq!((c.unscaled_ground, c.scaled_ground), (1000, 799));
        assert_eq!(brightness_row(799, 53, DEFAULT_LIGHT_ALPHA), 20);
    }

    /// spec: S-0104 claim 9, Formulas "Incremental update", Scenario L17
    #[test]
    fn incremental_scales_unclamped_ground() {
        let full = full_recompute(250, 0, 0, [80; 3], (0, 0), 0, &[], DetailLevel::High, &Q);
        assert_eq!(full.normalised.k, 52428);
        assert_eq!(full.scaled_ground, 1599);
        assert_eq!(brightness_row(1599, 53, DEFAULT_LIGHT_ALPHA), 41);
        let inc = incremental_update(&full, 240, HeightTerms::uniform(0, 0), 0, &Q);
        assert_eq!(inc.scaled_ground, 1919);
        assert_eq!(brightness_row(1919, 53, DEFAULT_LIGHT_ALPHA), 49);
        assert_eq!(inc.table, full.table);
        let again = full_recompute(240, 0, 0, [80; 3], (0, 0), 0, &[], DetailLevel::High, &Q);
        assert_eq!(again.scaled_ground, 1599);
        let off = LightQuirks {
            incremental_relight_differs: false,
            ..Q
        };
        assert_eq!(
            incremental_update(&full, 240, HeightTerms::uniform(0, 0), 0, &off),
            again
        );
    }

    /// spec: S-0104 claim 8 (near-black zeroes only the scaled ground), Scenario L18 Part A
    #[test]
    fn near_black_scaled_ground() {
        let full = full_recompute(100, 0, 0, [0; 3], (0, 0), 0, &[], DetailLevel::High, &Q);
        assert_eq!(full.table, TileTable::Neutral);
        assert_eq!(full.scaled_ground, 0);
        assert_eq!(full.unscaled_ground, 1000);
        assert_eq!(full.scaled_raised, 1000);
        assert_eq!(
            brightness_row(full.scaled_ground, 53, DEFAULT_LIGHT_ALPHA),
            0
        );
        let inc = incremental_update(&full, 90, HeightTerms::uniform(0, 0), 0, &Q);
        assert_eq!((inc.scaled_ground, inc.unscaled_ground), (900, 900));
        assert_eq!(brightness_row(900, 53, DEFAULT_LIGHT_ALPHA), 23);
        let off = LightQuirks {
            near_black_tint_zeroes_scaled_ground: false,
            ..Q
        };
        let full_off = full_recompute(100, 0, 0, [0; 3], (0, 0), 0, &[], DetailLevel::High, &off);
        assert_eq!(full_off.scaled_ground, 1000);
    }

    /// spec: S-0104 claim 12 (an untinted building light's 1000.0 tint saturates to 2000, scale 2)
    #[test]
    fn untinted_building_light_doubles_scaled_ground() {
        let light = LightSource::building((0, 0), (1, 1), 2560, 1, [1_000_000; 3]);
        let c = full_recompute(
            60,
            0,
            0,
            [100; 3],
            (3, 0),
            0,
            &[light],
            DetailLevel::High,
            &Q,
        );
        assert_eq!(normalise(c.raw_tint, &Q).k, 131072);
        assert_eq!(c.normalised.tint, [1000; 3]);
        assert_eq!(c.table, TileTable::Neutral);
        assert_eq!((c.unscaled_ground, c.scaled_ground), (600, 1200));
        let c = full_recompute(
            150,
            0,
            0,
            [100; 3],
            (3, 0),
            0,
            &[light],
            DetailLevel::High,
            &Q,
        );
        assert_eq!((c.unscaled_ground, c.scaled_ground), (1500, 2000));
    }

    /// spec: S-0104 claim 10 (dominator height bug), Scenario L8
    #[test]
    fn dominator_height_bug() {
        let terms = HeightTerms::dominator_full_recompute(0, 0, &Q);
        let c =
            full_recompute_with_terms(100, terms, [100; 3], (0, 0), 2, &[], DetailLevel::High, &Q);
        assert_eq!(c.unscaled_ground, 1200);
        assert_eq!(c.scaled_raised, 1000);
        let inc = incremental_update(&c, 100, HeightTerms::uniform(0, 0), 2, &Q);
        assert_eq!(inc.unscaled_ground, 1000);
        let off = LightQuirks {
            dominator_ground_uses_nuke_level: false,
            ..Q
        };
        assert_eq!(
            HeightTerms::dominator_full_recompute(7, 3, &off),
            HeightTerms::uniform(7, 3)
        );
    }

    /// spec: S-0104 claim 12, Edge cases (quirks.building_light_tint_default_1000)
    #[test]
    fn building_tint_default_switch() {
        let on = LightQuirks::default();
        let off = LightQuirks {
            building_light_tint_default_1000: false,
            ..on
        };
        assert_eq!(building_light_tint_default(&on), 1_000_000);
        assert_eq!(building_light_tint_default(&off), 1000);
        // A light without tint keys at falloff 500 ‰ on a neutral map tint:
        // on, each channel adds 500 000 ‰ and reaches the 2000 clamp; off,
        // it adds 500 ‰.
        let falloff = 500;
        let add = |t: i32| i64::from(t) * falloff / 1000;
        assert!(1000 + add(building_light_tint_default(&on)) >= i64::from(LEVEL_MAX));
        assert_eq!(1000 + add(building_light_tint_default(&off)), 1500);
    }

    /// spec: S-0104 claim 4, Formulas "Ambient fade"
    #[test]
    fn fade_steps() {
        assert_eq!(ambient_step(100, 50, 10), (90, 50));
        assert_eq!(ambient_step(55, 50, 10), (50, 50));
        assert_eq!(ambient_step(50, 200, 10), (60, 200));
        assert_eq!(ambient_step(195, 200, 10), (200, 200));
        assert_eq!(ambient_step(5, -30, 10), (0, 0));
        assert_eq!(ambient_step(50, -30, 10), (40, 0));
        assert_eq!(ambient_step(50, 50, 10), (50, 50));
    }

    /// spec: S-0104 Formulas, "Shade table" (L = 6 for 53 rows, 3 for 27)
    #[test]
    fn exempt_row_counts() {
        assert_eq!(exempt_rows(53), 6);
        assert_eq!(exempt_rows(27), 3);
        assert_eq!(exempt_rows(13), 0);
        assert_eq!(exempt_rows(14), 1);
    }

    /// spec: S-0104 claim 7 (neutral light: every non-zero channel one less; tint-exempt kept)
    #[test]
    fn neutral_row_is_one_less() {
        let p = synthetic();
        let table = shade_table(&p, [1000; 3], 53, Some(TINT_EXEMPT));
        assert_eq!(table.len(), 53);
        let row = &table[26];
        assert_eq!(row[0], Rgb8::default());
        for i in 1..=255u8 {
            let u = p.colors[usize::from(i)];
            let got = row[usize::from(i)];
            if TINT_EXEMPT.contains(&i) {
                assert_eq!(got, u, "index {i}");
            } else {
                let less = |c: u8| c.saturating_sub(1);
                assert_eq!(got, Rgb8::new(less(u.r), less(u.g), less(u.b)), "index {i}");
            }
        }
        // 248 shows 247's 5-bit step (Scenario L5).
        assert_eq!(shade_table(&grey(), [1000; 3], 53, None)[26][248].r, 247);
    }

    /// spec: S-0104 claim 6 (rows 0 and 52; tint-exempt ramp over the first L rows; r18: in 53 rows, row 6 and later are full palette colour, only rows below 6 darken)
    #[test]
    fn shade_rows() {
        let p = synthetic();
        let table = shade_table(&p, [1000; 3], 53, Some(TINT_EXEMPT));
        // Row 0 is black, tint-exempt entries included.
        assert!(table[0].iter().all(|c| *c == Rgb8::default()));
        // Row 52 is twice the tint, capped at 255: 2·65535 → 131056 after dropping bits.
        let u = p.colors[100];
        let twice = |c: u8| (i64::from(c) * 131056 / 65536).min(255) as u8;
        assert_eq!(
            table[52][100],
            Rgb8::new(twice(u.r), twice(u.g), twice(u.b))
        );
        // Tint-exempt: W(3) = 32768 of 6 rows, full from row 6 on.
        let e = p.colors[250];
        assert_eq!(table[3][250], Rgb8::new(e.r / 2, e.g / 2, e.b / 2));
        assert_eq!(table[5][250].r, (i64::from(e.r) * 54608 / 65536) as u8);
        assert_eq!(table[6][250], e);
        assert_eq!(table[52][250], e);
        assert!((0..6).all(|j| table[j][250].r < e.r));
        assert!((6..53).all(|j| table[j][250] == e));
        // No exempt indices (tile palette): index 250 follows the tint.
        let tile = shade_table(&p, [1000; 3], 53, None);
        assert_eq!(tile[52][250].r, 255);
    }

    /// spec: S-0104 claim 6, Scenarios L18–L22 (tinted rows with dropped low bits)
    #[test]
    fn tinted_rows_worked_values() {
        let p = grey();
        // Quantised white (896) at row 26: 248 → 222.
        assert_eq!(shade_table(&p, [896; 3], 53, None)[26][248].r, 222);
        // (500, 1000, 1000) at row 26: red 248 → 123, green 247.
        let t = shade_table(&p, [500, 1000, 1000], 53, None);
        assert_eq!((t[26][248].r, t[26][248].g), (123, 247));
        // Quantised (480, 992, 992): red 248 → 119.
        assert_eq!(shade_table(&p, [480, 992, 992], 53, None)[26][248].r, 119);
        // Row 36: 168 → 230 under (992, 992, 992), 232 under neutral.
        assert_eq!(shade_table(&p, [992; 3], 53, None)[36][168].r, 230);
        assert_eq!(shade_table(&p, [1000; 3], 53, None)[36][168].r, 232);
        assert_eq!(hicolor_565(Rgb8::new(230, 0, 0)) >> 11, 28);
        assert_eq!(hicolor_565(Rgb8::new(232, 0, 0)) >> 11, 29);
    }

    /// spec: S-0104 claim 6 (single row: the palette times the tint), Formulas "Shade table"
    #[test]
    fn single_row() {
        let p = synthetic();
        let t = shade_table(&p, [500, 1000, 2000], 1, Some(TINT_EXEMPT));
        assert_eq!(t.len(), 1);
        let u = p.colors[100];
        assert_eq!(
            t[0][100],
            Rgb8::new(
                (i64::from(u.r) * 32752 / 65536) as u8,
                (i64::from(u.g) * 65520 / 65536) as u8,
                (i64::from(u.b) * 131056 / 65536).min(255) as u8
            )
        );
        assert_eq!(t[0][245], p.colors[245]);
        assert_eq!(t[0][0], Rgb8::default());
    }

    /// #152: spec: S-0104 claim 6, Formulas "Shade table" (without tint-exempt indices only
    /// T_c(j) = ⌊2j·M_c / (N − 1)⌋ is used, so every shade count from 2 to 13, where L = 0,
    /// gives a table; W is undefined only for tint-exempt indices)
    #[test]
    fn small_shade_counts_without_exempt_indices() {
        let p = synthetic();
        for rows in 2..=13 {
            let t = shade_table(&p, [1000; 3], rows, None);
            assert_eq!(t.len(), rows);
            let u = p.colors[100];
            let m = tint_factor(1000);
            let span = rows as i64 - 1;
            assert_eq!(t[0][100], Rgb8::default(), "row 0 is black");
            let f = 2 * span * m / span / 16 * 16;
            assert_eq!(
                t[rows - 1][100].r,
                (i64::from(u.r) * f / 65536).min(255) as u8,
                "{rows} rows"
            );
        }
    }

    /// spec: S-0104 Formulas, "Shade table" (the full-factor form differs in one 5-bit channel)
    #[test]
    fn full_factor_form_differs_once() {
        let m = tint_factor(1000);
        let mut differ = Vec::new();
        for j in 0..53i64 {
            let t = 2 * j * m / 52;
            for u in 0..=255i64 {
                let dropped = (u * (t / 16 * 16) / 65536).min(255);
                let full = (u * t / 65536).min(255);
                if dropped >> 3 != full >> 3 {
                    differ.push((j, u));
                }
                assert_eq!(shade_channel(u as u8, t), dropped as u8);
            }
        }
        assert_eq!(differ, [(11, 227)]);
    }

    /// spec: S-0104 claims 6 and 7 (the untinted table's middle row reproduces the palette)
    #[test]
    fn untinted_table_middle_row() {
        let p = synthetic();
        let t = untinted_table(&p);
        assert_eq!(t.len(), 53);
        assert_eq!(t[26][0], Rgb8::default());
        assert_eq!(t[26][1..], p.colors[1..]);
        assert!(t[0].iter().all(|c| *c == Rgb8::default()));
        // Row 52: twice, capped (no tint-exempt indices).
        assert_eq!(t[52][200], Rgb8::new(255, 110, 200));
    }

    /// spec: S-0103 claim 9 (5-6-5 packing of shade rows)
    #[test]
    fn packing() {
        let rows = shade_table(&grey(), [1000; 3], 53, None);
        let packed = pack_565(&rows);
        assert_eq!(packed.len(), 53);
        assert_eq!(packed[26][248], hicolor_565(Rgb8::new(247, 247, 247)));
    }

    /// spec: S-0104 claim 15, Measurement scenarios (Common setup row list)
    #[test]
    fn brightness_rows() {
        let a = DEFAULT_LIGHT_ALPHA;
        for (b, row) in [
            (1000, 26),
            (2000, 52),
            (950, 24),
            (500, 12),
            (1075, 27),
            (1076, 28),
            (1400, 36),
            (1500, 39),
            (990, 25),
            (0, 0),
            (-300, 0),
            (5000, 52),
        ] {
            assert_eq!(brightness_row(b, 53, a), row, "b = {b}");
        }
        // S-0103 claim 7: row 6 from 236 ‰ up.
        assert_eq!(brightness_row(236, 53, a), 6);
        assert_eq!(brightness_row(235, 53, a), 5);
        assert_eq!(brightness_row(1000, 1, a), 0);
        assert_eq!(brightness_row(2000, 27, a), 26);
        // A brighter light-alpha is capped at the last row.
        assert_eq!(brightness_row(2000, 53, 255), 52);
    }

    /// spec: S-0104 claim 15 (the light-alpha after the covers picks the row; whatever a, b ≤ 0
    /// gives row 0); S-0110 F1 (shroud value 0 gives row 0)
    #[test]
    fn rows_follow_the_light_alpha() {
        assert_eq!(brightness_row(1000, 53, 0), 0);
        assert_eq!(brightness_row(1000, 53, 64), 13);
        assert_eq!(brightness_row(1000, 53, 255), 52);
        assert_eq!(brightness_row(0, 53, 255), 0);
        assert_eq!(brightness_row(-10, 53, 200), 0);
        assert_eq!(alpha_row(127, brightness_rho(1000), 53), 26);
    }

    /// spec: S-0104 claim 13, Formulas "Object brightness"
    #[test]
    fn object_brightness_table() {
        let cell = CellLevels {
            unscaled_ground: 1000,
            scaled_ground: 799,
            scaled_raised: 1200,
        };
        let extra = ExtraLight {
            unit: 7,
            infantry: 11,
            aircraft: 13,
        };
        let b = |o| object_brightness(o, cell, 100, 8, extra, &Q);
        let vehicle = |on_bridge_deck, shadowed_cell, height| LitObject::Vehicle {
            on_bridge_deck,
            shadowed_cell,
            height,
        };
        assert_eq!(b(vehicle(true, true, 0)), 1000 + 400 + 7);
        assert_eq!(b(vehicle(false, true, 207)), 1000 - 500 + 7);
        assert_eq!(b(vehicle(false, true, 208)), 1000 + 7);
        assert_eq!(b(vehicle(false, false, 900)), 1000 + 7);
        let inf = |on_bridge_deck, shadowed_cell, height| LitObject::Infantry {
            on_bridge_deck,
            shadowed_cell,
            height,
        };
        assert_eq!(b(inf(true, false, 0)), 1000 + 11);
        assert_eq!(b(inf(true, false, 420)), 1000 + 200 + 11);
        assert_eq!(b(inf(false, true, 209)), 1000 + 100 + 11);
        assert_eq!(b(inf(false, true, 208)), 1000 - 500 + 11);
        assert_eq!(b(inf(false, false, 900)), 1000 + 11);
        assert_eq!(b(LitObject::Aircraft { height: 630 }), 1000 + 3 * 8 + 13);
        let all_lambda = LightQuirks {
            aircraft_height_light_ion_only: false,
            ..Q
        };
        assert_eq!(
            object_brightness(
                LitObject::Aircraft { height: 630 },
                cell,
                100,
                8,
                extra,
                &all_lambda
            ),
            1000 + 300 + 13
        );
        for o in [
            LitObject::OrdinaryBuilding,
            LitObject::TerrainPaletteBuildingVoxel,
            LitObject::OreSpawningTerrainObject,
            LitObject::WallOverlay,
            LitObject::AnimPaletteAnimation,
            LitObject::BuildingOwnAnimation,
            LitObject::Parachute,
        ] {
            assert_eq!(b(o), 1000, "{o:?}");
        }
        for o in [
            LitObject::TerrainPaletteBuildingBody,
            LitObject::PlainTerrainObject,
            LitObject::OtherOverlay,
            LitObject::TerrainAnimation,
        ] {
            assert_eq!(b(o), 799, "{o:?}");
        }
        assert_eq!(b(LitObject::DeckOverlay), 1200);
        let dark = CellLevels::default();
        for o in [
            LitObject::OreOrGemOverlay,
            LitObject::DeployAnimation,
            LitObject::NormalLightAnimation,
        ] {
            assert_eq!(object_brightness(o, dark, 0, 0, extra, &Q), 1000, "{o:?}");
        }
        let lit_deploy = LightQuirks {
            deploy_animation_unlit: false,
            ..Q
        };
        assert_eq!(
            object_brightness(LitObject::DeployAnimation, dark, 0, 0, extra, &lit_deploy),
            0
        );
        // Not clamped here.
        assert_eq!(
            object_brightness(
                vehicle(false, true, 0),
                dark,
                0,
                0,
                ExtraLight::default(),
                &Q
            ),
            -500
        );
    }

    /// spec: S-0104 claim 13 (a building's top-left cell; the centre cell takes the larger coordinate)
    #[test]
    fn building_cells() {
        assert_eq!(building_centre_cell((10, 20), (3, 3)), (11, 21));
        assert_eq!(building_centre_cell((10, 20), (2, 4)), (11, 22));
        assert_eq!(building_light_cell((10, 20), (2, 4), &Q), (10, 20));
        let off = LightQuirks {
            building_light_from_top_left_cell: false,
            ..Q
        };
        assert_eq!(building_light_cell((10, 20), (2, 4), &off), (11, 22));
        // A building's light sits at its centre: a cell centre only for odd sides.
        let l = LightSource::building((10, 20), (3, 2), 5000, 100, [1000; 3]);
        assert_eq!((l.x, l.y), (11 * 256 + 128, 21 * 256));
        assert_eq!(l.required_detail, DetailLevel::High);
    }

    /// spec: S-0104 claim 18, Scenario L25 (quirks.infantry_height_light_limited off: the height
    /// term always, h above the terrain, plus −500 in a shadowed cell off a deck at h ≤ 208);
    /// claim 13 (on)
    #[test]
    fn infantry_light_switch_off() {
        let off = LightQuirks {
            infantry_height_light_limited: false,
            ..LightQuirks::default()
        };
        let cell = CellLevels {
            unscaled_ground: 1000,
            scaled_ground: 1000,
            scaled_raised: 1000,
        };
        let b = |shadowed_cell, on_bridge_deck, height, q: &LightQuirks| {
            object_brightness(
                LitObject::Infantry {
                    on_bridge_deck,
                    shadowed_cell,
                    height,
                },
                cell,
                100,
                100,
                ExtraLight::default(),
                q,
            )
        };
        // Part A: shadowed ordinary ground.
        let a: Vec<i32> = [0, 104, 207, 208, 209, 416]
            .iter()
            .map(|&h| b(true, false, h, &off))
            .collect();
        assert_eq!(a, [500, 500, 500, 600, 1100, 1200]);
        // Part B: unshadowed ordinary ground.
        let bb: Vec<i32> = [0, 104, 207, 208, 209, 416]
            .iter()
            .map(|&h| b(false, false, h, &off))
            .collect();
        assert_eq!(bb, [1000, 1000, 1000, 1100, 1100, 1200]);
        // Part C: standing on a deck (height 0 above the deck, 416 above the terrain), shadowed
        // or not: 2λ and no penalty.
        assert_eq!(b(true, true, 0, &off), 1200);
        assert_eq!(b(false, true, 0, &off), 1200);
        // The two states differ in a shadowed cell only at h = 208.
        let on = LightQuirks::default();
        assert_eq!(b(true, false, 208, &on), 500);
        assert_eq!(b(true, false, 207, &on), b(true, false, 207, &off));
        assert_eq!(b(true, false, 209, &on), b(true, false, 209, &off));
        // On a deck the on state measures h from the deck: nothing extra.
        assert_eq!(b(false, true, 0, &on), 1000);
    }
}
