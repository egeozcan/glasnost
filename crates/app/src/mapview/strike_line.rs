//! The air strike's strike line (S-0519 claim 27, F8, #448): for each
//! designator whose strike holds a building as its strike target, a red
//! line from the designator's primary fire point to a jittered point on
//! the building, cut into 4 parts of falling intensity, with a plus-sign
//! end marker when the end lies higher on screen than the start.

use glasnost_render::projection::{height_lift, project_lifted};
use glasnost_render::spots::{Spot, SpotKind, pack};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{Pos, World};

use super::beams::segment_pixels;

/// The colour draw's range (S-0519 F8: *r* = R(190, 270)).
pub const DRAW: (i32, i32) = (190, 270); // spec: S-0519 F8 (one inclusive ranged draw r = R(190, 270))

/// The jitter's amplitude in leptons (S-0519 F8: 15).
const JITTER: f64 = 15.0; // spec: S-0519 F8 (E = (trunc(X + 15 cos(0.37 t)), Y, trunc(Z + 15 sin t)))
/// The X jitter's angle step (S-0519 F8: 0.37 radians a step).
const X_RATE: f64 = 0.37; // spec: S-0519 F8 (cos(0.37 t))
/// Ticks a jitter step (S-0519 F8: *t* = ⌊*n* / 4⌋ in our engine).
const STEP_TICKS: i32 = 4; // spec: S-0519 F8 (our engine: t = ⌊n / 4⌋, n the tick number)
/// The depth nearness of the line's ends (S-0519 F8: *b* = −*L*(*z*) − 32).
const NEARNESS: i32 = 32; // spec: S-0519 F8 (b₀ = −L(z₀) − 32, b₁ = −L(z₁) − 32)
/// The parts' centre-line intensities (S-0519 F8).
const CENTRE: [u8; 4] = [255, 192, 128, 64]; // spec: S-0519 F8 (centre-line intensity α of parts 1 to 4)
/// The parts' side-line intensities (S-0519 F8).
const SIDE: [u8; 4] = [100, 75, 50, 25]; // spec: S-0519 F8 (side-line intensity α of parts 1 to 4)

/// The ends, start and end, in world leptons, of every strike line of `w`
/// in the drawn frame of tick `n` (S-0519 claim 27): one for each
/// designator whose strike target is a building, in designator order
/// (design: render (#448): the order between two lines is not asserted,
/// S-0519 Open questions).
pub fn lines(w: &World, n: i32) -> Vec<(Pos, Pos)> {
    w.strike
        .strikes
        .iter()
        .filter_map(|(&d, s)| {
            // spec: S-0519 claim 27 (drawn exactly while the strike target is a building; no test of house, visibility or the designator's place)
            let b = s.target?;
            w.obj(b).filter(|o| o.kind == Kind::Building)?;
            let firer = w.obj(d)?;
            // spec: S-0519 claim 27 (start: the fire point of a shot from the primary slot in that drawn frame, burst-counter parity included; S-0313)
            let from = glasnost_sim::combat::position::fire_point(w, d, 0, firer.combat.burst);
            // spec: S-0519 claim 27 (end: the strike target's reference point, S-0204 claim 4, jittered by F8)
            Some((from, end_point(w.reference_point(b), n)))
        })
        .collect()
}

/// The end point *E* of F8 for reference point `r` at tick `n`.
pub fn end_point(r: Pos, n: i32) -> Pos {
    // spec: S-0519 F8 (our engine: t = ⌊n / 4⌋)
    let t = f64::from(n.div_euclid(STEP_TICKS));
    // spec: S-0519 F8 (the sums in binary64, truncated toward zero; sine and cosine in binary64, our design)
    let x = (f64::from(r.x) + JITTER * (X_RATE * t).cos()).trunc() as i32;
    let z = (f64::from(r.z) + JITTER * t.sin()).trunc() as i32;
    // spec: S-0519 F8 (never along world Y)
    Pos::new(x, r.y, z)
}

/// The line colour *C* of F8 for draw `r`, 8 bits per channel.
pub fn colour(r: i32) -> [u8; 3] {
    if r <= 255 {
        // spec: S-0519 F8 (C = (r, 0, 0) when r ≤ 255)
        [r.clamp(0, 255) as u8, 0, 0]
    } else {
        // spec: S-0519 F8 (C = (255, trunc(110k / 14), trunc(30k / 14)) with k = r − 256 when r ≥ 256)
        let k = r - 256;
        [
            255,
            (110 * k / 14).clamp(0, 255) as u8,
            (30 * k / 14).clamp(0, 255) as u8,
        ]
    }
}

/// The end marker's colour (S-0519 claim 27: opaque pure red (255, 0, 0)).
fn marker_colour() -> u16 {
    // spec: S-0519 claim 27 (opaque pure red (255, 0, 0))
    pack([255, 0, 0])
}

/// The pixels of the strike line from world point `from` to `to` in colour
/// `rgb`: the end marker's spots, drawn first (S-0519 claim 27: before the
/// line), empty unless the end lies above the start on screen, and the
/// line's spots in drawing order.
pub fn spots(from: Pos, to: Pos, rgb: [u8; 3], over_shroud: bool) -> (Vec<Spot>, Vec<Spot>) {
    // spec: S-0519 claim 27 (from the start's screen point to the end's, S-0101 projection, height lift included)
    let s0 = project_lifted(from.x, from.y, from.z);
    let s4 = project_lifted(to.x, to.y, to.z);
    let (dx, dy) = (i64::from(s4.0 - s0.0), i64::from(s4.1 - s0.1));
    // spec: S-0519 F8 (sₖ = s₀ + (trunc(kΔx / 4), trunc(kΔy / 4)))
    let s: [(i32, i32); 5] = std::array::from_fn(|k| {
        (
            s0.0 + (k as i64 * dx / 4) as i32,
            s0.1 + (k as i64 * dy / 4) as i32,
        )
    });
    // spec: S-0519 F8 (b₀ = −L(z₀) − 32, b₁ = −L(z₁) − 32; point sₖ has b₀ + trunc(k(b₁ − b₀) / 4))
    let (b0, b1) = (
        -height_lift(from.z) - NEARNESS,
        -height_lift(to.z) - NEARNESS,
    );
    let b: [i32; 5] = std::array::from_fn(|k| b0 + (k as i32) * (b1 - b0) / 4);
    // spec: S-0519 F8 (side lines moved by (0, ∓1) when |Δx| > |Δy|, otherwise by (∓1, 0))
    let sides = if dx.abs() > dy.abs() {
        [(0, -1), (0, 1)]
    } else {
        [(-1, 0), (1, 0)]
    };
    let mut marker = Vec::new();
    if s4.1 < s0.1 {
        let (x, y) = s4;
        let kind = SpotKind::Marker {
            colour: marker_colour(),
            over_shroud,
        };
        let at = |x, y| Spot {
            x,
            y,
            bias: 0,
            kind,
        };
        // spec: S-0519 F8 (end marker: rows y and y + 1 from column x − 2 to x + 1, and columns x − 1 and x from row y − 1 to y + 2, each line with its end pixels)
        for row in [y, y + 1] {
            marker.extend((x - 2..=x + 1).map(|c| at(c, row)));
        }
        for col in [x - 1, x] {
            marker.extend((y - 1..=y + 2).map(|r| at(col, r)));
        }
    }
    let mut line = Vec::new();
    for k in 1..=4 {
        let pixels = segment_pixels(s[k - 1], s[k]);
        let h = (pixels.len() as i32 - 1).max(1);
        // design: render (#448): a part after the first leaves out its first pixel, the last one
        // of the part before; the interior bias runs linearly from one point's bias to the next
        // along the part's steps (S-0519 Open questions: exact interior sampling is open)
        let skip = usize::from(k > 1);
        let bias = |j: usize| b[k - 1] + (j as i32) * (b[k] - b[k - 1]) / h;
        let mut part = |alpha: u8, (ox, oy): (i32, i32)| {
            for (j, &(x, y)) in pixels.iter().enumerate().skip(skip) {
                line.push(Spot {
                    x: x + ox,
                    y: y + oy,
                    bias: bias(j),
                    kind: SpotKind::Strike { rgb, alpha },
                });
            }
        };
        // design: render (#448): each part's centre line, then its two side lines
        part(CENTRE[k - 1], (0, 0));
        part(SIDE[k - 1], sides[0]);
        part(SIDE[k - 1], sides[1]);
    }
    (marker, line)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0519 F8 (C = (r, 0, 0) for r ≤ 255; (255, trunc(110k/14), trunc(30k/14)) above)
    #[test]
    fn colour_runs_from_dark_red_to_orange() {
        assert_eq!(colour(190), [190, 0, 0]);
        assert_eq!(colour(255), [255, 0, 0]);
        assert_eq!(colour(256), [255, 0, 0]);
        assert_eq!(colour(263), [255, 55, 15]);
        assert_eq!(colour(270), [255, 110, 30]);
    }

    /// spec: S-0519 F8 (t = ⌊n / 4⌋: the end point holds for 4 ticks; never along Y; at most 15 leptons)
    #[test]
    fn end_point_steps_every_four_ticks() {
        let r = Pos::new(1000, 2000, 300);
        assert_eq!(end_point(r, 0), Pos::new(1015, 2000, 300));
        for n in 0..64 {
            let e = end_point(r, n);
            assert_eq!(e, end_point(r, n - n % 4), "tick {n}");
            assert_eq!(e.y, 2000);
            assert!((e.x - r.x).abs() <= 15 && (e.z - r.z).abs() <= 15);
        }
        assert_ne!(end_point(r, 3), end_point(r, 4));
    }

    /// spec: S-0519 F8 (the end marker when s₄ lies above s₀; side lines one pixel above and below a wide line)
    #[test]
    fn marker_only_when_the_end_is_higher_on_screen() {
        let low = Pos::new(256 * 10, 256 * 10, 0);
        let high = Pos::new(256 * 4, 256 * 4, 0);
        let (m, line) = spots(low, high, [200, 0, 0], true);
        assert_eq!(m.len(), 16, "two rows of 4 and two columns of 4");
        assert!(!line.is_empty());
        let (m, _) = spots(high, low, [200, 0, 0], true);
        assert!(m.is_empty());
        // A wide line (Δx > Δy): the side lines lie above and below.
        let a = Pos::new(256 * 10, 256 * 4, 0);
        let b = Pos::new(256 * 4, 256 * 10, 0);
        let (_, line) = spots(a, b, [200, 0, 0], true);
        let s0 = project_lifted(a.x, a.y, a.z);
        assert!(line.iter().any(|p| (p.x, p.y) == (s0.0, s0.1 - 1)));
        assert!(line.iter().any(|p| (p.x, p.y) == (s0.0, s0.1 + 1)));
    }
}
