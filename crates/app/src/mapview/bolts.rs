//! Electric bolts (S-0316): the drawn effect of an electric-bolt shot,
//! three jagged lines from the firer's fire point to the bolt end, made
//! afresh in each drawn frame of its 17-frame life by midpoint
//! displacement (F1 to F3) and drawn as single-pixel lines of the beam
//! flash's line kind (claim 11, S-0514 claim 25) with F4's depth biases.
//!
//! All of this is presentation: nothing here feeds back into the game. The
//! bolt's draws come from the view's cosmetic stream (S-0316 claim 13: the
//! unsynchronised stream), as the beam spreads' do. The spark system at the
//! bolt end is made in play (`glasnost_sim::emitters::electric_bolt_sparks`).
//!
//! Interim rules where the specs leave a case open (as for the beam flash,
//! docs/development.md, *Visual review 2026-10-19*):
//! - a piece covers the nearest-integer 8-connected line with both ends and
//!   uses its leftmost end's unclipped bias (S-0316 Open questions: interior
//!   pixels; S-0514 F8's clipped-end bias is not applied);
//! - one drawn frame per tick: a bolt's drawn frame *n* is its age in ticks
//!   (so its life does not burn down during a freeze in our client).

use glasnost_num::soft::{F64, Rounding};
use glasnost_render::projection::{height_lift, project_lifted};
use glasnost_render::spots::{Spot, SpotKind};
use glasnost_sim::world::{ObjectId, Pos};

/// A bolt's life in drawn frames (S-0316 claim 5).
pub const LIFE: i32 = 17; // spec: S-0316 claim 5 and Constants (bolt life 17 drawn frames)

/// The longest unsplit piece, as a length counter (F1).
const UNSPLIT: i64 = 64; // spec: S-0316 F1 and Constants (longest unsplit piece 64 leptons)

/// The most split levels (F1).
const MOST_LEVELS: u32 = 8; // spec: S-0316 F1 and Constants (most split levels 8)

/// The jitter amplitude factor's numerator, over 256 (F1).
const JITTER: i64 = 23; // spec: S-0316 F1 and Constants (jitter amplitude factor 23 / 256)

/// The pinning threshold factor's numerator, over 256 (F1).
const PINNING: i64 = 102; // spec: S-0316 F1 and Constants (pinning threshold factor 102 / 256)

/// The denominator of F1's two factors.
const FACTOR_SCALE: i64 = 256; // spec: S-0316 F1 (J₀ = ⌊23 λ / 256⌋, P = ⌊102 λ / 256⌋)

/// The coarse-jitter limit, as a length counter (F3).
const COARSE: i64 = 128; // spec: S-0316 F3 and Constants (coarse-jitter limit 128 leptons)

/// The first sway divisor (F2: *i* + 7 for *i* = 0 … 5).
const SWAY_DIVISOR: i64 = 7; // spec: S-0316 F2 and Constants (sway divisors 7 to 12)

/// The top of the phase draw R(0, 256) (claim 13).
const PHASE_TOP: i32 = 256; // spec: S-0316 claim 13 and Constants (phase range 0 to 256)

/// The general palette entry of strands B and C (claim 10).
pub const SIDE_ENTRY: usize = 10; // spec: S-0316 claim 10 and Constants (strand B and C colour entry 10)

/// The entry of strands B and C with the alternate-colour flag (claim 10).
pub const ALT_SIDE_ENTRY: usize = 5; // spec: S-0316 claim 10 and Constants (entry 5 with the alternate-colour flag)

/// The general palette entry of the main strand A (claim 10).
pub const MAIN_ENTRY: usize = 15; // spec: S-0316 claim 10 and Constants (strand A colour entry 15)

/// The general palette's neutral shade row (claims 10 and 11).
pub const NEUTRAL_ROW: usize = 26; // spec: S-0316 claims 10 and 11 (the neutral shade row 26 of 53, S-0104 claim 15)

/// The depth constant of F4.
const DEPTH: i32 = 2; // spec: S-0316 F4 and Constants (depth constant −2)

/// The double nearest π (F2's argument φ × π / (*i* + 7)).
const PI: F64 = F64(0x4009_21FB_5444_2D18); // spec: S-0316 F2 (φ × π / (i + 7), double precision)

/// Strands A, B and C (claim 9), by index.
const A: usize = 0; // nocite: an index, not a value
const B: usize = 1; // nocite: an index, not a value
const C: usize = 2; // nocite: an index, not a value

/// A world point as three integer leptons.
type P3 = [i64; 3];

/// One electric bolt (S-0316 Terms and claims 3 to 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bolt {
    /// The firer.
    pub firer: ObjectId,
    /// The weapon slot that fired (claim 4: an attached bolt follows that
    /// slot's fire point).
    pub slot: usize,
    /// The shot's burst counter (claim 3: the side of the shot's own
    /// projectile in every frame).
    pub burst: i32,
    /// The bolt start: moved each frame while attached (claim 4).
    pub start: Pos,
    /// The bolt end, fixed when the bolt is made (claim 3).
    pub end: Pos,
    /// Whether the start follows the firer (claim 4).
    pub attached: bool,
    /// The phase φ drawn when the bolt was made (Terms, claim 13).
    pub phase: i32,
    /// The tick whose drawn frame is the bolt's first (claim 5).
    pub born: i32,
    /// The weapon's alternate-colour flag (claim 10).
    pub alt: bool,
    /// The building-source adjustment *a* of F4 (0 for a firer that is not
    /// a building).
    pub a: i32,
}

impl Bolt {
    /// A new bolt drawing its phase φ = R(0, 256) from `draw` (claim 13).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        firer: ObjectId,
        slot: usize,
        burst: i32,
        start: Pos,
        end: Pos,
        attached: bool,
        born: i32,
        alt: bool,
        a: i32,
        draw: &mut dyn FnMut(i32, i32) -> i32,
    ) -> Self {
        // spec: S-0316 claim 13 (making the bolt takes one draw, the phase φ = R(0, 256))
        let phase = draw(0, PHASE_TOP);
        Self {
            firer,
            slot,
            burst,
            start,
            end,
            attached,
            phase,
            born,
            alt,
            a,
        }
    }

    /// The bolt's drawn frame *n* at tick `now` (0 to 16), or `None` when
    /// it is not yet made or gone (claim 5).
    pub fn frame(&self, now: i32) -> Option<i32> {
        let n = now - self.born;
        // spec: S-0316 claim 5 (drawn in 17 consecutive drawn frames, gone from the 18th)
        (0..LIFE).contains(&n).then_some(n)
    }

    /// The phase in drawn frame `n`: φ + *n*, raised in every drawn frame of
    /// the life whether or not the bolt was drawn (Terms, claim 5).
    pub fn phase_at(&self, n: i32) -> i32 {
        // spec: S-0316 Terms (drawn frame n of the life uses φ + n) and claim 5 (frames out of view raise the phase too)
        self.phase + n
    }
}

/// The bolt length λ between `p` and `q` in leptons (S-0316 Terms): the
/// truncated table root of the exact sum of squares, under `table`
/// (`quirks.table_root_distance`) and `mode` (`quirks.chop_rounding`).
pub fn length(p: Pos, q: Pos, table: bool, mode: Rounding) -> i64 {
    let d = |a: i32, b: i32| i64::from(b) - i64::from(a);
    let (dx, dy, dz) = (d(p.x, q.x), d(p.y, q.y), d(p.z, q.z));
    // spec: S-0316 Terms (the sum of squares, exact below 2⁵³, height term included; λ = trunc(√̃(Δx² + Δy² + Δz²)), S-0204 claim 5)
    let n = (dx * dx + dy * dy + dz * dz) as u64;
    glasnost_num::geom::root_floor(n, table, mode) as i64
}

/// The number of split levels *D* of a bolt of length `lambda` (F1).
pub fn levels(lambda: i64) -> u32 {
    let mut d = 0;
    // spec: S-0316 F1 (D = min(8, the least d ≥ 0 with ⌊λ / 2^d⌋ ≤ 64))
    while d < MOST_LEVELS && (lambda >> d) > UNSPLIT {
        d += 1;
    }
    d
}

/// The number of draws a drawn frame takes for a bolt with *D* levels
/// (claim 13: 9 per split, 9 × (2^*D* − 1)).
pub fn draws_per_frame(d: u32) -> i64 {
    // spec: S-0316 claim 13 (9 draws per split: 3 for strand A, then 3 for each of B and C)
    9 * ((1i64 << d) - 1)
}

/// ⌊v / 2⌋, rounding towards negative infinity (F1).
fn half(v: i64) -> i64 {
    // spec: S-0316 F1 (every halving written ⌊v / 2⌋ rounds towards negative infinity)
    v.div_euclid(2)
}

/// The sway shift of F2 at the first split, from phase `phase` and the
/// jitter amplitude `j0`: (Δx, Δy, Δz).
pub fn sway(phase: i32, j0: i64, mode: Rounding) -> P3 {
    let num = |v: i64| F64::from_i64(v, mode);
    // spec: S-0316 F2 (sᵢ = trunc(sinT(φ × π / (i + 7)) × J₀), double precision, the table sine of S-0105 F1)
    let s = |i: i64| -> i64 {
        let arg = num(i64::from(phase))
            .mul(PI, mode)
            .div(num(i + SWAY_DIVISOR), mode);
        glasnost_num::trig::sin_t(arg, mode)
            .mul(num(j0), mode)
            .trunc_i128()
            .map_or(0, |x| x as i64)
    };
    // spec: S-0316 F2 (Δx = s₀ + s₃, Δy = s₁ + s₅, Δz = ⌊(s₄ + 2 J₀ + s₂) / 2⌋)
    [s(0) + s(3), s(1) + s(5), half(s(4) + 2 * j0 + s(2))]
}

/// One piece of a bolt at its finest level (F3): the end points of strands
/// A, B and C, and the depth terms *e* of F4 at its two ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    /// Strands A, B and C: (*p*, *q*).
    pub strands: [(P3, P3); 3],
    /// The depth terms *e*(*p*) and *e*(*q*).
    pub e: (i64, i64),
}

/// The bolt's shape in one drawn frame (claim 9, F1 to F4): its pieces in
/// order from the start towards the end, with `draw` taking the ranged
/// draws (claim 13) and `a` the building-source adjustment of F4. A bolt of
/// length 0 has no pieces and takes no draws.
pub fn shape(
    start: Pos,
    end: Pos,
    phase: i32,
    a: i32,
    table: bool,
    mode: Rounding,
    draw: &mut dyn FnMut(i32, i32) -> i32,
) -> Vec<Piece> {
    let lambda = length(start, end, table, mode);
    // spec: S-0316 claim 9 (a bolt of length 0 is not drawn)
    if lambda == 0 {
        return Vec::new();
    }
    let d = levels(lambda);
    // spec: S-0316 F1 (J₀ = ⌊23 λ / 256⌋, P = ⌊102 λ / 256⌋)
    let j0 = (JITTER * lambda).div_euclid(FACTOR_SCALE);
    let pin = (PINNING * lambda).div_euclid(FACTOR_SCALE);
    let p3 = |p: Pos| [i64::from(p.x), i64::from(p.y), i64::from(p.z)];
    // spec: S-0316 F3 (each strand starts as the straight line from the bolt start to the bolt end)
    let (s, t) = (p3(start), p3(end));
    let mut ctx = Split {
        lambda,
        levels: d,
        j0,
        pin,
        sway: sway(phase, j0, mode),
        draw,
        out: Vec::with_capacity(1 << d),
    };
    // spec: S-0316 F4 (e = a at the start, 0 at the end)
    ctx.split(0, [s; 3], [t; 3], (i64::from(a), 0));
    ctx.out
}

/// The state of one frame's midpoint displacement.
struct Split<'a> {
    lambda: i64,
    levels: u32,
    j0: i64,
    pin: i64,
    sway: P3,
    draw: &'a mut dyn FnMut(i32, i32) -> i32,
    out: Vec<Piece>,
}

impl Split<'_> {
    /// A ranged draw R(−*j*, *j*) (S-0202 claim 4).
    fn jitter(&mut self, j: i64) -> i64 {
        let j = j as i32;
        i64::from((self.draw)(-j, j))
    }

    /// Splits the pieces of the three strands from `p` to `q` at level `k`
    /// (F3), depth first, parent before children, the half towards the
    /// start first (claim 13); `e` the depth terms at their ends (F4).
    fn split(&mut self, k: u32, p: [P3; 3], q: [P3; 3], e: (i64, i64)) {
        // spec: S-0316 F1 (a piece is split while ℓₖ > 64 and k < 8; every piece at a level is split or none is)
        if k == self.levels {
            self.out.push(Piece {
                strands: [(p[A], q[A]), (p[B], q[B]), (p[C], q[C])],
                e,
            });
            return;
        }
        // spec: S-0316 F1 (ℓₖ = ⌊λ / 2^k⌋, Jₖ = ⌊J₀ / 2^k⌋)
        let l = self.lambda >> k;
        let j = self.j0 >> k;
        // spec: S-0316 F3 (m = (⌊(pₓ + qₓ) / 2⌋, ⌊(p_y + q_y) / 2⌋, ⌊(p_z + q_z) / 2⌋))
        let mid =
            |p: P3, q: P3| -> P3 { [half(p[0] + q[0]), half(p[1] + q[1]), half(p[2] + q[2])] };
        // spec: S-0316 F2 (at the first split only, the same shift is added to each strand's midpoint, after the plain midpoint and before the jitter)
        let shift = if k == 0 { self.sway } else { [0; 3] };
        let plain = |s: usize| -> P3 {
            let m = mid(p[s], q[s]);
            [m[0] + shift[0], m[1] + shift[1], m[2] + shift[2]]
        };
        let mut m = [[0i64; 3]; 3];
        // spec: S-0316 claim 13 (3 draws for strand A, one per world axis x, y, z, before B's and C's)
        let mut ma = plain(A);
        for v in &mut ma {
            // spec: S-0316 F3 (strand A: R(−Jₖ, Jₖ) when ℓₖ > 128, and 2 Jₖ × R(−1, 1) when ℓₖ ≤ 128)
            *v += if l > COARSE {
                self.jitter(j)
            } else {
                2 * j * self.jitter(1)
            };
        }
        m[A] = ma;
        // spec: S-0316 claim 13 (then 3 draws for each of B and C; B before C and the axes x, y, z are our engine's order, design)
        for s in [B, C] {
            // spec: S-0316 F3 (ℓₖ > P: strand A's displaced midpoint plus ⌊R(−Jₖ, Jₖ) / 2⌋ per axis; ℓₖ ≤ P: its own midpoint plus R(−Jₖ, Jₖ) per axis)
            let pinned = l > self.pin;
            let mut ms = if pinned { ma } else { plain(s) };
            for v in &mut ms {
                let r = self.jitter(j);
                *v += if pinned { half(r) } else { r };
            }
            m[s] = ms;
        }
        // spec: S-0316 F4 (e(m) = ⌊(e(p) + e(q)) / 2⌋)
        let em = half(e.0 + e.1);
        // spec: S-0316 F3 (the two halves p–m and m–q are then split in turn at level k + 1) and claim 13 (depth first, the half towards the start first)
        self.split(k + 1, p, m, (e.0, em));
        self.split(k + 1, m, q, (em, e.1));
    }
}

/// Whether the straight screen line from `a` to `b` meets the view whose
/// top-left world pixel is `origin` and whose size is `size` (claim 5's
/// view test, on the straight line between the screen positions of the
/// two ends, not on the drawn strands).
pub fn line_in_view(a: (i32, i32), b: (i32, i32), origin: (i32, i32), size: (u32, u32)) -> bool {
    if size.0 == 0 || size.1 == 0 {
        return false;
    }
    let (x0, y0) = (i64::from(origin.0), i64::from(origin.1));
    let (x1, y1) = (x0 + i64::from(size.0) - 1, y0 + i64::from(size.1) - 1);
    let (ax, ay, bx, by) = (
        i64::from(a.0),
        i64::from(a.1),
        i64::from(b.0),
        i64::from(b.1),
    );
    // The bounding boxes must overlap.
    if ax.max(bx) < x0 || ax.min(bx) > x1 || ay.max(by) < y0 || ay.min(by) > y1 {
        return false;
    }
    // And the view's corners must not all lie strictly on one side of the line.
    let side = |x: i64, y: i64| (bx - ax) * (y - ay) - (by - ay) * (x - ax);
    let corners = [side(x0, y0), side(x1, y0), side(x0, y1), side(x1, y1)];
    !(corners.iter().all(|&c| c > 0) || corners.iter().all(|&c| c < 0))
}

/// The bolt colours from the general palette's neutral shade row: strands
/// B and C, B and C with the alternate-colour flag, and A (claim 10).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Colours {
    /// Entry 10.
    pub side: [u8; 3],
    /// Entry 5.
    pub alt_side: [u8; 3],
    /// Entry 15.
    pub main: [u8; 3],
}

impl Colours {
    /// The colours from the general palette file's bytes (`PALETTE.PAL`):
    /// the untinted table's row 26 (claim 11, S-0104 claim 6), `None` when
    /// the file is unreadable.
    pub fn from_palette_file(bytes: &[u8]) -> Option<Self> {
        let p = glasnost_fmt::pal::Palette6::parse(bytes).ok()?;
        // spec: S-0306 Terms and S-0103 claim 1 (the general palette: the file's components multiplied by 4)
        let pal = glasnost_render::palette::palette_from_file(
            &p,
            &glasnost_render::palette::PaletteQuirks::default(),
        );
        // spec: S-0316 claim 11 (row 26 of S-0104 claim 6's untinted table reproduces the palette file's colour)
        let t = glasnost_render::light::untinted_table(&pal);
        let row = t.get(NEUTRAL_ROW)?;
        let rgb = |i: usize| [row[i].r, row[i].g, row[i].b];
        Some(Self {
            side: rgb(SIDE_ENTRY),
            alt_side: rgb(ALT_SIDE_ENTRY),
            main: rgb(MAIN_ENTRY),
        })
    }
}

/// The spots of one piece strand from `p` to `q` with depth terms `e` and
/// colour `rgb` (claim 11, F4, S-0514 claim 25 and F8).
fn strand_spots(p: P3, q: P3, e: (i64, i64), rgb: [u8; 3], out: &mut Vec<Spot>) {
    let at = |v: P3| (v[0] as i32, v[1] as i32, v[2] as i32);
    let (pp, qq) = (at(p), at(q));
    // spec: S-0316 claim 11 (each piece is a solid line one pixel wide between the screen positions of its two points, each with its own height lift)
    let a = project_lifted(pp.0, pp.1, pp.2);
    let b = project_lifted(qq.0, qq.1, qq.2);
    // spec: S-0316 F4 (b(p) = e(p) − L(p_z) − 2, b(q) = e(q) − L(q_z) − 2, at the displaced heights)
    let bp = e.0 as i32 - height_lift(pp.2) - DEPTH;
    let bq = e.1 as i32 - height_lift(qq.2) - DEPTH;
    // spec: S-0514 F8 (the whole segment uses the bias of its leftmost end; our interim: unclipped)
    let bias = if b.0 < a.0 { bq } else { bp };
    out.extend(
        super::beams::segment_pixels(a, b)
            .into_iter()
            .map(|(x, y)| Spot {
                x,
                y,
                bias,
                // spec: S-0316 claim 11 (the per-pixel rules of S-0514 claim 25 on the palette entry's colour)
                kind: SpotKind::Flash { rgb },
            }),
    );
}

/// The spots of a bolt's pieces in drawing order (claim 12): from the start
/// towards the end, each piece's strands B, then C, then A.
pub fn spots(pieces: &[Piece], alt: bool, colours: &Colours) -> Vec<Spot> {
    // spec: S-0316 claim 10 (strands B and C entry 10, or 5 with the alternate-colour flag; A always 15)
    let side = if alt { colours.alt_side } else { colours.side };
    let mut out = Vec::new();
    for piece in pieces {
        // spec: S-0316 claim 12 (within each piece B, then C, then A; pieces from the start towards the end)
        for (s, rgb) in [(B, side), (C, side), (A, colours.main)] {
            let (p, q) = piece.strands[s];
            strand_spots(p, q, piece.e, rgb, &mut out);
        }
    }
    out
}

#[cfg(test)]
#[path = "bolts_tests.rs"]
mod tests;
