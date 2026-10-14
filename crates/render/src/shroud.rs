//! Shroud and fog cover pieces (S-0110) and the light-alpha values they
//! set (S-0104 claim 15).
//!
//! A cover piece is not painted: its pixels change the light-alpha value of
//! the screen pixels under it, and every drawing shaded through S-0104
//! claim 15 reads the resulting value ([`crate::compose::Paint::Lit`]).
//! [`light_alpha`] builds a view's light-alpha values from the pieces of
//! the cells in view, in the observable order of S-0110 (*State and
//! ordering*): cells by increasing *x* + *y*, each cell's shroud piece then
//! its fog piece, then the light-glow images.
//!
//! The pieces follow the explored and seen states at the time of the drawn
//! frame (claim 9); the caller gives them per cell ([`Cover`]), read from
//! the simulation (S-0409 owns the states). Fog of war is always off in
//! skirmish (S-0409 claim 19, S-0412 claim 1), so the fog rules here are
//! the deferred ones of claims 3 and 4; no skirmish reaches them.

use glasnost_fmt::image::IndexedImage;
use glasnost_fmt::shp::Shp;

use crate::light::DEFAULT_LIGHT_ALPHA;
use crate::projection::{TERRAIN_LIFT, cell_top};

/// Number of cover pieces (image frames 0 … 46).
pub const PIECES: usize = 47; // spec: S-0110 Constants (cover pieces: 47)

/// The piece of a cell that is fully covered.
pub const FULL_COVER: u8 = 15; // spec: S-0110 Constants (full-cover piece 15)

/// The shroud index that leaves the light-alpha value unchanged.
pub const SHROUD_TRANSPARENT: u8 = 254; // spec: S-0110 Constants (shroud transparent index 254)

/// The fog index that leaves the light-alpha value unchanged, and the
/// first fog index that is ignored is one above it.
pub const FOG_NEUTRAL: i32 = 127; // spec: S-0110 Constants (fog neutral index 127; 128 … 255 ignored)

/// The glow value of a factor 1.0.
pub const GLOW_SCALE: i32 = 127; // spec: S-0110 Constants (glow scale 127)

/// Width of a cell's canvas in pixels.
pub const CANVAS_WIDTH: i32 = 60; // spec: S-0110 Constants (canvas 60 × 30)

/// Height of a cell's canvas in pixels.
pub const CANVAS_HEIGHT: i32 = 30; // spec: S-0110 Constants (canvas 60 × 30)

/// How far left of the cell's top corner the canvas starts.
pub const CANVAS_SHIFT: i32 = 30; // spec: S-0110 Constants (canvas shift 30 pixels left)

/// The switch of S-0110.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShroudQuirks {
    /// `quirks.shroud_on_flat_grid` (claim 7, Edge cases): the pieces lie
    /// on the height-0 grid. Off: each canvas is placed at its cell's top
    /// corner at the cell's own terrain height. The switch also governs
    /// claim 8's point test, which is the simulation's
    /// (`econ::shroud::point_shrouded`) and keeps the on state there, so
    /// the simulation's `quirks.shroud_negative_odd_pair` always applies.
    pub shroud_on_flat_grid: bool,
}

impl Default for ShroudQuirks {
    fn default() -> Self {
        // spec: S-0110 Edge cases (quirks.shroud_on_flat_grid default on)
        Self {
            shroud_on_flat_grid: true,
        }
    }
}

impl ShroudQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 1] {
        [("shroud_on_flat_grid", self.shroud_on_flat_grid)]
    }
}

/// Which map neighbours of a cell are covered (S-0110 Terms): for the
/// shroud, not explored; for the fog, not seen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Covered {
    /// (x, y − 1).
    pub n: bool,
    /// (x + 1, y).
    pub e: bool,
    /// (x, y + 1).
    pub s: bool,
    /// (x − 1, y).
    pub w: bool,
    /// (x − 1, y − 1).
    pub nw: bool,
    /// (x + 1, y − 1).
    pub ne: bool,
    /// (x + 1, y + 1).
    pub se: bool,
    /// (x − 1, y + 1).
    pub sw: bool,
}

impl Covered {
    /// The neighbours of cell (x, y) for which `covered(x, y)` holds.
    pub fn of(x: i32, y: i32, covered: impl Fn(i32, i32) -> bool) -> Self {
        // spec: S-0110 Terms (map neighbours: N (x, y − 1), E (x + 1, y), S (x, y + 1), W (x − 1, y),
        // NW (x − 1, y − 1), NE (x + 1, y − 1), SE (x + 1, y + 1), SW (x − 1, y + 1))
        Self {
            n: covered(x, y - 1),
            e: covered(x + 1, y),
            s: covered(x, y + 1),
            w: covered(x - 1, y),
            nw: covered(x - 1, y - 1),
            ne: covered(x + 1, y - 1),
            se: covered(x + 1, y + 1),
            sw: covered(x - 1, y + 1),
        }
    }
}

/// Bits of a reduced set, for table T1.
const N: u8 = 1;
const E: u8 = 2;
const S: u8 = 4;
const W: u8 = 8;
const NW: u8 = 16;
const NE: u8 = 32;
const SE: u8 = 64;
const SW: u8 = 128;

/// Table T1: the piece of each reduced set that holds a corner neighbour.
// spec: S-0110 T1 (pieces for reduced sets that contain a corner neighbour)
const T1: [(u8, u8); 31] = [
    (N | SE, 16),
    (E | SW, 17),
    (N | E | SW, 18),
    (S | NW, 19),
    (E | S | NW, 20),
    (W | NE, 21),
    (N | W | SE, 22),
    (S | W | NE, 23),
    (N | SW, 24),
    (E | NW, 25),
    (S | NE, 26),
    (W | SE, 27),
    (N | SE | SW, 28),
    (E | NW | SW, 29),
    (S | NW | NE, 30),
    (W | NE | SE, 31),
    (NW, 32),
    (NE, 33),
    (SE, 34),
    (SW, 35),
    (NW | NE, 36),
    (NE | SE, 37),
    (SE | SW, 38),
    (NW | SW, 39),
    (NW | NE | SE, 40),
    (NE | SE | SW, 41),
    (NW | SE | SW, 42),
    (NW | NE | SW, 43),
    (NW | SE, 44),
    (NE | SW, 45),
    (NW | NE | SE | SW, 46),
];

/// The piece for a set of covered neighbours (S-0110 claim 2).
pub fn piece(c: Covered) -> u8 {
    // spec: S-0110 claim 2 (the reduced set: the covered sides, and each covered corner neither of
    // whose two adjacent sides is covered)
    let corner = |on: bool, a: bool, b: bool| on && !a && !b;
    let mut set = 0u8;
    for (on, bit) in [
        (c.n, N),
        (c.e, E),
        (c.s, S),
        (c.w, W),
        (corner(c.nw, c.n, c.w), NW),
        (corner(c.ne, c.n, c.e), NE),
        (corner(c.se, c.s, c.e), SE),
        (corner(c.sw, c.s, c.w), SW),
    ] {
        if on {
            set |= bit;
        }
    }
    if set & 0xF0 == 0 {
        // spec: S-0110 claim 2 (sides only: 1·[N] + 2·[E] + 4·[S] + 8·[W])
        return set;
    }
    // spec: S-0110 claim 2 (otherwise the entry of table T1)
    T1.iter()
        .find(|(s, _)| *s == set)
        .map(|(_, p)| *p)
        .expect("every reduced set with a corner is in T1")
}

/// The piece of a cell (S-0110 claims 1 and 4): 15 when the cell itself is
/// covered (not explored, or not seen), else the piece of claim 2 for its
/// covered neighbours.
pub fn cell_piece(own_covered: bool, neighbours: Covered) -> u8 {
    if own_covered {
        // spec: S-0110 claim 1 (a cell that is not explored gets piece 15); claim 4 (fog: not seen)
        FULL_COVER
    } else {
        piece(neighbours)
    }
}

/// F1, shroud: the light-alpha value after a shroud pixel of index `v`.
pub fn shroud_pixel(a: u8, v: u8) -> u8 {
    // spec: S-0110 F1 (shroud: a′ = v if v ≠ 254, else a)
    if v == SHROUD_TRANSPARENT { a } else { v }
}

/// F1, fog: the light-alpha value after a fog pixel of index `v`.
pub fn fog_pixel(a: u8, v: u8) -> u8 {
    // spec: S-0110 F1 (fog: a′ = max(0, a + v − 127) if v < 128, else a)
    if i32::from(v) <= FOG_NEUTRAL {
        (i32::from(a) + i32::from(v) - FOG_NEUTRAL).max(0) as u8
    } else {
        a
    }
}

/// F1, glow (claim 6): the light-alpha value after a glow pixel `g`.
pub fn glow_pixel(a: u8, g: u8) -> u8 {
    // spec: S-0110 F1, claim 6 (glow: a′ = min(255, ⌊a·g / 127⌋))
    (i32::from(a) * i32::from(g) / GLOW_SCALE).min(255) as u8
}

/// One image frame of a cover sprite: the pixels of its own rectangle and
/// the rectangle's position on the sprite's canvas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverFrame {
    /// The frame's rectangle (palette indices, all of them meaningful:
    /// index 0 is a value, not transparency).
    pub image: IndexedImage,
    /// Left edge of the rectangle inside the cell's canvas.
    pub x: i32,
    /// Top edge of the rectangle inside the cell's canvas.
    pub y: i32,
}

/// The shroud sprite or the fog sprite (S-0110 claim 3).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoverSprite {
    /// Image frames in order; `None` for an empty or undecodable frame.
    pub frames: Vec<Option<CoverFrame>>,
}

impl CoverSprite {
    /// The frames of a sprite file.
    pub fn from_shp(shp: &Shp<'_>) -> Self {
        let frames = (0..shp.frames.len())
            .map(|i| {
                let f = shp.frames[i];
                let image = shp.decode_frame(i).ok()?;
                (image.width() > 0 && image.height() > 0).then(|| CoverFrame {
                    image,
                    x: i32::from(f.x),
                    y: i32::from(f.y),
                })
            })
            .collect();
        Self { frames }
    }

    /// Image frame `i`, if the sprite has it (Import mapping: an image frame
    /// index the sprite does not have draws nothing).
    pub fn frame(&self, i: u8) -> Option<&CoverFrame> {
        self.frames.get(usize::from(i))?.as_ref()
    }
}

/// The covers of one cell in a drawn frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cover {
    /// The cell.
    pub cell: (i32, i32),
    /// The cell's terrain height level (used only with
    /// `quirks.shroud_on_flat_grid` off).
    pub level: i32,
    /// The shroud piece (claim 1).
    pub shroud: u8,
    /// The fog piece (claim 4), when fog of war is on and the local
    /// player's house is not defeated.
    pub fog: Option<u8>,
}

/// The world pixel of the top-left corner of a cell's canvas (S-0110 F2,
/// claim 7; with `quirks.shroud_on_flat_grid` off, at the cell's own
/// terrain height).
pub fn canvas_origin(cell: (i32, i32), level: i32, q: &ShroudQuirks) -> (i32, i32) {
    let (px, py) = cell_top(cell.0, cell.1);
    // spec: S-0110 F2 (the canvas spans P_x − 30 … P_x + 29 and P_y … P_y + 29, P at height 0)
    let lift = if q.shroud_on_flat_grid {
        0
    } else {
        // spec: S-0110 Edge cases (quirks.shroud_on_flat_grid off: the top corner at the cell's own
        // terrain height, S-0101 claim 4)
        TERRAIN_LIFT * level
    };
    (px - CANVAS_SHIFT, py - lift)
}

/// Whether a cell's canvas meets a view at world pixel `origin` of `size`.
pub fn canvas_in_view(
    cell: (i32, i32),
    level: i32,
    origin: (i32, i32),
    size: (u32, u32),
    q: &ShroudQuirks,
) -> bool {
    let (l, t) = canvas_origin(cell, level, q);
    let (r, b) = (l + CANVAS_WIDTH, t + CANVAS_HEIGHT);
    r > origin.0 && b > origin.1 && l < origin.0 + size.0 as i32 && t < origin.1 + size.1 as i32
}

/// Which sprites the covers take their pieces from (S-0110 claim 3).
#[derive(Clone, Copy, Debug)]
pub struct Sprites<'a> {
    /// The shroud sprite, if the data has it.
    pub shroud: Option<&'a CoverSprite>,
    /// The fog sprite, if the data has it.
    pub fog: Option<&'a CoverSprite>,
    /// Whether fog of war is on (never in skirmish).
    pub fog_of_war: bool,
}

/// A light-glow image placed in world pixels (claim 6).
#[derive(Clone, Copy, Debug)]
pub struct Glow<'a> {
    /// Its image frame's rectangle; every pixel takes part.
    pub image: &'a IndexedImage,
    /// World pixel of the rectangle's top-left corner.
    pub x: i32,
    /// World pixel of the rectangle's top-left corner.
    pub y: i32,
}

/// The light-alpha values of a view at world pixel `origin` of `size`,
/// row by row, after the covers and glows of one drawn frame (S-0110
/// claims 5 to 7 and 10, *Observable order*). `covers` must be ordered by
/// increasing *x* + *y* of their cells (cells with equal sums have canvases
/// that do not overlap). Glows act after every cover, in the order given
/// (their order is an open question of S-0110).
pub fn light_alpha(
    origin: (i32, i32),
    size: (u32, u32),
    covers: &[Cover],
    sprites: Sprites<'_>,
    glows: &[Glow<'_>],
    q: &ShroudQuirks,
) -> Vec<u8> {
    let (w, h) = (size.0 as i32, size.1 as i32);
    // spec: S-0110 claim 5 (the value is 127 before any cover or glow acts in a drawn frame)
    let mut a = vec![DEFAULT_LIGHT_ALPHA as u8; (w.max(0) * h.max(0)) as usize];
    // spec: S-0110 claim 3 (fog off: shroud pieces from the shroud sprite; fog on: both covers from
    // the fog sprite)
    let shroud_sprite = if sprites.fog_of_war {
        sprites.fog
    } else {
        sprites.shroud
    };
    for c in covers {
        let (cl, ct) = canvas_origin(c.cell, c.level, q);
        let mut apply = |sprite: Option<&CoverSprite>, piece: u8, f: fn(u8, u8) -> u8| {
            let Some(frame) = sprite.and_then(|s| s.frame(piece)) else {
                return;
            };
            let (fw, fh) = (frame.image.width() as i32, frame.image.height() as i32);
            // spec: S-0110 claim 7 (pixels outside the 60 × 30 canvas are not applied); claim 5
            // (pixels outside the frame's own rectangle are not touched)
            let x0 = frame.x.max(0);
            let y0 = frame.y.max(0);
            let x1 = (frame.x + fw).min(CANVAS_WIDTH);
            let y1 = (frame.y + fh).min(CANVAS_HEIGHT);
            let px = frame.image.pixels();
            for cy in y0..y1 {
                let vy = ct + cy - origin.1;
                if vy < 0 || vy >= h {
                    continue;
                }
                for cx in x0..x1 {
                    let vx = cl + cx - origin.0;
                    if vx < 0 || vx >= w {
                        continue;
                    }
                    let v = px[((cy - frame.y) * fw + (cx - frame.x)) as usize];
                    let i = (vy * w + vx) as usize;
                    a[i] = f(a[i], v);
                }
            }
        };
        // spec: S-0110 State and ordering (S_i then F_i per cell, cells by increasing x + y)
        apply(shroud_sprite, c.shroud, shroud_pixel);
        if let Some(fog) = c.fog {
            apply(sprites.fog, fog, fog_pixel);
        }
    }
    for g in glows {
        let (gw, gh) = (g.image.width() as i32, g.image.height() as i32);
        let px = g.image.pixels();
        for gy in 0..gh {
            let vy = g.y + gy - origin.1;
            if vy < 0 || vy >= h {
                continue;
            }
            for gx in 0..gw {
                let vx = g.x + gx - origin.0;
                if vx < 0 || vx >= w {
                    continue;
                }
                // spec: S-0110 claim 6 (every pixel of the glow's rectangle in the view takes part;
                // no index is skipped)
                let i = (vy * w + vx) as usize;
                a[i] = glow_pixel(a[i], px[(gy * gw + gx) as usize]);
            }
        }
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sides(n: bool, e: bool, s: bool, w: bool) -> Covered {
        Covered {
            n,
            e,
            s,
            w,
            ..Covered::default()
        }
    }

    /// spec: S-0110 claim 2 (sides only: 1·[N] + 2·[E] + 4·[S] + 8·[W]; empty 0, all four 15)
    #[test]
    fn side_pieces_are_the_sum() {
        assert_eq!(piece(Covered::default()), 0);
        assert_eq!(piece(sides(true, false, false, false)), 1);
        assert_eq!(piece(sides(false, true, false, false)), 2);
        assert_eq!(piece(sides(false, false, true, false)), 4);
        assert_eq!(piece(sides(false, false, false, true)), 8);
        assert_eq!(piece(sides(true, true, true, true)), 15);
        assert_eq!(piece(sides(true, false, true, false)), 5);
    }

    /// spec: S-0110 claim 2 (a covered corner counts only when neither adjacent side is covered),
    /// T1
    #[test]
    fn corners_reduce_and_use_t1() {
        let all = Covered {
            n: true,
            e: true,
            s: true,
            w: true,
            nw: true,
            ne: true,
            se: true,
            sw: true,
        };
        assert_eq!(piece(all), 15);
        let corners = Covered {
            nw: true,
            ne: true,
            se: true,
            sw: true,
            ..Covered::default()
        };
        assert_eq!(piece(corners), 46);
        // N with SE: SE's sides S and E are clear, so it stays: 16.
        let c = Covered {
            n: true,
            se: true,
            ..Covered::default()
        };
        assert_eq!(piece(c), 16);
        // N with NE: NE's side N is covered, so NE drops out: piece 1.
        let c = Covered {
            n: true,
            ne: true,
            ..Covered::default()
        };
        assert_eq!(piece(c), 1);
        let c = Covered {
            w: true,
            ne: true,
            se: true,
            ..Covered::default()
        };
        assert_eq!(piece(c), 31);
    }

    /// spec: S-0110 claim 2 (the 256 sets reduce to 47 distinct sets with 47 different pieces
    /// 0 … 46)
    #[test]
    fn every_set_gives_one_of_47_pieces() {
        let mut seen = std::collections::BTreeSet::new();
        for m in 0..=255u8 {
            let b = |i: u8| m & (1 << i) != 0;
            let c = Covered {
                n: b(0),
                e: b(1),
                s: b(2),
                w: b(3),
                nw: b(4),
                ne: b(5),
                se: b(6),
                sw: b(7),
            };
            seen.insert(piece(c));
        }
        assert_eq!(seen.len(), PIECES);
        assert_eq!(seen.iter().copied().max(), Some(46));
    }

    /// spec: S-0110 claim 1 (a cell that is not explored gets piece 15, whatever its neighbours)
    #[test]
    fn covered_cell_is_full_cover() {
        assert_eq!(cell_piece(true, Covered::default()), FULL_COVER);
        assert_eq!(cell_piece(false, sides(true, true, false, false)), 3);
    }

    /// spec: S-0110 F1 (shroud sets v except 254; fog adds v − 127 below 128, floored at 0;
    /// glow ⌊a·g/127⌋ capped at 255)
    #[test]
    fn pixel_rules() {
        assert_eq!(shroud_pixel(127, 0), 0);
        assert_eq!(shroud_pixel(127, 64), 64);
        assert_eq!(shroud_pixel(90, 254), 90);
        assert_eq!(shroud_pixel(90, 255), 255);
        assert_eq!(fog_pixel(127, 127), 127);
        assert_eq!(fog_pixel(127, 100), 100);
        assert_eq!(fog_pixel(20, 100), 0);
        assert_eq!(fog_pixel(60, 128), 60);
        assert_eq!(fog_pixel(60, 255), 60);
        assert_eq!(glow_pixel(127, 254), 254);
        assert_eq!(glow_pixel(200, 255), 255);
        assert_eq!(glow_pixel(0, 255), 0);
        assert_eq!(glow_pixel(64, 127), 64);
        assert_eq!(glow_pixel(100, 64), 50);
    }

    fn uniform_sprite(v: u8, w: u32, h: u32, x: i32, y: i32) -> CoverSprite {
        CoverSprite {
            frames: (0..PIECES)
                .map(|_| {
                    Some(CoverFrame {
                        image: IndexedImage::new(w, h, vec![v; (w * h) as usize]).unwrap(),
                        x,
                        y,
                    })
                })
                .collect(),
        }
    }

    fn sprites(s: &CoverSprite) -> Sprites<'_> {
        Sprites {
            shroud: Some(s),
            fog: None,
            fog_of_war: false,
        }
    }

    /// spec: S-0110 claims 5 and 7, F2 (a piece sets the values of its canvas, P_x − 30 … P_x + 29
    /// by P_y … P_y + 29 at height 0; elsewhere the value stays 127)
    #[test]
    fn canvas_placement_on_the_flat_grid() {
        let s = uniform_sprite(64, 60, 30, 0, 0);
        let q = ShroudQuirks::default();
        // Cell (2, 1), level 4: top corner (30, 45) at height 0 whatever its level.
        let cover = Cover {
            cell: (2, 1),
            level: 4,
            shroud: 15,
            fog: None,
        };
        let a = light_alpha((0, 40), (100, 40), &[cover], sprites(&s), &[], &q);
        let at = |x: i32, y: i32| a[((y - 40) * 100 + x) as usize];
        assert_eq!(at(0, 45), 64);
        assert_eq!(at(59, 74), 64);
        assert_eq!(at(60, 45), 127);
        assert_eq!(at(0, 44), 127);
        assert_eq!(at(10, 74), 64);
        // Off: the canvas moves up by 15 per level (S-0110 Edge cases).
        let off = ShroudQuirks {
            shroud_on_flat_grid: false,
        };
        let a = light_alpha((0, 0), (100, 80), &[cover], sprites(&s), &[], &off);
        let at = |x: i32, y: i32| a[(y * 100 + x) as usize];
        // The canvas now spans rows 45 − 60 … 74 − 60, that is −15 … 14.
        assert_eq!(at(0, 0), 64);
        assert_eq!(at(59, 14), 64);
        assert_eq!(at(0, 15), 127);
        assert_eq!(at(0, 45), 127);
    }

    /// spec: S-0110 claim 7 (frame pixels outside the 60 × 30 canvas are not applied); claim 5
    /// (pixels outside the frame's own rectangle are not touched)
    #[test]
    fn frame_rectangle_and_canvas_clip() {
        // A 10 × 6 frame at (25, 12) inside the canvas.
        let small = uniform_sprite(64, 10, 6, 25, 12);
        let q = ShroudQuirks::default();
        let cover = Cover {
            cell: (0, 0),
            level: 0,
            shroud: 0,
            fog: None,
        };
        let a = light_alpha((-30, 0), (60, 30), &[cover], sprites(&small), &[], &q);
        let at = |x: i32, y: i32| a[(y * 60 + x) as usize];
        assert_eq!(at(25, 12), 64);
        assert_eq!(at(34, 17), 64);
        assert_eq!(at(24, 12), 127);
        assert_eq!(at(35, 17), 127);
        assert_eq!(at(25, 18), 127);
        // A 90 × 60 frame at (−10, −10): only its part inside the canvas acts.
        let big = uniform_sprite(0, 90, 60, -10, -10);
        let a = light_alpha((-60, -10), (120, 60), &[cover], sprites(&big), &[], &q);
        let at = |x: i32, y: i32| a[((y + 10) * 120 + x + 60) as usize];
        assert_eq!(at(-30, 0), 0);
        assert_eq!(at(29, 29), 0);
        assert_eq!(at(-31, 0), 127);
        assert_eq!(at(30, 0), 127);
        assert_eq!(at(0, -1), 127);
        assert_eq!(at(0, 30), 127);
    }

    /// spec: S-0110 claim 10, State and ordering (in the 30 × 15 overlap, the lower cell's shroud
    /// pixel sets the value unless it is 254, which keeps the upper cell's)
    #[test]
    fn lower_cell_wins_the_overlap() {
        let q = ShroudQuirks::default();
        let mut s = uniform_sprite(127, 60, 30, 0, 0);
        s.frames[15] = uniform_sprite(64, 60, 30, 0, 0).frames[15].clone();
        let upper = Cover {
            cell: (0, 0),
            level: 0,
            shroud: 15,
            fog: None,
        };
        let lower = Cover {
            cell: (1, 0),
            level: 0,
            shroud: 0,
            fog: None,
        };
        // Overlap: x 0 … 29, y 15 … 29.
        let a = light_alpha((-30, 0), (120, 45), &[upper, lower], sprites(&s), &[], &q);
        let at = |x: i32, y: i32| a[(y * 120 + x + 30) as usize];
        assert_eq!(at(0, 15), 127);
        assert_eq!(at(29, 29), 127);
        assert_eq!(at(-30, 15), 64);
        assert_eq!(at(0, 14), 64);
        // A transparent lower piece keeps the upper cell's value.
        let mut t = s.clone();
        t.frames[0] = uniform_sprite(254, 60, 30, 0, 0).frames[0].clone();
        let a = light_alpha((-30, 0), (120, 45), &[upper, lower], sprites(&t), &[], &q);
        let at = |x: i32, y: i32| a[(y * 120 + x + 30) as usize];
        assert_eq!(at(0, 15), 64);
        assert_eq!(at(30, 15), 127);
    }

    /// spec: S-0110 claims 3 and 4, State and ordering (fog on: both covers from the fog sprite,
    /// the fog piece acting on the value its cell's shroud piece produced)
    #[test]
    fn fog_acts_after_its_shroud_piece() {
        let q = ShroudQuirks::default();
        let shroud = uniform_sprite(0, 60, 30, 0, 0);
        let fog = uniform_sprite(100, 60, 30, 0, 0);
        let cover = Cover {
            cell: (0, 0),
            level: 0,
            shroud: 0,
            fog: Some(15),
        };
        // Fog of war on: the shroud piece comes from the fog sprite (100), then fog 100 − 127.
        let on = Sprites {
            shroud: Some(&shroud),
            fog: Some(&fog),
            fog_of_war: true,
        };
        let a = light_alpha((-30, 0), (60, 30), &[cover], on, &[], &q);
        assert_eq!(a[0], 73);
    }

    /// spec: S-0110 claim 6 (glows scale the covered value, after every cover; a value of 0 stays
    /// black)
    #[test]
    fn glow_scales_the_covered_value() {
        let q = ShroudQuirks::default();
        let s = uniform_sprite(64, 60, 30, 0, 0);
        let img = IndexedImage::new(2, 1, vec![254, 0]).unwrap();
        let cover = Cover {
            cell: (0, 0),
            level: 0,
            shroud: 3,
            fog: None,
        };
        let glow = Glow {
            image: &img,
            x: 0,
            y: 0,
        };
        let a = light_alpha((-30, 0), (60, 30), &[cover], sprites(&s), &[glow], &q);
        assert_eq!(a[30], 128);
        assert_eq!(a[31], 0);
        let black = uniform_sprite(0, 60, 30, 0, 0);
        let a = light_alpha((-30, 0), (60, 30), &[cover], sprites(&black), &[glow], &q);
        assert_eq!(a[30], 0);
    }

    /// spec: S-0110 Import mapping (a missing sprite or frame draws no cover)
    #[test]
    fn missing_sprite_or_frame_draws_nothing() {
        let q = ShroudQuirks::default();
        let cover = Cover {
            cell: (0, 0),
            level: 0,
            shroud: 15,
            fog: None,
        };
        let none = Sprites {
            shroud: None,
            fog: None,
            fog_of_war: false,
        };
        let a = light_alpha((-30, 0), (60, 30), &[cover], none, &[], &q);
        assert!(a.iter().all(|&v| v == 127));
        let mut short = uniform_sprite(0, 60, 30, 0, 0);
        short.frames.truncate(15);
        let a = light_alpha((-30, 0), (60, 30), &[cover], sprites(&short), &[], &q);
        assert!(a.iter().all(|&v| v == 127));
    }
}
