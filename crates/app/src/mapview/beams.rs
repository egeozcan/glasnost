//! Line effects of the battlefield view: beams (S-0518 claims 17 to 23),
//! beam flashes (S-0514 claims 23 to 26) and the magnetic wave's picture
//! (S-0607 claims 25 and 26). Each becomes single-pixel drawings
//! ([`glasnost_render::spots`]) that the scene places in its draw order.
//!
//! All of this is presentation: nothing here feeds back into the game.
//! Beam spreads draw from the view's cosmetic stream (S-0518 claim 21: the
//! unsynchronised stream).
//!
//! Interim rules where the specs leave a case open (docs/development.md,
//! *Visual review 2026-10-19*):
//! - lines are computed unclipped and the view clips their pixels (S-0518
//!   Open questions: clipping; S-0514 F8's clipped-end bias is not applied);
//! - a blended line's interior bias steps as F9's do along its long axis,
//!   and its coverage follows F8 whatever its bias difference (S-0518 Open
//!   questions); on a slope of exactly one the long axis is x; on an
//!   axis-aligned blended line the zero-coverage second pixel lies toward
//!   increasing x or y, and is blended with weight 0 like any other;
//! - a beam whose ends share their plan position takes sector 0 (S-0518
//!   claim 20 asserts no half-way direction);
//! - a beam flash segment covers the nearest-integer 8-connected line with
//!   both ends (S-0514 Open questions), uses its leftmost end's unclipped
//!   bias, and the segment length is the full-rate 10 leptons (S-0514 F6's
//!   frame-rate term *p* is open);
//! - the magnetic wave covers the pixels whose centres lie inside or on its
//!   band (S-0607 claim 25: the fill rule is unmeasured); its ends follow
//!   the positions at the end of the tick;
//! - one frame is drawn per tick: the activity test's clock advances by the
//!   tick's paced period, and a flash's frame *k* is its age in ticks.

use glasnost_num::soft::{F32, F64, Rounding};
use glasnost_render::projection::{height_lift, project_lifted};
use glasnost_render::spots::{Spot, SpotKind, easing, pack};
use glasnost_sim::world::Pos;
use std::cmp::Ordering;

// ---------------------------------------------------------------------------
// Beam parameters

/// Intensity 1 as a single (S-0518 claim 18: S = 1).
pub const ONE: F32 = F32(0x3F80_0000); // spec: S-0518 claim 18 (S = 1)
/// Intensity 0 as a single (S-0518 claim 18: E = 0).
pub const NONE: F32 = F32::ZERO;
/// Intensity ½ as a single (S-0615 claim 15: the arc beams' fade parameter *E* = 0.5).
pub const HALF: F32 = F32(0x3F00_0000); // spec: S-0615 claim 15 (revision 4: fade parameters S = 1, E = 0.5, S-0518 claim 19 and F6)

/// The house laser colour *L* of a house display colour (S-0518 claim 17,
/// F5). The arithmetic is double precision until the final truncation.
pub fn laser_colour(rgb: [u8; 3]) -> [u8; 3] {
    let [r, g, b] = rgb.map(f64::from);
    // spec: S-0518 F5 (n = √(r² + g² + b²); n = 0 gives (255, 255, 255))
    let n = (r * r + g * g + b * b).sqrt();
    if n == 0.0 {
        return [255, 255, 255]; // spec: S-0518 F5 (n = 0: white)
    }
    // spec: S-0518 F5 (c′ = min(255, c × 240 / n); c′ = 0 if c′ < 96)
    let first = [r, g, b].map(|c| {
        let v = (c * 240.0 / n).min(255.0);
        if v < 96.0 { 0.0 } else { v }
    });
    let [r1, g1, b1] = first;
    // spec: S-0518 F5 (n′ = √(r′² + g′² + b′²); should it be 0, white)
    let n1 = (r1 * r1 + g1 * g1 + b1 * b1).sqrt();
    if n1 == 0.0 {
        return [255, 255, 255];
    }
    // spec: S-0518 F5 (c″ = min(255, c′ × 240 / n′), truncated toward zero)
    first.map(|c| (c * 240.0 / n1).min(255.0) as u8)
}

/// Each channel halved, rounding down (S-0518 claims 18 and 22).
pub fn halved(c: [u8; 3]) -> [u8; 3] {
    c.map(|v| v / 2)
}

/// A colour of three integer channels held as bytes: values outside
/// 0 … 255 are clamped (our interim: S-0514 Open questions leaves how
/// out-of-range colour values are stored open).
pub fn channels(c: [i32; 3]) -> [u8; 3] {
    c.map(|v| v.clamp(0, 255) as u8)
}

/// What a beam looks like (S-0518 claim 18 and Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    /// Without the house-colour flag: ordinary drawing (claim 21).
    Ordinary {
        /// Inner colour.
        inner: [u8; 3],
        /// Outer colour; black means no outer lines.
        outer: [u8; 3],
        /// Spread colour σ.
        spread: [u8; 3],
        /// The blink flag.
        blink: bool,
    },
    /// With the house-colour flag: house-colour drawing (claim 22).
    House {
        /// The inner colour *L*.
        colour: [u8; 3],
        /// Thickness *W*.
        width: i32,
        /// The supported flag.
        supported: bool,
    },
}

/// What made a weapon's beam (S-0518 claim 18).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// A prism master's shot with support count *N*.
    Prism(i32),
    /// Another building's laser weapon.
    Building,
    /// A vehicle's, infantry unit's or aircraft's laser weapon.
    Unit,
    /// A shrapnel fragment's beam (S-0318 claim 8), whatever the attacker.
    Fragment,
}

/// The look of a laser weapon's beam (S-0518 claim 18): `colours` the
/// weapon's inner, outer and spread colours, `house` its house-colour flag
/// and `l` the firer's house laser colour.
pub fn weapon_look(source: Source, colours: [[u8; 3]; 3], house: bool, l: [u8; 3]) -> Look {
    let [inner, outer, spread] = colours;
    if !house {
        // spec: S-0518 claim 18 (without the flag, the weapon's three laser colours; ordinary drawing)
        // spec: S-0518 claim 18 (thickness and the supported flag show only in the house-colour drawing)
        return Look::Ordinary {
            inner,
            outer,
            spread,
            blink: false,
        };
    }
    // spec: S-0518 claim 18 (W and the supported flag by source)
    let (width, supported) = match source {
        Source::Prism(0) => (3, false),
        Source::Prism(_) => (5, true),
        Source::Building => (1, false),
        Source::Unit => (2, false),
        // spec: S-0318 claim 8 (a fragment's beam has thickness 1 whatever the attacker's type)
        Source::Fragment => (1, false),
    };
    // spec: S-0518 claim 18 (with the flag the inner colour is the firer's L; the house-colour drawing reads only it)
    Look::House {
        colour: l,
        width,
        supported,
    }
}

/// The support beam's look (S-0518 claim 18: inner *L* of the supporter's
/// house, house-colour flag on, *W* = 3, not supported).
pub fn support_look(l: [u8; 3]) -> Look {
    Look::House {
        colour: l,
        width: 3, // spec: S-0518 claim 18 (support beam W = 3)
        supported: false,
    }
}

/// Which line effect a shot draws (S-0514 claim 23).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineEffect {
    /// A laser beam (S-0518).
    Laser,
    /// An electric bolt (S-0316: drawn from its own report,
    /// `Effect::Bolt`, by `super::bolts`).
    Bolt,
    /// A beam flash (S-0514 claims 23 to 26).
    Flash,
    /// None.
    None,
}

/// The line effect of a weapon with the laser, electric-bolt and
/// radiation-beam flags (S-0514 claim 23: laser first, then bolt, then
/// radiation beam).
pub fn line_effect(laser: bool, bolt: bool, rad: bool) -> LineEffect {
    // spec: S-0514 claim 23 (the laser flag wins first, then the electric-bolt flag, then the radiation-beam flag)
    if laser {
        LineEffect::Laser
    } else if bolt {
        LineEffect::Bolt
    } else if rad {
        LineEffect::Flash
    } else {
        LineEffect::None
    }
}

/// A beam (S-0518 Terms): fixed screen ends and depth biases, a sector, a
/// life in ticks and a fade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Beam {
    /// World pixel of the source end.
    pub from: (i32, i32),
    /// World pixel of the target end.
    pub to: (i32, i32),
    /// Depth biases *b*₀ (source) and *b*₁ (target).
    pub bias: (i32, i32),
    /// Sector *s*.
    pub sector: u8,
    /// Tick of creation (*P* = 0).
    pub born: i32,
    /// Duration *D* in ticks.
    pub duration: i32,
    /// Start intensity *S*.
    pub start: F32,
    /// End intensity *E*.
    pub end: F32,
    /// The fade flag.
    pub fade: bool,
    /// Its look.
    pub look: Look,
}

/// A railgun system's laser beam (S-0616 claim 31, S-0518 claim 18's
/// "Other specs"): from the system's position to its target point, inner
/// colour the type's laser colour, outer and spread black, 10 ticks, not
/// blinking, fading from 0.5 to 0; the Terms defaults leave the
/// house-colour flag off, so it takes the ordinary drawing (claim 21).
pub fn rail_beam(from: Pos, to: Pos, born: i32, inner: [u8; 3]) -> Beam {
    let look = Look::Ordinary {
        inner,
        // spec: S-0616 claim 31 (outer colour and spread black)
        outer: [0; 3],
        spread: [0; 3],
        // spec: S-0616 claim 31 (not blinking)
        blink: false,
    };
    // spec: S-0616 claim 31 (lasting 10 ticks, fading to 0); S-0518 claim 20 (not a building: a = 0)
    let mut b = Beam::new(from, to, 0, born, 10, NONE, look);
    // spec: S-0616 claim 31 (fading from intensity 0.5)
    b.start = HALF;
    b
}

impl Beam {
    /// A beam between world points `from` and `to` (S-0518 claim 20):
    /// projected with their height lift, with the biases
    /// *b*₀ = *a* − *L*(*z*₀) − 2 and *b*₁ = −*L*(*z*₁) − 2. It fades from 1
    /// to `end` over `duration` ticks from tick `born` (claims 18 and 19).
    pub fn new(from: Pos, to: Pos, a: i32, born: i32, duration: i32, end: F32, look: Look) -> Self {
        Self {
            from: project_lifted(from.x, from.y, from.z),
            to: project_lifted(to.x, to.y, to.z),
            // spec: S-0518 claim 20 (b₀ = a − L(z₀) − 2, b₁ = −L(z₁) − 2)
            bias: (
                a - height_lift(from.z) - NEARNESS,
                -height_lift(to.z) - NEARNESS,
            ),
            sector: sector(
                i64::from(to.x) - i64::from(from.x),
                i64::from(to.y) - i64::from(from.y),
            ),
            born,
            duration,
            // spec: S-0518 claim 18 (every source fades from S = 1)
            start: ONE,
            end,
            // spec: S-0518 claim 18 (every source uses the fade flag and leaves the blink flag off)
            fade: true,
            look,
        }
    }

    /// The age *P* at tick `now` while the beam is drawn (S-0518 claim 19:
    /// *P* = 0 … *D* − 1; a *D* ≤ 0 is never drawn).
    pub fn age(&self, now: i32) -> Option<i32> {
        let p = now - self.born;
        (0..self.duration).contains(&p).then_some(p)
    }
}

/// The beam depth nearness (S-0518 Constants: 2 depth units).
const NEARNESS: i32 = 2; // spec: S-0518 Constants (beam depth nearness 2)

/// The bias term *a* of a building's weapon beam (S-0518 claim 20):
/// min(0, *y*ₛ − *y*ₒ), *y*ₛ the screen row of the fire point and *y*ₒ that
/// of the point 128 leptons from the building's position toward decreasing
/// X and decreasing Y.
pub fn building_nearness(fire_point: Pos, building: Pos) -> i32 {
    let ys = project_lifted(fire_point.x, fire_point.y, fire_point.z).1;
    // spec: S-0518 claim 20 (the point 128 leptons toward decreasing X and decreasing Y)
    let o = Pos::new(building.x - 128, building.y - 128, building.z);
    let yo = project_lifted(o.x, o.y, o.z).1;
    // spec: S-0518 claim 20 (a = min(0, yₛ − yₒ))
    (ys - yo).min(0)
}

/// The beam's intensity *I* at age `p` (S-0518 claim 19, F6), in single
/// precision with the game's rounding.
pub fn intensity(b: &Beam, p: i32, mode: Rounding) -> F32 {
    if !b.fade {
        return ONE; // spec: S-0518 F6 (I = 1 without the fade flag)
    }
    let single = |x: F64| x.to_f32(mode).to_f64();
    let (s, e) = (b.start.to_f64(), b.end.to_f64());
    // spec: S-0518 F6 (I = E + (S − E) × (D − P) / D, in single precision)
    let diff = single(s.sub(e, mode));
    let left = single(F64::from_i64(i64::from(b.duration - p), mode));
    let d = single(F64::from_i64(i64::from(b.duration), mode));
    let t = single(diff.mul(left, mode));
    let t = single(t.div(d, mode));
    e.add(t, mode).to_f32(mode)
}

/// trunc(`v` × `i`) with the single `i` widened to double (S-0518 claim 22,
/// claim 23, F8).
fn scaled(v: i64, i: F32, mode: Rounding) -> i64 {
    F64::from_i64(v, mode)
        .mul(i.to_f64(), mode)
        .trunc_i128()
        .map_or(0, |x| x as i64)
}

// ---------------------------------------------------------------------------
// Geometry

/// a√2 compared with b, exactly.
fn cmp_root2(a: i64, b: i64) -> Ordering {
    let (a, b) = (i128::from(a), i128::from(b));
    match (a.signum(), b.signum()) {
        (0, 0) => Ordering::Equal,
        (sa, sb) if sa >= 0 && sb <= 0 => Ordering::Greater,
        (sa, sb) if sa <= 0 && sb >= 0 => Ordering::Less,
        // Both positive: 2a² against b²; both negative: reversed.
        (1, _) => (2 * a * a).cmp(&(b * b)),
        _ => (b * b).cmp(&(2 * a * a)),
    }
}

/// The sector *s* of a plan direction (`dx`, `dy`) in world leptons
/// (S-0518 claim 20): the nearest eighth of a turn, clockwise from north
/// (decreasing Y), so 0 is north and 2 is east. A zero direction takes 0.
pub fn sector(dx: i64, dy: i64) -> u8 {
    // spec: S-0518 claim 20 (eight directions clockwise from world north, decreasing Y)
    const DIRS: [(i64, i64); 8] = [
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
    ];
    let mut best = 0usize;
    for s in 1..8 {
        let dot = |k: usize| dx * DIRS[k].0 + dy * DIRS[k].1;
        let (a, b) = (dot(s), dot(best));
        // Compare dot/|v| between a diagonal (|v| = √2) and an axis (|v| = 1).
        let better = match (s % 2 == 1, best % 2 == 1) {
            (false, false) | (true, true) => a > b,
            // s is a diagonal, best an axis: a/√2 > b ⇔ b√2 < a.
            (true, false) => cmp_root2(b, a) == Ordering::Less,
            (false, true) => cmp_root2(a, b) == Ordering::Greater,
        };
        if better {
            best = s;
        }
    }
    best as u8
}

/// F9's displacement of a component difference `delta` at sample `k` of
/// `h`: sign(Δ) × ⌊(2k|Δ| + H − 1) / (2H)⌋.
fn step(delta: i64, k: i64, h: i64) -> i64 {
    // spec: S-0518 F9 (sign(Δ) × ⌊(2k × |Δ| + H − 1) / (2H)⌋)
    delta.signum() * ((2 * k * delta.abs() + h - 1) / (2 * h))
}

/// The samples (screen x, screen y, bias) of an unclipped opaque or
/// additive line from `a` (the source) to `b` (S-0518 F9).
pub fn samples(a: (i32, i32, i32), b: (i32, i32, i32)) -> Vec<(i32, i32, i32)> {
    // spec: S-0518 F9 (reorder by increasing screen x, source first on equal x)
    let (p, q) = if b.0 < a.0 { (b, a) } else { (a, b) };
    let d = (
        i64::from(q.0 - p.0),
        i64::from(q.1 - p.1),
        i64::from(q.2 - p.2),
    );
    // spec: S-0518 F9 (H = max(|Δx|, |Δy|, |Δb|); H = 0 contributes no sample)
    let h = d.0.abs().max(d.1.abs()).max(d.2.abs());
    (0..h)
        .map(|k| {
            (
                p.0 + step(d.0, k, h) as i32,
                p.1 + step(d.1, k, h) as i32,
                p.2 + step(d.2, k, h) as i32,
            )
        })
        .collect()
}

/// One step of a blended line (S-0518 F8): two bracketing pixels with
/// their coverage and the bias at that step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cover {
    /// The first pixel.
    pub first: (i32, i32),
    /// The second pixel, one further along the short axis.
    pub second: (i32, i32),
    /// Coverage *q*₁ of the first pixel.
    pub q1: u32,
    /// Coverage *q*₂ of the second.
    pub q2: u32,
    /// The bias at this step (our interim: F9's steps along the long axis).
    pub bias: i32,
}

/// The coverage steps of a blended line from `a` (the source) to `b`, each
/// (screen x, screen y, bias) (S-0518 F8).
pub fn coverage(a: (i32, i32, i32), b: (i32, i32, i32)) -> Vec<Cover> {
    // spec: S-0518 F8 (the first end has the smaller screen x; source first on equal x)
    let (p, q) = if b.0 < a.0 { (b, a) } else { (a, b) };
    let (dx, dy, db) = (
        i64::from(q.0 - p.0),
        i64::from(q.1 - p.1),
        i64::from(q.2 - p.2),
    );
    // spec: S-0518 F8 (H = max(|Δx|, |Δy|); h = trunc(256 × min(|Δx|, |Δy|) / H))
    let long_x = dx.abs() >= dy.abs();
    let h_len = dx.abs().max(dy.abs());
    if h_len == 0 {
        return Vec::new();
    }
    let h = 256 * dx.abs().min(dy.abs()) / h_len;
    // On an axis-aligned line the short axis has no sign toward the other end; the
    // zero-coverage second pixel then lies toward increasing x or y (our interim).
    let toward = |d: i64| if d == 0 { 1 } else { d.signum() as i32 };
    let (sx, sy) = (toward(dx), toward(dy));
    (0..h_len)
        .map(|k| {
            // spec: S-0518 F8 (v = k × h; q₂ = 0 for v = 0, else 1 + ((v − 1) mod 256); q₁ = 256 − q₂; j = (v − q₂) / 256)
            let v = k * h;
            let q2 = if v == 0 { 0 } else { 1 + (v - 1) % 256 };
            let q1 = 256 - q2;
            let j = ((v - q2) / 256) as i32;
            let k = k as i32;
            // spec: S-0518 F8 (k pixels along the long axis and j along the short one, toward the other end; the second one further along the short axis)
            let (first, second) = if long_x {
                (
                    (p.0 + sx * k, p.1 + sy * j),
                    (p.0 + sx * k, p.1 + sy * (j + 1)),
                )
            } else {
                (
                    (p.0 + sx * j, p.1 + sy * k),
                    (p.0 + sx * (j + 1), p.1 + sy * k),
                )
            };
            Cover {
                first,
                second,
                q1: q1 as u32,
                q2: q2 as u32,
                bias: p.2 + step(db, i64::from(k), h_len) as i32,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Fine and coarse mode

/// The activity test of S-0518 claim 22: a tick counter published into *f*
/// every 60 clock units of 16 ms, and the shared low-rate state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Activity {
    /// Ticks counted since the last publication.
    counter: i32,
    /// The published count *f*.
    f: i32,
    /// The remembered low-rate state.
    low: bool,
    /// Clock reading (units) of the last publication.
    published: Option<i64>,
}

/// The activity publication window (S-0518 Constants: 60 units of 16 ms).
const WINDOW_UNITS: i64 = 60; // spec: S-0518 Constants (activity publication window 60 clock units)
/// One clock unit in host milliseconds (S-0518 claim 22).
const CLOCK_UNIT_MS: i64 = 16; // spec: S-0518 claim 22 (one clock unit is 16 ms)

impl Activity {
    /// The start of a simulation tick: the counter gains 1.
    pub fn tick(&mut self) {
        // spec: S-0518 claim 22 (a separate integer counter gains 1 at the start of every simulation tick)
        self.counter += 1;
    }

    /// The cycle-end rate check at host time `ms`.
    pub fn check(&mut self, ms: i64) {
        // spec: S-0518 claim 22 (the clock reading is the millisecond clock divided by 16, rounding down)
        let now = ms.div_euclid(CLOCK_UNIT_MS);
        // spec: S-0518 claim 22 (the first check publishes immediately; later ones once the clock advanced at least 60 units)
        if self.published.is_none_or(|p| now - p >= WINDOW_UNITS) {
            self.f = self.counter;
            self.counter = 0;
            self.published = Some(now);
        }
    }

    /// The published count *f*.
    pub fn count(&self) -> i32 {
        self.f
    }

    /// The test with floor `m` and margin `z` (S-0518 claim 22).
    pub fn test(&mut self, m: i32, z: i32) -> bool {
        if self.low {
            // spec: S-0518 claim 22 (while set, passes only when f ≥ m + z, and a pass clears it)
            let pass = self.f >= m + z;
            if pass {
                self.low = false;
            }
            pass
        } else {
            // spec: S-0518 claim 22 (while clear, passes when f ≥ m; a failure sets it)
            let pass = self.f >= m;
            if !pass {
                self.low = true;
            }
            pass
        }
    }
}

// ---------------------------------------------------------------------------
// Drawing

/// The ordinary drawing's outer-line shifts by sector (S-0518 claim 21,
/// the fixed sector 5).
const OUTER_SHIFTS: [[(i32, i32); 2]; 8] = [
    [(0, -1), (1, 0)],
    [(0, -1), (0, 1)],
    [(1, 0), (0, 1)],
    [(-1, 0), (1, 0)],
    [(-1, 0), (0, 1)],
    [(0, -1), (0, 1)],
    [(0, -1), (-1, 0)],
    [(-1, 0), (1, 0)],
]; // spec: S-0518 claim 21 (shift table, sector 5 fixed)

/// The house-colour drawing's vectors *e*ₐ and *e*ᵦ by sector (S-0518
/// claim 22).
const PAIR_VECTORS: [[(i32, i32); 2]; 8] = [
    [(-1, -1), (1, 1)],
    [(0, -1), (0, 1)],
    [(-1, 1), (1, -1)],
    [(-1, 0), (1, 0)],
    [(-1, -1), (1, 1)],
    [(0, -1), (0, 1)],
    [(-1, 1), (1, -1)],
    [(-1, 0), (1, 0)],
]; // spec: S-0518 claim 22 (e table)

/// The additive-line skip threshold (S-0518 Constants: 8).
const SKIP_BELOW: u8 = 8; // spec: S-0518 Constants (additive-line skip threshold 8)

/// The shifts of pair `i` in `sector` (S-0518 claim 22).
pub fn pair_shifts(sector: u8, i: i32) -> [(i32, i32); 2] {
    let [ea, eb] = PAIR_VECTORS[sector as usize % 8];
    if sector % 2 == 1 {
        // spec: S-0518 claim 22 (sectors 1, 3, 5, 7: i × eₐ and i × eᵦ)
        [(i * ea.0, i * ea.1), (i * eb.0, i * eb.1)]
    } else {
        // spec: S-0518 claim 22 (sectors 0, 2, 4, 6: (⌈i/2⌉eₐₓ, ⌊i/2⌋eₐᵧ) and (⌊i/2⌋eᵦₓ, ⌈i/2⌉eᵦᵧ))
        let (up, down) = ((i + 1) / 2, i / 2);
        [(up * ea.0, down * ea.1), (down * eb.0, up * eb.1)]
    }
}

/// How the drawing of one frame is set up.
pub struct Frame<'a> {
    /// Detail level 0, 1 or 2.
    pub detail: u8,
    /// The game's rounding.
    pub mode: Rounding,
    /// The activity test, run when a house-colour beam asks for it.
    pub fine: &'a mut dyn FnMut() -> bool,
    /// A ranged draw R(a, b) from the unsynchronised stream.
    pub draw: &'a mut dyn FnMut(i32, i32) -> i32,
}

/// One line of a beam: its colour, shift and pixel rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Line {
    Opaque([u8; 3]),
    Additive([u8; 3]),
    Blended([u8; 3], F32),
}

/// The spots of beam `b` at age `p` in the frame `f` (S-0518 claims 21 to
/// 23), in drawing order.
pub fn beam_spots(b: &Beam, p: i32, f: &mut Frame<'_>) -> Vec<Spot> {
    let i = intensity(b, p, f.mode);
    let lines: Vec<(Line, (i32, i32))> = match b.look {
        Look::Ordinary {
            inner,
            outer,
            spread,
            blink,
        } => {
            // spec: S-0518 claim 19 (with the blink flag, an ordinary beam is drawn only at odd P)
            if blink && p % 2 == 0 {
                return Vec::new();
            }
            // spec: S-0518 claim 21 (two outer lines when the outer colour is not black)
            let outer_lines = outer != [0, 0, 0];
            let [s1, s2] = OUTER_SHIFTS[b.sector as usize % 8];
            let oc = if outer_lines {
                // spec: S-0518 claim 21 (one draw per channel, red, green, blue, R(−σ, σ), clamped to 0 … 255)
                let mut c = [0u8; 3];
                for k in 0..3 {
                    let sigma = i32::from(spread[k]);
                    let v = i32::from(outer[k]) + (f.draw)(-sigma, sigma);
                    c[k] = v.clamp(0, 255) as u8;
                }
                c
            } else {
                outer
            };
            let mut v = Vec::new();
            if f.detail == 0 {
                // spec: S-0518 claim 21 (detail 0: opaque, full colours; the second outer line wins over the first, either over the centre)
                v.push((Line::Opaque(inner), (0, 0)));
                if outer_lines {
                    v.push((Line::Opaque(oc), s1));
                    v.push((Line::Opaque(oc), s2));
                }
            } else if !blink && outer_lines {
                // spec: S-0518 claim 21 (additive at I; C(O₂(O₁(d))))
                v.push((Line::Additive(oc), s1));
                v.push((Line::Additive(oc), s2));
                v.push((Line::Additive(inner), (0, 0)));
            } else {
                // spec: S-0518 claim 21 (the centre blended at I; with blink, outer lines blended at 1: O₂(O₁(C(d))))
                v.push((Line::Blended(inner, i), (0, 0)));
                if outer_lines {
                    v.push((Line::Blended(oc, ONE), s1));
                    v.push((Line::Blended(oc, ONE), s2));
                }
            }
            v
        }
        Look::House {
            colour,
            width,
            supported,
        } => {
            // spec: S-0518 claim 22 (fine mode when the detail level is not 0 and the frame rate test passes)
            let fine = f.detail != 0 && (f.fine)();
            // spec: S-0518 claim 22 and Constants (stop threshold 8 fine, 64 coarse)
            let threshold = if fine { 8 } else { 64 };
            // spec: S-0518 claim 22 (K = L, or 2L clamped to 255 when supported)
            let k = if supported {
                colour.map(|c| c.saturating_mul(2))
            } else {
                colour
            };
            let line = |c: [u8; 3]| {
                if fine {
                    // spec: S-0518 claim 22 (fine: every line additive at I)
                    Line::Additive(c)
                } else {
                    // spec: S-0518 claim 22 (coarse: opaque, c′ = c + trunc((255 − c) × r / 256), r = trunc(I × 255))
                    let r = scaled(255, i, f.mode);
                    Line::Opaque(c.map(|c| {
                        let c = i64::from(c);
                        (c + (255 - c) * r / 256).clamp(0, 255) as u8
                    }))
                }
            };
            let mut v = Vec::new();
            // spec: S-0518 claim 22 (the first pair is L halved, or K when supported)
            let mut pair = if supported { k } else { halved(colour) };
            for n in 1..=width {
                // spec: S-0518 claim 22 (no further pairs once the colour has every channel below the threshold)
                if pair.iter().all(|&c| c < threshold) {
                    break;
                }
                let [sa, sb] = pair_shifts(b.sector, n);
                v.push((line(pair), sa));
                v.push((line(pair), sb));
                // spec: S-0518 claim 22 (the next pair is L when supported and i = 1, otherwise halved)
                pair = if supported && n == 1 {
                    colour
                } else {
                    halved(pair)
                };
            }
            // spec: S-0518 claim 22 (C ∘ Bⱼ ∘ Aⱼ ∘ … ∘ B₁ ∘ A₁: the centre last; coarse: the centre wins, then the highest pair)
            v.push((line(k), (0, 0)));
            v
        }
    };
    let a = (b.from.0, b.from.1, b.bias.0);
    let z = (b.to.0, b.to.1, b.bias.1);
    let centre = samples(a, z);
    let cover = if lines.iter().any(|(l, _)| matches!(l, Line::Blended(..))) {
        coverage(a, z)
    } else {
        Vec::new()
    };
    let mut out = Vec::new();
    for (line, (sx, sy)) in lines {
        match line {
            Line::Opaque(c) => {
                // spec: S-0518 claim 23 (opaque: the line's colour reduced to 16-bit colour)
                let colour = pack(c.map(u32::from));
                out.extend(centre.iter().map(|&(x, y, bias)| Spot {
                    x: x + sx,
                    y: y + sy,
                    bias,
                    kind: SpotKind::Opaque { colour },
                }));
            }
            Line::Additive(c) => {
                // spec: S-0518 claim 23 (scaled colour trunc(c × I); not drawn when all three channels are below 8)
                let rgb = c.map(|c| scaled(i64::from(c), i, f.mode).clamp(0, 255) as u8);
                if rgb.iter().all(|&c| c < SKIP_BELOW) {
                    continue;
                }
                out.extend(centre.iter().map(|&(x, y, bias)| Spot {
                    x: x + sx,
                    y: y + sy,
                    bias,
                    kind: SpotKind::Additive { rgb },
                }));
            }
            Line::Blended(rgb, li) => {
                for c in &cover {
                    for (q, (px, py), from) in [
                        (c.q1, c.first, (0, 0)),
                        (
                            c.q2,
                            c.second,
                            (c.first.0 - c.second.0, c.first.1 - c.second.1),
                        ),
                    ] {
                        // spec: S-0518 F8 (w = trunc(q × I), the single I widened to double)
                        let w = scaled(i64::from(q), li, f.mode).clamp(0, 256) as u32;
                        out.push(Spot {
                            x: px + sx,
                            y: py + sy,
                            bias: c.bias,
                            kind: SpotKind::Blended {
                                rgb,
                                line: easing(w),
                                screen: easing(256 - w),
                                // spec: S-0518 claim 23 (a sample pair uses the first pixel's light-alpha)
                                alpha_from: (from.0 as i8, from.1 as i8),
                            },
                        });
                    }
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Beam flashes (S-0514 claims 23 to 26)

/// A beam flash: its two world points, its colour and its first drawn tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flash {
    /// *S*.
    pub s: Pos,
    /// *T*.
    pub t: Pos,
    /// The flash colour.
    pub rgb: [u8; 3],
    /// The tick of its first drawn frame (*k* = 0).
    pub born: i32,
}

/// A beam flash's life (S-0514 Constants: 15 drawn frames).
pub const FLASH_FRAMES: i32 = 15; // spec: S-0514 Constants (beam flash life 15 drawn frames)

/// The double nearest π/180 (S-0514 F6: *c*).
const DEGREE: F64 = F64(0x3F91_DF46_A252_9D39); // spec: S-0514 F6 (c, the double nearest π/180)

/// The pixels of a nearest-integer 8-connected line from `a` to `b`, both
/// ends included (our interim, S-0514 Open questions).
pub(crate) fn segment_pixels(a: (i32, i32), b: (i32, i32)) -> Vec<(i32, i32)> {
    let (dx, dy) = (i64::from(b.0 - a.0), i64::from(b.1 - a.1));
    let h = dx.abs().max(dy.abs());
    if h == 0 {
        return vec![a];
    }
    (0..=h)
        .map(|k| (a.0 + step(dx, k, h) as i32, a.1 + step(dy, k, h) as i32))
        .collect()
}

/// The spots of flash `fl` in its drawn frame `k` (S-0514 claims 24 and
/// 25, F6 to F8).
pub fn flash_spots(fl: &Flash, k: i32, mode: Rounding) -> Vec<Spot> {
    let (s, t) = (fl.s, fl.t);
    // spec: S-0514 F6 (D = T − S; squares and sum in signed 32-bit arithmetic with wrap)
    let (dx, dy, dz) = (
        t.x.wrapping_sub(s.x),
        t.y.wrapping_sub(s.y),
        t.z.wrapping_sub(s.z),
    );
    let n2 = dx
        .wrapping_mul(dx)
        .wrapping_add(dy.wrapping_mul(dy))
        .wrapping_add(dz.wrapping_mul(dz));
    // spec: S-0514 F6 (ℓ = r(…), S-0204 claim 5's table root, unrounded; no flash when ℓ ≤ 0)
    let len = glasnost_num::geom::table_root(u64::from(n2.unsigned_abs()), mode).to_f64();
    if !len.gt(F64::ZERO) {
        return Vec::new();
    }
    let num = |v: i64| F64::from_i64(v, mode);
    let trunc = |v: F64| v.trunc_i128().map_or(0, |x| x as i64);
    // spec: S-0514 F6 (A = (k / 15) × 40, double precision)
    let amp = num(i64::from(k))
        .div(num(i64::from(FLASH_FRAMES)), mode)
        .mul(num(40), mode);
    // spec: S-0514 F6 (s = 10 leptons at full frame rate; p is our interim 1)
    let seg = num(10);
    // spec: S-0514 F6 (n = trunc(ℓ / s))
    let n = trunc(len.div(seg, mode));
    // spec: S-0514 F6 (M = max(ℓ, 360))
    let m = len.max(num(360));
    // spec: S-0514 F6 (θⱼ = c × (2M × (j / n)), hⱼ = sin*(θⱼ) × A)
    let height = |j: i64| -> i64 {
        if n == 0 {
            return 0;
        }
        let theta = DEGREE.mul(
            num(2).mul(m, mode).mul(num(j).div(num(n), mode), mode),
            mode,
        );
        trunc(glasnost_num::trig::sin_t(theta, mode).mul(amp, mode))
    };
    let point = |j: i64| -> Pos {
        if n == 0 {
            // spec: S-0514 F6 (n = 0: one straight segment from S to T)
            return if j == 0 { s } else { t };
        }
        // spec: S-0514 F6 (step e = (trunc(Dx / n), trunc(Dy / n), trunc(Dz / n)); Pⱼ = S + j × e, raised by trunc(hⱼ))
        let e = |d: i32| i64::from(d) / n;
        Pos::new(
            (i64::from(s.x) + j * e(dx)) as i32,
            (i64::from(s.y) + j * e(dy)) as i32,
            (i64::from(s.z) + j * e(dz) + height(j)) as i32,
        )
    };
    // spec: S-0514 F8 (Δ is the screen row of S minus that of T, both lifted)
    let delta = project_lifted(s.x, s.y, s.z).1 - project_lifted(t.x, t.y, t.z).1;
    let mut out = Vec::new();
    let mut i = 0i64;
    // spec: S-0514 F6 (segments Pᵢ to Pᵢ₊₁ for every i with i × s < ℓ; one when n = 0)
    while if n == 0 {
        i == 0
    } else {
        num(i).mul(seg, mode).lt(len)
    } {
        let (p0, p1) = (point(i), point(i + 1));
        // spec: S-0514 F7 (fᵢ = 0.5 + (|trunc(hᵢ₊₁)| × 0.5) / 40; channel′ = trunc(channel × fᵢ))
        let half = F64::from_parts(false, 1, -1, mode);
        let f = half.add(
            num(height(i + 1).abs()).mul(half, mode).div(num(40), mode),
            mode,
        );
        let rgb = fl
            .rgb
            .map(|c| trunc(num(i64::from(c)).mul(f, mode)).clamp(0, 255) as u8);
        let a = project_lifted(p0.x, p0.y, p0.z);
        let b = project_lifted(p1.x, p1.y, p1.z);
        // spec: S-0514 F8 (b₀ = Δ − L(z₀) − 2 at the first point, b₁ = −L(z₁) − 2 at the second, raised heights)
        let b0 = delta - height_lift(p0.z) - NEARNESS;
        let b1 = -height_lift(p1.z) - NEARNESS;
        // spec: S-0514 F8 (the whole segment uses the bias of its leftmost end)
        let bias = if b.0 < a.0 { b1 } else { b0 };
        out.extend(segment_pixels(a, b).into_iter().map(|(x, y)| Spot {
            x,
            y,
            bias,
            kind: SpotKind::Flash { rgb },
        }));
        i += 1;
    }
    out
}

// ---------------------------------------------------------------------------
// The magnetic wave (S-0607 claims 25 and 26)

/// The band's half width (S-0607 claim 25: 100 leptons wide).
const HALF_BAND: i64 = 50; // spec: S-0607 claim 25 (corners 50 leptons to either side)
/// The second end's raise (S-0607 claim 25: 50 leptons).
pub const END_RAISE: i32 = 50; // spec: S-0607 claim 25 (raised by 50 leptons)

/// 50 × |`d`| / √`sq` as its integer part *m* and whether it is exact
/// (*m*² × `sq` = 2500 × `d`²); `sq` > 0.
fn band_offset(d: i64, sq: i64) -> (i64, bool) {
    let num = i128::from(HALF_BAND * HALF_BAND) * i128::from(d) * i128::from(d);
    let m = glasnost_num::geom::isqrt((num / i128::from(sq)) as u128) as i64;
    (m, i128::from(m) * i128::from(m) * i128::from(sq) == num)
}

/// `e` + `sign` × *x* rounded toward zero, where *x* ≥ 0 has the integer
/// part `m` and is an integer when `exact`.
fn corner_component(e: i32, sign: i64, m: i64, exact: bool) -> i32 {
    let base = i64::from(e) + sign * m;
    if exact || sign == 0 {
        return base as i32;
    }
    // The exact value lies strictly between base and base + sign.
    let value_sign = if base != 0 { base.signum() } else { sign };
    (if value_sign == sign {
        base
    } else {
        base + sign
    }) as i32
}

/// The band's four corners in world leptons (S-0607 F5), in the order
/// *E*₂ − 50*n*, *E*₂ + 50*n*, *E*₁ + 50*n*, *E*₁ − 50*n*.
pub fn wave_corners(e1: Pos, e2: Pos) -> [Pos; 4] {
    let (dx, dy) = (i64::from(e2.x - e1.x), i64::from(e2.y - e1.y));
    let sq = dx * dx + dy * dy;
    // spec: S-0607 F5 (u = (Δx, Δy) / h; h = 0 gives u = (1, 0); n = (−uᵧ, uₓ), so 50n = (−50Δy/h, 50Δx/h))
    let (nx, ny) = if sq == 0 {
        ((0, 0, true), (1, HALF_BAND, true))
    } else {
        let (mx, ex) = band_offset(dy, sq);
        let (my, ey) = band_offset(dx, sq);
        ((-dy.signum(), mx, ex), (dx.signum(), my, ey))
    };
    // spec: S-0607 F5 (each component rounded toward zero to whole leptons)
    let corner = |e: Pos, side: i64| {
        Pos::new(
            corner_component(e.x, side * nx.0, nx.1, nx.2),
            corner_component(e.y, side * ny.0, ny.1, ny.2),
            e.z,
        )
    };
    [corner(e2, -1), corner(e2, 1), corner(e1, 1), corner(e1, -1)]
}

/// The wave's shift direction from the shot's firing point and impact
/// point (S-0607 F5), as a unit pixel offset.
pub fn shift_direction(firing: Pos, impact: Pos) -> (i8, i8) {
    // spec: S-0607 F5 (E₁ the firing point, E₂ the impact point raised by 50)
    let e2 = Pos::new(impact.x, impact.y, impact.z + END_RAISE);
    let s = project_lifted(firing.x, firing.y, firing.z);
    let c = wave_corners(firing, e2)[1];
    let p = project_lifted(c.x, c.y, c.z);
    // spec: S-0607 F5 (Δx = Pₓ − Sₓ, Δy = Sᵧ − Pᵧ)
    let (dx, dy) = (p.0 - s.0, s.1 - p.1);
    // spec: S-0607 F5 (right or left when Δx ≠ 0 and −1 ≤ Δy / Δx < 1)
    let flat = match dx.cmp(&0) {
        Ordering::Greater => -dx <= dy && dy < dx,
        Ordering::Less => -dx >= dy && dy > dx,
        Ordering::Equal => false,
    };
    if flat {
        (dx.signum() as i8, 0)
    } else if dy >= 1 {
        (0, -1) // spec: S-0607 F5 (up when Δy ≥ 1)
    } else {
        (0, 1) // spec: S-0607 F5 (down when Δy ≤ 0)
    }
}

/// The visual countdown *c* `age` ticks after the wave's creation
/// (S-0607 F4): 100 at creation, falling by 1 per tick and restarting at
/// 64 when an update finds 19, so 63 is the next drawn value.
pub fn countdown(age: i32) -> i32 {
    // spec: S-0607 F4 (starts at 100, decreases by 1 each tick, reset to 64 when an update finds it at 19)
    let mut c = 100;
    for _ in 0..age.max(0) {
        if c == 19 {
            c = 64;
        }
        c -= 1;
    }
    c
}

/// *W*(*i*) of S-0607 F5.
pub fn ripple_wave(i: i64, mode: Rounding) -> i64 {
    let tenth = F64(0x3FB9_9999_9999_999A); // spec: S-0607 F5 (0.1, the nearest double)
    let bump = F64(0x3FDF_5C28_F5C2_8F5C); // spec: S-0607 F5 (0.49, the nearest double)
    // spec: S-0607 F5 (W(i) = trunc(RZ(RZ(12 × sinₜ(RZ(i × 0.1))) + 0.49)))
    let x = F64::from_i64(i, mode).mul(tenth, mode);
    let s = glasnost_num::trig::sin_t(x, mode);
    let v = F64::from_i64(12, mode).mul(s, mode).add(bump, mode);
    v.trunc_i128().map_or(0, |x| x as i64)
}

/// The displacement *s* and boost *k* of a band pixel whose squared screen
/// distance from the first end is `dist_sq`, at countdown `c` (S-0607 F5).
pub fn ripple(dist_sq: u64, c: i32, mode: Rounding) -> (i32, u16) {
    // spec: S-0607 F5 (D = trunc(r(…)) with the table root)
    let d = glasnost_num::geom::root_floor(dist_sq, true, mode) as i64;
    // spec: S-0607 F5 (j = min(|W(D + c)|, 9))
    let j = ripple_wave(d + i64::from(c), mode).abs().min(9);
    // spec: S-0607 F5 (s = 0 for j ≤ 1, 1 for 2 ≤ j ≤ 6, 2 for 7 ≤ j ≤ 9; k = 110 + 8j)
    let s = match j {
        0 | 1 => 0,
        2..=6 => 1,
        _ => 2,
    };
    (s, (110 + 8 * j) as u16)
}

/// Whether the doubled point `p` lies inside or on the convex polygon `v`
/// (doubled coordinates).
fn inside(v: &[(i64, i64); 4], p: (i64, i64)) -> bool {
    let mut pos = false;
    let mut neg = false;
    for i in 0..4 {
        let (a, b) = (v[i], v[(i + 1) % 4]);
        let cross = (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
        pos |= cross > 0;
        neg |= cross < 0;
    }
    !(pos && neg) && (pos || neg)
}

/// The spots of a magnetic wave with ends `e1` and `e2` (`e2` already
/// raised), shift direction `dir` and countdown `c` (S-0607 claim 25, F5).
pub fn wave_spots(e1: Pos, e2: Pos, dir: (i8, i8), c: i32, mode: Rounding) -> Vec<Spot> {
    let corners = wave_corners(e1, e2).map(|p| project_lifted(p.x, p.y, p.z));
    let doubled = corners.map(|(x, y)| (2 * i64::from(x), 2 * i64::from(y)));
    let (x0, y0) = project_lifted(e1.x, e1.y, e1.z);
    // spec: S-0607 claim 25 (flat profile, bias −L(z) − 2 with z the first end's height)
    let bias = -height_lift(e1.z) - NEARNESS;
    let left = corners.iter().map(|c| c.0).min().unwrap_or(0);
    let right = corners.iter().map(|c| c.0).max().unwrap_or(0);
    let top = corners.iter().map(|c| c.1).min().unwrap_or(0);
    let bottom = corners.iter().map(|c| c.1).max().unwrap_or(0);
    let mut out = Vec::new();
    for y in top..=bottom {
        for x in left..=right {
            // Our interim fill rule: the pixel's centre inside or on the band.
            if !inside(&doubled, (2 * i64::from(x) + 1, 2 * i64::from(y) + 1)) {
                continue;
            }
            let (dx, dy) = (i64::from(x - x0), i64::from(y - y0));
            let (s, k) = ripple((dx * dx + dy * dy) as u64, c, mode);
            out.push(Spot {
                x,
                y,
                bias,
                kind: SpotKind::Wave {
                    // spec: S-0607 F5 (the source pixel s pixels along the shift direction)
                    shift: (dir.0 * s as i8, dir.1 * s as i8),
                    k,
                },
            });
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The view's line effects

/// The imported settings the line effects read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// Each weapon's laser, electric-bolt and radiation-beam flags, by
    /// simulation weapon index.
    pub flags: Vec<(bool, bool, bool)>,
    /// The chrono beam colour (S-0514 claim 23).
    pub chrono: [u8; 3],
    /// The radiation colour (S-0514 claim 23).
    pub rad: [u8; 3],
    /// The activity floor *m* and margin *z* (S-0518 claim 22).
    pub limits: (i32, i32),
    /// Each house's laser colour *L* by house index (S-0518 claim 17).
    pub house_laser: Vec<[u8; 3]>,
    /// Each particle-system type's laser colour when it has the laser
    /// flag, by simulation system-type index (S-0616 claim 31).
    pub rails: Vec<Option<[u8; 3]>>,
    /// The electric bolts' colours from the general palette (S-0316 claim
    /// 10), when the palette file is there.
    pub bolt: Option<super::bolts::Colours>,
}

impl Settings {
    /// Reads the settings from the import, the game's rules and the
    /// houses' display colours (S-0103 claim 15).
    pub fn read(
        import: &glasnost_convert::Import,
        rules: &glasnost_sim::rules::SimRules,
        house_rgb: &[[u8; 3]],
    ) -> Self {
        use glasnost_convert::kinds::Kind;
        use glasnost_ini::value;
        let flags = rules
            .combat
            .weapons
            .iter()
            .map(|wr| {
                let idx = super::build::find_type(import, Kind::Weapons, wr.name.as_bytes());
                let flag = |key: &[u8]| {
                    idx.is_some_and(|i| {
                        super::build::rule_bool(import, Kind::Weapons, i, key, false)
                    })
                };
                // spec: S-0518 Import mapping and S-0514 Import mapping (IsLaser, IsElectricBolt, IsRadBeam: booleans, absent no)
                (
                    flag(b"IsLaser"),
                    flag(b"IsElectricBolt"),
                    flag(b"IsRadBeam"),
                )
            })
            .collect();
        let rails = rules
            .gas
            .systems
            .iter()
            .map(|t| {
                let idx =
                    super::build::find_type(import, Kind::ParticleSystems, t.name.as_bytes())?;
                // spec: S-0616 Import mapping (Laser: boolean, absent no; LaserColor: colour, absent 0, 0, 0)
                super::build::rule_bool(import, Kind::ParticleSystems, idx, b"Laser", false).then(
                    || {
                        let c =
                            super::build::rule(import, Kind::ParticleSystems, idx, b"LaserColor")
                                .map_or([0; 3], |v| value::int_tuple_or(v, [0; 3]).0);
                        channels(c)
                    },
                )
            })
            .collect();
        let global = |section: &[u8], key: &[u8]| {
            import
                .outcome
                .globals
                .get(section)
                .and_then(|s| s.get(key))
                .and_then(|k| k.value.as_deref())
        };
        // spec: S-0514 Import mapping (ChronoBeamColor: three integers, absent 0, 0, 0)
        let chrono = global(b"AudioVisual", b"ChronoBeamColor")
            .map_or([0; 3], |v| value::int_tuple_or(v, [0; 3]).0);
        let int = |key: &[u8], held: i32| {
            global(b"AudioVisual", key)
                .and_then(value::integer)
                .unwrap_or(held)
        };
        Self {
            flags,
            chrono: channels(chrono),
            // spec: S-0525 Import mapping and S-0514 Import mapping (revision 11: RadColor, a byte triple; the value in force, (0, 0, 0) at a fresh start)
            rad: channels(rules.combat.general.radiation.colour),
            // spec: S-0518 Import mapping (DetailMinFrameRateNormal: held value 15; DetailBufferZoneWidth: held value 5)
            limits: (
                int(b"DetailMinFrameRateNormal", 15),
                int(b"DetailBufferZoneWidth", 5),
            ),
            // spec: S-0518 claim 17 (each house's L from its display colour, S-0103 claim 15)
            house_laser: house_rgb.iter().map(|&c| laser_colour(c)).collect(),
            rails,
            bolt: None,
        }
    }
}

/// A magnetic wave's picture (S-0607 claim 25).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wave {
    /// The first end *E*₁.
    pub e1: Pos,
    /// The second end *E*₂, raised.
    pub e2: Pos,
    /// The shift direction, fixed at creation.
    pub dir: (i8, i8),
    /// The creation tick (countdown 100).
    pub born: i32,
    /// The serial of the next animation at its creation: it is drawn after
    /// the air layer's animations created before it (S-0607 claim 26).
    pub after: u32,
}

/// The line effects the battlefield view shows (presentation state).
#[derive(Clone, Debug, Default)]
pub struct Lines {
    /// The settings.
    pub settings: Settings,
    /// Beams still shown, in creation order.
    pub beams: Vec<Beam>,
    /// Beam flashes still shown, in creation order.
    pub flashes: Vec<Flash>,
    /// Electric bolts still shown, in creation order (S-0316).
    pub bolts: Vec<super::bolts::Bolt>,
    /// The magnetic waves' pictures, by wave.
    pub waves: std::collections::BTreeMap<glasnost_sim::world::ObjectId, Wave>,
    /// The activity test's state (S-0518 claim 22).
    pub activity: Activity,
    /// Our synthetic host clock, in milliseconds: each tick adds its paced
    /// period (S-0201 claim 4).
    clock_ms: i64,
}

impl Lines {
    /// The line effects with `settings` and none shown.
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            ..Self::default()
        }
    }

    /// The laser colour *L* of `house` (white without one: S-0518 F5's
    /// *n* = 0 case).
    fn laser(&self, house: Option<glasnost_sim::world::HouseId>) -> [u8; 3] {
        house
            .and_then(|h| self.settings.house_laser.get(usize::from(h.0)))
            .copied()
            .unwrap_or([255, 255, 255]) // spec: S-0518 F5 (white)
    }

    /// Takes one of a tick's effects at tick `now`; `next_anim` is the
    /// serial of the next animation.
    pub fn take(
        &mut self,
        w: &glasnost_sim::world::World,
        e: &glasnost_sim::combat::effects::Effect,
        now: i32,
        next_anim: u32,
    ) {
        use glasnost_sim::combat::effects::Effect;
        let rules = w.rules();
        match *e {
            Effect::SupportBeam {
                from,
                to,
                duration,
                house,
            } => {
                // spec: S-0518 claim 18 (support beam: D = Lₛ, inner L of the supporter's house, W = 3, S = 1, E = 0)
                let look = support_look(self.laser(house));
                self.beams
                    .push(Beam::new(from, to, 0, now, duration, NONE, look));
            }
            Effect::Beam {
                from,
                to,
                duration,
                fade_out,
                colours,
                house,
            } => {
                // spec: S-0615 claim 15 (with the house-colour flag the inner colour is the house laser colour, the outer that colour halved; the spread is kept)
                let colours = match house {
                    Some(_) => glasnost_sim::disc::house_colours(
                        self.laser(house).map(i32::from),
                        colours[2],
                    ),
                    None => colours,
                };
                let [inner, outer, spread] = colours.map(channels);
                // spec: S-0518 cross-spec changes (disc beams use ordinary drawing even when their colours come from the house)
                let look = Look::Ordinary {
                    inner,
                    outer,
                    spread,
                    blink: false,
                };
                // spec: S-0615 claims 15 and 16 (revision 4: arc beams S = 1, E = 0.5; the final beam S = 1, E = 0; S-0518 claim 19 and F6 own the lifetime and intensity)
                let end = if fade_out { NONE } else { HALF };
                self.beams
                    .push(Beam::new(from, to, 0, now, duration, end, look));
            }
            Effect::Line {
                firer,
                weapon,
                from,
                primary,
                to,
                building,
                prism,
            } => {
                let Some(&(laser, bolt, rad)) = self.settings.flags.get(weapon as usize) else {
                    return;
                };
                let wr = rules.combat.weapon(weapon);
                match line_effect(laser, bolt, rad) {
                    LineEffect::Laser => {
                        let source = match (building, prism) {
                            (Some(_), Some(n)) => Source::Prism(n),
                            (Some(_), None) => Source::Building,
                            (None, _) => Source::Unit,
                        };
                        let house = w.obj(firer).and_then(|o| o.owner);
                        let look = weapon_look(
                            source,
                            wr.laser_colours.map(channels),
                            wr.house_colour,
                            self.laser(house),
                        );
                        // spec: S-0518 claim 20 (a building's weapon beam: a = min(0, yₛ − yₒ); otherwise 0)
                        let a = building.map_or(0, |b| building_nearness(from, b));
                        // spec: S-0518 claim 18 (D is the weapon's laser duration)
                        self.beams
                            .push(Beam::new(from, to, a, now, wr.laser_duration, NONE, look));
                    }
                    LineEffect::Flash => {
                        // spec: S-0514 claim 23 (the chrono beam colour with a temporal warhead, the radiation colour otherwise)
                        let temporal = wr.warhead.is_some_and(|h| rules.combat.warhead(h).temporal);
                        let rgb = if temporal {
                            self.settings.chrono
                        } else {
                            self.settings.rad
                        };
                        // spec: S-0514 claim 24 (S the primary slot's fire point, T the target point; both fixed)
                        self.flashes.push(Flash {
                            s: primary,
                            t: to,
                            rgb,
                            born: now,
                        });
                    }
                    // spec: S-0316 claim 2 (the bolt wins over the radiation beam: no beam flash; the bolt is drawn from Effect::Bolt)
                    LineEffect::Bolt | LineEffect::None => {}
                }
            }
            Effect::FragmentLine {
                attacker,
                weapon,
                from,
                to,
            } => {
                let wr = rules.combat.weapon(weapon);
                let house = w.obj(attacker).and_then(|o| o.owner);
                // spec: S-0318 claim 8 (the look of S-0518 claim 18 for a laser shot of the fragment weapon by the attacker, thickness 1)
                let look = weapon_look(
                    Source::Fragment,
                    wr.laser_colours.map(channels),
                    wr.house_colour,
                    self.laser(house),
                );
                // spec: S-0318 claim 8 (from the burst point, a = 0 whatever the attacker; the fragment weapon's laser duration, S-0518 claim 18)
                self.beams
                    .push(Beam::new(from, to, 0, now, wr.laser_duration, NONE, look));
            }
            Effect::RailLine {
                system_type,
                from,
                to,
            } => {
                // spec: S-0616 claim 31 (only a type with the laser flag makes a beam)
                let Some(Some(inner)) = self.settings.rails.get(system_type as usize).copied()
                else {
                    return;
                };
                self.beams.push(rail_beam(from, to, now, inner));
            }
            Effect::Wave { id, from, to } => {
                self.waves.insert(
                    id,
                    Wave {
                        e1: from,
                        e2: to,
                        // spec: S-0607 claim 25 (the shift direction is chosen once, at creation, from the shot's firing and impact points)
                        dir: shift_direction(from, to),
                        born: now,
                        after: next_anim,
                    },
                );
                self.follow(w, id);
            }
            _ => {}
        }
    }

    /// Sets wave `id`'s ends from current positions while it has not ended
    /// (S-0607 claim 25).
    fn follow(&mut self, w: &glasnost_sim::world::World, id: glasnost_sim::world::ObjectId) {
        use glasnost_sim::world::Target;
        let Some(state) = w.waves.get(&id) else {
            return;
        };
        // spec: S-0607 claim 25 (once the wave has ended its ends are no longer updated)
        if state.ended {
            return;
        }
        // spec: S-0607 claim 21 (the wave ends at its next turn when its firer or its target has left play, or its firer's target is no longer the wave's), claim 25 (the band stays where it last was)
        // design: render (#72) an object removed or put away by a turn after the wave's own leaves the wave not yet ended at the end of the tick; its ends are kept as its next turn's end will keep them, not taken from an object no longer in play
        let in_play = |x: glasnost_sim::world::ObjectId| {
            w.obj(x).is_some_and(|o| {
                o.on_map
                    && o.structure.inside.is_none()
                    && glasnost_sim::transport::riding(w, x).is_none()
            }) && glasnost_sim::structures::bunker::bunkered_in(w, x).is_none()
        };
        let target_gone = matches!(state.target, Target::Object(t) if !in_play(t));
        let retargeted = w.obj(state.firer).and_then(|o| o.target) != Some(state.target);
        if !in_play(state.firer) || target_gone || retargeted {
            return;
        }
        let Some(v) = self.waves.get_mut(&id) else {
            return;
        };
        let raise = |p: Pos| Pos::new(p.x, p.y, p.z + END_RAISE);
        // spec: S-0607 claim 25 (the fire point of the firer's first weapon)
        let fire = glasnost_sim::combat::position::fire_point(w, state.firer, 0, 0);
        match state.target {
            Target::Object(t)
                if w.obj(t)
                    .is_some_and(|o| o.kind == glasnost_sim::rules::Kind::Vehicle) =>
            {
                // spec: S-0607 claim 25 (a vehicle target: E₁ the target's centre, E₂ the fire point raised by 50)
                v.e1 = w.reference_point(t);
                v.e2 = raise(fire);
            }
            target => {
                // spec: S-0607 claim 25 (any other target: E₁ the fire point, E₂ the target's centre raised by 50)
                v.e1 = fire;
                // A cell target's centre at its surface height (our interim, as a shot's target point).
                v.e2 = raise(match target {
                    Target::Object(t) => w.reference_point(t),
                    Target::Cell(_) => glasnost_sim::combat::range::target_point(w, target),
                });
            }
        }
    }

    /// The end of a tick at game speed `speed`: the activity counter
    /// (S-0518 claim 22), the waves' ends and removals, and the beams and
    /// flashes that are gone by tick `now`.
    pub fn after_tick(&mut self, w: &glasnost_sim::world::World, now: i32, speed: u8) {
        self.activity.tick();
        self.clock_ms += super::live::period_ms(speed);
        self.activity.check(self.clock_ms);
        // spec: S-0607 claim 25 (a wave is drawn until it is removed, then vanishes at once)
        self.waves.retain(|id, _| w.waves.contains_key(id));
        let ids: Vec<_> = self.waves.keys().copied().collect();
        for id in ids {
            self.follow(w, id);
        }
        // spec: S-0518 claim 19 (gone from the tick where P = D)
        self.beams.retain(|b| now - b.born < b.duration);
        // spec: S-0514 claim 24 (gone from the 16th drawn frame; our engine draws one frame per tick)
        self.flashes.retain(|f| now - f.born < FLASH_FRAMES);
        // spec: S-0316 claim 5 (gone after its 17th drawn frame; our engine draws one frame per tick)
        self.bolts.retain(|b| b.frame(now).is_some());
        self.follow_bolts(w);
    }

    /// Takes an electric bolt's report at tick `now` (S-0316 claims 3, 4
    /// and 13), drawing its phase from `draw`.
    pub fn take_bolt(
        &mut self,
        w: &glasnost_sim::world::World,
        e: &glasnost_sim::combat::effects::Effect,
        now: i32,
        draw: &mut dyn FnMut(i32, i32) -> i32,
    ) {
        use glasnost_sim::combat::effects::Effect;
        let Effect::Bolt {
            firer,
            weapon,
            slot,
            burst,
            from,
            to,
            building,
            vehicle_on_map,
        } = *e
        else {
            return;
        };
        // spec: S-0316 claim 4 (attached only when the firer is a vehicle on the map with no bolt attached to it already)
        let attached = vehicle_on_map
            && !self
                .bolts
                .iter()
                .any(|b| b.attached && b.firer == firer && b.frame(now).is_some());
        // spec: S-0316 claim 10 (the weapon's alternate-colour flag)
        let alt = w.rules().combat.weapon(weapon).alt_colour;
        // spec: S-0316 F4 (a building firer: S-0518 claim 20's a = min(0, yₛ − yₒ); otherwise 0)
        let a = building.map_or(0, |b| building_nearness(from, b));
        // spec: S-0316 claim 5 (its first drawn frame is the one after the shot's tick: our shown tick)
        self.bolts.push(super::bolts::Bolt::new(
            firer, slot, burst, from, to, attached, now, alt, a, draw,
        ));
    }

    /// Moves each attached bolt's start to its firer's current fire point
    /// for the same slot and burst side (S-0316 claim 4).
    fn follow_bolts(&mut self, w: &glasnost_sim::world::World) {
        for b in self.bolts.iter_mut().filter(|b| b.attached) {
            if w.obj(b.firer).is_none() {
                // spec: S-0316 claim 4 (the firer's removal ends the attachment; the bolt keeps its last start)
                b.attached = false;
                continue;
            }
            // spec: S-0316 claim 4 (each drawn frame, before the view test, the start moves to the current fire point of the slot that fired, the shot's own side; a destroyed firer still present and one off the map keep following)
            b.start = glasnost_sim::combat::position::fire_point(w, b.firer, b.slot, b.burst);
        }
    }

    /// The electric bolts' spots at tick `now` (S-0316 claims 5, 9 to 13):
    /// the newest bolt first, each out of view skipped with no draws;
    /// `view` the battlefield view's origin and size (`None`: all in view),
    /// `table` and `mode` the switches of the bolt length.
    pub fn bolt_spots(
        &self,
        now: i32,
        view: Option<((i32, i32), (u32, u32))>,
        table: bool,
        mode: Rounding,
        draw: &mut dyn FnMut(i32, i32) -> i32,
    ) -> Vec<Spot> {
        use super::bolts;
        let mut out = Vec::new();
        // spec: S-0316 claims 12 and 13 (the newest bolt first)
        for b in self.bolts.iter().rev() {
            let Some(n) = b.frame(now) else {
                continue;
            };
            let at = |p: Pos| project_lifted(p.x, p.y, p.z);
            // spec: S-0316 claim 5 (a bolt whose straight screen line between its two ends lies wholly outside the battlefield view is not drawn and takes no draws)
            if let Some((origin, size)) = view
                && !bolts::line_in_view(at(b.start), at(b.end), origin, size)
            {
                continue;
            }
            let pieces = bolts::shape(b.start, b.end, b.phase_at(n), b.a, table, mode, draw);
            if let Some(c) = &self.settings.bolt {
                out.extend(bolts::spots(&pieces, b.alt, c));
            }
        }
        out
    }

    /// The beams' spots at tick `now` (S-0518 claims 19 to 23), in creation
    /// order, and the beam flashes' (S-0514 claims 24 to 26).
    pub fn spots(
        &mut self,
        now: i32,
        detail: u8,
        mode: Rounding,
        draw: &mut dyn FnMut(i32, i32) -> i32,
    ) -> (Vec<Spot>, Vec<Spot>) {
        let (m, z) = self.settings.limits;
        let activity = &mut self.activity;
        let mut fine = || activity.test(m, z);
        let mut f = Frame {
            detail,
            mode,
            fine: &mut fine,
            draw,
        };
        let beams = self
            .beams
            .iter()
            .filter_map(|b| Some(beam_spots(b, b.age(now)?, &mut f)))
            .flatten()
            .collect();
        let flashes = self
            .flashes
            .iter()
            .filter(|fl| (0..FLASH_FRAMES).contains(&(now - fl.born)))
            .flat_map(|fl| flash_spots(fl, now - fl.born, mode))
            .collect();
        (beams, flashes)
    }

    /// Each wave's spots at tick `now` with the serial it follows, in
    /// creation order.
    pub fn wave_spots(&self, now: i32, mode: Rounding) -> Vec<(u32, Vec<Spot>)> {
        let mut out: Vec<(u32, Vec<Spot>)> = self
            .waves
            .values()
            .map(|v| {
                (
                    v.after,
                    wave_spots(v.e1, v.e2, v.dir, countdown(now - v.born), mode),
                )
            })
            .collect();
        // spec: S-0607 claim 26 (the air layer's list in entering order)
        out.sort_by_key(|(after, _)| *after);
        out
    }
}

#[cfg(test)]
#[path = "beams_tests.rs"]
mod tests;
