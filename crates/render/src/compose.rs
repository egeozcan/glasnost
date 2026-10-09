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

/// Depth of the view's top row in the first drawn frame after a reset of
/// *C* (S-0102 F1).
pub const DEPTH_REFERENCE_START: i32 = 32768; // spec: S-0102 Constants (depth reference start)

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

/// What a drawing's visible pixels do to the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paint {
    /// Each pixel takes `rows[row][index]`.
    Row(usize),
    /// Each non-zero pixel darkens the pixel beneath (provisional, Q-0015:
    /// every channel halved).
    Shadow,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComposeQuirks {
    /// `quirks.upright_depth_anchor_after_clip`: the upright profile is
    /// anchored at the last visible row (F2 note).
    pub upright_depth_anchor_after_clip: bool,
}

impl Default for ComposeQuirks {
    fn default() -> Self {
        // spec: S-0102 Edge cases and quirks (default: on)
        Self {
            upright_depth_anchor_after_clip: true,
        }
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
        }
    }

    /// *C* after a reset with world row `top_row` at the view's top (F1:
    /// C = 32768 + w_top).
    pub fn reset_reference(top_row: i32) -> i32 {
        // spec: S-0102 F1 (at a reset, C = 32768 + w_top)
        DEPTH_REFERENCE_START + top_row
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
        // The drawing rectangle after clipping (F2: r₀ and r₁).
        let (r0, r1_own) = (y0, y1 - 1);
        // spec: S-0102 F2 note (quirks.upright_depth_anchor_after_clip: anchor at the last visible row)
        let r1 = if q.upright_depth_anchor_after_clip {
            r1_own
        } else {
            d.y + h - 1
        };
        let row_table = match d.paint {
            Paint::Row(i) => rows.get(i),
            Paint::Shadow => None,
        };
        if matches!(d.paint, Paint::Row(_)) && row_table.is_none() {
            return;
        }
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
                // spec: S-0104 claim 6 (entry 0 is transparent in every row)
                if index == 0 {
                    continue;
                }
                let depth = match &d.depth {
                    DepthSource::Tile { tile_depth, .. } => match tile_depth {
                        Some(td) => base + i32::from(td.get(ix, iy).unwrap_or(0)),
                        // spec: S-0102 claim 9 (tiles without depth data do not touch the buffer)
                        None => {
                            self.paint(x, y, index, d.paint, row_table);
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
                };
                if !pass {
                    continue;
                }
                if d.test != DepthTest::Reader {
                    // spec: S-0102 F2 (a writer stores the pixel depth, reduced modulo 65536)
                    self.depth[i] = depth.rem_euclid(DEPTH_MOD) as u16;
                }
                self.paint(x, y, index, d.paint, row_table);
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

    fn paint(&mut self, x: i32, y: i32, index: u8, paint: Paint, row: Option<&Row565>) {
        let i = (y as u32 * self.width + x as u32) as usize;
        match (paint, row) {
            (Paint::Row(_), Some(row)) => self.color[i] = row[usize::from(index)],
            (Paint::Shadow, _) => self.color[i] = darken565(self.color[i]),
            _ => {}
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

/// Halves each channel of a 5-6-5 pixel (provisional shadow treatment,
/// Q-0015). Halving the 8-bit channel before packing gives the same top
/// bits.
pub fn darken565(c: u16) -> u16 {
    // provisional (Q-0015): each channel halved, rounding down
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

    /// spec: S-0102 F1 (depth decreases by 1 per row down; C = 32768 + w_top)
    #[test]
    fn row_depth() {
        let c = Frame::reset_reference(100);
        let f = Frame::new(4, 4, 100, c);
        assert_eq!(f.row_depth(0), 32768);
        assert_eq!(f.row_depth(3), 32765);
    }

    /// spec: S-0102 claims 9, 10 (tiles ≤, writers and readers <; readers store nothing)
    #[test]
    fn depth_tests() {
        let q = ComposeQuirks::default();
        let mut f = Frame::new(2, 1, 0, Frame::reset_reference(0));
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
        let mut f = Frame::new(1, 7, 0, Frame::reset_reference(0));
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
        let mut f = Frame::new(1, 4, 0, Frame::reset_reference(0));
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

    /// spec: S-0102 F2 note (quirks.upright_depth_anchor_after_clip)
    #[test]
    fn upright_anchor_after_clip() {
        let a = img(1, 6, 1);
        let d = Drawing {
            image: &a,
            x: 0,
            y: 0,
            test: DepthTest::Writer,
            depth: profile(Profile::Upright, 0),
            paint: Paint::Row(0),
        };
        let mut on = Frame::new(1, 3, 0, Frame::reset_reference(0));
        on.draw(&d, &rows(), &ComposeQuirks::default());
        // Anchored at view row 2: Q(32766) = 32766.
        assert_eq!(on.depth[2], 32766);
        let mut off = Frame::new(1, 3, 0, Frame::reset_reference(0));
        off.draw(
            &d,
            &rows(),
            &ComposeQuirks {
                upright_depth_anchor_after_clip: false,
            },
        );
        // Anchored at the image's own row 5: Q(32763) − ⌊3 / 3⌋ at row 2.
        assert_eq!(off.depth[2], 32762);
    }

    /// spec: S-0102 F2 (with a companion image every row uses the anchor value, minus the image)
    #[test]
    fn companion_depth() {
        let q = ComposeQuirks::default();
        let mut f = Frame::new(1, 3, 0, Frame::reset_reference(0));
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

    /// Q-0015 provisional shadow halving.
    #[test]
    fn shadow_halves() {
        assert_eq!(darken565(0xFFFF), 0x7BEF);
        assert_eq!(expand565(0xFFFF), (255, 255, 255));
    }
}
