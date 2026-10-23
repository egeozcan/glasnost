//! Unit tests for the electric bolts' drawing (S-0316 claims 4, 5 and 9 to
//! 13, F1 to F4), on synthetic positions and an invented palette.

use super::*;
use glasnost_num::soft::Rounding;
use glasnost_sim::world::{ObjectId, Pos};

const RZ: Rounding = Rounding::Nearest;

fn pos(x: i32, y: i32, z: i32) -> Pos {
    Pos::new(x, y, z)
}

/// The shape with every draw answering 0, recording each draw's range.
fn recorded(start: Pos, end: Pos, phase: i32, a: i32) -> (Vec<Piece>, Vec<(i32, i32)>) {
    let mut ranges = Vec::new();
    let mut draw = |lo: i32, hi: i32| {
        ranges.push((lo, hi));
        0
    };
    let pieces = shape(start, end, phase, a, false, RZ, &mut draw);
    (pieces, ranges)
}

/// spec: S-0316 F1 (D = min(8, the least d with ⌊λ / 2^d⌋ ≤ 64))
#[test]
fn f1_split_levels() {
    assert_eq!(levels(0), 0);
    assert_eq!(levels(64), 0);
    assert_eq!(levels(65), 1);
    assert_eq!(levels(129), 1);
    assert_eq!(levels(130), 2);
    assert_eq!(levels(64 << 8), 8);
    assert_eq!(levels((64 << 8) + 256), 8, "at most 8 levels");
    assert_eq!(levels(i64::from(i32::MAX)), 8);
}

/// spec: S-0316 Terms (λ the truncated root of the sum of squares, the height term included)
#[test]
fn terms_bolt_length() {
    assert_eq!(length(pos(0, 0, 0), pos(300, 400, 0), false, RZ), 500);
    assert_eq!(length(pos(0, 0, 0), pos(0, 0, -77), false, RZ), 77);
    assert_eq!(length(pos(10, 10, 10), pos(11, 11, 11), false, RZ), 1);
    assert_eq!(length(pos(5, 5, 5), pos(5, 5, 5), false, RZ), 0);
}

/// spec: S-0316 claim 9 (a bolt of length 0 is not drawn and takes no draws; at most 64 one straight piece per strand, no draws)
#[test]
fn claim_9_short_bolts() {
    let (pieces, ranges) = recorded(pos(7, 7, 7), pos(7, 7, 7), 3, 0);
    assert!(pieces.is_empty());
    assert!(ranges.is_empty());
    let (s, t) = (pos(100, 100, 0), pos(140, 150, 0));
    let (pieces, ranges) = recorded(s, t, 3, 0);
    assert!(ranges.is_empty());
    assert_eq!(pieces.len(), 1);
    let line = ([100, 100, 0], [140, 150, 0]);
    assert_eq!(pieces[0].strands, [line; 3], "all three straight");
}

/// spec: S-0316 claim 13 (9 draws per split, 9 × (2^D − 1) per drawn frame)
#[test]
fn claim_13_draw_count() {
    for len in [65, 128, 129, 300, 1000, 5000, 20000, 100000] {
        let (pieces, ranges) = recorded(pos(0, 0, 0), pos(len, 0, 0), 11, 0);
        let d = levels(i64::from(len));
        assert_eq!(ranges.len() as i64, draws_per_frame(d), "λ = {len}");
        assert_eq!(pieces.len(), 1 << d, "λ = {len}");
    }
}

/// spec: S-0316 F3 (strand A: R(−J, J) above 128, 2J × R(−1, 1) at or below; B and C pinned to A above P, their own midpoint at or below P) and claim 13 (A's three draws before B's and C's; depth first, the start half first)
#[test]
fn f3_draw_ranges_in_order() {
    // λ = 1000: D = 4; J₀ = ⌊23000 / 256⌋ = 89, P = ⌊102000 / 256⌋ = 398.
    let (_, ranges) = recorded(pos(0, 0, 0), pos(1000, 0, 0), 0, 0);
    // The levels of the splits in depth-first order, parent first.
    let order = [0, 1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3];
    assert_eq!(ranges.len(), 9 * order.len());
    for (i, &k) in order.iter().enumerate() {
        let j = 89 >> k;
        let l = 1000 >> k;
        let a = if l > 128 { (-j, j) } else { (-1, 1) };
        let split = &ranges[9 * i..9 * i + 9];
        assert_eq!(split[..3], [a; 3], "split {i} level {k}: strand A");
        assert_eq!(split[3..], [(-j, j); 6], "split {i} level {k}: B then C");
    }
}

/// spec: S-0316 F2 (φ = 0: every sᵢ is 0, so Δ = (0, 0, ⌊2 J₀ / 2⌋) = (0, 0, J₀)) and F3 (the plain midpoint floors towards negative infinity)
#[test]
fn f2_f3_first_midpoint() {
    assert_eq!(sway(0, 9, RZ), [0, 0, 9]);
    assert_eq!(sway(17, 0, RZ), [0, 0, 0]);
    // |sᵢ| ≤ J₀: Δx and Δy within ±2 J₀, Δz within 0 to 2 J₀.
    for phase in 0..300 {
        let [dx, dy, dz] = sway(phase, 50, RZ);
        assert!(dx.abs() <= 100 && dy.abs() <= 100, "φ = {phase}");
        assert!((0..=100).contains(&dz), "φ = {phase}");
    }
    // φ = 7: s₀ = trunc(sinT(π) × J₀) = 0 (sinT(π) is within one unit of 0).
    let s0 = glasnost_num::trig::sin_t(
        F64::from_i64(7, RZ)
            .mul(PI, RZ)
            .div(F64::from_i64(7, RZ), RZ),
        RZ,
    );
    assert_eq!(s0.mul(F64::from_i64(1000, RZ), RZ).trunc_i128(), Some(0));
    // λ = 101: D = 1, J₀ = ⌊2323 / 256⌋ = 9, P = 40: B and C pinned.
    let (pieces, _) = recorded(pos(-101, 0, 0), pos(0, 0, 0), 0, 0);
    assert_eq!(pieces.len(), 2);
    for s in 0..3 {
        // ⌊−101 / 2⌋ = −51, not −50.
        assert_eq!(pieces[0].strands[s].1, [-51, 0, 9], "strand {s}");
        assert_eq!(pieces[1].strands[s].0, [-51, 0, 9], "strand {s}");
    }
}

/// spec: S-0316 F3 (at ℓ ≤ 128 strand A's jitter is 2 Jₖ × R(−1, 1); pinned B and C take A's displaced midpoint plus ⌊R(−J, J) / 2⌋)
#[test]
fn f3_jitter_values() {
    // λ = 100: D = 1, J₀ = 8, P = 39 < 100: B and C pinned; φ = 0.
    let mut answers = [1, -1, 1, -3, 5, -7, 2, 2, 0].into_iter();
    let mut draw = |_: i32, _: i32| answers.next().unwrap();
    let pieces = shape(pos(0, 0, 0), pos(100, 0, 0), 0, 0, false, RZ, &mut draw);
    let a = [50 + 16, -16, 8 + 16];
    assert_eq!(pieces[0].strands[A].1, a);
    // ⌊−3 / 2⌋ = −2, ⌊5 / 2⌋ = 2, ⌊−7 / 2⌋ = −4.
    assert_eq!(pieces[0].strands[B].1, [a[0] - 2, a[1] + 2, a[2] - 4]);
    assert_eq!(pieces[0].strands[C].1, [a[0] + 1, a[1] + 1, a[2]]);
}

/// spec: S-0316 F3 (at ℓₖ ≤ P strands B and C take their own midpoints plus R(−Jₖ, Jₖ))
#[test]
fn f3_unpinned_strands() {
    // λ = 300: D = 3, J₀ = 26, P = 119. Level 2: ℓ = 75 ≤ P.
    let mut n = 0;
    let mut draw = |lo: i32, _: i32| {
        n += 1;
        // Every draw answers its low end, except at level 2 (the third split).
        if (19..=27).contains(&n) { 0 } else { lo }
    };
    let pieces = shape(pos(0, 0, 0), pos(300, 0, 0), 0, 0, false, RZ, &mut draw);
    assert_eq!(pieces.len(), 8);
    // The first level-2 split (draws 19 to 27) answered 0: B's and C's
    // midpoints are their own strands' plain midpoints, not A's.
    for s in [B, C] {
        let (p, q) = (pieces[0].strands[s].0, pieces[1].strands[s].1);
        let m = pieces[0].strands[s].1;
        assert_eq!(
            m,
            [
                (p[0] + q[0]).div_euclid(2),
                (p[1] + q[1]).div_euclid(2),
                (p[2] + q[2]).div_euclid(2)
            ],
            "strand {s}"
        );
    }
}

/// spec: S-0316 F4 (e = a at the start, 0 at the end, e(m) = ⌊(e(p) + e(q)) / 2⌋)
#[test]
fn f4_depth_terms() {
    let (pieces, _) = recorded(pos(0, 0, 0), pos(200, 0, 0), 0, -9);
    // D = 2: ⌊−9 / 2⌋ = −5 at the middle, then ⌊−14 / 2⌋ = −7 and ⌊−5 / 2⌋ = −3.
    let es: Vec<(i64, i64)> = pieces.iter().map(|p| p.e).collect();
    assert_eq!(es, vec![(-9, -7), (-7, -5), (-5, -3), (-3, 0)]);
}

fn colours() -> Colours {
    Colours {
        side: [10, 10, 10],
        alt_side: [5, 5, 5],
        main: [15, 15, 15],
    }
}

/// spec: S-0316 claim 10 (B and C entry 10, or 5 with the alternate-colour flag; A entry 15) and claim 12 (each piece B, then C, then A)
#[test]
fn claims_10_12_colours_and_order() {
    let (pieces, _) = recorded(pos(0, 0, 0), pos(40, 0, 0), 0, 0);
    let rgb = |s: &Spot| match s.kind {
        SpotKind::Flash { rgb } => rgb,
        _ => panic!("a flash pixel"),
    };
    for (alt, side) in [(false, [10; 3]), (true, [5; 3])] {
        let spots = spots(&pieces, alt, &colours());
        let third = spots.len() / 3;
        assert_eq!(spots.len(), 3 * third);
        assert!(spots[..2 * third].iter().all(|s| rgb(s) == side));
        assert!(spots[2 * third..].iter().all(|s| rgb(s) == [15; 3]));
    }
}

/// spec: S-0316 claim 11 and F4 (b = e − L(z) − 2 at each end; a piece takes its leftmost end's bias, S-0514 F8)
#[test]
fn claim_11_piece_bias() {
    use glasnost_render::projection::{height_lift, project_lifted};
    let p = [0, 0, 104];
    let q = [40, -40, 0];
    let piece = Piece {
        strands: [(p, q); 3],
        e: (-6, 0),
    };
    let spots = spots(&[piece], false, &colours());
    let (ps, qs) = (project_lifted(0, 0, 104), project_lifted(40, -40, 0));
    let bp = -6 - height_lift(104) - 2;
    let bq = -height_lift(0) - 2;
    let want = if qs.0 < ps.0 { bq } else { bp };
    assert!(spots.iter().all(|s| s.bias == want));
    assert_eq!(spots[0].x, ps.0);
    assert_eq!(spots[0].y, ps.1);
}

/// spec: S-0316 claim 5 (17 drawn frames, the first the drawn frame after the shot's tick) and Terms (frame n uses φ + n) and claim 13 (making the bolt takes the one draw R(0, 256))
#[test]
fn claim_5_life_and_phase() {
    let mut ranges = Vec::new();
    let mut draw = |lo: i32, hi: i32| {
        ranges.push((lo, hi));
        77
    };
    let b = Bolt::new(
        ObjectId(1),
        0,
        0,
        pos(0, 0, 0),
        pos(500, 0, 0),
        true,
        40,
        false,
        0,
        &mut draw,
    );
    assert_eq!(ranges, vec![(0, 256)]);
    assert_eq!(b.phase, 77);
    assert_eq!(b.frame(39), None);
    assert_eq!(b.frame(40), Some(0));
    assert_eq!(b.frame(56), Some(16));
    assert_eq!(b.frame(57), None, "gone from the 18th");
    assert_eq!(b.phase_at(5), 82);
}

/// spec: S-0316 claim 5 (the view test is on the straight screen line between the two ends)
#[test]
fn claim_5_view_test() {
    let (o, s) = ((0, 0), (100, 50));
    assert!(line_in_view((10, 10), (20, 20), o, s), "inside");
    assert!(
        line_in_view((-50, 25), (150, 25), o, s),
        "crossing, ends outside"
    );
    assert!(!line_in_view((-50, -50), (-10, 40), o, s), "to the left");
    assert!(!line_in_view((200, 0), (300, 40), o, s), "to the right");
    // The bounding boxes overlap but the line passes outside the corner.
    assert!(!line_in_view((90, -30), (130, 10), o, s));
    assert!(
        line_in_view((99, 49), (300, 300), o, s),
        "touching the corner"
    );
    assert!(!line_in_view((10, 10), (20, 20), o, (0, 0)));
}

/// spec: S-0316 claim 11 (the general palette's row 26 reproduces the palette file's colours) and claim 10 (entries 10, 5 and 15)
#[test]
fn claim_10_colours_from_the_palette() {
    let mut bytes = vec![0u8; 768];
    for i in 0..256 {
        bytes[3 * i] = (i % 64) as u8;
        bytes[3 * i + 1] = ((i + 7) % 64) as u8;
        bytes[3 * i + 2] = ((i * 3) % 64) as u8;
    }
    let c = Colours::from_palette_file(&bytes).expect("a palette");
    let p = glasnost_fmt::pal::Palette6::parse(&bytes).unwrap();
    let pal = glasnost_render::palette::palette_from_file(
        &p,
        &glasnost_render::palette::PaletteQuirks::default(),
    );
    let rgb = |i: usize| [pal.colors[i].r, pal.colors[i].g, pal.colors[i].b];
    assert_eq!(c.side, rgb(10));
    assert_eq!(c.alt_side, rgb(5));
    assert_eq!(c.main, rgb(15));
    assert_eq!(Colours::from_palette_file(&[1, 2, 3]), None);
}

// ---------------------------------------------------------------------------
// The view's bolts (`beams::Lines`)

use super::super::beams::{Lines, Settings};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::World;

const SIZE: i32 = 24;

/// A synthetic combat world with an Alpha CART in the middle.
fn cart_world() -> (World, ObjectId) {
    use glasnost_sim::testkit;
    let mut w = testkit::game_with(&testkit::combat_ruleset(), SIZE, 3);
    let c = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    (w, c)
}

fn report(firer: ObjectId, from: Pos, to: Pos, vehicle_on_map: bool) -> Effect {
    Effect::Bolt {
        firer,
        weapon: 0,
        slot: 0,
        burst: 0,
        from,
        to,
        building: None,
        vehicle_on_map,
    }
}

fn lines() -> Lines {
    Lines::new(Settings {
        bolt: Some(colours()),
        ..Settings::default()
    })
}

/// spec: S-0316 claim 4 (attached only for a vehicle on the map with no attached bolt; the start follows the fire point of the slot that fired each drawn frame; the firer's removal ends the attachment keeping the last start) and claim 5 (gone after 17 drawn frames)
#[test]
fn claim_4_attachment_and_following() {
    let (mut w, c) = cart_world();
    let mut lines = lines();
    let mut draw = |_: i32, _: i32| 3;
    let (from, to) = (pos(1, 2, 3), pos(4000, 4000, 0));
    lines.take_bolt(&w, &report(c, from, to, true), 5, &mut draw);
    lines.take_bolt(&w, &report(c, from, to, true), 5, &mut draw);
    lines.take_bolt(&w, &report(c, from, to, false), 5, &mut draw);
    let att: Vec<bool> = lines.bolts.iter().map(|b| b.attached).collect();
    assert_eq!(att, vec![true, false, false], "one attached bolt per firer");
    assert!(lines.bolts.iter().all(|b| b.phase == 3 && b.born == 5));
    lines.after_tick(&w, 5, 4);
    let fp = glasnost_sim::combat::position::fire_point(&w, c, 0, 0);
    assert_eq!(lines.bolts[0].start, fp, "moved to the current fire point");
    assert_eq!(lines.bolts[1].start, from, "an unattached start stays");
    // The firer moves: the start follows.
    let p = w.obj(c).unwrap().pos;
    w.obj_mut(c).unwrap().pos = Pos::new(p.x + 256, p.y, p.z);
    lines.after_tick(&w, 6, 4);
    let moved = glasnost_sim::combat::position::fire_point(&w, c, 0, 0);
    assert_ne!(moved, fp);
    assert_eq!(lines.bolts[0].start, moved);
    assert_eq!(lines.bolts[0].end, to, "the end is fixed");
    // Removed: no longer attached, the last start kept.
    glasnost_sim::objects::remove(&mut w, c);
    assert!(w.obj(c).is_none());
    lines.after_tick(&w, 7, 4);
    assert!(!lines.bolts[0].attached);
    assert_eq!(lines.bolts[0].start, moved);
    lines.after_tick(&w, 21, 4);
    assert_eq!(lines.bolts.len(), 3, "frame 16 still drawn");
    lines.after_tick(&w, 22, 4);
    assert!(lines.bolts.is_empty(), "gone from the 18th drawn frame");
}

/// spec: S-0316 claim 4 (once the attached bolt is gone, the firer's next bolt can be attached)
#[test]
fn claim_4_attached_again_after_the_last_ends() {
    let (w, c) = cart_world();
    let mut lines = lines();
    let mut draw = |_: i32, _: i32| 0;
    let (from, to) = (pos(1, 2, 3), pos(4000, 4000, 0));
    lines.take_bolt(&w, &report(c, from, to, true), 0, &mut draw);
    lines.after_tick(&w, 17, 4);
    lines.take_bolt(&w, &report(c, from, to, true), 17, &mut draw);
    assert_eq!(lines.bolts.len(), 1);
    assert!(lines.bolts[0].attached);
}

/// spec: S-0316 claim 5 (a bolt out of view is not drawn and takes no draws) and claims 12 and 13 (the newest bolt first)
#[test]
fn claims_5_12_13_view_test_and_newest_first() {
    let (w, c) = cart_world();
    let mut lines = lines();
    let mut zero = |_: i32, _: i32| 0;
    // An older long bolt (λ = 1000, J₀ = 89) and a newer short one (λ = 300, J₀ = 26).
    let s = pos(10000, 10000, 0);
    lines.take_bolt(&w, &report(c, s, pos(11000, 10000, 0), false), 0, &mut zero);
    lines.take_bolt(&w, &report(c, s, pos(10300, 10000, 0), false), 1, &mut zero);
    let mut ranges = Vec::new();
    let mut rec = |lo: i32, hi: i32| {
        ranges.push((lo, hi));
        0
    };
    let spots = lines.bolt_spots(2, None, false, RZ, &mut rec);
    assert!(!spots.is_empty());
    assert_eq!(ranges.len() as i64, draws_per_frame(3) + draws_per_frame(4));
    assert_eq!(ranges[0], (-26, 26), "the newer bolt's draws first");
    assert_eq!(ranges[draws_per_frame(3) as usize], (-89, 89));
    // A view far from both: no spots and no draws.
    let mut count = 0;
    let mut rec = |_: i32, _: i32| {
        count += 1;
        0
    };
    let far = Some(((-100000, -100000), (640, 400)));
    assert!(lines.bolt_spots(3, far, false, RZ, &mut rec).is_empty());
    assert_eq!(count, 0);
    // Before its first frame or after its last: none.
    let mut none = |_: i32, _: i32| -> i32 { panic!("no draw") };
    assert!(lines.bolt_spots(-1, None, false, RZ, &mut none).is_empty());
    assert!(lines.bolt_spots(18, None, false, RZ, &mut none).is_empty());
}

/// spec: S-0316 F4 (a building firer's bolt: e = a at the start, S-0518 claim 20's building-source adjustment; never attached)
#[test]
fn f4_building_adjustment() {
    let (w, c) = cart_world();
    let mut lines = lines();
    let mut zero = |_: i32, _: i32| 0;
    let from = pos(5000, 5000, 400);
    let building = pos(5100, 5100, 0);
    lines.take_bolt(
        &w,
        &Effect::Bolt {
            firer: c,
            weapon: 0,
            slot: 0,
            burst: 0,
            from,
            to: pos(6000, 5000, 0),
            building: Some(building),
            vehicle_on_map: false,
        },
        0,
        &mut zero,
    );
    let a = super::super::beams::building_nearness(from, building);
    assert_eq!(lines.bolts[0].a, a);
    assert!(!lines.bolts[0].attached);
}
