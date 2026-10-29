//! Combat in the live view: S-0111's status extras (health bars, unit
//! selection brackets, veterancy insignia) and our own provisional
//! placeholders for what the simulation's combat creates (projectiles in
//! flight, shots, detonations, animations).
//!
//! The placeholders are presentation of our own design, not game art: the
//! importer looks up no projectile art yet (S-0007 claims 26 to 30 are not
//! implemented), and muzzle flashes have no art path. Animation events play
//! their art (S-0109, `mapview::anims`); the animation marker is the
//! viewer's debug drawing of an animation whose type has no sprite, and the
//! game client draws none. They are marked provisional and can be hidden
//! with the other provisional items, or with their own layer toggle.
//!
//! The culling of claim 1 ([`in_status_view`]), the shading of claim 2
//! (the bracket, insignia, wrench and bomb icon take the light-alpha shade
//! of S-0104 claim 15 at neutral brightness; [`Cache::overlay_lit`]) and the
//! corner brackets of selected buildings (claim 8, [`corner_segments`],
//! [`corner_image`]) are drawn. The pip scale is drawn
//! whole: its origin (claim 10, [`pip_origin`]), who sees it (claim 9), its
//! rows and self-heal marker (claims 10 to 16, [`super::pip_scale`]) and the
//! control-group number (claim 17, [`group_number_at`],
//! [`Cache::group_number_item`]). The bomb icon of S-0519 claim 24
//! is drawn: the first status extra of its carrier, through the cursor
//! palette ([`Cache::bomb_item`], [`CURSOR_PALETTE`]; S-0111 claim 19a),
//! and so is the repair wrench of claim 19 ([`Cache::wrench_item`]).

use std::collections::BTreeMap;

use glasnost_fmt::fnt::{Font, Mask};
use glasnost_fmt::image::{IndexedImage, Rgb8};
use glasnost_fmt::pal::Palette6;
use glasnost_fmt::shp::Shp;
use glasnost_num::{F32, F64, Rounding};
use glasnost_render::compose::{DepthTest, Paint, Profile, Shade, pack565, row565};
use glasnost_render::palette::{PaletteQuirks, palette_from_file};
use glasnost_render::projection::height_lift;
use glasnost_render::scene::{Group, Item, ItemDepth, Scene};

use super::build::Assets;

/// The pip sprite (S-0111 Import mapping).
pub const PIP_SPRITE: &[u8] = b"PIPS.SHP"; // spec: S-0111 Import mapping (overlay sprites)
/// The second pip sprite (S-0111 Import mapping).
pub const PIP2_SPRITE: &[u8] = b"PIPS2.SHP"; // spec: S-0111 Import mapping (overlay sprites: PIPS2.SHP, the second pip sprite)
/// The bracket sprite (S-0111 Import mapping).
pub const BRACKET_SPRITE: &[u8] = b"PIPBRD.SHP"; // spec: S-0111 Import mapping (overlay sprites)
/// The overlay palette the pips and brackets use (S-0111 Import mapping).
pub const OVERLAY_PALETTE: &[u8] = b"PALETTE.PAL"; // spec: S-0111 Import mapping (overlay palette)
/// The bomb icon sprite (S-0519 claim 24, S-0111 Import mapping).
pub const BOMB_SPRITE: &[u8] = b"BOMBCURS.SHP"; // spec: S-0111 Import mapping (BOMBCURS.SHP, the bomb icon of S-0519 claim 24)
/// The wrench sprite (S-0111 claim 19 and Import mapping).
pub const WRENCH_SPRITE: &[u8] = b"WRENCH.SHP"; // spec: S-0111 Import mapping (WRENCH.SHP, the wrench sprite)
/// The wrench's base delay before normalisation, in ticks (S-0111
/// Constants).
const WRENCH_DELAY: i32 = 14; // spec: S-0111 Constants (wrench base delay 14 ticks before normalisation)
/// The cursor palette: the palette of the wrench, the bomb icon (S-0519
/// claim 24, S-0111 claim 19a and Import mapping) and the cursor images
/// (S-0214 Import mapping).
pub const CURSOR_PALETTE: &[u8] = b"MOUSEPAL.PAL"; // spec: S-0111 Import mapping (the wrench and the bomb sprite use the cursor palette MOUSEPAL.PAL, the same as S-0214's cursor images)

/// The cursor palette loaded from its file (S-0111 claim 19a): each file
/// component *c* becomes (4·*c*) mod 256 by S-0103 claim 1, following
/// `quirks.palette_component_wrap` as the palettes of that claim do
/// (off: a component above 63 becomes 252). `None` when the file does not
/// parse as a palette.
pub fn cursor_palette(file: &[u8], q: &PaletteQuirks) -> Option<glasnost_fmt::image::Palette> {
    let pal = Palette6::parse(file).ok()?;
    // spec: S-0111 claim 19a, S-0103 claim 1 and Edge cases (the cursor palette's components ×4, wrapping, or 252 above 63 with the switch off)
    Some(palette_from_file(&pal, q))
}

/// The repair wrench's image frame in tick `t` at game speed `speed`
/// (S-0111 claim 19, F4); `normalise` is `quirks.speed_normalised_delays`.
pub fn wrench_frame(t: i32, speed: u8, normalise: bool) -> usize {
    // spec: S-0111 claim 19 and F4 (N(14, s), the game-speed normalisation of S-0107 F3; with quirks.speed_normalised_delays off, 14 at every speed)
    let n = if normalise {
        glasnost_sim::infantry::normalised(WRENCH_DELAY, speed)
    } else {
        WRENCH_DELAY
    };
    // spec: S-0111 F4 (p = max(2, ⌊N(14, s) / 4⌋))
    let p = (n / 4).max(2);
    // spec: S-0111 F4 (image frame = ⌊6 (t mod p) / (p − 1)⌋, t the ticks since the game started)
    usize::try_from(6 * t.rem_euclid(p) / (p - 1)).unwrap_or(0)
}

/// The image frame of a bomb icon in tick `n` for a bomb planted in tick
/// `t`, with timed delay `delay` and flicker rate `flicker` (S-0519 F3).
pub fn bomb_frame(n: i32, t: i32, delay: i32, flicker: i32) -> usize {
    // spec: S-0519 F3 (q = ⌊D / 6⌋; when q = 0 our engine substitutes q = 1)
    let q = (delay / 6).max(1);
    // spec: S-0519 F3 (b = 1 when (n mod 2F) ≥ F, else 0; when F = 0 our engine uses b = 0)
    let b = if flicker > 0 {
        i32::from((n % (2 * flicker)) >= flicker)
    } else {
        0
    };
    // spec: S-0519 F3 (frame = min(11, 2⌊(n − t) / q⌋ + b), divisions truncated toward zero, frames from 0)
    let frame = (2 * ((n - t) / q) + b).min(11);
    usize::try_from(frame).unwrap_or(0)
}

/// Health segments of a vehicle or aircraft bar (S-0111 claim 6).
pub const UNIT_SEGMENTS: i32 = 17; // spec: S-0111 Constants (health segments, vehicle or aircraft)
/// Health segments of an infantry bar (S-0111 claim 6).
pub const INFANTRY_SEGMENTS: i32 = 8; // spec: S-0111 Constants (health segments, infantry)
/// The height of one level used by the building box (S-0111 Terms).
pub const LEVEL_HEIGHT: i32 = 104; // spec: S-0111 Constants (height of one level)
/// The art height field's absent-key default, in levels (S-0111 Import
/// mapping).
pub const DEFAULT_ART_HEIGHT: i32 = 2; // spec: S-0111 Import mapping (building art Height absent: 2)

/// A health colour (S-0111 claim 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthColour {
    /// Above both ratios.
    Green,
    /// Above the red ratio, at or below the yellow ratio.
    Yellow,
    /// At or below the red ratio, or undefined.
    Red,
}

/// The two condition ratios with the rounding mode in force.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratios {
    /// Yellow ratio.
    pub yellow: F64,
    /// Red ratio.
    pub red: F64,
    /// Rounding mode (S-0205: round toward zero under `quirks.chop_rounding`).
    pub mode: Rounding,
}

/// The health ratio *h* = ⟨H / S⟩ (S-0111 Terms, S-0108 F3).
pub fn health_ratio(health: i32, strength: i32, mode: Rounding) -> F64 {
    // spec: S-0108 F3 (h = H / S in binary64, rounded by the mode in force)
    F64::from_i64(i64::from(health), mode).div(F64::from_i64(i64::from(strength), mode), mode)
}

/// The filled segments *f* of a bar of `n` segments (S-0111 claim 4, F1;
/// claim 5 for a full strength of 0 or below).
pub fn filled_segments(health: i32, strength: i32, n: i32, mode: Rounding) -> i32 {
    if strength <= 0 {
        // spec: S-0111 claim 5 (full strength 0 or below: f = 1 in every case)
        return 1;
    }
    let h = health_ratio(health, strength, mode);
    // spec: S-0111 F1 (f = min(n, max(1, trunc(⟨h × n⟩))), the product rounded by the mode)
    let p = h
        .mul(F64::from_i64(i64::from(n), mode), mode)
        .trunc_i128()
        .unwrap_or(0);
    (p.clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32).clamp(1, n.max(1))
}

/// The health colour (S-0111 claim 5): `building` picks the buildings'
/// column.
pub fn health_colour(health: i32, strength: i32, building: bool, r: &Ratios) -> HealthColour {
    let h = health_ratio(health, strength, r.mode);
    // spec: S-0111 claim 5 (h ≤ red ratio or undefined: red; a building at H = 0 shows green)
    if h.is_nan() || h.le(r.red) {
        if building && health == 0 {
            HealthColour::Green
        } else {
            HealthColour::Red
        }
    } else if h.le(r.yellow) {
        // spec: S-0111 claim 5 (red ratio < h ≤ yellow ratio: yellow)
        HealthColour::Yellow
    } else {
        // spec: S-0111 claim 5 (h above both ratios: green)
        HealthColour::Green
    }
}

/// An overlay sprite.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sprite {
    /// The pip sprite.
    Pips,
    /// The second pip sprite (S-0111 claim 10).
    Pips2,
    /// The bracket sprite.
    Bracket,
    /// The bomb icon sprite (S-0519 claim 24).
    Bomb,
    /// The wrench sprite (S-0111 claim 19).
    Wrench,
}

impl Sprite {
    /// The file.
    pub fn file(self) -> &'static [u8] {
        match self {
            Sprite::Pips => PIP_SPRITE,
            Sprite::Pips2 => PIP2_SPRITE,
            Sprite::Bracket => BRACKET_SPRITE,
            Sprite::Bomb => BOMB_SPRITE,
            Sprite::Wrench => WRENCH_SPRITE,
        }
    }
}

/// One overlay image: a sprite's image frame drawn as a centred image at a
/// displacement from the status anchor (S-0111 Terms), or as a shadow
/// (darkening) of that image's shape (the self-heal blink, claim 16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pip {
    /// Sprite.
    pub sprite: Sprite,
    /// Image frame.
    pub frame: usize,
    /// Displacement from the status anchor, screen pixels.
    pub at: (i32, i32),
    /// Drawn as a shadow instead of as an image.
    pub shadow: bool,
    /// Shaded by the light-alpha values (S-0111 claim 2: the bracket image
    /// and the insignia); segments and pips keep their palette colours.
    pub shaded: bool,
}

impl Pip {
    /// An image `frame` of `sprite` at `at`.
    pub fn image(sprite: Sprite, frame: usize, at: (i32, i32)) -> Pip {
        Pip {
            sprite,
            frame,
            at,
            shadow: false,
            shaded: false,
        }
    }

    /// As [`Pip::image`], shaded (S-0111 claim 2).
    pub fn shaded(sprite: Sprite, frame: usize, at: (i32, i32)) -> Pip {
        Pip {
            shaded: true,
            ..Pip::image(sprite, frame, at)
        }
    }
}

/// The health bar of a vehicle, aircraft or infantry unit (S-0111 claim 6),
/// in drawing order: the bracket image when selected, then the filled
/// segments.
pub fn unit_bar(
    infantry: bool,
    delta: i32,
    health: i32,
    strength: i32,
    selected: bool,
    r: &Ratios,
) -> Vec<Pip> {
    // spec: S-0111 claim 6 (table: segments, segment place and bracket image by kind)
    let (n, x0, y, bracket_frame, bracket_at) = if infantry {
        (INFANTRY_SEGMENTS, -5, delta - 24, 1, (11, delta - 25))
    } else {
        (UNIT_SEGMENTS, -15, delta - 25, 0, (1, delta - 26))
    };
    let mut out = Vec::new();
    if selected {
        // spec: S-0111 claim 6 (the bracket image when selected only; it shows through as the empty part)
        // spec: S-0111 claim 2 (the bracket is shaded)
        out.push(Pip::shaded(Sprite::Bracket, bracket_frame, bracket_at));
    }
    // spec: S-0111 claim 6 (pip sprite image frames 16 green, 17 yellow, 18 red)
    let frame = match health_colour(health, strength, false, r) {
        HealthColour::Green => 16,
        HealthColour::Yellow => 17,
        HealthColour::Red => 18,
    };
    let f = filled_segments(health, strength, n, r.mode);
    // spec: S-0111 claim 6 (segment i at x0 + 2i; only the filled segments are drawn)
    out.extend((0..f).map(|i| Pip::image(Sprite::Pips, frame, (x0 + 2 * i, y))));
    out
}

/// The projected displacement P(ξ, η, ζ) of S-0111 Terms (S-0101 F1 and F2
/// applied to a displacement).
pub fn displacement(xi: i32, eta: i32, zeta: i32) -> (i32, i32) {
    // spec: S-0111 Terms (P = (trunc((30ξ − 30η) / 256), trunc((15ξ + 15η) / 256) − L(ζ)))
    (
        (30 * xi - 30 * eta) / 256,
        (15 * xi + 15 * eta) / 256 - height_lift(zeta),
    )
}

/// The health bar of a building of foundation `w` × `d` cells with an art
/// height field of `levels` (S-0111 claim 7, F5).
pub fn building_bar(
    w: i32,
    d: i32,
    levels: i32,
    health: i32,
    strength: i32,
    r: &Ratios,
) -> Vec<Pip> {
    // spec: S-0111 Terms (α = 128·W, β = 128·D; V = art height × 104 leptons)
    let (alpha, beta, v) = (128 * w, 128 * d, levels * LEVEL_HEIGHT);
    // spec: S-0111 F5 (Q = P(−α, β, V), Q′ = P(−α, −β, V), n = trunc((Q_y − Q′_y) / 2))
    let q = displacement(-alpha, beta, v);
    let q2 = displacement(-alpha, -beta, v);
    let n = (q.1 - q2.1) / 2;
    if n <= 0 {
        return Vec::new();
    }
    let f = filled_segments(health, strength, n, r.mode);
    // spec: S-0111 claim 7 (filled pips in image frame 1 green, 2 yellow, 4 red; the rest image frame 0)
    let frame = match health_colour(health, strength, true, r) {
        HealthColour::Green => 1,
        HealthColour::Yellow => 2,
        HealthColour::Red => 4,
    };
    (0..n)
        .map(|i| {
            Pip::image(
                Sprite::Pips,
                if i < f { frame } else { 0 },
                // spec: S-0111 claim 7 (pip i centred at Q + (4(n − i) + 3, 2(i − n) + 4), top right first)
                (q.0 + 4 * (n - i) + 3, q.1 + 2 * (i - n) + 4),
            )
        })
        .collect()
}

/// The veterancy insignia of a unit or building with veterancy value
/// `v` (S-0111 claim 18), if it shows one.
pub fn insignia(v: F32, infantry: bool) -> Option<Pip> {
    let v = v.to_f64();
    let rz = Rounding::TowardZero;
    // spec: S-0111 claim 18 (veteran 14, elite 15, below 0 19; none in the rookie band)
    let frame = if v.lt(F64::ZERO) {
        19
    } else if v.ge(F64::from_i64(2, rz)) {
        15
    } else if v.ge(F64::from_i64(1, rz)) {
        14
    } else {
        return None;
    };
    // spec: S-0111 claim 2 (the insignia is shaded)
    Some(Pip::shaded(
        Sprite::Pips,
        frame,
        // spec: S-0111 claim 18 (at (5, 2) for infantry, (10, 6) for everything else)
        if infantry { (5, 2) } else { (10, 6) },
    ))
}

/// The pip-scale origin *o* from the status anchor (S-0111 claim 10):
/// (−10, +10) for a vehicle, aircraft or infantry unit, and for a building
/// of foundation `w` × `d` (`building`) the projected displacement of its
/// box's left corner at the base of the box, P(−α, β, 0).
pub fn pip_origin(building: Option<(i32, i32)>) -> (i32, i32) {
    match building {
        Some((w, d)) => {
            // spec: S-0111 claim 10 (a building: o = P(−α, β, 0), α = 128·W, β = 128·D, at the base of the box)
            displacement(-128 * w, 128 * d, 0)
        }
        // spec: S-0111 claim 10 (vehicles, aircraft and infantry: o = (−10, +10))
        None => (-10, 10),
    }
}

/// The digit control group `index` shows (S-0111 claim 17). Our group
/// keys put the group of key *k* at index *k* (Q-1988), so the keyboard's
/// group 1 is index 1 and its tenth group, key 0, is index 0: index *k* is
/// claim 17's group γ = (*k* + 9) mod 10.
pub fn group_digit(index: usize) -> u8 {
    let gamma = (index + 9) % 10;
    // spec: S-0111 claim 17 (group γ shows the digit (γ + 1) mod 10: the first group 1, the tenth 0)
    ((gamma + 1) % 10) as u8
}

/// The point the control-group number is placed at, from the status
/// anchor (S-0111 claim 17): (−4, −39) from the pip-scale origin for
/// infantry and (−4, −36) for everything else.
pub fn group_number_at(infantry: bool, building: Option<(i32, i32)>) -> (i32, i32) {
    let o = pip_origin(building);
    // spec: S-0111 claim 17 ((−4, −39) from o for infantry, (−4, −36) for others)
    let (dx, dy) = if infantry && building.is_none() {
        (-4, -39)
    } else {
        (-4, -36)
    };
    (o.0 + dx, o.1 + dy)
}

/// The game font's data the control-group number is drawn with (S-0111
/// claim 17, F6 and Import mapping): each digit's glyph mask, whose width is
/// the digit's imported width *G* and whose height is the font's bitmap
/// height *H*<sub>f</sub>, and the font's line pitch *P*<sub>f</sub>.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupFont {
    digits: Vec<Mask>,
    pitch: i64,
}

impl GroupFont {
    /// The digits of `font`. `None` for a font without a mapped capital X,
    /// which is rejected as a whole (S-0014 claim 6): then no group number
    /// is drawn.
    pub fn new(font: &Font) -> Option<Self> {
        // spec: S-0111 claim 17 (a missing glyph uses S-0213 claim 25's generated fallback: the complement of the capital X's mask, with X's stored width as G); S-0014 claims 4 and 6
        let fallback = font.fallback()?;
        let digits = (0..10u8)
            .map(|d| {
                // spec: S-0111 claim 17 (the digit is one glyph of the game bitmap font, the text unit of its character; no separate digit images)
                let unit = u16::from(b'0' + d);
                font.glyph_number(unit)
                    .and_then(|j| font.mask(j))
                    .unwrap_or_else(|| fallback.clone())
            })
            .collect();
        Some(Self {
            digits,
            pitch: i64::from(font.line_pitch),
        })
    }

    /// The glyph mask of `digit`.
    pub fn mask(&self, digit: u8) -> &Mask {
        &self.digits[usize::from(digit % 10)]
    }

    /// The line pitch *P*<sub>f</sub>.
    pub fn pitch(&self) -> i64 {
        self.pitch
    }
}

/// The group number's image index of the house colour (index 0 is
/// transparent, S-0104 claim 6).
pub const GROUP_INK: u8 = 1;
/// The group number's image index of the box's opaque black fill.
pub const GROUP_FILL: u8 = 2;

/// The most rows a group number's box is given.
// design: a cap on the box's height for a font with a huge line pitch, so that the image fits in memory; no battlefield view is that tall, and claim 17's view edge cuts the box anyway (S-0111 sets no bound)
const GROUP_BOX_MAX_ROWS: i64 = 4096;

/// The group number of glyph `mask` for line pitch `pitch` (S-0111 claim
/// 17, F6): its image and the displacement of the image's top-left pixel
/// from the placement point *d*. Index [`GROUP_INK`] is the house colour,
/// [`GROUP_FILL`] the box's black fill, 0 transparent. `None` when nothing
/// is drawn (a glyph with no pixels and no box).
pub fn group_number_image(mask: &Mask, pitch: i64) -> Option<(IndexedImage, (i32, i32))> {
    let g = mask.width as i64;
    let h = mask.height as i64;
    // spec: S-0111 F6 (box: d_x − 3 ≤ x ≤ d_x + G + 2 and d_y + 2 ≤ y ≤ d_y + P_f − 3, outer size (G + 6) × (P_f − 4)); claim 17 (when P_f − 4 ≤ 0 our engine draws no box, only the glyph)
    let boxed = (pitch - 4 > 0).then(|| (-3, g + 2, 2, (pitch - 3).min(1 + GROUP_BOX_MAX_ROWS)));
    // spec: S-0111 F6 (glyph mask pixel (i, r), 0 ≤ i < G, 0 ≤ r < H_f, at d + (i, r): left-aligned, its top stored row at d; every stored row paints, with no pitch limit)
    let glyph = (g > 0 && h > 0).then_some((0, g - 1, 0, h - 1));
    let (x0, x1, y0, y1) = match (boxed, glyph) {
        (Some(b), Some(m)) => (b.0.min(m.0), b.1.max(m.1), b.2.min(m.2), b.3.max(m.3)),
        (Some(r), None) | (None, Some(r)) => r,
        (None, None) => return None,
    };
    let w = x1 - x0 + 1;
    let mut px = vec![0u8; (w * (y1 - y0 + 1)) as usize];
    let at = |x: i64, y: i64| ((y - y0) * w + (x - x0)) as usize;
    if let Some((bx0, bx1, by0, by1)) = boxed {
        for y in by0..=by1 {
            for x in bx0..=bx1 {
                // spec: S-0111 claim 17 and F6 (the box is first filled with opaque black; then its outermost one-pixel ring is painted in the house colour, a closed rectangle round the digit alone)
                let border = x == bx0 || x == bx1 || y == by0 || y == pitch - 3;
                px[at(x, y)] = if border { GROUP_INK } else { GROUP_FILL };
            }
        }
    }
    for r in 0..h {
        for i in 0..g {
            // spec: S-0111 claim 17 and F6 (the glyph's set mask pixels are painted in the house colour last, over the border, the fill or the scene; clear mask pixels leave what is under them)
            if mask.get(i as usize, r as usize) {
                px[at(i, r)] = GROUP_INK;
            }
        }
    }
    let img = IndexedImage::new(w as u32, (y1 - y0 + 1) as u32, px).ok()?;
    Some((img, (x0 as i32, y0 as i32)))
}

// ---------------------------------------------------------------------------
// Placeholders (our own design, provisional)
// ---------------------------------------------------------------------------

/// What a placeholder marks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Marker {
    /// A projectile in flight; `arcing` and `homing` pick its colour.
    Projectile {
        /// Arcing flight.
        arcing: bool,
        /// Steering (a turn rate above 0).
        homing: bool,
    },
    /// A shot left a firer at this point.
    Muzzle,
    /// A detonation of this half-width in pixels, at this stage.
    Burst {
        /// Half-width, pixels.
        half: i32,
        /// Stage, 0 first.
        stage: i32,
    },
    /// An animation whose type has no sprite was created here (the
    /// viewer's debug layer).
    Anim,
    /// The reach of a radiation field (S-0525): a green ring of this
    /// half-width in pixels. Our placeholder for the field's light, which
    /// the view does not draw (it never relights the map).
    Field {
        /// Half-width, pixels.
        half: i32,
    },
    /// A crate on the map (S-0520): a yellow box, our placeholder for the
    /// crate overlay's picture, which the view draws only from the map.
    Crate,
    /// A gas particle (S-0613) whose type has no sprite: a green cloud, our
    /// placeholder, drawn only in the debug-markers view (the game view
    /// draws nothing for it, Q-1520 interim rule).
    Gas,
    /// A particle of a smoke system (S-0616): a small white puff, our
    /// placeholder like [`Marker::Gas`].
    Smoke,
    /// A particle of a fire system (S-0616): an orange flame dot.
    Fire,
    /// A particle of a spark system (S-0616): a small yellow dot.
    Spark,
    /// A particle of a railgun system (S-0616): a small cyan dot.
    Rail,
}

/// Placeholder palette indices (our own colours).
mod ink {
    pub const OUTLINE: u8 = 1;
    pub const WHITE: u8 = 2;
    pub const CYAN: u8 = 3;
    pub const ORANGE: u8 = 4;
    pub const MAGENTA: u8 = 5;
    pub const YELLOW: u8 = 6;
    pub const RED: u8 = 7;
    pub const GREEN: u8 = 8;
}

/// The placeholder palette row: our own colours.
fn placeholder_row() -> [u16; 256] {
    let mut row = [0u16; 256];
    for (i, c) in [
        (ink::OUTLINE, Rgb8::new(24, 24, 32)),
        (ink::WHITE, Rgb8::new(255, 255, 255)),
        (ink::CYAN, Rgb8::new(80, 230, 255)),
        (ink::ORANGE, Rgb8::new(255, 150, 20)),
        (ink::MAGENTA, Rgb8::new(255, 70, 230)),
        (ink::YELLOW, Rgb8::new(255, 245, 120)),
        (ink::RED, Rgb8::new(230, 40, 30)),
        (ink::GREEN, Rgb8::new(90, 255, 120)),
    ] {
        row[usize::from(i)] = pack565(c);
    }
    row
}

/// Ticks a muzzle marker stays (our own choice).
pub const MUZZLE_TICKS: i32 = 3;
/// Ticks a detonation burst stays (our own choice); it grows in
/// [`BURST_STAGES`] stages.
pub const BURST_TICKS: i32 = 8;
/// Stages of a burst.
pub const BURST_STAGES: i32 = 4;
/// Ticks an animation marker stays (our own choice).
pub const ANIM_TICKS: i32 = 12;

/// The half-width in pixels of a detonation burst of spread radius `radius`
/// leptons: the projected half-width of a ground circle of that radius
/// (S-0101 F1: 30 pixels per 256 leptons along each axis, so a radius
/// reaches 30·√2 / 256 pixels across), kept between 4 and 96 and rounded to
/// a multiple of 4 so that bursts share images.
pub fn burst_half_width(radius: i32) -> i32 {
    // spec: S-0101 F1 (x = (30ξ − 30η) / 256: a ground circle of radius R spans 30·√2·R / 256 pixels each side)
    let px = i64::from(radius.max(0)) * 30 * 1414 / (256 * 1000);
    let px = px.clamp(4, 96) as i32; // nocite: our own size limits of a placeholder
    (px + 2) / 4 * 4
}

/// A marker's image, drawn by our own rules.
fn marker_image(m: Marker) -> IndexedImage {
    let disc = |r: i32, ry: i32, ring: Option<i32>, fill: u8, edge: u8| {
        let (w, h) = (2 * r + 1, 2 * ry + 1);
        let mut px = vec![0u8; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                // Ellipse distance in 1/1000ths of the radius.
                let dx = i64::from(x - r) * 1000 / i64::from(r.max(1));
                let dy = i64::from(y - ry) * 1000 / i64::from(ry.max(1));
                let d2 = dx * dx + dy * dy;
                let inside = d2 <= 1_000_000;
                let rim = d2 > 640_000;
                let v = match ring {
                    Some(inner) => {
                        let inner2 = i64::from(inner) * i64::from(inner);
                        if inside && d2 >= inner2 {
                            if rim { edge } else { fill }
                        } else {
                            0
                        }
                    }
                    None if inside => {
                        if rim {
                            edge
                        } else {
                            fill
                        }
                    }
                    None => 0,
                };
                px[(y * w + x) as usize] = v;
            }
        }
        IndexedImage::new(w as u32, h as u32, px).expect("sized")
    };
    match m {
        Marker::Projectile { arcing, homing } => {
            // A 7 × 7 diamond with a white core.
            let fill = if homing {
                ink::MAGENTA
            } else if arcing {
                ink::ORANGE
            } else {
                ink::CYAN
            };
            let mut px = vec![0u8; 49];
            for y in 0..7i32 {
                for x in 0..7i32 {
                    let d = (x - 3).abs() + (y - 3).abs();
                    px[(y * 7 + x) as usize] = match d {
                        0 => ink::WHITE,
                        1 | 2 => fill,
                        3 => ink::OUTLINE,
                        _ => 0,
                    };
                }
            }
            IndexedImage::new(7, 7, px).expect("sized")
        }
        Marker::Muzzle => {
            // A 7 × 7 plus.
            let mut px = vec![0u8; 49];
            for i in 0..7 {
                px[3 * 7 + i] = ink::YELLOW;
                px[i * 7 + 3] = ink::YELLOW;
            }
            px[3 * 7 + 3] = ink::WHITE;
            IndexedImage::new(7, 7, px).expect("sized")
        }
        Marker::Burst { half, stage } => {
            // An ellipse twice as wide as high (S-0101 F1's ground
            // projection), filled at the first stage, then an opening ring.
            let r = (half * (stage + 1) / BURST_STAGES).max(2);
            let ry = (r / 2).max(1);
            if stage == 0 {
                disc(r, ry, None, ink::YELLOW, ink::ORANGE)
            } else {
                // Inner edge in 1/1000ths of the radius.
                let inner = 400 + 150 * stage;
                let fill = if stage + 1 == BURST_STAGES {
                    ink::RED
                } else {
                    ink::ORANGE
                };
                disc(r, ry, Some(inner), fill, ink::RED)
            }
        }
        Marker::Field { half } => {
            // A ring twice as wide as high (S-0101 F1's ground projection).
            let r = half.max(4);
            disc(r, (r / 2).max(1), Some(880), ink::GREEN, ink::GREEN)
        }
        // A cloud twice as wide as high (S-0101 F1's ground projection).
        Marker::Gas => disc(14, 7, None, ink::GREEN, ink::OUTLINE),
        Marker::Smoke => disc(8, 4, None, ink::WHITE, ink::OUTLINE),
        Marker::Fire => disc(4, 3, None, ink::ORANGE, ink::RED),
        Marker::Spark => disc(2, 2, None, ink::YELLOW, ink::WHITE),
        Marker::Rail => disc(2, 2, None, ink::CYAN, ink::WHITE),
        Marker::Crate => {
            // A 13 × 11 box with an outline and a cross.
            let mut px = vec![0u8; 143];
            for y in 0..11usize {
                for x in 0..13usize {
                    let edge = x == 0 || y == 0 || x == 12 || y == 10;
                    let cross = x == 6 || y == 5;
                    px[y * 13 + x] = if edge {
                        ink::ORANGE
                    } else if cross {
                        ink::WHITE
                    } else {
                        ink::YELLOW
                    };
                }
            }
            IndexedImage::new(13, 11, px).expect("sized")
        }
        Marker::Anim => {
            // A 9 × 9 hollow square with a diagonal cross.
            let mut px = vec![0u8; 81];
            for y in 0..9usize {
                for x in 0..9usize {
                    let edge = x == 0 || y == 0 || x == 8 || y == 8;
                    let cross = x == y || x + y == 8;
                    px[y * 9 + x] = if edge {
                        ink::MAGENTA
                    } else if cross {
                        ink::GREEN
                    } else {
                        0
                    };
                }
            }
            IndexedImage::new(9, 9, px).expect("sized")
        }
    }
}

/// A short-lived marker: what, where (world pixel of its centre), and the
/// tick it was created.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transient {
    /// What.
    pub kind: TransientKind,
    /// World pixel of the centre.
    pub at: (i32, i32),
    /// Tick of creation (the tick that ran when the event came).
    pub born: i32,
}

/// What a transient marks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransientKind {
    /// A muzzle.
    Muzzle,
    /// A burst of this half-width.
    Burst(i32),
    /// An animation.
    Anim,
}

impl Transient {
    /// The marker shown at tick `now`, or `None` once it has gone.
    pub fn marker(&self, now: i32) -> Option<Marker> {
        let age = now - self.born;
        if age < 0 {
            return None;
        }
        match self.kind {
            TransientKind::Muzzle => (age < MUZZLE_TICKS).then_some(Marker::Muzzle),
            TransientKind::Burst(half) => (age < BURST_TICKS).then_some(Marker::Burst {
                half,
                stage: age * BURST_STAGES / BURST_TICKS,
            }),
            TransientKind::Anim => (age < ANIM_TICKS).then_some(Marker::Anim),
        }
    }
}

/// Counts of the combat events the view received.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// Shots.
    pub shots: u64,
    /// Detonations.
    pub detonations: u64,
    /// Animations created.
    pub anims: u64,
    /// Animations created whose type has a sprite (played with art,
    /// S-0109), the move-feedback animation included.
    pub anims_with_art: u64,
    /// Animations created whose type has no sprite (or that the import
    /// does not list): they draw nothing in the game view.
    pub anims_without_sprite: u64,
    /// Animations that drew items in the last redraw.
    pub anims_drawn: usize,
    /// Most animations drawn in one redraw.
    pub most_anims_drawn: usize,
    /// Particles that drew items in the last redraw (S-0613, S-0616).
    pub particles_drawn: usize,
    /// Objects killed.
    pub kills: u64,
    /// Most projectiles in flight at once.
    pub most_in_flight: usize,
    /// Hits that gas clouds dealt (S-0613 claim 13).
    pub gas_hits: u64,
    /// Most gas particles in the world at once (S-0613).
    pub most_gas: usize,
    /// Electric bolts made (S-0316).
    pub bolts: u64,
    /// Electric-bolt pixels drawn in the last redraw (S-0316).
    pub bolt_pixels: usize,
}

/// A decoded overlay sprite frame: image, frame offset, canvas size.
type Frame = (u32, (i32, i32), (i32, i32));

/// A control-group digit's scene image and its offset from the placement
/// point (S-0111 F6).
type GroupImage = (u32, (i32, i32));

/// The images and palette rows the combat drawing added to a scene, kept so
/// that each is added once (a scene's images only grow).
#[derive(Clone, Debug, Default)]
pub struct Cache {
    placeholder_row: Option<usize>,
    overlay_row: Option<Option<usize>>,
    mouse_row: Option<Option<usize>>,
    markers: BTreeMap<Marker, u32>,
    frames: BTreeMap<(Sprite, usize), Option<Frame>>,
    group_rows: BTreeMap<[u8; 3], usize>,
    group_digits: BTreeMap<u8, Option<GroupImage>>,
    overlay_lit: Option<Option<Shade>>,
    mouse_lit: Option<Option<Shade>>,
    corners: BTreeMap<(i32, i32, i32, u8), Option<GroupImage>>,
}

const NEUTRAL_BRIGHTNESS: i32 = 1000; // spec: S-0111 claim 2 (neutral brightness), S-0104 claim 15 (brightness in per mille)

/// The widening of the battlefield view within which status extras are
/// drawn, left and right (S-0111 claim 1).
pub const CULL_X: i32 = 360; // spec: S-0111 claim 1
/// As [`CULL_X`], above and below.
pub const CULL_Y: i32 = 180; // spec: S-0111 claim 1

/// Whether the status anchor `anchor` (world pixels) lies within the
/// battlefield view `view` (origin, size) widened by [`CULL_X`] and
/// [`CULL_Y`], equality included (S-0111 claim 1, scenario OV1). No view
/// known: everything is in view.
pub fn in_status_view(anchor: (i32, i32), view: Option<((i32, i32), (u32, u32))>) -> bool {
    let Some((o, (w, h))) = view else {
        return true;
    };
    let (x, y) = (i64::from(anchor.0 - o.0), i64::from(anchor.1 - o.1));
    // spec: S-0111 claim 1 (within 360 pixels left or right and 180 above or below, equality included)
    (-i64::from(CULL_X)..=i64::from(w) + i64::from(CULL_X)).contains(&x)
        && (-i64::from(CULL_Y)..=i64::from(h) + i64::from(CULL_Y)).contains(&y)
}

/// The overlay-palette index of a selected building's corner brackets.
pub const CORNER_INDEX: u8 = 15; // spec: S-0111 claim 8 (palette index 15)
/// The index when the building's height above the terrain is below
/// [`CORNER_LOW`].
pub const CORNER_INDEX_LOW: u8 = 12; // spec: S-0111 claim 8 (index 12)
/// The height above the terrain, in leptons, below which index 12 is used.
pub const CORNER_LOW: i32 = -4; // spec: S-0111 claim 8 (below -4 leptons)

/// The line segments of a selected building's corner brackets, as screen
/// displacements from the status anchor (S-0111 claim 8): for a building
/// of foundation `w` x `d` and box height `v` leptons, at each drawn corner
/// of the building box a line along each edge that meets there, for a
/// quarter of that edge's length rounded toward zero in leptons, projected
/// by [`displacement`].
// design: S-0111 gives low confidence on which corners are drawn; we draw the four corners of the top face and the three base corners the box's silhouette shows (left, front, right); the hidden back base corner is left out
pub fn corner_segments(w: i32, d: i32, v: i32) -> Vec<((i32, i32), (i32, i32))> {
    let (alpha, beta) = (128 * w, 128 * d);
    // spec: S-0111 claim 8 (a quarter of the edge's length, toward zero: the edges are 256W, 256D and V leptons long)
    let (qx, qy, qz) = (256 * w / 4, 256 * d / 4, v / 4);
    let mut corners = Vec::new();
    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        corners.push((sx, sy, v, -1)); // top face: its vertical edge runs down
    }
    for (sx, sy) in [(-1, 1), (1, 1), (1, -1)] {
        corners.push((sx, sy, 0, 1)); // base: its vertical edge runs up
    }
    let mut out = Vec::new();
    for (sx, sy, z, up) in corners {
        let at = (sx * alpha, sy * beta, z);
        let from = displacement(at.0, at.1, at.2);
        for end in [
            (at.0 - sx * qx, at.1, at.2),
            (at.0, at.1 - sy * qy, at.2),
            (at.0, at.1, at.2 + up * qz),
        ] {
            out.push((from, displacement(end.0, end.1, end.2)));
        }
    }
    out
}

/// The image of the corner brackets of a building (`w` x `d`, box height
/// `v`) in palette index `index`, and the displacement of its top-left
/// pixel from the status anchor; `None` when there is nothing to draw.
pub fn corner_image(w: i32, d: i32, v: i32, index: u8) -> Option<(IndexedImage, (i32, i32))> {
    let segs = corner_segments(w, d, v);
    let pts = || segs.iter().flat_map(|&(a, b)| [a, b]);
    let (x0, x1) = (pts().map(|p| p.0).min()?, pts().map(|p| p.0).max()?);
    let (y0, y1) = (pts().map(|p| p.1).min()?, pts().map(|p| p.1).max()?);
    let (iw, ih) = ((x1 - x0 + 1) as usize, (y1 - y0 + 1) as usize);
    let mut px = vec![0u8; iw * ih];
    for &(a, b) in &segs {
        // spec: S-0111 claim 8 (thin lines: one pixel wide)
        let (mut x, mut y) = a;
        let (dx, dy) = ((b.0 - a.0).abs(), -(b.1 - a.1).abs());
        let (sx, sy) = ((b.0 - a.0).signum(), (b.1 - a.1).signum());
        let mut err = dx + dy;
        loop {
            px[(y - y0) as usize * iw + (x - x0) as usize] = index;
            if (x, y) == b {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
    let img = IndexedImage::new(iw as u32, ih as u32, px).ok()?;
    Some((img, (x0, y0)))
}

impl Cache {
    /// The placeholder row, added on first use.
    fn placeholder_row(&mut self, scene: &mut Scene) -> usize {
        *self
            .placeholder_row
            .get_or_insert_with(|| scene.add_row(placeholder_row()))
    }

    /// The overlay palette's row, unshaded (S-0111 claim 2: segments and
    /// pips keep their palette colours), added on first use; `None` when
    /// the palette file is missing or unreadable.
    pub fn overlay_row(&mut self, scene: &mut Scene, assets: &Assets) -> Option<usize> {
        *self.overlay_row.get_or_insert_with(|| {
            let pal = Palette6::parse(assets.get(OVERLAY_PALETTE)?).ok()?;
            // spec: S-0103 claim 1 (a palette file's components are loaded ×4)
            let pal = palette_from_file(&pal, &PaletteQuirks::default());
            Some(scene.add_row(row565(&pal.colors)))
        })
    }

    /// The cursor palette's row, added on first use; `None` when the
    /// palette is missing (S-0111 claim 19a: then neither the wrench nor
    /// the bomb icon is drawn).
    fn mouse_row(&mut self, scene: &mut Scene, assets: &Assets) -> Option<usize> {
        *self.mouse_row.get_or_insert_with(|| {
            // spec: S-0111 claim 19a (the wrench and the bomb icon are drawn through the cursor palette)
            let pal = cursor_palette(assets.get(CURSOR_PALETTE)?, &PaletteQuirks::default())?;
            Some(scene.add_row(row565(&pal.colors)))
        })
    }

    /// The shade table of a palette at neutral brightness (S-0111 claim 2,
    /// S-0104 claim 15), added to the scene once.
    fn lit(scene: &mut Scene, pal: &glasnost_fmt::image::Palette) -> Shade {
        // spec: S-0104 claims 6 and 15 (the untinted table; the light-alpha value picks the row), S-0111 claim 2 (neutral brightness)
        let t = glasnost_render::light::untinted_table(pal);
        let base = scene.rows.len();
        for row in &t {
            scene.add_row(row565(row));
        }
        Shade::new(base, t.len(), NEUTRAL_BRIGHTNESS)
    }

    /// The overlay palette's shade table (S-0111 claim 2: bracket,
    /// insignia, corner brackets).
    pub fn overlay_lit(&mut self, scene: &mut Scene, assets: &Assets) -> Option<Shade> {
        *self.overlay_lit.get_or_insert_with(|| {
            let pal = Palette6::parse(assets.get(OVERLAY_PALETTE)?).ok()?;
            let pal = palette_from_file(&pal, &PaletteQuirks::default());
            Some(Self::lit(scene, &pal))
        })
    }

    /// The cursor palette's shade table (S-0111 claim 2: wrench and bomb).
    fn mouse_lit(&mut self, scene: &mut Scene, assets: &Assets) -> Option<Shade> {
        *self.mouse_lit.get_or_insert_with(|| {
            let pal = cursor_palette(assets.get(CURSOR_PALETTE)?, &PaletteQuirks::default())?;
            Some(Self::lit(scene, &pal))
        })
    }

    /// The corner brackets of a selected building (S-0111 claim 8) at the
    /// status anchor `anchor`, in overlay-palette `index`; `None` when the
    /// overlay palette is missing.
    #[allow(clippy::too_many_arguments)]
    pub fn corner_item(
        &mut self,
        scene: &mut Scene,
        assets: &Assets,
        anchor: (i32, i32),
        foundation: (i32, i32),
        v: i32,
        index: u8,
        object: Option<u32>,
    ) -> Option<Item> {
        let shade = self.overlay_lit(scene, assets)?;
        let (image, (ox, oy)) = (*self
            .corners
            .entry((foundation.0, foundation.1, v, index))
            .or_insert_with(|| {
                let (img, off) = corner_image(foundation.0, foundation.1, v, index)?;
                Some((scene.add_image(img), off))
            }))?;
        Some(Item {
            image,
            x: anchor.0 + ox,
            y: anchor.1 + oy,
            test: DepthTest::Ignore,
            depth: flat(),
            // spec: S-0111 claim 2 (selection marks are shaded by the light-alpha values)
            paint: Paint::Lit(shade),
            group: Group::Status,
            provisional: false,
            object,
            stack: None,
        })
    }

    /// The bomb icon's item at image frame `frame`, centred on the
    /// carrier's draw point `at` (S-0519 claim 24); `None` when the sprite,
    /// its frame or the cursor palette is missing.
    pub fn bomb_item(
        &mut self,
        scene: &mut Scene,
        assets: &Assets,
        at: (i32, i32),
        frame: usize,
        object: Option<u32>,
    ) -> Option<Item> {
        // spec: S-0519 claim 24 (the fixed cursor palette, no house-colour remapping, neutral brightness, no translucency)
        let row = self.mouse_row(scene, assets)?;
        let lit = self.mouse_lit(scene, assets);
        let (image, (fx, fy), (cw, ch)) = self.frame(scene, assets, Sprite::Bomb, frame)?;
        Some(Item {
            image,
            // spec: S-0519 claim 24, S-0102 claim 19 (centred placement on the draw point, no further displacement)
            x: at.0 - cw / 2 + fx,
            y: at.1 - ch / 2 + fy,
            // spec: S-0519 claim 24 (a status extra: after the object layers, with no depth test)
            test: DepthTest::Ignore,
            depth: flat(),
            // spec: S-0111 claim 2 (shaded by the light-alpha values at neutral brightness)
            paint: lit.map_or(Paint::Row(row), Paint::Lit),
            group: Group::Status,
            provisional: false,
            object,
            stack: None,
        })
    }

    /// The repair wrench's item at image frame `frame`, a centred image at
    /// the status anchor `anchor` (S-0111 claims 1 and 19, Terms); `None`
    /// when the sprite, its frame or the cursor palette is missing.
    pub fn wrench_item(
        &mut self,
        scene: &mut Scene,
        assets: &Assets,
        anchor: (i32, i32),
        frame: usize,
        object: Option<u32>,
    ) -> Option<Item> {
        // spec: S-0111 claim 19 and Import mapping (the wrench is drawn through the cursor palette)
        let row = self.mouse_row(scene, assets)?;
        let lit = self.mouse_lit(scene, assets);
        let (image, (fx, fy), (cw, ch)) = self.frame(scene, assets, Sprite::Wrench, frame)?;
        Some(Item {
            image,
            // spec: S-0111 claim 1 and Terms (placed from the status anchor, a centred image; claim 19 gives the wrench no displacement)
            x: anchor.0 - cw / 2 + fx,
            y: anchor.1 - ch / 2 + fy,
            // spec: S-0102 claim 1 (a status extra: after the object layers, with no depth test)
            test: DepthTest::Ignore,
            depth: flat(),
            // spec: S-0111 claim 2 (shaded by the light-alpha values at neutral brightness)
            paint: lit.map_or(Paint::Row(row), Paint::Lit),
            group: Group::Status,
            provisional: false,
            object,
            stack: None,
        })
    }

    /// Forgets the group numbers' images, for a new font.
    pub fn forget_group_digits(&mut self) {
        self.group_digits.clear();
    }

    /// The control-group number's item (S-0111 claim 17, F6): `digit`'s
    /// glyph of `font` and its box, the glyph's top stored row and left
    /// column at the placement point `at` from the status anchor `anchor`,
    /// in the house colour `colour`; `None` when it has no pixels.
    #[allow(clippy::too_many_arguments)]
    pub fn group_number_item(
        &mut self,
        scene: &mut Scene,
        font: &GroupFont,
        anchor: (i32, i32),
        at: (i32, i32),
        digit: u8,
        colour: [u8; 3],
        object: Option<u32>,
    ) -> Option<Item> {
        let row = *self.group_rows.entry(colour).or_insert_with(|| {
            let [r, g, b] = colour;
            let mut pal = [Rgb8 { r: 0, g: 0, b: 0 }; 256];
            // spec: S-0111 claim 17 (the digit and the border share one colour: the owner's house display RGB of S-0103 claim 15, at display precision)
            pal[usize::from(GROUP_INK)] = Rgb8 { r, g, b };
            // spec: S-0111 claim 17 and Constants (the box's fill: black, opaque)
            pal[usize::from(GROUP_FILL)] = Rgb8 { r: 0, g: 0, b: 0 };
            scene.add_row(row565(&pal))
        });
        let (image, (ox, oy)) = (*self.group_digits.entry(digit).or_insert_with(|| {
            let (img, off) = group_number_image(font.mask(digit), font.pitch())?;
            Some((scene.add_image(img), off))
        }))?;
        Some(Item {
            image,
            // spec: S-0111 claim 17 and F6 (the glyph left-aligned with its top stored row at the placement point d; no centring, baseline shift or shadow)
            x: anchor.0 + at.0 + ox,
            y: anchor.1 + at.1 + oy,
            // spec: S-0102 claim 1 (a status extra: after the object layers, with no depth test)
            test: DepthTest::Ignore,
            depth: flat(),
            // spec: S-0111 claims 2 and 17 (the number and its box are drawn unshaded: lighting, shroud and the object's colour-adjustment tint change neither digit, border nor fill)
            paint: Paint::Row(row),
            group: Group::Status,
            provisional: false,
            object,
            stack: None,
        })
    }

    /// A marker's image, added on first use.
    fn marker(&mut self, scene: &mut Scene, m: Marker) -> u32 {
        *self
            .markers
            .entry(m)
            .or_insert_with(|| scene.add_image(marker_image(m)))
    }

    /// An overlay sprite frame, decoded on first use; `None` when the sprite
    /// or the frame is missing (S-0111 Import mapping: a missing sprite
    /// draws nothing).
    fn frame(
        &mut self,
        scene: &mut Scene,
        assets: &Assets,
        s: Sprite,
        frame: usize,
    ) -> Option<Frame> {
        *self.frames.entry((s, frame)).or_insert_with(|| {
            let shp = Shp::parse(assets.get(s.file())?).ok()?;
            let f = *shp.frames.get(frame)?;
            let img = shp
                .decode_frame(frame)
                .ok()
                .filter(|i| i.width() > 0 && i.height() > 0)?;
            Some((
                scene.add_image(img),
                (i32::from(f.x), i32::from(f.y)),
                (i32::from(shp.width), i32::from(shp.height)),
            ))
        })
    }

    /// A placeholder item centred on world pixel `at`.
    pub fn marker_item(&mut self, scene: &mut Scene, m: Marker, at: (i32, i32)) -> Item {
        let row = self.placeholder_row(scene);
        let image = self.marker(scene, m);
        let img = &scene.images[image as usize];
        let (w, h) = (img.width() as i32, img.height() as i32);
        Item {
            image,
            x: at.0 - w / 2,
            y: at.1 - h / 2,
            test: DepthTest::Ignore,
            depth: flat(),
            paint: Paint::Row(row),
            group: Group::Effects,
            provisional: true,
            object: None,
            stack: None,
        }
    }

    /// The items of overlay images `pips` placed from the status anchor
    /// `anchor` (world pixels), each a centred image (S-0111 Terms); a
    /// missing sprite, frame or palette draws nothing.
    pub fn pip_items(
        &mut self,
        scene: &mut Scene,
        assets: &Assets,
        anchor: (i32, i32),
        pips: &[Pip],
        object: Option<u32>,
    ) -> Vec<Item> {
        let Some(row) = self.overlay_row(scene, assets) else {
            return Vec::new();
        };
        let lit = self.overlay_lit(scene, assets);
        pips.iter()
            .filter_map(|p| {
                let (image, (fx, fy), (cw, ch)) = self.frame(scene, assets, p.sprite, p.frame)?;
                let (cx, cy) = (anchor.0 + p.at.0, anchor.1 + p.at.1);
                Some(Item {
                    image,
                    // spec: S-0111 Terms (centred image: the canvas centre, halved and truncated, at the point)
                    x: cx - cw / 2 + fx,
                    y: cy - ch / 2 + fy,
                    test: DepthTest::Ignore,
                    depth: flat(),
                    // spec: S-0111 claim 16 (the self-heal blink: drawn as a shadow, darkening, instead of as an image)
                    paint: if p.shadow {
                        Paint::Shadow
                    } else if p.shaded {
                        // spec: S-0111 claim 2 (bracket and insignia take the light-alpha shade; segments and pips keep their palette colours)
                        lit.map_or(Paint::Row(row), Paint::Lit)
                    } else {
                        Paint::Row(row)
                    },
                    group: Group::Status,
                    provisional: false,
                    object,
                    stack: None,
                })
            })
            .collect()
    }
}

fn flat() -> ItemDepth {
    ItemDepth::Profile {
        profile: Profile::Flat,
        bias: 0,
        companion: None,
    }
}

/// A synthetic game font of bitmap height `h` and line pitch `p`: the
/// digit 1 two pixels wide (its top two rows and its bottom row set
/// across), the digit 5 four pixels wide (a single column), the capital
/// X three pixels wide and asymmetric, the other digits unmapped.
#[cfg(test)]
pub(crate) fn digit_font(h: usize, p: u32) -> Font {
    let mut f = Font::empty(1, h, p);
    let mut one = vec!["##", "##"];
    one.extend(std::iter::repeat_n(".#", h.saturating_sub(3)));
    one.push("##");
    let j1 = f.push_glyph(2, &one);
    f.map_unit(u16::from(b'1'), Some(j1));
    let five = vec!["#..."; h];
    let j5 = f.push_glyph(4, &five);
    f.map_unit(u16::from(b'5'), Some(j5));
    let x = f.push_glyph(3, &["#..", ".#."]);
    f.map_unit(88, Some(x));
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ratios(yellow: &str, red: &str) -> Ratios {
        let rz = Rounding::TowardZero;
        let d = |s: &str| {
            let (num, den): (i64, i64) = match s {
                "0.5" => (1, 2),
                "0.25" => (1, 4),
                _ => unreachable!(),
            };
            F64::from_i64(num, rz).div(F64::from_i64(den, rz), rz)
        };
        Ratios {
            yellow: d(yellow),
            red: d(red),
            mode: rz,
        }
    }

    /// spec: S-0111 claims 4 and 5, scenario OV4 (colours switch at
    /// equality; 2 segments at 30 of 170, not 3; 1 segment at health 1)
    #[test]
    fn segments_and_colours_follow_ov4() {
        let r = ratios("0.5", "0.25");
        let c = |h| health_colour(h, 160, false, &r);
        assert_eq!(
            [c(81), c(80), c(41), c(40)],
            [
                HealthColour::Green,
                HealthColour::Yellow,
                HealthColour::Yellow,
                HealthColour::Red
            ]
        );
        let b = |h| health_colour(h, 160, true, &r);
        assert_eq!([b(81), b(80), b(41), b(40)], [c(81), c(80), c(41), c(40)]);
        let rz = Rounding::TowardZero;
        assert_eq!(filled_segments(30, 170, 17, rz), 2);
        assert_eq!(filled_segments(1, 170, 17, rz), 1);
        assert_eq!(filled_segments(170, 170, 17, rz), 17);
        assert_eq!(filled_segments(169, 170, 17, rz), 16);
        // F1: every k/17 with 2 ≤ k ≤ 16 shows one segment fewer.
        for k in 2..=16 {
            assert_eq!(filled_segments(k * 10, 170, 17, rz), k - 1);
        }
        // n = 8: no such case.
        assert_eq!(filled_segments(85, 170, 8, rz), 4);
    }

    /// spec: S-0111 claim 5 (full strength 0 or below: one segment; a
    /// building at health 0 is green, a unit red)
    #[test]
    fn bad_strength_and_zero_health() {
        let r = ratios("0.5", "0.25");
        let rz = Rounding::TowardZero;
        assert_eq!(filled_segments(50, 0, 17, rz), 1);
        assert_eq!(filled_segments(0, 0, 17, rz), 1);
        assert_eq!(filled_segments(50, -10, 17, rz), 1);
        assert_eq!(health_colour(5, 0, false, &r), HealthColour::Green);
        assert_eq!(health_colour(0, 0, false, &r), HealthColour::Red);
        assert_eq!(health_colour(0, 0, true, &r), HealthColour::Green);
        assert_eq!(health_colour(0, 100, true, &r), HealthColour::Green);
        assert_eq!(health_colour(0, 100, false, &r), HealthColour::Red);
    }

    /// spec: S-0111 claim 6 (segment and bracket places by kind)
    #[test]
    fn unit_bar_places() {
        let r = ratios("0.5", "0.25");
        let v = unit_bar(false, 3, 100, 100, true, &r);
        assert_eq!(v.len(), 18);
        assert_eq!(v[0], Pip::shaded(Sprite::Bracket, 0, (1, -23)));
        assert_eq!(v[1].at, (-15, -22));
        assert_eq!(v[17].at, (17, -22));
        assert!(v[1..].iter().all(|p| p.frame == 16));
        let i = unit_bar(true, 0, 10, 100, false, &r);
        // Hovered: no bracket; 8 × 0.1 → 0, at least 1, red.
        assert_eq!(i, [Pip::image(Sprite::Pips, 18, (-5, -24))]);
        let sel = unit_bar(true, 0, 60, 100, true, &r);
        assert_eq!(sel[0].frame, 1);
        assert_eq!(sel[0].at, (11, -25));
    }

    /// spec: S-0111 claim 7 and scenario OV5 (pip counts 7, 14, 29, 15, 29
    /// and 30; Q = (−60, −15e) for a 2 × 2 foundation; the last pip 7 right
    /// of and 2 below Q)
    #[test]
    fn building_bar_geometry() {
        let r = ratios("0.5", "0.25");
        for ((w, d), n) in [
            ((1, 1), 7),
            ((1, 2), 14),
            ((1, 4), 29),
            ((2, 2), 15),
            ((3, 4), 29),
            ((4, 4), 30),
        ] {
            assert_eq!(building_bar(w, d, 2, 10, 10, &r).len(), n, "{w} × {d}");
        }
        for e in [0, 1, 2, 5] {
            let q = displacement(-256, 256, e * LEVEL_HEIGHT);
            assert_eq!(q, (-60, -15 * e), "e = {e}");
            let v = building_bar(2, 2, e, 10, 10, &r);
            assert_eq!(v.last().unwrap().at, (q.0 + 7, q.1 + 2));
            assert_eq!(v[1].at.0 - v[0].at.0, -4);
            assert_eq!(v[1].at.1 - v[0].at.1, 2);
        }
        // Filled first in the colour, then empty frame 0: half health of 15 → 7.
        let v = building_bar(2, 2, 2, 50, 100, &r);
        assert_eq!(v.iter().filter(|p| p.frame == 2).count(), 7);
        assert!(v[7..].iter().all(|p| p.frame == 0));
    }

    /// spec: S-0111 claim 18 (bands, image frames and places)
    #[test]
    fn insignia_bands() {
        let rz = Rounding::TowardZero;
        let v = |num: i64, den: i64| {
            F64::from_i64(num, rz)
                .div(F64::from_i64(den, rz), rz)
                .to_f32(rz)
        };
        assert_eq!(
            insignia(v(-1, 2), true).map(|p| (p.frame, p.at)),
            Some((19, (5, 2)))
        );
        assert_eq!(insignia(v(1, 2), false), None);
        assert_eq!(
            insignia(v(3, 2), false).map(|p| (p.frame, p.at)),
            Some((14, (10, 6)))
        );
        assert_eq!(insignia(v(5, 2), false).map(|p| p.frame), Some(15));
        assert_eq!(insignia(F32::ZERO, false), None);
    }

    /// spec: S-0111 claim 17 and scenario OV12 (groups 1, 5 and 10 of the
    /// keyboard show 1, 5 and 0; the number at (−14, −26) from a vehicle's
    /// anchor, (−14, −29) from an infantry unit's, and (−4, −36) from a
    /// building's pip-scale origin), claim 10 (o = (−10, +10), and for a
    /// building P(−α, β, 0), the box's left corner at its base)
    #[test]
    fn group_number_digits_and_places() {
        // Our keys: group k of the keyboard at index k, the tenth (key 0) at index 0.
        assert_eq!(group_digit(1), 1);
        assert_eq!(group_digit(5), 5);
        assert_eq!(group_digit(0), 0);
        assert_eq!(
            (0..10).map(group_digit).collect::<Vec<_>>(),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
        assert_eq!(pip_origin(None), (-10, 10));
        assert_eq!(group_number_at(false, None), (-14, -26));
        assert_eq!(group_number_at(true, None), (-14, -29));
        // Buildings: o = (−15(W + D), trunc(15(D − W) / 2)) for whole cells, then (−4, −36).
        assert_eq!(pip_origin(Some((2, 2))), (-60, 0));
        assert_eq!(group_number_at(false, Some((2, 2))), (-64, -36));
        assert_eq!(pip_origin(Some((1, 2))), (-45, 7));
        assert_eq!(group_number_at(false, Some((1, 2))), (-49, -29));
        assert_eq!(pip_origin(Some((3, 1))), (-60, -15));
        assert_eq!(group_number_at(false, Some((3, 1))), (-64, -51));
        for (w, d) in [(1, 1), (1, 4), (3, 4), (4, 2), (5, 5)] {
            assert_eq!(
                pip_origin(Some((w, d))),
                (-15 * (w + d), 15 * (d - w) / 2),
                "{w} × {d}"
            );
        }
        // A building's place does not depend on the infantry flag.
        assert_eq!(group_number_at(true, Some((2, 2))), (-64, -36));
    }

    /// The pixels of a group number, by place from the placement point.
    fn number_pixels(mask: &Mask, pitch: i64) -> BTreeMap<(i32, i32), u8> {
        let Some((img, (ox, oy))) = group_number_image(mask, pitch) else {
            return BTreeMap::new();
        };
        let w = img.width() as i32;
        img.pixels()
            .iter()
            .enumerate()
            .filter(|(_, p)| **p != 0)
            .map(|(i, p)| ((ox + i as i32 % w, oy + i as i32 / w), *p))
            .collect()
    }

    /// spec: S-0111 claim 17, F6 and scenario OV12 a (the glyph of the game font, left-aligned
    /// with its top stored row at d; the box from (−3, +2), (G + 6) × (P_f − 4), black inside a
    /// one-pixel ring of the house colour; glyph pixels in the house colour over everything; rows
    /// above the box paint over the scene; nothing outside the box and the mask)
    #[test]
    fn claim_17_glyph_and_box_follow_f6() {
        let font = GroupFont::new(&digit_font(7, 10)).unwrap();
        let one = font.mask(1);
        assert_eq!((one.width, one.height), (2, 7), "G = 2, H_f = 7");
        let px = number_pixels(one, font.pitch());
        // Box: −3 ≤ x ≤ G + 2 = 4, 2 ≤ y ≤ P_f − 3 = 7.
        for y in 0..7 {
            for x in -3..=4 {
                let inside_box = y >= 2;
                let border = inside_box && (x == -3 || x == 4 || y == 2 || y == 7);
                let ink = (0..2).contains(&x) && one.get(x as usize, y as usize);
                let want = if ink || border {
                    Some(GROUP_INK)
                } else if inside_box {
                    Some(GROUP_FILL)
                } else {
                    None
                };
                assert_eq!(px.get(&(x, y)).copied(), want, "({x}, {y})");
            }
        }
        for x in -3..=4 {
            assert_eq!(px[&(x, 7)], GROUP_INK, "bottom ring at y = P_f − 3");
            assert!(!px.contains_key(&(x, 8)), "nothing below the box");
        }
        // The glyph's top rows paint above the box, over the scene.
        assert_eq!(px[&(0, 0)], GROUP_INK);
        assert_eq!(px[&(1, 1)], GROUP_INK);
        assert!(!px.contains_key(&(-1, 0)));
        assert!(
            !px.contains_key(&(2, 0)),
            "clear of the glyph above the box"
        );
        // The fill shows between glyph and border: the gap follows the font's data.
        assert_eq!(px[&(-2, 4)], GROUP_FILL);
        assert_eq!(px[&(0, 4)], GROUP_FILL, "a clear mask pixel keeps the fill");
        assert_eq!(px[&(1, 4)], GROUP_INK);
        // Every digit keeps its own width: the box behind 5 is wider by ΔG = 2.
        let width = |d: u8| {
            let px = number_pixels(font.mask(d), font.pitch());
            let xs: Vec<i32> = px.keys().filter(|p| p.1 == 3).map(|p| p.0).collect();
            xs.iter().max().unwrap() - xs.iter().min().unwrap() + 1
        };
        assert_eq!(width(1), 2 + 6);
        assert_eq!(width(5), 4 + 6);
        assert_eq!(width(5) - width(1), 2);
    }

    /// spec: S-0111 claim 17 (glyph rows: every stored row paints, 0 ≤ r < H_f, even when
    /// P_f < H_f, unlike status text) and F6, scenario OV12 g 2
    #[test]
    fn claim_17_rows_past_the_pitch_paint() {
        let font = GroupFont::new(&digit_font(12, 8)).unwrap();
        let px = number_pixels(font.mask(5), font.pitch());
        // The box ends at y = P_f − 3 = 5; the column of 5 still paints rows 6 to 11.
        for r in 0..12 {
            assert_eq!(px.get(&(0, r)), Some(&GROUP_INK), "row {r}");
        }
        assert_eq!(px.get(&(1, 6)), None, "below the box only the glyph");
        assert_eq!(px.get(&(-3, 5)), Some(&GROUP_INK));
        assert_eq!(px.get(&(-3, 6)), None);
        assert!(!px.contains_key(&(0, 12)));
    }

    /// spec: S-0111 claim 17 and F6 (when P_f − 4 ≤ 0 the digit is still drawn and our engine
    /// draws no box; scenario OV12 g 3)
    #[test]
    fn claim_17_no_box_at_a_small_pitch() {
        for p in [0u32, 3, 4] {
            let font = GroupFont::new(&digit_font(7, p)).unwrap();
            let px = number_pixels(font.mask(1), font.pitch());
            assert!(px.values().all(|&i| i == GROUP_INK), "P_f = {p}: no fill");
            assert_eq!(px.keys().map(|k| k.0).min(), Some(0), "no border at −3");
            assert_eq!(px[&(0, 0)], GROUP_INK);
        }
        let font = GroupFont::new(&digit_font(7, 5)).unwrap();
        let px = number_pixels(font.mask(1), font.pitch());
        assert_eq!(px[&(-3, 2)], GROUP_INK, "P_f = 5: a one-row box");
        assert!(!px.contains_key(&(-3, 3)));
    }

    /// spec: S-0111 claim 17 (a digit with no mapped glyph uses S-0213 claim 25's fallback, the
    /// complement of the capital X's mask with X's width, so the box takes X's width; our engine
    /// paints it in the house colour) and scenario OV12 g 4; S-0014 claim 6 (a font without a
    /// mapped capital X is rejected whole: no number)
    #[test]
    fn claim_17_a_missing_digit_takes_the_fallback() {
        let raw = digit_font(7, 10);
        let font = GroupFont::new(&raw).unwrap();
        let zero = font.mask(0);
        assert_eq!(zero.width, 3, "X's stored width");
        assert_eq!(*zero, raw.fallback().unwrap());
        assert!(!zero.get(0, 0) && zero.get(1, 0) && zero.get(0, 1) && !zero.get(1, 1));
        let px = number_pixels(zero, font.pitch());
        assert_eq!(px[&(1, 0)], GROUP_INK);
        assert_eq!(px[&(5, 4)], GROUP_INK, "border at G + 2 = 5");
        assert_eq!(px[&(4, 4)], GROUP_FILL);
        let mut no_x = raw;
        no_x.map_unit(88, None);
        assert_eq!(GroupFont::new(&no_x), None);
    }

    /// Our placeholder lifetimes and burst sizes.
    #[test]
    fn transients_age_out() {
        let t = Transient {
            kind: TransientKind::Burst(burst_half_width(192)),
            at: (0, 0),
            born: 10,
        };
        assert_eq!(burst_half_width(0), 4);
        assert_eq!(burst_half_width(192), 32);
        assert_eq!(burst_half_width(100_000), 96);
        assert!(matches!(t.marker(10), Some(Marker::Burst { stage: 0, .. })));
        assert!(matches!(t.marker(17), Some(Marker::Burst { stage: 3, .. })));
        assert_eq!(t.marker(18), None);
        assert_eq!(t.marker(9), None);
    }
}
