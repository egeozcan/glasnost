//! Tests of the line effects on synthetic values and small frames.

use super::*;
use glasnost_render::compose::{DEPTH_REFERENCE, Frame as Screen};
use glasnost_render::scene::Group;
use glasnost_render::spots::{SpotGroup, apply, widen};

const RZ: Rounding = Rounding::TowardZero;

fn pos(x: i32, y: i32, z: i32) -> Pos {
    Pos::new(x, y, z)
}

/// A beam between two screen points with flat biases.
fn beam(from: (i32, i32), to: (i32, i32), sector: u8, duration: i32, look: Look) -> Beam {
    Beam {
        from,
        to,
        bias: (-2, -2),
        sector,
        born: 0,
        duration,
        start: ONE,
        end: NONE,
        fade: true,
        look,
    }
}

/// Draws `spots` on a black `w × h` screen whose world origin is (0, 0).
fn draw(spots: Vec<Spot>, w: u32, h: u32) -> Screen {
    let mut s = Screen::new(w, h, 0, DEPTH_REFERENCE);
    apply(
        &mut s,
        &SpotGroup {
            at: usize::MAX,
            group: Group::Effects,
            provisional: false,
            spots,
        },
        (0, 0),
    );
    s
}

fn frame_with<'a>(
    detail: u8,
    fine: &'a mut dyn FnMut() -> bool,
    draw: &'a mut dyn FnMut(i32, i32) -> i32,
) -> super::Frame<'a> {
    super::Frame {
        detail,
        mode: RZ,
        fine,
        draw,
    }
}

/// spec: S-0518 claim 17, F5 (weak channels cut to 0, the rest scaled to length 240; n = 0 gives white)
#[test]
fn house_laser_colour() {
    assert_eq!(laser_colour([255, 0, 0]), [240, 0, 0]);
    assert_eq!(laser_colour([0, 0, 0]), [255, 255, 255]);
    assert_eq!(laser_colour([100, 100, 100]), [138, 138, 138]);
    // 50 · 240 / 206.2 = 58.2 < 96: cut to 0, then red alone scales to 240.
    assert_eq!(laser_colour([200, 50, 0]), [240, 0, 0]);
    assert_eq!(laser_colour([200, 150, 0]), [192, 144, 0]);
}

/// spec: S-0518 claim 18 (W and the supported flag by source; without the flag the weapon's colours and ordinary drawing)
#[test]
fn beam_parameters_by_source() {
    let colours = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    let l = [240, 0, 0];
    let house = |s| weapon_look(s, colours, true, l);
    assert_eq!(
        house(Source::Prism(0)),
        Look::House {
            colour: l,
            width: 3,
            supported: false
        }
    );
    assert_eq!(
        house(Source::Prism(2)),
        Look::House {
            colour: l,
            width: 5,
            supported: true
        }
    );
    assert!(matches!(
        house(Source::Building),
        Look::House { width: 1, .. }
    ));
    assert!(matches!(house(Source::Unit), Look::House { width: 2, .. }));
    // spec: S-0318 claim 8 (a fragment's beam has thickness 1 whatever the attacker; without the flag the fragment weapon's colours)
    assert_eq!(
        house(Source::Fragment),
        Look::House {
            colour: l,
            width: 1,
            supported: false
        }
    );
    assert_eq!(
        weapon_look(Source::Fragment, colours, false, l),
        Look::Ordinary {
            inner: [1, 2, 3],
            outer: [4, 5, 6],
            spread: [7, 8, 9],
            blink: false
        }
    );
    assert_eq!(
        weapon_look(Source::Unit, colours, false, l),
        Look::Ordinary {
            inner: [1, 2, 3],
            outer: [4, 5, 6],
            spread: [7, 8, 9],
            blink: false
        }
    );
    assert_eq!(
        support_look(l),
        Look::House {
            colour: l,
            width: 3,
            supported: false
        }
    );
}

/// spec: S-0518 claim 19, F6 (drawn for P = 0 … D − 1; I = (D − P) / D from 1 down to 1/D; D ≤ 0 never drawn)
#[test]
fn lifetime_and_intensity() {
    let b = beam((0, 0), (10, 0), 2, 5, support_look([240, 0, 0]));
    assert_eq!(b.age(0), Some(0));
    assert_eq!(b.age(4), Some(4));
    assert_eq!(b.age(5), None);
    assert_eq!(b.age(-1), None);
    let never = Beam { duration: 0, ..b };
    assert_eq!(never.age(0), None);
    assert_eq!(intensity(&b, 0, RZ), ONE);
    // 1/5 in single precision, rounded toward zero.
    let fifth = intensity(&b, 4, RZ).to_f64();
    assert_eq!(scaled(1 << 30, intensity(&b, 4, RZ), RZ), 214748352);
    assert!(fifth.lt(F64::from_i64(1, RZ)));
    // 3/5: LB6's intensity; 256 × I truncates to 153.
    assert_eq!(scaled(256, intensity(&b, 2, RZ), RZ), 153);
    let half = Beam { end: HALF, ..b };
    assert_eq!(scaled(256, intensity(&half, 4, RZ), RZ), 153);
}

/// spec: S-0518 claim 20 (sector: plan direction clockwise from north, nearest eighth)
#[test]
fn sectors() {
    assert_eq!(sector(0, -5), 0);
    assert_eq!(sector(5, -5), 1);
    assert_eq!(sector(5, 0), 2);
    assert_eq!(sector(5, 5), 3);
    assert_eq!(sector(0, 9), 4);
    assert_eq!(sector(-3, 3), 5);
    assert_eq!(sector(-7, 0), 6);
    assert_eq!(sector(-3, -3), 7);
    // 68.2° from north is nearer east; 63.4° nearer north-east.
    assert_eq!(sector(10, -4), 2);
    assert_eq!(sector(10, -5), 1);
}

/// spec: S-0518 claim 20 (b₀ = a − L(z₀) − 2, b₁ = −L(z₁) − 2; a building's a = min(0, yₛ − yₒ))
#[test]
fn end_biases() {
    let b = Beam::new(
        pos(1000, 1000, 208),
        pos(2000, 1000, 0),
        0,
        0,
        10,
        NONE,
        support_look([0; 3]),
    );
    // L(208) = 30.
    assert_eq!(b.bias, (-32, -2));
    assert_eq!(b.sector, 2);
    let tall = building_nearness(pos(1000, 1000, 832), pos(1000, 1000, 0));
    // A fire point lifted far above the point 128 leptons back: a < 0; at the building's foot a = 0.
    assert!(tall < 0);
    assert_eq!(building_nearness(pos(1000, 1000, 0), pos(1000, 1000, 0)), 0);
}

/// spec: S-0518 F9 and scenario LB7 (nearest-integer steps, half ties toward the first end; bias steps can dominate)
#[test]
fn unclipped_samples() {
    let xs: Vec<i32> = samples((0, 0, 0), (3, 0, 8)).iter().map(|s| s.0).collect();
    assert_eq!(xs, [0, 0, 1, 1, 1, 2, 2, 3]);
    let xs: Vec<i32> = samples((0, 0, 0), (4, 0, 8)).iter().map(|s| s.0).collect();
    assert_eq!(xs, [0, 0, 1, 1, 2, 2, 3, 3]);
    assert_eq!(
        samples((0, 0, 0), (2, 0, 4)),
        [(0, 0, 0), (0, 0, 1), (1, 0, 2), (1, 0, 3)]
    );
    // Reordered by screen x, the target end first; H = 0 gives nothing.
    assert_eq!(samples((2, 0, 0), (0, 0, 0)), [(0, 0, 0), (1, 0, 0)]);
    assert!(samples((5, 5, 1), (5, 5, 1)).is_empty());
}

/// spec: S-0518 F8 (slope one half: (256, 0), (128, 128), (0, 256), (128, 128), (0, 256))
#[test]
fn blended_coverage() {
    let c = coverage((0, 0, 0), (8, 4, 0));
    let q: Vec<(u32, u32)> = c.iter().take(5).map(|c| (c.q1, c.q2)).collect();
    assert_eq!(q, [(256, 0), (128, 128), (0, 256), (128, 128), (0, 256)]);
    assert_eq!(c[3].first, (3, 1));
    assert_eq!(c[3].second, (3, 2));
    // Axis-aligned: the first pixel alone has coverage.
    assert!(
        coverage((0, 0, 0), (0, -6, 0))
            .iter()
            .all(|c| c.q1 == 256 && c.q2 == 0)
    );
}

/// spec: S-0518 claim 22 (the activity test: f published every 60 units of 16 ms; floor m, margin z, shared low state)
#[test]
fn activity_test() {
    let mut a = Activity::default();
    assert_eq!(a.count(), 0);
    a.tick();
    a.check(0);
    // The first check publishes at once.
    assert_eq!(a.count(), 1);
    assert!(!a.test(15, 5));
    for t in 1..=20 {
        a.tick();
        a.check(t * 47);
    }
    // 940 ms: 58 units, not yet published.
    assert_eq!(a.count(), 1);
    a.tick();
    a.check(960);
    assert_eq!(a.count(), 21);
    // Low state set: needs m + z = 20.
    assert!(a.test(15, 5));
    // Cleared: f ≥ m passes.
    assert!(a.test(21, 5));
    assert!(!a.test(22, 5));
    assert!(!a.test(17, 5));
}

/// spec: S-0518 claim 21 (detail 0: opaque, second outer line over the first over the centre; spread draws R(−σ, σ) per channel)
#[test]
fn ordinary_detail_zero() {
    let look = Look::Ordinary {
        inner: [255, 0, 0],
        outer: [0, 0, 200],
        spread: [0, 0, 10],
        blink: false,
    };
    // Sector 2 (east): outer lines shifted (1, 0) and (0, 1).
    let b = beam((1, 1), (5, 1), 2, 10, look);
    let mut draws = Vec::new();
    let mut d = |a: i32, b: i32| {
        draws.push((a, b));
        b
    };
    let mut fine = || panic!("no activity test for an ordinary beam");
    let spots = beam_spots(&b, 0, &mut frame_with(0, &mut fine, &mut d));
    assert_eq!(draws, [(0, 0), (0, 0), (-10, 10)]);
    let s = draw(spots, 8, 4);
    let px = |x: usize, y: usize| widen(s.color[y * 8 + x]);
    // Centre row 1: x = 1 is centre only; x = 2…4 also the first outer line (shifted right): it wins.
    assert_eq!(px(1, 1), [248, 0, 0]);
    assert_eq!(px(2, 1), [0, 0, 208]);
    assert_eq!(px(1, 2), [0, 0, 208]);
}

/// spec: S-0518 claim 21 (no outer lines: no spread draw; detail 1 or 2 without outer lines: the centre blended at I)
#[test]
fn ordinary_blended_centre() {
    let look = Look::Ordinary {
        inner: [129, 129, 129],
        outer: [0, 0, 0],
        spread: [50, 50, 50],
        blink: false,
    };
    let b = beam((0, 0), (4, 0), 2, 5, look);
    let mut d = |_: i32, _: i32| -> i32 { panic!("no draw without outer lines") };
    let mut fine = || true;
    let spots = beam_spots(&b, 2, &mut frame_with(2, &mut fine, &mut d));
    let s = draw(spots, 6, 3);
    // LB6: inner 129 at I = 3/5 over black gives 63 per channel.
    assert_eq!(widen(s.color[2]), [56, 60, 56]);
    // The zero-coverage neighbour below takes the F8 mix with w = 0 (black stays black).
    assert_eq!(s.color[6 + 2], 0);
}

/// spec: S-0518 claim 21 (detail 1 or 2, outer lines: additive C(O₂(O₁(d))); blink draws only at odd P)
#[test]
fn ordinary_additive_and_blink() {
    let look = Look::Ordinary {
        inner: [0, 100, 0],
        outer: [100, 0, 0],
        spread: [0, 0, 0],
        blink: false,
    };
    // Sector 1: outer lines (0, −1) and (0, 1), so the three lines do not meet.
    let b = beam((0, 1), (3, 1), 1, 10, look);
    let mut d = |_: i32, _: i32| 0;
    let mut fine = || true;
    let spots = beam_spots(&b, 0, &mut frame_with(1, &mut fine, &mut d));
    let kinds: Vec<_> = spots.iter().map(|s| (s.y, s.kind)).collect();
    // O₁ first, then O₂, then the centre.
    assert_eq!(kinds[0], (0, SpotKind::Additive { rgb: [100, 0, 0] }));
    assert_eq!(
        kinds.last().copied(),
        Some((1, SpotKind::Additive { rgb: [0, 100, 0] }))
    );
    let s = draw(spots, 4, 3);
    // The centre row takes the inner colour alone, the rows above and below the outer.
    assert_eq!(widen(s.color[4]), [0, 96, 0]);
    assert_eq!(widen(s.color[0]), [96, 0, 0]);
    assert_eq!(widen(s.color[8]), [96, 0, 0]);
    let blink = Beam {
        look: Look::Ordinary {
            inner: [0, 100, 0],
            outer: [100, 0, 0],
            spread: [0, 0, 0],
            blink: true,
        },
        ..b
    };
    assert!(beam_spots(&blink, 0, &mut frame_with(1, &mut fine, &mut d)).is_empty());
    assert!(!beam_spots(&blink, 1, &mut frame_with(1, &mut fine, &mut d)).is_empty());
}

/// spec: S-0518 claim 21 (sector 5 uses the fixed pair (0, −1) and (0, 1))
#[test]
fn sector_five_fixed() {
    assert_eq!(OUTER_SHIFTS[5], [(0, -1), (0, 1)]);
}

/// spec: S-0518 claim 22 (pair shifts: i·eₐ and i·eᵦ on axis sectors, alternating steps on diagonal ones)
#[test]
fn house_pair_shifts() {
    assert_eq!(pair_shifts(1, 2), [(0, -2), (0, 2)]);
    assert_eq!(pair_shifts(3, 3), [(-3, 0), (3, 0)]);
    assert_eq!(pair_shifts(0, 1), [(-1, 0), (0, 1)]);
    assert_eq!(pair_shifts(0, 2), [(-1, -1), (1, 1)]);
    assert_eq!(pair_shifts(2, 3), [(-2, 1), (1, -2)]);
}

/// spec: S-0518 claim 22 (coarse mode: opaque lines moved toward white, c′ = c + trunc((255 − c) r / 256); pairs stop below 64)
#[test]
fn house_coarse() {
    let look = Look::House {
        colour: [100, 0, 0],
        width: 3,
        supported: false,
    };
    let b = beam((2, 4), (6, 4), 1, 10, look);
    let mut d = |_: i32, _: i32| 0;
    let mut fine = || false;
    let spots = beam_spots(&b, 0, &mut frame_with(2, &mut fine, &mut d));
    // L/2 = 50 is below 64: no pair, the centre only, at I = 1: 100 → 254.
    assert!(spots.iter().all(|s| s.y == 4));
    assert_eq!(
        spots[0].kind,
        SpotKind::Opaque {
            colour: pack([254, 255, 255])
        }
    );
    // Detail 0 is coarse without asking the test.
    let mut never = || -> bool { panic!("detail 0 runs no test") };
    assert!(!beam_spots(&b, 0, &mut frame_with(0, &mut never, &mut d)).is_empty());
}

/// spec: S-0518 claim 22 (fine mode: additive lines A₁, B₁, …, C; supported K = 2L, first pair K, then L, then halving; stop below 8)
#[test]
fn house_fine_supported() {
    let look = Look::House {
        colour: [200, 40, 0],
        width: 5,
        supported: true,
    };
    let b = beam((10, 10), (14, 10), 3, 10, look);
    let mut d = |_: i32, _: i32| 0;
    let mut fine = || true;
    let spots = beam_spots(&b, 0, &mut frame_with(1, &mut fine, &mut d));
    let colours: Vec<[u8; 3]> = spots
        .iter()
        .filter_map(|s| match s.kind {
            SpotKind::Additive { rgb } => Some(rgb),
            _ => None,
        })
        .collect();
    let mut distinct = colours.clone();
    distinct.dedup();
    // Pairs: K = (255, 80, 0), L, L/2, L/4, L/8 = (25, 5, 0); the centre K last.
    assert_eq!(
        distinct,
        [
            [255, 80, 0],
            [200, 40, 0],
            [100, 20, 0],
            [50, 10, 0],
            [25, 5, 0],
            [255, 80, 0]
        ]
    );
    // Sector 3: pairs move along x; the fifth pair is 5 pixels out.
    // The centre covers x = 10 … 13 (the terminal end excluded); the fifth pair lies 5 pixels out.
    assert!(spots.iter().all(|s| s.y == 10));
    assert_eq!(spots.iter().map(|s| s.x).min(), Some(5));
    assert_eq!(spots.iter().map(|s| s.x).max(), Some(18));
}

/// spec: S-0518 claim 23 (an additive line whose scaled colour has all channels below 8 is not drawn)
#[test]
fn faint_additive_line_skipped() {
    let look = Look::House {
        colour: [20, 0, 0],
        width: 1,
        supported: false,
    };
    let b = beam((0, 0), (4, 0), 1, 10, look);
    let mut d = |_: i32, _: i32| 0;
    let mut fine = || true;
    // P = 8: I = 2/10, 20 × 0.2 = 3.99… below 8: nothing.
    assert!(beam_spots(&b, 8, &mut frame_with(1, &mut fine, &mut d)).is_empty());
    assert!(!beam_spots(&b, 0, &mut frame_with(1, &mut fine, &mut d)).is_empty());
}

/// spec: S-0514 claim 23 (laser first, then electric bolt, then radiation beam)
#[test]
fn flash_selection() {
    assert_eq!(line_effect(true, true, true), LineEffect::Laser);
    assert_eq!(line_effect(false, true, true), LineEffect::Bolt);
    assert_eq!(line_effect(false, false, true), LineEffect::Flash);
    assert_eq!(line_effect(false, false, false), LineEffect::None);
}

/// spec: S-0514 claim 24, F6 (frame 0 is a straight line; zero length is never drawn; n = 0 is one segment S to T)
#[test]
fn flash_shape() {
    let fl = Flash {
        s: pos(0, 0, 0),
        t: pos(512, 0, 0),
        rgb: [200, 100, 50],
        born: 0,
    };
    let straight = flash_spots(&fl, 0, RZ);
    // Every pixel lies within a pixel of the projected axis S to T.
    let (a, b) = (project_lifted(0, 0, 0), project_lifted(512, 0, 0));
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    assert!(
        straight
            .iter()
            .all(|s| ((s.y - a.1) * dx - (s.x - a.0) * dy).abs() <= dx.abs().max(dy.abs()))
    );
    // F7 at frame 0: f = 0.5 everywhere.
    assert!(
        straight
            .iter()
            .all(|s| s.kind == SpotKind::Flash { rgb: [100, 50, 25] })
    );
    let wavy = flash_spots(&fl, 14, RZ);
    assert!(
        wavy.iter()
            .any(|s| s.kind != SpotKind::Flash { rgb: [100, 50, 25] })
    );
    assert!(flash_spots(&Flash { t: fl.s, ..fl }, 3, RZ).is_empty());
    let short = flash_spots(
        &Flash {
            t: pos(6, 0, 0),
            ..fl
        },
        7,
        RZ,
    );
    assert!(!short.is_empty());
}

/// spec: S-0514 F8 (b₀ = Δ − L(z₀) − 2, the segment's leftmost end's bias)
#[test]
fn flash_depth_bias() {
    let fl = Flash {
        s: pos(0, 0, 0),
        t: pos(0, 6, 0),
        rgb: [255, 255, 255],
        born: 0,
    };
    let spots = flash_spots(&fl, 0, RZ);
    // One segment: Δ = row(S) − row(T) = 0 − 0 (6 leptons project to row 0).
    assert!(spots.iter().all(|s| s.bias == -2));
    let fl = Flash {
        s: pos(0, 0, 208),
        t: pos(5, 0, 0),
        ..fl
    };
    // S lifted 30 rows (L(208) = 30): Δ = −30, and the first segment's leftmost end is S,
    // so b₀ = −30 − 30 − 2.
    let spots = flash_spots(&fl, 0, RZ);
    assert_eq!(spots[0].bias, -62);
    // Lower points are less near: the last segment's bias is −30 − L(z) − 2 with z small.
    assert!(spots.iter().all(|s| (-62..=-32).contains(&s.bias)));
}

/// spec: S-0607 F5 (corners 50 leptons either side at right angles, rounded toward zero; h = 0 takes u = (1, 0))
#[test]
fn wave_band_corners() {
    let c = wave_corners(pos(1000, 1000, 0), pos(2000, 1000, 50));
    assert_eq!(
        c,
        [
            pos(2000, 950, 50),
            pos(2000, 1050, 50),
            pos(1000, 1050, 0),
            pos(1000, 950, 0)
        ]
    );
    // 3-4-5: 50n = (−40, 30) exactly.
    let c = wave_corners(pos(1000, 1000, 0), pos(1300, 1400, 0));
    assert_eq!(c[2], pos(960, 1030, 0));
    // Inexact: 50 / √2 = 35.35…: 1000 + 35.35 → 1035, 1000 − 35.35 → 964.
    let c = wave_corners(pos(1000, 1000, 0), pos(1100, 1100, 0));
    assert_eq!(c[2], pos(964, 1035, 0));
    assert_eq!(c[3], pos(1035, 964, 0));
    let c = wave_corners(pos(500, 500, 0), pos(500, 500, 50));
    assert_eq!(c[2], pos(500, 550, 0));
}

/// spec: S-0607 F4, claim 25 (100 at creation, 19 at 81 ticks, then 63: a jump of 44 every 45 ticks)
#[test]
fn wave_countdown() {
    assert_eq!(countdown(0), 100);
    assert_eq!(countdown(81), 19);
    assert_eq!(countdown(82), 63);
    assert_eq!(countdown(126), 19);
    assert_eq!(countdown(127), 63);
}

/// spec: S-0607 F5 (W takes −11 … 12; j ≤ 1 no shift, 2 … 6 one, 7 … 9 two; k = 110 + 8j)
#[test]
fn wave_ripple() {
    let ws: Vec<i64> = (0..400).map(|i| ripple_wave(i, RZ)).collect();
    assert!(ws.iter().all(|w| (-11..=12).contains(w)));
    assert_eq!(ws[0], 0);
    assert!(ws.contains(&12) && ws.contains(&-11));
    for d in 0..200u64 {
        let (s, k) = ripple(d * d, 40, RZ);
        let j = (i64::from(k) - 110) / 8;
        assert!((0..=9).contains(&j));
        assert_eq!(
            s,
            if j <= 1 {
                0
            } else if j <= 6 {
                1
            } else {
                2
            }
        );
    }
}

/// spec: S-0607 F5 (shift right or left within 45° of horizontal on screen, else up or down)
#[test]
fn wave_shift_direction() {
    // Along world +x: the band's screen line runs down-right at slope ½.
    assert_eq!(shift_direction(pos(0, 0, 0), pos(2000, 0, 0)), (1, 0));
    assert_eq!(shift_direction(pos(2000, 0, 0), pos(0, 0, 0)), (-1, 0));
    // Along world +x and +y together: straight down the screen.
    assert_eq!(shift_direction(pos(0, 0, 0), pos(1000, 1000, 0)), (0, 1));
    assert_eq!(shift_direction(pos(1000, 1000, 0), pos(0, 0, 0)), (0, -1));
}

/// spec: S-0607 claim 25, F5 (band pixels take a displaced source pixel, boosted; flat bias −L(z₁) − 2)
#[test]
fn wave_pixels() {
    let spots = wave_spots(pos(0, 0, 104), pos(1024, 0, 104), (1, 0), 50, RZ);
    assert!(!spots.is_empty());
    // L(104) = 15.
    assert!(spots.iter().all(|s| s.bias == -17));
    assert!(
        spots.iter().all(
            |s| matches!(s.kind, SpotKind::Wave { shift: (dx, 0), .. } if (0..=2).contains(&dx))
        )
    );
    // Over a grey picture, a band pixel boosts red and blue but keeps green.
    let mut screen = Screen::new(140, 80, -40, DEPTH_REFERENCE);
    let grey = pack([100, 100, 100]);
    screen.color = vec![grey; 140 * 80];
    apply(
        &mut screen,
        &SpotGroup {
            at: 0,
            group: Group::Effects,
            provisional: false,
            spots,
        },
        (-10, -40),
    );
    let changed: Vec<[u32; 3]> = screen
        .color
        .iter()
        .filter(|&&c| c != grey)
        .map(|&c| widen(c))
        .collect();
    assert!(!changed.is_empty());
    assert!(
        changed
            .iter()
            .all(|c| c[1] == 100 && c[0] > 96 && c[2] > 96)
    );
}

/// spec: S-0615 claims 15 and 16 (revision 4: arc beams D = 8 − k with S = 1, E = 0.5, the last drawn
/// tick at 0.5 + 0.5/D, not 0.5; the final beam S = 1, E = 0, the last drawn tick at 1/D, no
/// zero-intensity tick, no beam for D ≤ 0; ordinary drawing, fading on and blinking off, even with
/// house colours, whose flag changes only the colours)
/// spec: S-0518 claim 19, F6 (the lifetime and intensity the disc laser's parameters feed)
#[test]
fn disc_laser_beam_parameters() {
    use glasnost_sim::combat::effects::Effect;
    let w = glasnost_sim::testkit::new_game(5);
    let mut lines = Lines::new(Settings {
        house_laser: vec![[240, 0, 0]],
        ..Settings::default()
    });
    let colours = [[200, 100, 40], [100, 50, 20], [7, 8, 9]];
    let beam = |lines: &mut Lines, duration, fade_out, house| {
        lines.take(
            &w,
            &Effect::Beam {
                from: pos(0, 0, 0),
                to: pos(2560, 0, 0),
                duration,
                fade_out,
                colours,
                house,
            },
            0,
            0,
        );
        lines.beams.pop().expect("a beam")
    };
    let to_256 = |b: &Beam, p| scaled(1 << 8, intensity(b, p, RZ), RZ);
    // An arc beam of step k = 5: D = 3; drawn at P = 0, 1, 2.
    let arc = beam(&mut lines, 3, false, None);
    assert!(arc.fade);
    assert_eq!((arc.start, arc.end), (ONE, HALF));
    assert_eq!(intensity(&arc, 0, RZ), ONE);
    // 0.5 + 0.5/3 = 0.666…: 256 × I truncates to 170, above 0.5's 128.
    assert_eq!(to_256(&arc, 2), 170);
    assert_eq!(arc.age(3), None, "E itself is never drawn");
    // The final beam: D = 4; its last drawn tick is 1/4.
    let fin = beam(&mut lines, 4, true, None);
    assert_eq!((fin.start, fin.end), (ONE, NONE));
    assert_eq!(to_256(&fin, 3), 64);
    assert_eq!(fin.age(4), None, "no zero-intensity tick");
    for d in [0, -2] {
        assert_eq!(
            beam(&mut lines, d, true, None).age(0),
            None,
            "D = {d}: no beam"
        );
    }
    // House colours: the house laser colour, that colour halved, the spread
    // kept, drawn ordinarily and not blinking.
    let house = beam(&mut lines, 3, false, Some(glasnost_sim::world::HouseId(0)));
    assert_eq!(
        house.look,
        Look::Ordinary {
            inner: [240, 0, 0],
            outer: [120, 0, 0],
            spread: [7, 8, 9],
            blink: false
        }
    );
    assert!(house.fade);
}

/// spec: S-0616 claim 31 (railgun laser: inner the type's laser colour, outer and spread black, 10 ticks, not blinking, 0.5 to 0)
/// spec: S-0518 claim 18 (other specs' beams carry their parameters; the Terms defaults leave the house-colour flag off)
#[test]
fn railgun_laser_beam() {
    let b = rail_beam(pos(1000, 1000, 0), pos(2000, 1000, 0), 7, [0, 200, 0]);
    assert_eq!(b.born, 7);
    assert_eq!(b.duration, 10);
    assert_eq!((b.start, b.end, b.fade), (HALF, NONE, true));
    assert_eq!(
        b.look,
        Look::Ordinary {
            inner: [0, 200, 0],
            outer: [0, 0, 0],
            spread: [0, 0, 0],
            blink: false,
        }
    );
    // spec: S-0518 claim 20 (not a building: a = 0, so b₀ = b₁ = −L(0) − 2 on flat ground)
    assert_eq!(b.bias, (-2, -2));
    assert_eq!(b.age(16), Some(9));
    assert_eq!(b.age(17), None);
    // spec: S-0518 claim 19 (I runs from S = 0.5 to E + (S − E)/D = 0.05 on the last drawn tick)
    assert_eq!(intensity(&b, 0, RZ), HALF);
    assert_eq!(scaled(256, intensity(&b, 9, RZ), RZ), 12);
    // Pixels: the centre line only (black outer: no outer lines), dimmer as it fades.
    let flat = Beam {
        from: (0, 1),
        to: (3, 1),
        sector: 1,
        born: 0,
        ..b
    };
    let mut d = |_: i32, _: i32| 0;
    let mut fine = || true;
    let first = draw(
        beam_spots(&flat, 0, &mut frame_with(1, &mut fine, &mut d)),
        4,
        3,
    );
    let last = draw(
        beam_spots(&flat, 9, &mut frame_with(1, &mut fine, &mut d)),
        4,
        3,
    );
    let g0 = widen(first.color[4]);
    let g9 = widen(last.color[4]);
    assert!(
        g0[0] == 0 && g0[2] == 0 && g0[1] > g9[1] && g9[1] > 0,
        "{g0:?} {g9:?}"
    );
    assert_eq!(widen(first.color[0]), [0, 0, 0]);
    assert_eq!(widen(first.color[8]), [0, 0, 0]);
}

/// spec: S-0616 claim 31 (a detonation railgun's beam is of zero length; whether it shows is the beam drawing's), S-0518 F9 (a zero-length triple, H = 0, contributes no sample)
#[test]
fn railgun_zero_length_beam_draws_nothing() {
    let at = pos(1000, 1000, 0);
    let b = rail_beam(at, at, 0, [0, 200, 0]);
    assert_eq!(b.from, b.to);
    let point = Beam {
        from: (1, 1),
        to: (1, 1),
        ..b
    };
    let mut d = |_: i32, _: i32| 0;
    let mut fine = || true;
    let out = draw(
        beam_spots(&point, 0, &mut frame_with(1, &mut fine, &mut d)),
        4,
        3,
    );
    assert!(out.color.iter().all(|&c| widen(c) == [0, 0, 0]), "no pixel");
}

/// spec: S-0607 claim 21 (the wave ends when its firer or its target leaves play, or when its
/// firer's target is no longer the wave's), claim 25 (once ended, its ends are no longer updated)
///
/// #72: a target removed by a turn after the wave's own leaves the wave not yet ended at the end
/// of the tick; its ends stay where they last were instead of jumping to the world origin.
#[test]
fn a_wave_keeps_its_ends_when_its_target_leaves_play_before_it_ends() {
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::rules::Kind as SimKind;
    use glasnost_sim::testkit;
    use glasnost_sim::world::{ObjectId, Target};
    let mut w = testkit::game_with(&testkit::combat_ruleset(), 24, 3);
    let cart = testkit::spawn(
        &mut w,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(24, -2, 0),
        16384,
    );
    let walker = testkit::spawn(
        &mut w,
        SimKind::Infantry,
        "WALKER",
        1,
        testkit::mid(24, 2, 0),
        49152,
    );
    w.obj_mut(walker).expect("the walker").target = Some(Target::Object(cart));
    let id = ObjectId(1 << 20);
    w.waves.insert(
        id,
        glasnost_sim::lift::Wave {
            firer: walker,
            target: Target::Object(cart),
            a: glasnost_num::soft::F64::ZERO,
            b: glasnost_num::soft::F64::ZERO,
            ended: false,
        },
    );
    let mut lines = Lines::new(Settings::default());
    lines.take(
        &w,
        &Effect::Wave {
            id,
            from: w.reference_point(walker),
            to: w.reference_point(cart),
        },
        0,
        0,
    );
    lines.after_tick(&w, 1, 1);
    let held = lines.waves[&id];
    assert_ne!(held.e1, Pos::default(), "the vehicle target's centre");
    w.objects.remove(&cart);
    lines.after_tick(&w, 2, 1);
    let after = lines.waves[&id];
    assert_eq!((after.e1, after.e2), (held.e1, held.e2));
}
