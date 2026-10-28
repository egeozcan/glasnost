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
//! What S-0111 asks that is not drawn yet: the culling of claim 1 (the
//! scene has no on-screen culling, S-0101 claim 6), the shading of claim 2
//! (no shroud or fog), the corner brackets of selected buildings (claim 8:
//! which corners are drawn is not established), the pip scale's rows
//! (claims 11 to 15) and the self-heal marker (claim 16). Of the pip scale
//! only its origin (claim 10, [`pip_origin`]), who sees it (claim 9) and the
//! control-group number (claim 17, [`group_number_at`],
//! [`Cache::group_number_item`]) are drawn. The bomb icon of S-0519 claim 24
//! is drawn: the first status extra of its carrier, through the cursor
//! palette ([`Cache::bomb_item`], [`CURSOR_PALETTE`]; S-0111 claim 19a),
//! and so is the repair wrench of claim 19 ([`Cache::wrench_item`]).

use std::collections::BTreeMap;

use glasnost_fmt::image::{IndexedImage, Rgb8};
use glasnost_fmt::pal::Palette6;
use glasnost_fmt::shp::Shp;
use glasnost_num::{F32, F64, Rounding};
use glasnost_render::compose::{DepthTest, Paint, Profile, pack565, row565};
use glasnost_render::palette::{PaletteQuirks, palette_from_file};
use glasnost_render::projection::height_lift;
use glasnost_render::scene::{Group, Item, ItemDepth, Scene};

use super::build::Assets;

/// The pip sprite (S-0111 Import mapping).
pub const PIP_SPRITE: &[u8] = b"PIPS.SHP"; // spec: S-0111 Import mapping (overlay sprites)
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
            Sprite::Bracket => BRACKET_SPRITE,
            Sprite::Bomb => BOMB_SPRITE,
            Sprite::Wrench => WRENCH_SPRITE,
        }
    }
}

/// One overlay image: a sprite's image frame drawn as a centred image at a
/// displacement from the status anchor (S-0111 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pip {
    /// Sprite.
    pub sprite: Sprite,
    /// Image frame.
    pub frame: usize,
    /// Displacement from the status anchor, screen pixels.
    pub at: (i32, i32),
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
        out.push(Pip {
            sprite: Sprite::Bracket,
            frame: bracket_frame,
            at: bracket_at,
        });
    }
    // spec: S-0111 claim 6 (pip sprite image frames 16 green, 17 yellow, 18 red)
    let frame = match health_colour(health, strength, false, r) {
        HealthColour::Green => 16,
        HealthColour::Yellow => 17,
        HealthColour::Red => 18,
    };
    let f = filled_segments(health, strength, n, r.mode);
    // spec: S-0111 claim 6 (segment i at x0 + 2i; only the filled segments are drawn)
    out.extend((0..f).map(|i| Pip {
        sprite: Sprite::Pips,
        frame,
        at: (x0 + 2 * i, y),
    }));
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
        .map(|i| Pip {
            sprite: Sprite::Pips,
            frame: if i < f { frame } else { 0 },
            // spec: S-0111 claim 7 (pip i centred at Q + (4(n − i) + 3, 2(i − n) + 4), top right first)
            at: (q.0 + 4 * (n - i) + 3, q.1 + 2 * (i - n) + 4),
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
    Some(Pip {
        sprite: Sprite::Pips,
        frame,
        // spec: S-0111 claim 18 (at (5, 2) for infantry, (10, 6) for everything else)
        at: if infantry { (5, 2) } else { (10, 6) },
    })
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

/// Our digit glyphs, 3 × 5 pixels, one row of three bits per line (top
/// first, the high bit leftmost).
// design: our own digit glyphs for the control-group number (S-0111 claim 17 leaves the font open; Q-2040)
const DIGITS: [[u8; 5]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111],
    [0b010, 0b110, 0b010, 0b010, 0b111],
    [0b111, 0b001, 0b111, 0b100, 0b111],
    [0b111, 0b001, 0b011, 0b001, 0b111],
    [0b101, 0b101, 0b111, 0b001, 0b001],
    [0b111, 0b100, 0b111, 0b001, 0b111],
    [0b111, 0b100, 0b111, 0b101, 0b111],
    [0b111, 0b001, 0b010, 0b010, 0b010],
    [0b111, 0b101, 0b111, 0b101, 0b111],
    [0b111, 0b101, 0b111, 0b001, 0b111],
];

/// The size of the group number's image: the glyph, a one-pixel gap and a
/// one-pixel border round it.
// design: a 1-pixel border 1 pixel clear of the 3 × 5 glyph (S-0111 claim 17 leaves the border's size open; Q-2040)
pub const GROUP_NUMBER_SIZE: (u32, u32) = (7, 9);

/// The group number's image for `digit` (S-0111 claim 17: the digit inside
/// a small border of the same colour), every drawn pixel index 1, the rest
/// 0 (transparent). The image's top-left corner goes at the placement point.
// design: the image's top-left corner at claim 17's placement point (the alignment is open; Q-2040)
pub fn group_number_image(digit: u8) -> IndexedImage {
    let (w, h) = GROUP_NUMBER_SIZE;
    let glyph = DIGITS[usize::from(digit % 10)];
    let mut px = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let border = x == 0 || y == 0 || x == w - 1 || y == h - 1;
            let ink = (2..5).contains(&x)
                && (2..7).contains(&y)
                && glyph[(y - 2) as usize] & (0b100 >> (x - 2)) != 0;
            if border || ink {
                px[(y * w + x) as usize] = 1;
            }
        }
    }
    IndexedImage::new(w, h, px).expect("size matches")
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
    group_digits: BTreeMap<u8, u32>,
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
        let (image, (fx, fy), (cw, ch)) = self.frame(scene, assets, Sprite::Bomb, frame)?;
        Some(Item {
            image,
            // spec: S-0519 claim 24, S-0102 claim 19 (centred placement on the draw point, no further displacement)
            x: at.0 - cw / 2 + fx,
            y: at.1 - ch / 2 + fy,
            // spec: S-0519 claim 24 (a status extra: after the object layers, with no depth test)
            test: DepthTest::Ignore,
            depth: flat(),
            paint: Paint::Row(row),
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
        let (image, (fx, fy), (cw, ch)) = self.frame(scene, assets, Sprite::Wrench, frame)?;
        Some(Item {
            image,
            // spec: S-0111 claim 1 and Terms (placed from the status anchor, a centred image; claim 19 gives the wrench no displacement)
            x: anchor.0 - cw / 2 + fx,
            y: anchor.1 - ch / 2 + fy,
            // spec: S-0102 claim 1 (a status extra: after the object layers, with no depth test)
            test: DepthTest::Ignore,
            depth: flat(),
            paint: Paint::Row(row),
            group: Group::Status,
            provisional: false,
            object,
            stack: None,
        })
    }

    /// The control-group number's item (S-0111 claim 17): `digit` in the
    /// house colour `colour`, placed at `at` from the status anchor
    /// `anchor`. Unshaded, as the pips are (claim 2).
    pub fn group_number_item(
        &mut self,
        scene: &mut Scene,
        anchor: (i32, i32),
        at: (i32, i32),
        digit: u8,
        colour: [u8; 3],
        object: Option<u32>,
    ) -> Item {
        let row = *self.group_rows.entry(colour).or_insert_with(|| {
            let [r, g, b] = colour;
            let mut pal = [Rgb8 { r: 0, g: 0, b: 0 }; 256];
            // spec: S-0111 claim 17 (the digit and its border in the owner's house colour)
            pal[1] = Rgb8 { r, g, b };
            scene.add_row(row565(&pal))
        });
        let image = *self
            .group_digits
            .entry(digit)
            .or_insert_with(|| scene.add_image(group_number_image(digit)));
        Item {
            image,
            // design: the image's top-left corner at claim 17's placement point (Q-2040)
            x: anchor.0 + at.0,
            y: anchor.1 + at.1,
            // spec: S-0102 claim 1 (a status extra: after the object layers, with no depth test)
            test: DepthTest::Ignore,
            depth: flat(),
            // design: unshaded like the pips (S-0111 claim 2 names the shaded overlay images; the group number is not among them; Q-2040)
            paint: Paint::Row(row),
            group: Group::Status,
            provisional: false,
            object,
            stack: None,
        }
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
                    paint: Paint::Row(row),
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
        assert_eq!(
            v[0],
            Pip {
                sprite: Sprite::Bracket,
                frame: 0,
                at: (1, -23)
            }
        );
        assert_eq!(v[1].at, (-15, -22));
        assert_eq!(v[17].at, (17, -22));
        assert!(v[1..].iter().all(|p| p.frame == 16));
        let i = unit_bar(true, 0, 10, 100, false, &r);
        // Hovered: no bracket; 8 × 0.1 → 0, at least 1, red.
        assert_eq!(
            i,
            [Pip {
                sprite: Sprite::Pips,
                frame: 18,
                at: (-5, -24)
            }]
        );
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

    /// spec: S-0111 claim 17 (the digit inside a small border of the same
    /// colour); the glyphs and the border's size are our design (Q-2040)
    #[test]
    fn group_number_images() {
        let (w, h) = GROUP_NUMBER_SIZE;
        let mut seen = std::collections::BTreeSet::new();
        for d in 0..10u8 {
            let img = group_number_image(d);
            assert_eq!((img.width(), img.height()), (w, h));
            let px = img.pixels();
            // The border is drawn all round, the gap inside it is clear.
            for x in 0..w {
                assert_eq!(px[x as usize], 1);
                assert_eq!(px[((h - 1) * w + x) as usize], 1);
            }
            for y in 0..h {
                assert_eq!(px[(y * w) as usize], 1);
                assert_eq!(px[(y * w + w - 1) as usize], 1);
            }
            for x in 1..w - 1 {
                assert_eq!(px[(w + x) as usize], 0);
                assert_eq!(px[((h - 2) * w + x) as usize], 0);
            }
            assert!(px.iter().all(|&p| p <= 1));
            assert!(seen.insert(px.to_vec()), "digit {d} has its own glyph");
        }
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
