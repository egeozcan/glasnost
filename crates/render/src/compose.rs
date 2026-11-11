//! The battlefield compositor on the CPU: the reference implementation of
//! S-0102's depth compositing, which the GPU path must match pixel for
//! pixel.
//!
//! A frame is a list of [`Drawing`]s in draw order: the cell pass first
//! (tiles, then the other writers in claim 16's ranks), then the object pass
//! (the five layers in order, claim 1). Each drawing is a palette-indexed
//! image placed at a view pixel, with a palette row (a shade-table row of
//! 256 hi-colour pixels, S-0103 claim 9 and S-0104 claim 6) and a depth
//! rule. Index 0 is transparent in every drawing (S-0104 claim 6).
//!
//! Depth (S-0102 F1 to F3): every view row *r* has the screen-row depth
//! d(r) = (C − w) mod 65536 with *w* = r + O_y the world pixel row and *C*
//! the depth reference. Smaller is nearer.

use glasnost_fmt::image::{IndexedImage, Rgb8};

/// Depth clear value (S-0102 Constants).
pub const DEPTH_CLEAR: u16 = 65535; // spec: S-0102 Constants (depth clear value)

/// The depth reference *C*: one value for the whole game, whatever the
/// view, scrolls, jumps and redraws (S-0102 F1, a departure).
pub const DEPTH_REFERENCE: i32 = 32768; // spec: S-0102 F1, Constants (fixed depth reference)

/// Tile depth row shift (S-0102 F3).
pub const TILE_DEPTH_SHIFT: i32 = 30; // spec: S-0102 Constants (tile depth row shift)

/// Rounding quantum of the slope and upright profiles (S-0102 F2).
pub const QUANTUM: i32 = 3; // spec: S-0102 Constants (slope / upright quantum)

/// Depth range modulus (S-0102 Constants: values wrap modulo 65536).
const DEPTH_MOD: i32 = 65536; // spec: S-0102 Constants (depth value range)

/// One row of 256 hi-colour pixels (5-6-5, S-0103 claim 9).
pub type Row565 = [u16; 256];

/// Packs an 8-bit colour into 5-6-5 (S-0103 claim 9, Formulas "16-bit
/// step").
pub fn pack565(c: Rgb8) -> u16 {
    crate::palette::hicolor_565(c)
}

/// A shade-table row as hi-colour pixels.
pub fn row565(row: &[Rgb8; 256]) -> Row565 {
    std::array::from_fn(|i| pack565(row[i]))
}

/// The depth profile of a drawing (S-0102 Terms, F2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    /// Lying on the ground: depth follows the ground row by row.
    Flat,
    /// 45°: loses 1 on the second and third of every three rows.
    Slope,
    /// Standing: anchored at the last row, nearer by 1 every 3 rows up.
    Upright,
}

/// How a drawing takes part in the depth test (S-0102 claims 9, 10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthTest {
    /// A terrain tile: drawn and stored where depth ≤ stored (claim 9).
    Tile,
    /// Another cell-pass writer: drawn and stored where depth < stored.
    Writer,
    /// A reader: drawn where depth < stored, never stores (claim 10).
    Reader,
    /// Neither reads nor writes depth: every pixel is painted (a smudge,
    /// S-0102 claim 25).
    Ignore,
}

/// A signed per-pixel depth image (S-0102 Terms: companion depth image,
/// −128 … 127 per pixel).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedImage {
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
    /// Values, row by row.
    pub values: Vec<i8>,
}

impl SignedImage {
    /// The value at column `u`, row `v`, or `None` outside the image.
    pub fn get(&self, u: i32, v: i32) -> Option<i8> {
        let (u, v) = (u32::try_from(u).ok()?, u32::try_from(v).ok()?);
        if u >= self.width || v >= self.height {
            return None;
        }
        self.values.get((v * self.width + u) as usize).copied()
    }

    /// A building companion depth image from the bytes of a shape frame's
    /// stored rectangle, `width` × `height`, row by row (S-0102 F5).
    pub fn from_companion_bytes(width: u32, height: u32, bytes: &[u8]) -> Self {
        Self {
            width,
            height,
            values: bytes.iter().map(|&b| companion_adjustment(b)).collect(),
        }
    }
}

/// The signed depth adjustment *a* of a companion depth image byte *b*
/// (S-0102 F5): 0 for *b* = 0, otherwise ((*b* − 65 + 128) mod 256) − 128.
pub fn companion_adjustment(b: u8) -> i8 {
    if b == 0 {
        return 0;
    }
    // spec: S-0102 F5 (a = ((b − 65 + 128) mod 256) − 128, the remainder non-negative)
    ((i32::from(b) - 65 + 128).rem_euclid(256) - 128) as i8
}

/// Which building body a companion depth image is placed for (S-0102 F5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanionBody {
    /// A body drawn from the building itself.
    Live,
    /// A body drawn from the remembered image under fog (S-0102 claim 16).
    Remembered,
}

impl CompanionBody {
    /// The sampling constants (*Kₓ*, *Kᵧ*), pixels from the body's draw point.
    pub fn constants(self) -> (i32, i32) {
        match self {
            // spec: S-0102 F5 (K = (198, 446) for live bodies)
            Self::Live => (198, 446),
            // spec: S-0102 F5 (K = (144, 172) for remembered bodies)
            Self::Remembered => (144, 172),
        }
    }
}

/// Foundations this many cells wide or wider use no companion depth image
/// (S-0102 F5).
pub const COMPANION_WIDTH_LIMIT: i32 = 8; // spec: S-0102 F5

/// The screen pixel where a building companion depth image's stored
/// rectangle begins (S-0102 F5): (*pₓ* − *Dₓ* − *c_z*, *pᵧ* − *Dᵧ* − *r_z*),
/// for a body with draw point `p` on a foundation of `foundation` cells,
/// art displacement `m` ((0, 0) when omitted) and the companion's stored
/// rectangle at canvas column and row `stored`. `None` when the foundation
/// is too wide for a companion.
pub fn companion_origin(
    p: (i32, i32),
    foundation: (i32, i32),
    m: (i32, i32),
    body: CompanionBody,
    stored: (i32, i32),
) -> Option<(i32, i32)> {
    let (w, h) = foundation;
    // spec: S-0102 F5 (a foundation less than 8 cells wide uses the companion)
    if w >= COMPANION_WIDTH_LIMIT {
        return None;
    }
    let (kx, ky) = body.constants();
    // spec: S-0102 F5 (Dₓ = Kₓ + mₓ − 30(w − h), Dᵧ = Kᵧ + mᵧ − 15(w + h − 2))
    let dx = kx + m.0 - 30 * (w - h);
    let dy = ky + m.1 - 15 * (w + h - 2);
    // spec: S-0102 F5 (the stored rectangle begins at (pₓ − Dₓ − c_z, pᵧ − Dᵧ − r_z))
    Some((p.0 - dx - stored.0, p.1 - dy - stored.1))
}

/// Where a drawing's per-pixel depth comes from.
#[derive(Clone, Debug)]
pub enum DepthSource<'a> {
    /// Terrain tile (F3): `tile_depth` holds *e* per pixel of the image (same
    /// size); `reference_row` is the view row *t* of the tile's ground
    /// reference top row (its top row as drawn plus 15 × its height level).
    Tile {
        /// Tile depth data, same size as the image; `None` when the tile has
        /// none (drawn without touching depth, claim 9).
        tile_depth: Option<&'a IndexedImage>,
        /// View row *t*.
        reference_row: i32,
    },
    /// A profile drawing (F2) with bias β over the drawing rectangle, which
    /// is the image's own rectangle.
    Profile {
        /// Profile.
        profile: Profile,
        /// Depth bias β.
        bias: i32,
        /// Optional companion depth image and its top-left view pixel.
        companion: Option<(&'a SignedImage, i32, i32)>,
    },
}

/// A shade table and a brightness: the rows `base … base + rows − 1` of
/// the scene, of which each pixel takes the row its light-alpha value
/// gives (S-0104 claim 15), and the protection colour mask an opaque pixel
/// takes after that (S-0102 claim 34, F9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shade {
    /// The table's first row.
    pub base: u32,
    /// The table's number of rows *N*.
    pub rows: u16,
    /// ρ of the drawing's brightness.
    pub rho: u16,
    /// The protection colour mask *K* (5-6-5), 0 for none.
    pub mask: u16,
}

impl Shade {
    /// The table of `rows` rows from `base`, at brightness `b` ‰, with no
    /// protection colour mask.
    pub fn new(base: usize, rows: usize, b: i32) -> Self {
        Self {
            base: base as u32,
            rows: rows as u16,
            rho: crate::light::brightness_rho(b) as u16,
            // spec: S-0102 F9 (with no selected adjustment, K = 0)
            mask: 0,
        }
    }

    /// The same shading with protection colour mask `k` (S-0102 claim 34).
    pub fn with_mask(self, k: u16) -> Self {
        Self { mask: k, ..self }
    }

    /// The row a pixel of light-alpha `alpha` takes.
    pub fn row(&self, alpha: u8) -> usize {
        self.base as usize
            + crate::light::alpha_row(
                i32::from(alpha),
                i32::from(self.rho),
                usize::from(self.rows),
            )
    }
}

/// What a drawing's visible pixels do to the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paint {
    /// Each pixel takes `rows[row][index]`: drawing that does not read the
    /// light-alpha values (S-0110 claim 5).
    Row(usize),
    /// A tile diamond (S-0010 claim 22): as `Row`, but every pixel of the
    /// diamond shape is painted, palette index 0 included.
    Diamond(usize),
    /// Each pixel takes the row of its light-alpha value in a shade table
    /// (S-0104 claim 15, S-0110 claim 5).
    Lit(Shade),
    /// A tile diamond shaded as `Lit`.
    LitDiamond(Shade),
    /// As `Lit`, but each pixel blends with the pixel already painted by a
    /// translucency code of 1 to 3 (S-0102 claim 32, F8). Code 0 is `Lit`.
    Translucent(Shade, Translucency),
    /// Each non-zero pixel darkens the pixel beneath: every hi-colour
    /// channel halved, rounded down (S-0102 claim 26, F7).
    Shadow,
    /// Each non-zero pixel takes this 8-bit colour with no palette, scaled
    /// by its light-alpha value *a* when *a* < 127 and not drawn when
    /// *a* = 0, then packed to 5-6-5; always opaque (S-0616 claim 28, the
    /// pixel particles).
    Pixel(Rgb8),
}

/// A blending translucency code of S-0102 claim 32 and F8 (S-0109 claims
/// 20 and 21 choose it). Code 0 replaces the destination and is not one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Translucency {
    /// Code 1, 25 % translucent.
    Quarter,
    /// Code 2, 50 % translucent.
    Half,
    /// Code 3, 75 % translucent.
    ThreeQuarter,
}

impl Translucency {
    /// The blend of code `code`; `None` for code 0 (opaque, it replaces
    /// the destination) and for any other value.
    pub fn from_code(code: i32) -> Option<Self> {
        // spec: S-0102 claim 32, F8 (code 0 replaces the destination; codes 1, 2 and 3 blend)
        match code {
            1 => Some(Self::Quarter),
            2 => Some(Self::Half),
            3 => Some(Self::ThreeQuarter),
            _ => None,
        }
    }

    /// The code, 1 to 3.
    pub fn code(self) -> u32 {
        match self {
            Self::Quarter => 1,
            Self::Half => 2,
            Self::ThreeQuarter => 3,
        }
    }
}

/// Width of a tile diamond in pixels (S-0010 claim 22).
pub const DIAMOND_WIDTH: i32 = 60; // spec: S-0010 Constants (diamond size 60 × 29)

/// Rows of a tile diamond (S-0010 claim 22).
pub const DIAMOND_ROWS: i32 = 29; // spec: S-0010 Constants (diamond size 60 × 29)

/// Whether pixel (c, r) of a 60 × 29 diamond box is one of the diamond's 900
/// pixels (S-0010 Formulas, *Diamond rows*).
pub fn in_diamond(c: i32, r: i32) -> bool {
    if !(0..DIAMOND_ROWS).contains(&r) {
        return false;
    }
    // spec: S-0010 Formulas (f(r) = 28 − 2r, n(r) = 4(r + 1) for r ≤ 14; f(r) = 2(r − 14), n(r) = 4(29 − r) for r ≥ 14)
    let (f, n) = if r <= 14 {
        (28 - 2 * r, 4 * (r + 1))
    } else {
        (2 * (r - 14), 4 * (29 - r))
    };
    c >= f && c < f + n
}

/// One image to draw.
#[derive(Clone, Debug)]
pub struct Drawing<'a> {
    /// Palette indices; 0 is transparent.
    pub image: &'a IndexedImage,
    /// View pixel of the image's top-left corner.
    pub x: i32,
    /// View pixel of the image's top-left corner.
    pub y: i32,
    /// Depth test.
    pub test: DepthTest,
    /// Depth source.
    pub depth: DepthSource<'a>,
    /// Paint.
    pub paint: Paint,
}

/// Compositing switches of S-0102 implemented here.
///
/// The fixed depth reference and the unclipped profile rectangle of F1
/// and F2 are departures without switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComposeQuirks {
    /// `quirks.cell_pass_shadows_write_depth` (claim 27): a shadow drawn as
    /// a writer (the cell pass's shadows) stores its depth where it darkens;
    /// off: every shadow darkens without storing depth.
    pub cell_pass_shadows_write_depth: bool,
}

impl Default for ComposeQuirks {
    fn default() -> Self {
        // spec: S-0102 Edge cases and quirks (default: on)
        Self {
            cell_pass_shadows_write_depth: true,
        }
    }
}

impl ComposeQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 1] {
        [(
            "cell_pass_shadows_write_depth",
            self.cell_pass_shadows_write_depth,
        )]
    }
}

/// The battlefield view's frame: hi-colour pixels and the depth buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// 5-6-5 pixels, row by row.
    pub color: Vec<u16>,
    /// Depth per pixel (S-0102 Terms: smaller is nearer).
    pub depth: Vec<u16>,
    /// World pixel row of view row 0 (O_y).
    pub origin_row: i32,
    /// Depth reference *C* (F1).
    pub reference: i32,
    /// Light-alpha value per pixel (S-0104 claim 15, S-0110 claim 5): 127
    /// unless covers or glows changed it ([`Frame::set_light_alpha`]).
    pub alpha: Vec<u8>,
}

/// Q(v) = trunc₀(v / 3) × 3 (S-0102 F2).
pub fn quantise_depth(v: i32) -> i32 {
    // spec: S-0102 F2 (Q(v) = trunc₀(v / 3) × 3)
    (v / QUANTUM) * QUANTUM
}

impl Frame {
    /// A cleared frame (claim 8: depth 65535) of `width × height` whose row 0
    /// shows world row `origin_row`, with depth reference `reference`.
    pub fn new(width: u32, height: u32, origin_row: i32, reference: i32) -> Self {
        let n = width as usize * height as usize;
        Self {
            width,
            height,
            color: vec![0; n],
            depth: vec![DEPTH_CLEAR; n],
            origin_row,
            reference,
            // spec: S-0110 claim 5 (127 before any cover or glow acts in a drawn frame)
            alpha: vec![crate::light::DEFAULT_LIGHT_ALPHA as u8; n],
        }
    }

    /// Sets the light-alpha values the covers and glows of this drawn frame
    /// produced (S-0110 claims 5 and 6), one per pixel, row by row. A
    /// buffer of another size is ignored.
    pub fn set_light_alpha(&mut self, alpha: Vec<u8>) {
        if alpha.len() == self.alpha.len() {
            self.alpha = alpha;
        }
    }

    /// Screen-row depth d(r) of view row `r` (F1), in 0 … 65535.
    pub fn row_depth(&self, r: i32) -> i32 {
        // spec: S-0102 F1 (d = (C − w) mod 65536)
        (self.reference - (r + self.origin_row)).rem_euclid(DEPTH_MOD)
    }

    /// Draws `d` with the palette rows `rows`, and returns how many of its
    /// pixels were painted (passed every test of the drawing), which the
    /// pointer pick reads ([`crate::scene::Scene::object_at`]).
    pub fn draw(&mut self, d: &Drawing<'_>, rows: &[Row565], q: &ComposeQuirks) -> usize {
        let (w, h) = (d.image.width() as i32, d.image.height() as i32);
        let (fw, fh) = (self.width as i32, self.height as i32);
        let x0 = d.x.max(0);
        let x1 = (d.x + w).min(fw);
        let y0 = d.y.max(0);
        let y1 = (d.y + h).min(fh);
        if x0 >= x1 || y0 >= y1 {
            return 0;
        }
        let mut painted = 0;
        // spec: S-0102 F2 (r₀ and r₁ are the drawing rectangle before clipping; clipping removes pixels only)
        let (r0, r1) = (d.y, d.y + h - 1);
        let first_row = match d.paint {
            Paint::Row(i) | Paint::Diamond(i) => rows.get(i),
            Paint::Lit(s) | Paint::LitDiamond(s) | Paint::Translucent(s, _) => {
                rows.get(s.base as usize)
            }
            Paint::Shadow | Paint::Pixel(_) => None,
        };
        if !matches!(d.paint, Paint::Shadow | Paint::Pixel(_)) && first_row.is_none() {
            return 0;
        }
        let diamond = matches!(d.paint, Paint::Diamond(_) | Paint::LitDiamond(_));
        for y in y0..y1 {
            let base = match &d.depth {
                DepthSource::Tile { reference_row, .. } => {
                    // spec: S-0102 F3 (depth = e + d(t + 30))
                    self.row_depth(reference_row + TILE_DEPTH_SHIFT)
                }
                DepthSource::Profile {
                    profile,
                    bias,
                    companion,
                } => self.profile_row_depth(*profile, *bias, companion.is_some(), r0, r1, y),
            };
            for x in x0..x1 {
                let (ix, iy) = ((x - d.x) as u32, (y - d.y) as u32);
                let Some(index) = d.image.get(ix, iy) else {
                    continue;
                };
                // spec: S-0104 claim 6 (entry 0 is transparent in every row);
                // S-0010 claim 22 (a diamond has no transparent index)
                if index == 0 && !(diamond && in_diamond(ix as i32, iy as i32)) {
                    continue;
                }
                let depth = match &d.depth {
                    DepthSource::Tile { tile_depth, .. } => match tile_depth {
                        Some(td) => base + i32::from(td.get(ix, iy).unwrap_or(0)),
                        // spec: S-0102 claim 9 (tiles without depth data do not touch the buffer)
                        None => {
                            self.paint(x, y, index, d.paint, rows);
                            painted += 1;
                            continue;
                        }
                    },
                    DepthSource::Profile { companion, .. } => {
                        // spec: S-0102 F2 (pixel depth = row depth − companion value)
                        let a = match companion {
                            None => 0,
                            Some((img, cx, cy)) => match img.get(x - cx, y - cy) {
                                Some(a) => i32::from(a),
                                // spec: S-0102 claim 31, F5 (a body pixel the companion's stored
                                // rectangle does not cover is not drawn and writes no depth)
                                None => continue,
                            },
                        };
                        base - a
                    }
                };
                let i = (y as u32 * self.width + x as u32) as usize;
                let stored = i32::from(self.depth[i]);
                let pass = match d.test {
                    // spec: S-0102 claim 9 (a tile pixel is drawn where depth ≤ stored)
                    DepthTest::Tile => depth <= stored,
                    // spec: S-0102 claim 10, F2 (drawn iff pixel depth < stored)
                    DepthTest::Writer | DepthTest::Reader => depth < stored,
                    // spec: S-0102 claim 25 (a smudge neither reads nor writes depth)
                    DepthTest::Ignore => {
                        self.paint(x, y, index, d.paint, rows);
                        painted += 1;
                        continue;
                    }
                };
                if !pass {
                    continue;
                }
                // spec: S-0102 claim 27 (a cell-pass shadow stores its depth, quirks.cell_pass_shadows_write_depth)
                let stores = d.test != DepthTest::Reader
                    && (q.cell_pass_shadows_write_depth || !matches!(d.paint, Paint::Shadow));
                if stores {
                    // spec: S-0102 F2 (a writer stores the pixel depth, reduced modulo 65536)
                    self.depth[i] = depth.rem_euclid(DEPTH_MOD) as u16;
                }
                self.paint(x, y, index, d.paint, rows);
                painted += 1;
            }
        }
        painted
    }

    fn profile_row_depth(
        &self,
        p: Profile,
        bias: i32,
        companion: bool,
        r0: i32,
        r1: i32,
        r: i32,
    ) -> i32 {
        match (p, companion) {
            // spec: S-0102 F2 (flat: d(r) + β; with a companion image, d(r₀) + β on every row)
            (Profile::Flat, false) => self.row_depth(r) + bias,
            (Profile::Flat, true) => self.row_depth(r0) + bias,
            // spec: S-0102 F2 (slope: Q(d(r₀) + β) − ⌊2j / 3⌋; with a companion image, Q(d(r₀) + β))
            (Profile::Slope, false) => {
                quantise_depth(self.row_depth(r0) + bias) - (2 * (r - r0)) / QUANTUM
            }
            (Profile::Slope, true) => quantise_depth(self.row_depth(r0) + bias),
            // spec: S-0102 F2 (upright: Q(d(r₁) + β) − ⌊(r₁ − r) / 3⌋; with a companion image, d(r₁) + β)
            (Profile::Upright, false) => {
                quantise_depth(self.row_depth(r1) + bias) - (r1 - r) / QUANTUM
            }
            (Profile::Upright, true) => self.row_depth(r1) + bias,
        }
    }

    fn paint(&mut self, x: i32, y: i32, index: u8, paint: Paint, rows: &[Row565]) {
        let i = (y as u32 * self.width + x as u32) as usize;
        let row = match paint {
            Paint::Row(r) | Paint::Diamond(r) => rows.get(r),
            // spec: S-0104 claim 15, S-0110 claim 5 (the pixel's light-alpha value picks the row)
            Paint::Lit(s) | Paint::LitDiamond(s) => {
                if let Some(row) = rows.get(s.row(self.alpha[i])) {
                    // spec: S-0102 claim 34, F9 (an opaque, depth-passing pixel becomes S ∨ K after
                    // palette conversion and shade selection)
                    self.color[i] = row[usize::from(index)] | s.mask;
                }
                return;
            }
            Paint::Translucent(s, t) => {
                // spec: S-0102 claim 32 (the selected, lit hi-colour source pixel blends with the
                // already-painted destination by F8)
                if let Some(row) = rows.get(s.row(self.alpha[i])) {
                    self.color[i] = blend(
                        row[usize::from(index)],
                        self.color[i],
                        t,
                        PixelLayout::Rgb565,
                    );
                }
                return;
            }
            Paint::Shadow => {
                self.color[i] = darken565(self.color[i]);
                return;
            }
            Paint::Pixel(c) => {
                if let Some(c) = pixel_colour(c, self.alpha[i]) {
                    self.color[i] = pack565(c);
                }
                return;
            }
        };
        if let Some(row) = row {
            self.color[i] = row[usize::from(index)];
        }
    }

    /// The frame as RGBA bytes: each 5-6-5 channel widened by repeating its
    /// top bits (a display-path choice of ours; S-0103 claim 9 fixes only
    /// the top bits).
    pub fn to_rgba(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.color.len() * 4);
        for &c in &self.color {
            let (r, g, b) = expand565(c);
            out.extend_from_slice(&[r, g, b, 255]);
        }
        out
    }
}

/// The colour a pixel particle of colour `c` takes where the light-alpha
/// value is `a` (S-0616 claim 28, F16): none for *a* = 0, each channel
/// times *a* / 128 truncated for *a* < 127, unchanged otherwise.
pub fn pixel_colour(c: Rgb8, a: u8) -> Option<Rgb8> {
    // spec: S-0616 claim 28 (not drawn when a = 0, fully shrouded)
    if a == 0 {
        return None;
    }
    // spec: S-0616 claim 28 and Constants (pixel full light-alpha 127; below it each channel × a / 128, truncated)
    if a >= 127 {
        return Some(c);
    }
    let s = |x: u8| ((u32::from(x) * u32::from(a)) / 128) as u8; // spec: S-0616 claim 28 (divided by 128)
    Some(Rgb8 {
        r: s(c.r),
        g: s(c.g),
        b: s(c.b),
    })
}

/// Halves each channel of a 5-6-5 pixel (S-0102 claim 26, F7). Halving the
/// 8-bit channel before packing gives the same top bits.
pub fn darken565(c: u16) -> u16 {
    // spec: S-0102 F7 (each channel halved separately, rounded down)

    let r = (c >> 11) >> 1;
    let g = ((c >> 5) & 0x3F) >> 1;
    let b = (c & 0x1F) >> 1;
    (r << 11) | (g << 5) | b
}

/// The hi-colour pixel layouts of S-0103 claim 9.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelLayout {
    /// 5 bits red, 6 green, 5 blue (the layout our frames use).
    Rgb565,
    /// 5 bits for each channel.
    Rgb555,
}

/// The protection colour mask *K* of an imported colour-adjustment triple
/// (S-0102 F9), packed once in `layout`: the triple's bytes are already
/// mask contributions, with no 8-bit-to-hi-colour conversion, saturation
/// or scaling, and only the low 16 bits are kept.
pub fn protection_mask(rgb: [u8; 3], layout: PixelLayout) -> u16 {
    let [r, g, b] = rgb.map(u32::from);
    let k = match layout {
        // spec: S-0102 F9 (5-6-5: K = (2048·R ∨ 32·G ∨ B) modulo 65536)
        PixelLayout::Rgb565 => (2048 * r) | (32 * g) | b,
        // spec: S-0102 F9 (5-5-5: K = (1024·R ∨ 32·⌊G/2⌋ ∨ B) modulo 65536)
        PixelLayout::Rgb555 => (1024 * r) | (32 * (g / 2)) | b,
    };
    // spec: S-0102 F9 (only the low 16 bits are retained)
    (k % 65536) as u16
}

/// One channel of F8 (S-0102): `s` the source channel after palette
/// selection and lighting, `d` the destination channel, both packed.
pub fn blend_channel(s: u16, d: u16, t: Translucency) -> u16 {
    // spec: S-0102 F8 (each division rounds down separately before the multiplication or addition)
    match t {
        // spec: S-0102 F8 (code 1, 25 %: 3·⌊s/4⌋ + ⌊d/4⌋)
        Translucency::Quarter => 3 * (s / 4) + d / 4,
        // spec: S-0102 F8 (code 2, 50 %: ⌊s/2⌋ + ⌊d/2⌋)
        Translucency::Half => s / 2 + d / 2,
        // spec: S-0102 F8 (code 3, 75 %: ⌊s/4⌋ + 3·⌊d/4⌋)
        Translucency::ThreeQuarter => s / 4 + 3 * (d / 4),
    }
}

/// Blends a packed source pixel over a packed destination pixel of the
/// same layout (S-0102 claim 32, F8): every channel by itself, with no
/// carry between channels, then repacked in that layout.
pub fn blend(s: u16, d: u16, t: Translucency, layout: PixelLayout) -> u16 {
    // spec: S-0102 F8 (5-6-5: red/blue 0…31, green 0…63; 5-5-5: every channel 0…31)
    let (green_bits, red_shift) = match layout {
        PixelLayout::Rgb565 => (6, 11),
        PixelLayout::Rgb555 => (5, 10),
    };
    let field = |c: u16, shift: u16, bits: u16| (c >> shift) & ((1 << bits) - 1);
    let mix =
        |shift: u16, bits: u16| blend_channel(field(s, shift, bits), field(d, shift, bits), t);
    let r = mix(red_shift, 5);
    let g = mix(5, green_bits);
    let b = mix(0, 5);
    // spec: S-0102 F8 (repack the three results in the same layout)
    (r << red_shift) | (g << 5) | b
}

/// A 5-6-5 pixel as 8-bit channels, top bits repeated into the low bits.
pub fn expand565(c: u16) -> (u8, u8, u8) {
    let r = ((c >> 11) & 0x1F) as u8;
    let g = ((c >> 5) & 0x3F) as u8;
    let b = (c & 0x1F) as u8;
    (
        (r << 3) | (r >> 2),
        (g << 2) | (g >> 4),
        (b << 3) | (b >> 2),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(w: u32, h: u32, v: u8) -> IndexedImage {
        IndexedImage::new(w, h, vec![v; (w * h) as usize]).unwrap()
    }

    fn rows() -> Vec<Row565> {
        vec![[0xF800; 256], [0x07E0; 256], [0x001F; 256]]
    }

    fn profile(p: Profile, bias: i32) -> DepthSource<'static> {
        DepthSource::Profile {
            profile: p,
            bias,
            companion: None,
        }
    }

    /// spec: S-0102 F1 (d = (C − w) mod 65536: depth decreases by 1 per row down)
    #[test]
    fn row_depth() {
        let f = Frame::new(4, 4, 100, DEPTH_REFERENCE);
        assert_eq!(f.row_depth(0), 32668);
        assert_eq!(f.row_depth(3), 32665);
    }

    /// spec: S-0102 F1 (C = 32768 throughout the game, independent of the view; a departure)
    #[test]
    fn depth_reference_is_fixed() {
        assert_eq!(DEPTH_REFERENCE, 32768);
        // World row 500 seen from views whose top rows differ, as after a
        // scroll or a jump: its depth depends on the world row only.
        let depth_of_row_500 =
            |top: i32| Frame::new(1, 1, top, DEPTH_REFERENCE).row_depth(500 - top);
        assert_eq!(depth_of_row_500(0), 32268);
        assert_eq!(depth_of_row_500(100), 32268);
        assert_eq!(depth_of_row_500(300), 32268);
    }

    /// spec: S-0102 claims 9, 10 (tiles ≤, writers and readers <; readers store nothing)
    #[test]
    fn depth_tests() {
        let q = ComposeQuirks::default();
        let mut f = Frame::new(2, 1, 0, DEPTH_REFERENCE);
        let a = img(1, 1, 1);
        let flat = |x| Drawing {
            image: &a,
            x,
            y: 0,
            test: DepthTest::Writer,
            depth: profile(Profile::Flat, 0),
            paint: Paint::Row(0),
        };
        f.draw(&flat(0), &rows(), &q);
        assert_eq!((f.color[0], f.depth[0]), (0xF800, 32768));
        // An equal-depth writer does not replace (strictly less).
        let mut d = flat(0);
        d.paint = Paint::Row(1);
        f.draw(&d, &rows(), &q);
        assert_eq!(f.color[0], 0xF800);
        // An equal-depth tile does (≤): tile depth e = 0 at reference row −30 gives d(0).
        let td = img(1, 1, 0);
        let tile = Drawing {
            image: &a,
            x: 0,
            y: 0,
            test: DepthTest::Tile,
            depth: DepthSource::Tile {
                tile_depth: Some(&td),
                reference_row: -30,
            },
            paint: Paint::Row(2),
        };
        f.draw(&tile, &rows(), &q);
        assert_eq!((f.color[0], f.depth[0]), (0x001F, 32768));
        // A nearer reader paints but stores nothing.
        let reader = Drawing {
            test: DepthTest::Reader,
            depth: profile(Profile::Flat, -5),
            paint: Paint::Row(1),
            ..flat(0)
        };
        f.draw(&reader, &rows(), &q);
        assert_eq!((f.color[0], f.depth[0]), (0x07E0, 32768));
    }

    /// spec: S-0102 F2 (upright: bottom row Q(d(r₁) + β), nearer by 1 every 3 rows up)
    #[test]
    fn upright_profile() {
        let q = ComposeQuirks::default();
        let mut f = Frame::new(1, 7, 0, DEPTH_REFERENCE);
        let a = img(1, 7, 1);
        let d = Drawing {
            image: &a,
            x: 0,
            y: 0,
            test: DepthTest::Writer,
            depth: profile(Profile::Upright, -2),
            paint: Paint::Row(0),
        };
        f.draw(&d, &rows(), &q);
        // d(6) = 32762; + β = 32760; Q = 32760.
        let got: Vec<u16> = f.depth.clone();
        assert_eq!(got, vec![32758, 32759, 32759, 32759, 32760, 32760, 32760]);
    }

    /// spec: S-0102 F2 (slope: Q(d(r₀) + β) − ⌊2j / 3⌋)
    #[test]
    fn slope_profile() {
        let q = ComposeQuirks::default();
        let mut f = Frame::new(1, 4, 0, DEPTH_REFERENCE);
        let a = img(1, 4, 1);
        let d = Drawing {
            image: &a,
            x: 0,
            y: 0,
            test: DepthTest::Writer,
            depth: profile(Profile::Slope, 0),
            paint: Paint::Row(0),
        };
        f.draw(&d, &rows(), &q);
        // Q(32768) = 32766; j = 0..3 → −0, −0, −1, −2.
        assert_eq!(f.depth, vec![32766, 32766, 32765, 32764]);
    }

    /// spec: S-0102 F2 (r₀ and r₁ are the rectangle before clipping; clipping removes pixels only)
    #[test]
    fn clipping_keeps_profile_depth() {
        let q = ComposeQuirks::default();
        let a = img(1, 6, 1);
        let at = |y: i32, profile_kind: Profile| Drawing {
            image: &a,
            x: 0,
            y,
            test: DepthTest::Writer,
            depth: profile(profile_kind, 0),
            paint: Paint::Row(0),
        };
        for kind in [Profile::Upright, Profile::Slope, Profile::Flat] {
            // The whole image inside a tall view, and the same image at the
            // same world rows seen through views that clip its bottom and its
            // top: every surviving pixel keeps its depth.
            let mut full = Frame::new(1, 10, 0, DEPTH_REFERENCE);
            full.draw(&at(2, kind), &rows(), &q);
            let mut bottom = Frame::new(1, 5, 0, DEPTH_REFERENCE);
            bottom.draw(&at(2, kind), &rows(), &q);
            assert_eq!(bottom.depth[2..5], full.depth[2..5], "{kind:?} bottom clip");
            // The view starts at world row 4: the image's world rows 2 and 3
            // are cut off at the top.
            let mut top = Frame::new(1, 6, 4, DEPTH_REFERENCE);
            top.draw(&at(-2, kind), &rows(), &q);
            assert_eq!(top.depth[0..4], full.depth[4..8], "{kind:?} top clip");
        }
        // Upright, anchored at the image's own last row 7 (not the last
        // visible row 4): Q(32761) − ⌊(7 − 4) / 3⌋.
        let mut bottom = Frame::new(1, 5, 0, DEPTH_REFERENCE);
        bottom.draw(&at(2, Profile::Upright), &rows(), &q);
        assert_eq!(bottom.depth[4], 32760 - 1);
    }

    /// spec: S-0102 F2 (with a companion image every row uses the anchor value, minus the image)
    #[test]
    fn companion_depth() {
        let q = ComposeQuirks::default();
        let mut f = Frame::new(1, 3, 0, DEPTH_REFERENCE);
        let a = img(1, 3, 1);
        let comp = SignedImage {
            width: 1,
            height: 3,
            values: vec![10, -5, 0],
        };
        let d = Drawing {
            image: &a,
            x: 0,
            y: 0,
            test: DepthTest::Writer,
            depth: DepthSource::Profile {
                profile: Profile::Upright,
                bias: -2,
                companion: Some((&comp, 0, 0)),
            },
            paint: Paint::Row(0),
        };
        f.draw(&d, &rows(), &q);
        // d(2) + β = 32764, no Q.
        assert_eq!(f.depth, vec![32754, 32769, 32764]);
    }

    /// spec: S-0102 F5 (a = 0 for b = 0, otherwise ((b − 65 + 128) mod 256) − 128: bytes 0 and 65
    /// give 0, 64 gives −1, 66 gives +1)
    #[test]
    fn f5_companion_bytes() {
        for (b, a) in [
            (0u8, 0i8),
            (64, -1),
            (65, 0),
            (66, 1),
            (1, -64),
            (192, 127),
            (193, -128),
            (255, -66),
        ] {
            assert_eq!(companion_adjustment(b), a, "byte {b}");
        }
        let img = SignedImage::from_companion_bytes(2, 2, &[0, 64, 65, 66]);
        assert_eq!(img.values, vec![0, -1, 0, 1]);
        assert_eq!(
            (img.get(1, 1), img.get(2, 0), img.get(0, -1)),
            (Some(1), None, None)
        );
    }

    /// spec: S-0102 F5 (Dₓ = Kₓ + mₓ − 30(w − h), Dᵧ = Kᵧ + mᵧ − 15(w + h − 2); K = (198, 446) live,
    /// (144, 172) remembered; the stored rectangle begins at (pₓ − Dₓ − c_z, pᵧ − Dᵧ − r_z); widths
    /// of 8 cells or more use none)
    #[test]
    fn f5_companion_placement() {
        let p = (1000, 1000);
        // Live, 3 × 2: D = (198 − 30, 446 − 45) = (168, 401).
        assert_eq!(
            companion_origin(p, (3, 2), (0, 0), CompanionBody::Live, (0, 0)),
            Some((832, 599))
        );
        // Remembered, 3 × 2: D = (144 − 30, 172 − 45) = (114, 127).
        assert_eq!(
            companion_origin(p, (3, 2), (0, 0), CompanionBody::Remembered, (0, 0)),
            Some((886, 873))
        );
        // A positive displacement (17, 23) moves coverage left and up; a stored
        // rectangle origin (5, 7) moves it further.
        assert_eq!(
            companion_origin(p, (3, 2), (17, 23), CompanionBody::Live, (5, 7)),
            Some((1000 - 185 - 5, 1000 - 424 - 7))
        );
        assert_eq!(
            companion_origin(p, (3, 2), (17, 23), CompanionBody::Remembered, (5, 7)),
            Some((1000 - 131 - 5, 1000 - 150 - 7))
        );
        // Width 7 uses the companion, width 8 does not, whatever the height.
        assert!(companion_origin(p, (7, 1), (0, 0), CompanionBody::Live, (0, 0)).is_some());
        assert_eq!(
            companion_origin(p, (8, 1), (0, 0), CompanionBody::Live, (0, 0)),
            None
        );
        assert_eq!(
            companion_origin(p, (9, 9), (0, 0), CompanionBody::Remembered, (0, 0)),
            None
        );
    }

    /// spec: S-0102 claim 31, F5 (a body pixel outside the companion's stored rectangle is not drawn
    /// and writes no depth, with no upright fallback; covered pixels take the unrounded upright anchor
    /// minus a; a zero byte adjusts nothing)
    #[test]
    fn companion_clips_uncovered_pixels() {
        let q = ComposeQuirks::default();
        let mut f = Frame::new(2, 3, 0, DEPTH_REFERENCE);
        let before = (f.color.clone(), f.depth.clone());
        let a = img(2, 3, 1);
        // Covers column 1, rows 1 and 2 only (the companion's rectangle at view (1, 1)).
        let comp = SignedImage::from_companion_bytes(1, 2, &[0, 66]);
        let d = Drawing {
            image: &a,
            x: 0,
            y: 0,
            test: DepthTest::Writer,
            depth: DepthSource::Profile {
                profile: Profile::Upright,
                bias: -2,
                companion: Some((&comp, 1, 1)),
            },
            paint: Paint::Row(0),
        };
        f.draw(&d, &rows(), &q);
        let anchor = f.row_depth(2) - 2;
        for (i, (c, dep)) in f.color.iter().zip(&f.depth).enumerate() {
            let (x, y) = (i % 2, i / 2);
            match (x, y) {
                (1, 1) => assert_eq!(i32::from(*dep), anchor),
                (1, 2) => assert_eq!(i32::from(*dep), anchor - 1),
                _ => assert_eq!((*c, *dep), (before.0[i], before.1[i]), "pixel {x}, {y}"),
            }
        }
        assert_ne!(f.color[3], before.0[3], "a covered pixel is drawn");
    }

    /// spec: S-0102 claim 33 (a valid empty companion rectangle covers no body pixel: the body
    /// draws nothing and writes no depth)
    #[test]
    fn empty_companion_covers_nothing() {
        let q = ComposeQuirks::default();
        for (w, h) in [(0, 2), (1, 0), (0, 0)] {
            let mut f = Frame::new(2, 3, 0, DEPTH_REFERENCE);
            let before = (f.color.clone(), f.depth.clone());
            let a = img(2, 3, 1);
            let comp = SignedImage::from_companion_bytes(w, h, &[]);
            let d = Drawing {
                image: &a,
                x: 0,
                y: 0,
                test: DepthTest::Writer,
                depth: DepthSource::Profile {
                    profile: Profile::Upright,
                    bias: -2,
                    companion: Some((&comp, 0, 0)),
                },
                paint: Paint::Row(0),
            };
            f.draw(&d, &rows(), &q);
            assert_eq!((f.color, f.depth), before, "{w} × {h}");
        }
    }

    /// spec: S-0102 claim 26, F7 (each hi-colour channel halved, rounded down; twice gives ⌊v/4⌋)
    #[test]
    fn shadow_halves() {
        assert_eq!(darken565(0xFFFF), 0x7BEF);
        assert_eq!(darken565(darken565(0xFFFF)), (7 << 11) | (15 << 5) | 7);
        assert_eq!(expand565(0xFFFF), (255, 255, 255));
        // Halving the 8-bit channel before packing gives the same packed pixel.
        let c = Rgb8 {
            r: 203,
            g: 77,
            b: 18,
        };
        let half = Rgb8 {
            r: c.r / 2,
            g: c.g / 2,
            b: c.b / 2,
        };
        assert_eq!(darken565(pack565(c)), pack565(half));
    }

    /// spec: S-0102 claim 27 (cell-pass shadows write depth: a later shadow darkens again only where nearer)
    #[test]
    fn overlapping_shadows() {
        let q = ComposeQuirks::default();
        let img = IndexedImage::new(1, 1, vec![5]).unwrap();
        let shadow = |bias, test| Drawing {
            image: &img,
            x: 0,
            y: 0,
            test,
            depth: DepthSource::Profile {
                profile: Profile::Flat,
                bias,
                companion: None,
            },
            paint: Paint::Shadow,
        };
        let mut f = Frame::new(1, 1, 0, DEPTH_REFERENCE);
        f.color[0] = 0xFFFF;
        // A terrain-object shadow (−3), then a building shadow (−4): darkened twice.
        f.draw(&shadow(-3, DepthTest::Writer), &rows(), &q);
        f.draw(&shadow(-4, DepthTest::Writer), &rows(), &q);
        assert_eq!(f.color[0], darken565(darken565(0xFFFF)));
        // Another building shadow (−4) at the same depth: not darkened again.
        f.draw(&shadow(-4, DepthTest::Writer), &rows(), &q);
        assert_eq!(f.color[0], darken565(darken565(0xFFFF)));
        // quirks.cell_pass_shadows_write_depth off: overlapping shadows always darken again.
        let off = ComposeQuirks {
            cell_pass_shadows_write_depth: false,
        };
        let mut g = Frame::new(1, 1, 0, DEPTH_REFERENCE);
        g.color[0] = 0xFFFF;
        for _ in 0..3 {
            g.draw(&shadow(-4, DepthTest::Writer), &rows(), &off);
        }
        assert_eq!(g.color[0], darken565(darken565(darken565(0xFFFF))));
    }

    /// spec: S-0102 claim 27 (the overlay-shadow rows), claim 13 (an overlay shadow: −2 − 15 × ℓ_s,
    /// flat, writes), claim 16 (rank 3: overlays before overlay shadows)
    /// spec: S-0616 claim 28 (one pixel of the particle's colour: not drawn
    /// at a = 0, each channel × a / 128 truncated below 127, unchanged at
    /// 127 and above; drawn only where d − L(z) − 50 is strictly less than
    /// the stored depth, never writing it; opaque)
    #[test]
    fn pixel_particles() {
        let q = ComposeQuirks::default();
        let img = IndexedImage::new(1, 1, vec![1]).unwrap();
        let c = Rgb8 {
            r: 200,
            g: 101,
            b: 255,
        };
        let draw = |bias| Drawing {
            image: &img,
            x: 0,
            y: 0,
            test: DepthTest::Reader,
            depth: DepthSource::Profile {
                profile: Profile::Flat,
                bias,
                companion: None,
            },
            paint: Paint::Pixel(c),
        };
        assert_eq!(pixel_colour(c, 0), None);
        assert_eq!(
            pixel_colour(c, 64),
            Some(Rgb8 {
                r: 100,
                g: 50,
                b: 127
            })
        );
        assert_eq!(
            pixel_colour(c, 126).map(|x| x.g),
            Some(99),
            "101 × 126 / 128"
        );
        assert_eq!(pixel_colour(c, 127), Some(c));
        assert_eq!(pixel_colour(c, 255), Some(c));
        let fresh = |alpha: u8, stored: u16| {
            let mut f = Frame::new(1, 1, 0, DEPTH_REFERENCE);
            f.color[0] = 0x1234;
            f.depth[0] = stored;
            f.set_light_alpha(vec![alpha]);
            f
        };
        let d = Frame::new(1, 1, 0, DEPTH_REFERENCE).row_depth(0);
        // Drawn opaque over what was there; the depth stays.
        let mut f = fresh(127, 60000);
        f.draw(&draw(-50), &rows(), &q);
        assert_eq!(f.color[0], pack565(c));
        assert_eq!(f.depth[0], 60000);
        // Scaled below 127; not drawn at 0.
        let mut f = fresh(64, 60000);
        f.draw(&draw(-50), &rows(), &q);
        assert_eq!(f.color[0], pack565(pixel_colour(c, 64).unwrap()));
        let mut f = fresh(0, 60000);
        f.draw(&draw(-50), &rows(), &q);
        assert_eq!(f.color[0], 0x1234);
        // Strictly less: equal depth is not drawn, one less is.
        let at = (d - 50) as u16;
        let mut f = fresh(127, at);
        f.draw(&draw(-50), &rows(), &q);
        assert_eq!(f.color[0], 0x1234);
        let mut f = fresh(127, at + 1);
        f.draw(&draw(-50), &rows(), &q);
        assert_eq!(f.color[0], pack565(c));
        assert_eq!(f.depth[0], at + 1);
    }

    #[test]
    fn overlay_shadow_rows() {
        let q = ComposeQuirks::default();
        let img = IndexedImage::new(1, 1, vec![5]).unwrap();
        let draw = |bias, paint| Drawing {
            image: &img,
            x: 0,
            y: 0,
            test: DepthTest::Writer,
            depth: DepthSource::Profile {
                profile: Profile::Flat,
                bias,
                companion: None,
            },
            paint,
        };
        let fresh = || {
            let mut f = Frame::new(1, 1, 0, DEPTH_REFERENCE);
            f.color[0] = 0xFFFF;
            f
        };
        let once = darken565(0xFFFF);
        // On a bare tile pixel (stored 65535): darkened once.
        let mut f = fresh();
        f.draw(&draw(-2, Paint::Shadow), &rows(), &q);
        assert_eq!(f.color[0], once);
        // A second overlay shadow at the same level: equal depths, once.
        f.draw(&draw(-2, Paint::Shadow), &rows(), &q);
        assert_eq!(f.color[0], once);
        // A tree's shadow (−3) and then a building's (−4) over it: darkened again each time.
        f.draw(&draw(-3, Paint::Shadow), &rows(), &q);
        assert_eq!(f.color[0], darken565(once));
        // An animation shadow (−2, a writer here as the worst case) over it: not again.
        f.draw(&draw(-2, Paint::Shadow), &rows(), &q);
        assert_eq!(f.color[0], darken565(once));
        // A flat ore body at the same level, drawn earlier (rank 3): the shadow ties and the ore
        // shows undarkened.
        let mut g = fresh();
        g.draw(&draw(-2, Paint::Row(0)), &rows(), &q);
        let ore = g.color[0];
        g.draw(&draw(-2, Paint::Shadow), &rows(), &q);
        assert_eq!(g.color[0], ore);
        // A deck cell at level 1: its body −2 − 15 × 5, its shadow −2 − 15 × 1, so the shadow
        // never darkens the deck's own pixels.
        let mut h = fresh();
        h.draw(&draw(-77, Paint::Row(0)), &rows(), &q);
        let deck = h.color[0];
        h.draw(&draw(-17, Paint::Shadow), &rows(), &q);
        assert_eq!(h.color[0], deck);
    }

    /// spec: S-0010 claim 22 (a diamond paints all 900 pixels, index 0 included; row 29 never)
    #[test]
    fn diamond_paints_index_zero() {
        let n: usize = (0..30)
            .map(|r| (0..60).filter(|&c| in_diamond(c, r)).count())
            .sum();
        assert_eq!(n, 900);
        assert!(in_diamond(28, 0) && in_diamond(31, 0) && !in_diamond(27, 0));
        assert!(in_diamond(0, 14) && in_diamond(59, 14));
        assert!(!(0..60).any(|c| in_diamond(c, 29)));
        let q = ComposeQuirks::default();
        let img = IndexedImage::new(60, 30, vec![0; 1800]).unwrap();
        let mut f = Frame::new(60, 30, 0, DEPTH_REFERENCE);
        let d = Drawing {
            image: &img,
            x: 0,
            y: 0,
            test: DepthTest::Tile,
            depth: DepthSource::Tile {
                tile_depth: None,
                reference_row: 0,
            },
            paint: Paint::Diamond(1),
        };
        f.draw(&d, &rows(), &q);
        let painted = f.color.iter().filter(|&&c| c == rows()[1][0]).count();
        assert_eq!(painted, 900);
    }

    /// spec: S-0102 claim 25 (a smudge neither reads nor writes depth)
    #[test]
    fn ignore_depth() {
        let q = ComposeQuirks::default();
        let img = IndexedImage::new(1, 1, vec![3]).unwrap();
        let mut f = Frame::new(1, 1, 0, DEPTH_REFERENCE);
        f.depth[0] = 0;
        let d = Drawing {
            image: &img,
            x: 0,
            y: 0,
            test: DepthTest::Ignore,
            depth: DepthSource::Profile {
                profile: Profile::Flat,
                bias: 0,
                companion: None,
            },
            paint: Paint::Row(1),
        };
        f.draw(&d, &rows(), &q);
        assert_eq!(f.color[0], rows()[1][3]);
        assert_eq!(f.depth[0], 0);
    }

    fn pack(r: u16, g: u16, b: u16, layout: PixelLayout) -> u16 {
        match layout {
            PixelLayout::Rgb565 => (r << 11) | (g << 5) | b,
            PixelLayout::Rgb555 => (r << 10) | (g << 5) | b,
        }
    }

    /// spec: S-0102 F8 and D41 (per-channel truncation before the weighted sum: s = 31, d = 15
    /// gives 24, 22 and 16 for codes 1 to 3, not 27, 23 and 19; green s = 63, d = 31 gives 52,
    /// 46 and 36; code 0 replaces)
    #[test]
    fn f8_channel_blend() {
        let codes = [
            Translucency::Quarter,
            Translucency::Half,
            Translucency::ThreeQuarter,
        ];
        let out: Vec<u16> = codes.iter().map(|&t| blend_channel(31, 15, t)).collect();
        assert_eq!(out, [24, 22, 16]);
        let out: Vec<u16> = codes.iter().map(|&t| blend_channel(63, 31, t)).collect();
        assert_eq!(out, [52, 46, 36]);
        assert_eq!(Translucency::from_code(0), None);
        assert_eq!(
            codes.map(|t| Translucency::from_code(t.code() as i32)),
            codes.map(Some)
        );
        // Whole pixels in both layouts: red and blue s = 31, d = 15, green s = 63, d = 31 in
        // 5-6-5 and s = 31, d = 15 in 5-5-5; no carry between channels.
        let s = pack(31, 63, 31, PixelLayout::Rgb565);
        let d = pack(15, 31, 15, PixelLayout::Rgb565);
        assert_eq!(
            codes.map(|t| blend(s, d, t, PixelLayout::Rgb565)),
            [
                pack(24, 52, 24, PixelLayout::Rgb565),
                pack(22, 46, 22, PixelLayout::Rgb565),
                pack(16, 36, 16, PixelLayout::Rgb565),
            ]
        );
        let s = pack(31, 31, 31, PixelLayout::Rgb555);
        let d = pack(15, 15, 15, PixelLayout::Rgb555);
        assert_eq!(
            codes.map(|t| blend(s, d, t, PixelLayout::Rgb555)),
            [
                pack(24, 24, 24, PixelLayout::Rgb555),
                pack(22, 22, 22, PixelLayout::Rgb555),
                pack(16, 16, 16, PixelLayout::Rgb555),
            ]
        );
        // Every channel value of both layouts stays in range.
        for t in codes {
            for s in 0..64 {
                for d in 0..64 {
                    assert!(blend_channel(s, d, t) <= s.max(d));
                }
            }
        }
    }

    /// spec: S-0102 claim 32 and D41 (a translucent reader blends with the colour already
    /// present, also over another translucent image; index 0 leaves colour and depth alone; a
    /// rejected pixel does not blend; translucency does not make the image a writer)
    #[test]
    fn translucent_readers_blend() {
        let q = ComposeQuirks::default();
        let background = pack(15, 31, 15, PixelLayout::Rgb565);
        let source = pack(31, 63, 31, PixelLayout::Rgb565);
        let rows: Vec<Row565> = vec![[background; 256], [source; 256]];
        let shade = Shade {
            base: 1,
            rows: 1,
            rho: 0,
            mask: 0,
        };
        let mut f = Frame::new(3, 1, 0, DEPTH_REFERENCE);
        let ground = img(3, 1, 1);
        f.draw(
            &Drawing {
                image: &ground,
                x: 0,
                y: 0,
                test: DepthTest::Writer,
                depth: profile(Profile::Flat, 0),
                paint: Paint::Row(0),
            },
            &rows,
            &q,
        );
        let depth_before = f.depth.clone();
        // Pixel 0 opaque index, pixel 1 index 0, pixel 2 behind the ground (rejected).
        let marker = IndexedImage::new(2, 1, vec![5, 0]).unwrap();
        let reader = |x, bias, t| Drawing {
            image: &marker,
            x,
            y: 0,
            test: DepthTest::Reader,
            depth: profile(Profile::Flat, bias),
            paint: Paint::Translucent(shade, t),
        };
        f.draw(&reader(0, -1, Translucency::Half), &rows, &q);
        assert_eq!(f.color[0], pack(22, 46, 22, PixelLayout::Rgb565));
        assert_eq!(f.color[1], background, "index 0 is transparent");
        assert_eq!(
            f.depth, depth_before,
            "a translucent reader stores no depth"
        );
        // A second translucent image blends with the colour now present.
        f.draw(&reader(0, -1, Translucency::Quarter), &rows, &q);
        assert_eq!(
            f.color[0],
            pack(
                3 * (31 / 4) + 22 / 4,
                3 * (63 / 4) + 46 / 4,
                3 * (31 / 4) + 22 / 4,
                PixelLayout::Rgb565
            )
        );
        // A rejected pixel does not blend.
        f.draw(&reader(2, 1, Translucency::ThreeQuarter), &rows, &q);
        assert_eq!(f.color[2], background);
        // The shadow keeps plain halving (claim 26), whatever the image's code.
        let mut g = Frame::new(1, 1, 0, DEPTH_REFERENCE);
        g.color[0] = background;
        let dot = img(1, 1, 1);
        g.draw(
            &Drawing {
                image: &dot,
                x: 0,
                y: 0,
                test: DepthTest::Reader,
                depth: profile(Profile::Flat, 0),
                paint: Paint::Shadow,
            },
            &rows,
            &q,
        );
        assert_eq!(g.color[0], darken565(background));
    }

    /// spec: S-0102 F9 (K = (2048·R ∨ 32·G ∨ B) mod 65536 on 5-6-5 and (1024·R ∨ 32·⌊G/2⌋ ∨ B)
    /// mod 65536 on 5-5-5: the bytes are mask contributions, not 8-bit colours; no saturation,
    /// oversized contributions set bits of another channel, only the low 16 bits are kept)
    #[test]
    fn f9_protection_mask_both_layouts() {
        assert_eq!(protection_mask([0, 0, 0], PixelLayout::Rgb565), 0);
        assert_eq!(protection_mask([1, 0, 0], PixelLayout::Rgb565), 1 << 11);
        assert_eq!(protection_mask([0, 3, 0], PixelLayout::Rgb565), 3 << 5);
        assert_eq!(protection_mask([0, 0, 1], PixelLayout::Rgb565), 1);
        assert_eq!(protection_mask([0, 3, 0], PixelLayout::Rgb555), 1 << 5);
        assert_eq!(protection_mask([1, 0, 0], PixelLayout::Rgb555), 1 << 10);
        // (255, 255, 255): 2048·255 ∨ 32·255 ∨ 255, low 16 bits.
        assert_eq!(
            protection_mask([255, 255, 255], PixelLayout::Rgb565),
            (((2048 * 255) | (32 * 255) | 255) % 65536) as u16
        );
        assert_eq!(
            protection_mask([255, 255, 255], PixelLayout::Rgb555),
            (((1024 * 255) | (32 * 127) | 255) % 65536) as u16
        );
        // An oversized blue contribution reaches the green bits.
        assert_eq!(protection_mask([0, 0, 40], PixelLayout::Rgb565), 40);
    }

    /// spec: S-0102 claim 34 and F9 (an opaque, depth-passing pixel becomes S ∨ K after shade
    /// selection, even a black shade-row pixel; a transparent pixel and one rejected by depth
    /// stay untouched; the mask is a union, not a saturating addition; depth rules unchanged)
    #[test]
    fn claim34_mask_is_a_union_on_opaque_depth_passing_pixels() {
        let q = ComposeQuirks::default();
        let background = pack(15, 31, 15, PixelLayout::Rgb565);
        let black = 0u16;
        let lit = pack(5, 9, 31, PixelLayout::Rgb565);
        let rows: Vec<Row565> = vec![[background; 256], [black; 256], [lit; 256]];
        let k = protection_mask([7, 19, 3], PixelLayout::Rgb565);
        let mut f = Frame::new(4, 1, 0, DEPTH_REFERENCE);
        let ground = img(4, 1, 1);
        f.draw(
            &Drawing {
                image: &ground,
                x: 0,
                y: 0,
                test: DepthTest::Writer,
                depth: profile(Profile::Flat, 0),
                paint: Paint::Row(0),
            },
            &rows,
            &q,
        );
        let depth_before = f.depth.clone();
        // Pixel 0 opaque, pixel 1 index 0 (transparent).
        let body = IndexedImage::new(2, 1, vec![5, 0]).unwrap();
        let shade = |base: usize| Shade::new(base, 1, 1000).with_mask(k);
        let reader = |x, bias, base| Drawing {
            image: &body,
            x,
            y: 0,
            test: DepthTest::Reader,
            depth: profile(Profile::Flat, bias),
            paint: Paint::Lit(shade(base)),
        };
        // A black shade-row pixel takes the mask's colour.
        f.draw(&reader(0, -1, 1), &rows, &q);
        assert_eq!(f.color[0], k);
        assert_eq!(f.color[1], background, "index 0 is transparent");
        assert_eq!(f.depth, depth_before, "the mask changes no depth rule");
        // Union: bits already set stay, no carry into another channel.
        f.draw(&reader(0, -1, 2), &rows, &q);
        assert_eq!(f.color[0], lit | k);
        assert_ne!(f.color[0], lit.wrapping_add(k));
        // Rejected by depth: untouched.
        f.draw(&reader(2, 1, 1), &rows, &q);
        assert_eq!(f.color[2], background);
        // A writer stores its depth as before.
        let mut g = Frame::new(1, 1, 0, DEPTH_REFERENCE);
        let dot = img(1, 1, 1);
        let plain = Drawing {
            image: &dot,
            x: 0,
            y: 0,
            test: DepthTest::Writer,
            depth: profile(Profile::Flat, 0),
            paint: Paint::Lit(Shade::new(2, 1, 1000)),
        };
        let mut h = g.clone();
        g.draw(&plain, &rows, &q);
        h.draw(
            &Drawing {
                paint: Paint::Lit(Shade::new(2, 1, 1000).with_mask(k)),
                ..plain.clone()
            },
            &rows,
            &q,
        );
        assert_eq!(g.depth, h.depth);
        assert_eq!(h.color[0], g.color[0] | k);
    }
}
