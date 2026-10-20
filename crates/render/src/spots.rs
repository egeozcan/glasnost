//! Single-pixel drawings: the pixels of beams (S-0518 claims 20 to 23),
//! beam flashes (S-0514 claim 25) and the magnetic wave (S-0607 claim 25).
//!
//! A [`Spot`] changes one screen pixel by an integer rule that reads the
//! pixel already drawn there. Spots come in [`SpotGroup`]s placed in the
//! scene's draw order; a group's spots are applied one after another, so a
//! pixel that two spots hit gets both transformations in order (S-0518
//! claims 21 and 22: *C*(*O*₂(*O*₁(*d*)))). Every spot reads depth and never
//! writes it; it passes where its depth, the view row's depth plus its bias,
//! is strictly less than the stored depth (S-0518 claim 20, S-0514 F8,
//! S-0607 claim 25).
//!
//! The CPU reference ([`apply`]) and the GPU path (`map_gpu`'s spot
//! pipeline) compute the same pixels.

use crate::compose::Frame;
use crate::scene::Group;

/// What a spot does to its pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpotKind {
    /// Replaces the pixel with a 5-6-5 colour; light-alpha does not apply
    /// (S-0518 claim 23, opaque line).
    Opaque {
        /// The colour, 5-6-5.
        colour: u16,
    },
    /// Adds an already intensity-scaled colour by S-0518 F7.
    Additive {
        /// The scaled line colour, 8 bits per channel.
        rgb: [u8; 3],
    },
    /// Mixes a colour in by S-0518 F8.
    Blended {
        /// The line colour, 8 bits per channel.
        rgb: [u8; 3],
        /// *T*[*w*], the line colour's easing weight.
        line: u16,
        /// *T*[256 − *w*], the screen colour's easing weight.
        screen: u16,
        /// Offset from this pixel to the pixel whose light-alpha value the
        /// pair uses (S-0518 claim 23: the first pixel's).
        alpha_from: (i8, i8),
    },
    /// A beam flash pixel (S-0514 claim 25).
    Flash {
        /// The segment colour, 8 bits per channel.
        rgb: [u8; 3],
    },
    /// A magnetic wave band pixel (S-0607 F5).
    Wave {
        /// Offset from this pixel to its source pixel.
        shift: (i8, i8),
        /// The boost factor *k*.
        k: u16,
    },
}

/// One pixel of a spot drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spot {
    /// World pixel.
    pub x: i32,
    /// World pixel row.
    pub y: i32,
    /// Depth bias added to the row's depth.
    pub bias: i32,
    /// What it does.
    pub kind: SpotKind,
}

/// Spots drawn together at one place of the scene's draw order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpotGroup {
    /// Drawn just before [`crate::scene::Scene::items`]`[at]`; at or past
    /// the end, after every item.
    pub at: usize,
    /// Group, for toggles.
    pub group: Group,
    /// Whether the drawing rests on a provisional choice (a spec request).
    pub provisional: bool,
    /// The pixels, in drawing order.
    pub spots: Vec<Spot>,
}

/// The magnetic wave's view-edge margin (S-0607 claim 25: not drawn within
/// 3 pixels of the battlefield view's edges).
pub const WAVE_MARGIN: i32 = 3; // spec: S-0607 claim 25 (we use 3 throughout)

/// Light-alpha scale (S-0518 Constants: 128 for F7 and F8).
const ALPHA_SCALE: u32 = 128; // spec: S-0518 Constants (light-alpha scale 128)

/// A 5-6-5 pixel widened to 8-bit channels by appending zero bits (S-0518
/// F7, F8; S-0514 F7; S-0607 F5; S-0103 claim 9).
pub fn widen(c: u16) -> [u32; 3] {
    let c = u32::from(c);
    [((c >> 11) & 31) << 3, ((c >> 5) & 63) << 2, (c & 31) << 3]
}

/// 8-bit channels reduced to 5-6-5 by dropping the low bits (S-0103
/// claim 9).
pub fn pack(c: [u32; 3]) -> u16 {
    (((c[0] >> 3) << 11) | ((c[1] >> 2) << 5) | (c[2] >> 3)) as u16
}

/// The easing table *T*[*i*] of S-0518 F8, for *i* = 0 … 256.
pub fn easing(i: u32) -> u16 {
    // spec: S-0518 F8 (T[i] = trunc(((i/128 − 1)³ + 1) × 128), in double precision, truncated toward zero)
    let x = f64::from(i.min(256)) / 128.0 - 1.0;
    ((x * x * x + 1.0) * 128.0) as u16
}

/// S-0518 F7: the screen pixel `d` with the scaled line colour `c` added at
/// light-alpha `a`.
pub fn additive(d: u16, c: [u8; 3], a: u8) -> u16 {
    let d = widen(d);
    // spec: S-0518 F7 (t = d + ⌊c × a / 128⌋)
    let mut t: [u32; 3] =
        std::array::from_fn(|i| d[i] + u32::from(c[i]) * u32::from(a) / ALPHA_SCALE);
    // spec: S-0518 F7 (k channels with t ≥ 255, X the sum of their t − 255)
    let over: Vec<usize> = (0..3).filter(|&i| t[i] >= 255).collect();
    let x: u32 = over.iter().map(|&i| t[i] - 255).sum();
    // spec: S-0518 F7 (spill ⌊X/2⌋ when k = 1, X when k = 2, to every channel with t < 255)
    let spill = match over.len() {
        1 => x / 2,
        2 => x,
        _ => 0,
    };
    for v in &mut t {
        if *v < 255 {
            *v += spill;
        }
    }
    // spec: S-0518 F7 (each channel clamped to 255, then reduced to 16-bit colour)
    pack(t.map(|v| v.min(255)))
}

/// S-0518 F8: the screen pixel `d` mixed with the line colour `c` by the
/// easing weights `line` = *T*[*w*] and `screen` = *T*[256 − *w*], at
/// light-alpha `a`.
pub fn blended(d: u16, c: [u8; 3], line: u16, screen: u16, a: u8) -> u16 {
    // spec: S-0518 F8 (a pixel with a = 0 is unchanged)
    if a == 0 {
        return d;
    }
    let mut v = widen(d);
    for i in 0..3 {
        // spec: S-0518 F8 (only channels with c > 0 change)
        if c[i] > 0 {
            // spec: S-0518 F8 (d′ = ⌊(d × T[256 − w] + c × T[w]) × a / 2¹⁵⌋)
            let s = v[i] * u32::from(screen) + u32::from(c[i]) * u32::from(line);
            v[i] = (s * u32::from(a)) >> 15;
        }
    }
    pack(v.map(|x| x.min(255)))
}

/// S-0514 claim 25: the flash pixel with segment colour `c` at light-alpha
/// `a`, or `None` where it is not drawn.
pub fn flash(c: [u8; 3], a: u8) -> Option<u16> {
    // spec: S-0514 claim 25 (not drawn when a = 0)
    if a == 0 {
        return None;
    }
    let packed = pack(c.map(u32::from));
    // spec: S-0514 claim 25 (with a = 127 the colour is unchanged)
    if a == 127 {
        return Some(packed);
    }
    // spec: S-0514 F7 (the 8-bit channel is the packed field restored with its dropped low bits zero)
    // spec: S-0514 claim 25 (each channel × a / 128, truncated, then capped at 255)
    let v = widen(packed).map(|ch| (ch * u32::from(a) / ALPHA_SCALE).min(255));
    Some(pack(v))
}

/// S-0607 F5: the band pixel made from source pixel `src` with boost `k`.
pub fn wave(src: u16, k: u16) -> u16 {
    let [r, g, b] = widen(src);
    let k = u32::from(k);
    // spec: S-0607 F5 (R′ = min(255, R + ⌊(k / 2) × R / 256⌋); G′ = G; B′ = min(255, B + ⌊4k × B / 256⌋))
    let r = (r + (k / 2) * r / 256).min(255);
    let b = (b + 4 * k * b / 256).min(255);
    pack([r, g, b])
}

/// The view pixel of `s` in `frame` for a view whose top-left world pixel
/// is `origin`, when it lies inside the view.
fn pixel(frame: &Frame, s: &Spot, origin: (i32, i32)) -> Option<(i32, i32)> {
    let (x, y) = (s.x - origin.0, s.y - origin.1);
    let (w, h) = (frame.width as i32, frame.height as i32);
    if x < 0 || y < 0 || x >= w || y >= h {
        return None;
    }
    if matches!(s.kind, SpotKind::Wave { .. })
        && (x < WAVE_MARGIN || y < WAVE_MARGIN || x >= w - WAVE_MARGIN || y >= h - WAVE_MARGIN)
    {
        return None;
    }
    Some((x, y))
}

/// The depth a spot at view row `r` is compared with: the row's depth plus
/// the bias, modulo 65536 for a beam flash (S-0514 F8), otherwise held to
/// the depth range as every drawing's depth is.
pub fn spot_depth(frame: &Frame, r: i32, s: &Spot) -> i32 {
    let v = frame.row_depth(r) + s.bias;
    match s.kind {
        // spec: S-0514 F8 (a pixel on row r passes when (d(r) + b) mod 65536 is less than the stored depth)
        SpotKind::Flash { .. } => v.rem_euclid(65536),
        _ => v.clamp(0, 65535),
    }
}

/// Applies `group` to `frame`, a view whose top-left world pixel is
/// `origin` (the CPU reference).
pub fn apply(frame: &mut Frame, group: &SpotGroup, origin: (i32, i32)) {
    // spec: S-0607 claim 25 (source pixels are read from the picture as it stood before the wave drew)
    let before = group
        .spots
        .iter()
        .any(|s| matches!(s.kind, SpotKind::Wave { .. }))
        .then(|| frame.color.clone());
    let w = frame.width as i32;
    for s in &group.spots {
        let Some((x, y)) = pixel(frame, s, origin) else {
            continue;
        };
        let i = (y * w + x) as usize;
        // spec: S-0518 claim 20 (a sample contributes only where its depth is strictly less than the stored depth; no depth write)
        if spot_depth(frame, y, s) >= i32::from(frame.depth[i]) {
            continue;
        }
        let a = frame.alpha[i];
        let d = frame.color[i];
        frame.color[i] = match s.kind {
            // spec: S-0518 claim 23 (opaque: replaced by the line's colour; light-alpha does not apply)
            SpotKind::Opaque { colour } => colour,
            SpotKind::Additive { rgb } => additive(d, rgb, a),
            SpotKind::Blended {
                rgb,
                line,
                screen,
                alpha_from: (dx, dy),
            } => {
                // spec: S-0518 claim 23 (a sample pair uses the first pixel's light-alpha for both pixels)
                // Where the first pixel lies outside the view, the nearest view pixel's value
                // stands in: clipped coverage is an open question of S-0518.
                let ax = (x + i32::from(dx)).clamp(0, w - 1);
                let ay = (y + i32::from(dy)).clamp(0, frame.height as i32 - 1);
                blended(d, rgb, line, screen, frame.alpha[(ay * w + ax) as usize])
            }
            SpotKind::Flash { rgb } => match flash(rgb, a) {
                Some(c) => c,
                None => continue,
            },
            SpotKind::Wave { shift: (dx, dy), k } => {
                let (sx, sy) = (x + i32::from(dx), y + i32::from(dy));
                let src = before.as_ref().map_or(d, |b| b[(sy * w + sx) as usize]);
                wave(src, k)
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::DEPTH_REFERENCE;

    fn frame(w: u32, h: u32) -> Frame {
        Frame::new(w, h, 0, DEPTH_REFERENCE)
    }

    fn group(spots: Vec<Spot>) -> SpotGroup {
        SpotGroup {
            at: 0,
            group: Group::Effects,
            provisional: false,
            spots,
        }
    }

    fn spot(x: i32, y: i32, kind: SpotKind) -> Spot {
        Spot {
            x,
            y,
            bias: 0,
            kind,
        }
    }

    /// spec: S-0518 F8 (T[i] = trunc(((i/128 − 1)³ + 1) × 128))
    #[test]
    fn easing_table_ends_and_middle() {
        assert_eq!(easing(0), 0);
        assert_eq!(easing(128), 128);
        assert_eq!(easing(256), 256);
        assert_eq!(easing(153), 128);
        assert_eq!(easing(103), 127);
    }

    /// spec: S-0518 F8 and scenario LB6 (inner 129 at w = 153 over black gives 63 per channel)
    #[test]
    fn blended_lb6_vector() {
        let c = blended(0, [129, 129, 129], easing(153), easing(103), 127);
        assert_eq!(widen(c), [56, 60, 56]);
        assert_eq!(c, pack([63, 63, 63]));
    }

    /// spec: S-0518 F8 (only channels with c > 0 change; a = 0 leaves the pixel unchanged)
    #[test]
    fn blended_keeps_zero_channels_and_shroud() {
        let d = pack([200, 100, 40]);
        let c = blended(d, [255, 0, 0], easing(256), easing(0), 127);
        assert_eq!(widen(c)[1..], widen(d)[1..]);
        assert_ne!(widen(c)[0], widen(d)[0]);
        assert_eq!(blended(d, [255, 255, 255], 256, 0, 0), d);
    }

    /// spec: S-0518 F7 (t = d + ⌊c a / 128⌋ on channels widened with zero bits)
    #[test]
    fn additive_adds_scaled_by_alpha() {
        let d = pack([16, 32, 64]);
        assert_eq!(
            additive(d, [64, 64, 64], 127),
            pack([16 + 63, 32 + 63, 64 + 63])
        );
        assert_eq!(additive(d, [64, 64, 64], 64), pack([48, 64, 96]));
        // spec: S-0518 claim 23 (where a = 0 nothing is added)
        assert_eq!(additive(d, [255, 255, 255], 0), d);
    }

    /// spec: S-0518 F7 (spill ⌊X/2⌋ for one overflowing channel, X for two)
    #[test]
    fn additive_spills_overflow() {
        // One channel over: red t = 248 + 127 = 375, X = 120, spill 60.
        let d = pack([248, 0, 0]);
        assert_eq!(widen(additive(d, [128, 0, 0], 128)), [248, 60 & !3, 56]);
        // Two over: red 375, green 252 + 127 = 379; X = 120 + 124 = 244.
        let d = pack([248, 252, 0]);
        assert_eq!(widen(additive(d, [128, 128, 0], 128)), [248, 252, 240]);
        // Three over: no spill, all clamp.
        let d = pack([248, 252, 248]);
        assert_eq!(widen(additive(d, [128, 128, 128], 128)), [248, 252, 248]);
    }

    /// spec: S-0514 claim 25 (a = 0 not drawn; a = 127 unchanged; otherwise × a / 128, capped)
    #[test]
    fn flash_pixel_rules() {
        assert_eq!(flash([200, 100, 50], 0), None);
        assert_eq!(flash([200, 100, 50], 127), Some(pack([200, 100, 50])));
        assert_eq!(flash([200, 100, 50], 64), Some(pack([100, 50, 24])));
        assert_eq!(flash([200, 100, 50], 255), Some(pack([255, 199, 95])));
    }

    /// spec: S-0607 F5 (R and B boosted, G kept; zero channels stay zero)
    #[test]
    fn wave_boost() {
        let src = pack([100, 100, 100]);
        // k = 126: R′ = 96 + ⌊63 × 96 / 256⌋ = 119; B′ = min(255, 96 + ⌊504 × 96 / 256⌋) = 255.
        assert_eq!(widen(wave(src, 126)), [112, 100, 248]);
        assert_eq!(wave(pack([0, 200, 0]), 182), pack([0, 200, 0]));
    }

    /// spec: S-0518 claim 20 (strict depth test, no depth write) and claim 21 (overlaps compose in order)
    #[test]
    fn apply_tests_depth_and_composes_in_order() {
        let mut f = frame(4, 2);
        let d0 = f.row_depth(0);
        f.depth[1] = d0 as u16; // equal: fails the strict test
        f.depth[2] = (d0 + 1) as u16;
        let g = group(vec![
            spot(1, 0, SpotKind::Opaque { colour: 0xFFFF }),
            spot(2, 0, SpotKind::Additive { rgb: [64, 0, 0] }),
            spot(2, 0, SpotKind::Additive { rgb: [64, 0, 0] }),
        ]);
        let before = f.depth.clone();
        apply(&mut f, &g, (0, 0));
        assert_eq!(f.color[1], 0);
        assert_eq!(widen(f.color[2]), [112, 0, 0]);
        assert_eq!(f.depth, before);
    }

    /// spec: S-0514 F8 (depth (d(r) + b) mod 65536)
    #[test]
    fn flash_depth_wraps() {
        let f = frame(1, 1);
        let s = Spot {
            x: 0,
            y: 0,
            bias: 65536 - f.row_depth(0) + 5,
            kind: SpotKind::Flash { rgb: [255; 3] },
        };
        assert_eq!(spot_depth(&f, 0, &s), 5);
    }

    /// spec: S-0518 claim 23 (a pair uses the first pixel's light-alpha)
    #[test]
    fn blended_pair_reads_first_alpha() {
        let mut f = frame(1, 2);
        f.alpha = vec![0, 127];
        let g = group(vec![spot(
            0,
            1,
            SpotKind::Blended {
                rgb: [255, 255, 255],
                line: 256,
                screen: 0,
                alpha_from: (0, -1),
            },
        )]);
        apply(&mut f, &g, (0, 0));
        assert_eq!(f.color[1], 0);
    }

    /// spec: S-0607 claim 25 (sources from the picture before the wave; 3-pixel view-edge margin)
    #[test]
    fn wave_reads_before_and_keeps_margin() {
        let mut f = frame(8, 8);
        let grey = pack([100, 100, 100]);
        f.color = vec![grey; 64];
        let k = 110;
        let g = group(vec![
            spot(3, 3, SpotKind::Wave { shift: (1, 0), k }),
            spot(4, 3, SpotKind::Wave { shift: (1, 0), k }),
            spot(2, 3, SpotKind::Wave { shift: (1, 0), k }),
        ]);
        apply(&mut f, &g, (0, 0));
        assert_eq!(f.color[3 * 8 + 3], wave(grey, k));
        assert_eq!(f.color[3 * 8 + 4], wave(grey, k));
        assert_eq!(f.color[3 * 8 + 2], grey);
    }
}
