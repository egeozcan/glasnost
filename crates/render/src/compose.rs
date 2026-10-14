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
/// gives (S-0104 claim 15).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shade {
    /// The table's first row.
    pub base: u32,
    /// The table's number of rows *N*.
    pub rows: u16,
    /// ρ of the drawing's brightness.
    pub rho: u16,
}

impl Shade {
    /// The table of `rows` rows from `base`, at brightness `b` ‰.
    pub fn new(base: usize, rows: usize, b: i32) -> Self {
        Self {
            base: base as u32,
            rows: rows as u16,
            rho: crate::light::brightness_rho(b) as u16,
        }
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
    /// Each non-zero pixel darkens the pixel beneath: every hi-colour
    /// channel halved, rounded down (S-0102 claim 26, F7).
    Shadow,
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

    /// Draws `d` with the palette rows `rows`.
    pub fn draw(&mut self, d: &Drawing<'_>, rows: &[Row565], q: &ComposeQuirks) {
        let (w, h) = (d.image.width() as i32, d.image.height() as i32);
        let (fw, fh) = (self.width as i32, self.height as i32);
        let x0 = d.x.max(0);
        let x1 = (d.x + w).min(fw);
        let y0 = d.y.max(0);
        let y1 = (d.y + h).min(fh);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        // spec: S-0102 F2 (r₀ and r₁ are the drawing rectangle before clipping; clipping removes pixels only)
        let (r0, r1) = (d.y, d.y + h - 1);
        let first_row = match d.paint {
            Paint::Row(i) | Paint::Diamond(i) => rows.get(i),
            Paint::Lit(s) | Paint::LitDiamond(s) => rows.get(s.base as usize),
            Paint::Shadow => None,
        };
        if !matches!(d.paint, Paint::Shadow) && first_row.is_none() {
            return;
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
                            continue;
                        }
                    },
                    DepthSource::Profile { companion, .. } => {
                        // spec: S-0102 F2 (pixel depth = row depth − companion value)
                        let a = companion.map_or(0, |(img, cx, cy)| {
                            let (u, v) = (x - cx, y - cy);
                            if u < 0 || v < 0 || u >= img.width as i32 || v >= img.height as i32 {
                                0
                            } else {
                                i32::from(img.values[(v as u32 * img.width + u as u32) as usize])
                            }
                        });
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
            }
        }
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
            Paint::Lit(s) | Paint::LitDiamond(s) => rows.get(s.row(self.alpha[i])),
            Paint::Shadow => {
                self.color[i] = darken565(self.color[i]);
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

/// Halves each channel of a 5-6-5 pixel (S-0102 claim 26, F7). Halving the
/// 8-bit channel before packing gives the same top bits.
pub fn darken565(c: u16) -> u16 {
    // spec: S-0102 F7 (each channel halved separately, rounded down)

    let r = (c >> 11) >> 1;
    let g = ((c >> 5) & 0x3F) >> 1;
    let b = (c & 0x1F) >> 1;
    (r << 11) | (g << 5) | b
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
}
