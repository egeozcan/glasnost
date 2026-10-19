//! The railgun shot (S-0306 claim 27) and its beam end point (S-0616 F14).
//!
//! At a shot of a weapon with the railgun flag whose firer's railgun
//! emitter place is empty, [`shot`] walks the fifty samples of F14 from the
//! emitter's position *P* to the target's position *Q*, finds the end point
//! (the first tested sample below the terrain, else *Q*), and, when no
//! sample stops in terrain, hits every qualifying object of the cells the
//! samples newly enter with one ordinary hit of the weapon's raw damage. A
//! terrain stop on a destroyable-cliff cell, or an unobstructed shot at a
//! destroyable-cliff cell target, takes one draw R(0, 99) for the cliff
//! collapse. The collapse itself is the seam [`cliff_collapse`].
//!
//! Every parameter *t*ᵢ of F14, the last one included, is formed in the
//! program's rounding mode (S-0616 revision 10; S-0205 claim 5), so under
//! round-toward-zero *t*₄₉ = 1 − 2⁻²⁴; the end point without a terrain stop
//! is still the exact *Q*. The line distance follows S-0306 claim 27's
//! mixed-precision equations (revision 15); a distance that is not a
//! number qualifies with `quirks.railgun_collinear_hits` (Q-1405 answered).

use glasnost_map::CellXY;
use glasnost_num::{F32, F64, Rounding};

use crate::combat::data::WeaponId;
use crate::rules::Kind;
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// The number of samples along the beam (F14: *i* = 0 … 49).
const SAMPLES: i32 = 50; // spec: S-0616 Constants (railgun end-point samples 50, i = 0 … 49)

/// The significand of the step δ, the single-precision value nearest 1/49:
/// δ = 10956549 × 2⁻²⁹.
const STEP_SIGNIFICAND: u128 = 10_956_549; // spec: S-0616 Constants and F14 (the railgun end-point step: 10956549 / 536870912, the fixed single-precision value nearest 1/49, unchanged by the rounding mode)

/// The binary exponent of δ.
const STEP_EXPONENT: i32 = -29; // spec: S-0616 Constants and F14 (10956549 / 536870912 = 10956549 × 2⁻²⁹)

/// Leptons per cell, for a sample's cell (F14: trunc(x / 256), trunc(y / 256)).
const CELL: i32 = 256; // spec: S-0616 F14 (a sample's cell is (trunc(x / 256), trunc(y / 256)))

/// The height of one height level, leptons (S-0306 claim 27's deck test).
const LEVEL: i32 = 104; // spec: S-0306 claim 27 (the sample's world height is at least (the cell's height level + 4) × 104 leptons)

/// The levels between a cell's ground and its bridge deck (claim 27).
const DECK_LEVELS: i32 = 4; // spec: S-0306 claim 27 ((the cell's height level + 4) × 104 leptons)

/// The top of the cliff-collapse draw (claim 27: R(0, 99)).
const COLLAPSE_TOP: i32 = 99; // spec: S-0306 claim 27 (one ranged scenario draw R(0, 99))

/// The step δ of F14.
pub fn step() -> F32 {
    // Exact: a 24-bit significand, so the mode does not matter.
    F64::from_parts(false, STEP_SIGNIFICAND, STEP_EXPONENT, Rounding::Nearest)
        .to_f32(Rounding::Nearest)
}

/// *t*ᵢ = fl(*i* × δ) of F14, in the program's mode for every *i*, the
/// last one included (S-0616 F14, *Last sample*): under round-toward-zero
/// *t*₄₉ = 1 − 2⁻²⁴.
pub fn fraction(i: i32, mode: Rounding) -> F64 {
    // spec: S-0616 F14 (tᵢ = fl(i × δ): the product formed in double precision, then narrowed to single precision, both in the program's rounding mode, S-0205 claims 4 and 5; no special assignment of 1.0 at the last sample)
    F64::from_i64(i64::from(i), mode)
        .mul(step().to_f64(), mode)
        .to_f32(mode)
        .to_f64()
}

/// The sample *s*ᵢ of F14 between `p` and `q`.
pub fn sample(p: Pos, q: Pos, i: i32, mode: Rounding) -> Pos {
    let t = fraction(i, mode);
    let one = F64::from_i64(1, mode);
    let u = one.sub(t, mode);
    // spec: S-0616 F14 (sᵢ = trunc((Q × tᵢ) + (P × (1 − tᵢ))) per component: every subtraction, product and sum in double precision in the program's mode, no single-precision narrowing, no fused multiply-add, truncated toward zero)
    let c = |qv: i32, pv: i32| {
        F64::from_i64(i64::from(qv), mode)
            .mul(t, mode)
            .add(F64::from_i64(i64::from(pv), mode).mul(u, mode), mode)
            .to_i32_common()
    };
    Pos::new(c(q.x, p.x), c(q.y, p.y), c(q.z, p.z))
}

/// A sample's cell (F14).
fn cell_of(s: Pos) -> CellXY {
    // spec: S-0616 F14 (the cell (trunc(x / 256), trunc(y / 256)))
    CellXY::new(s.x / CELL, s.y / CELL)
}

/// The walk of F14: the end point, the cell of a terrain stop (if any) and
/// the cells newly entered before it with the height of the sample that
/// entered each, in sample order.
pub struct Walk {
    /// The beam's end point.
    pub end: Pos,
    /// The cell of the sample that stopped in terrain, if one did.
    pub stop: Option<CellXY>,
    /// The cells newly entered, with the entering sample's height.
    pub visited: Vec<(CellXY, i32)>,
}

/// F14: the samples from `p` to `q` in the order of *i*.
pub fn walk(w: &World, p: Pos, q: Pos) -> Walk {
    let mode = w.rules().mode;
    let mut prev = cell_of(p);
    let mut visited = Vec::new();
    for i in 0..SAMPLES {
        let s = sample(p, q, i, mode);
        let c = cell_of(s);
        // spec: S-0616 F14 (a sample is tested only when its cell differs from the cell of the sample before it; for i = 0, the cell of P)
        if c == prev {
            continue;
        }
        prev = c;
        // spec: S-0616 F14 (a tested sample below the terrain height there, strictly, is the end point; H is never a bridge deck)
        if s.z < w.surface_at(s.x, s.y) {
            return Walk {
                end: s,
                stop: Some(c),
                visited,
            };
        }
        visited.push((c, s.z));
    }
    // spec: S-0616 F14 (when no tested sample is below the terrain, the end point is the exact Q, not s₄₉)
    Walk {
        end: q,
        stop: None,
        visited,
    }
}

/// The line distance *D* of S-0306 claim 27 (revision 15) of `c` from the
/// beam from `p` to `q`, with its mixed-precision equations. A NaN result
/// (a calculated *h* of zero, exact collinearity among others) is returned
/// as it is.
pub fn line_distance(p: Pos, q: Pos, c: Pos, table: bool, mode: Rounding) -> F64 {
    let f = |v: i32| F64::from_i64(i64::from(v), mode);
    // spec: S-0306 claim 27 (RZ₂₄: the narrowing to single precision, then widened exactly; with quirks.chop_rounding off RN₂₄)
    let n = |v: F64| v.to_f32(mode).to_f64();
    // spec: S-0306 claim 27 (p = (RZ₂₄(Pₓ), RZ₂₄(Pᵧ), RZ₂₄(P_z)))
    let p = [n(f(p.x)), n(f(p.y)), n(f(p.z))];
    // spec: S-0306 claim 27 (d = (RZ₂₄(Qₓ − pₓ), RZ₂₄(RZ₂₄(Qᵧ) − pᵧ), RZ₂₄(RZ₂₄(Q_z) − p_z)))
    let d = [
        n(f(q.x).sub(p[0], mode)),
        n(n(f(q.y)).sub(p[1], mode)),
        n(n(f(q.z)).sub(p[2], mode)),
    ];
    // spec: S-0306 claim 27 (u = (Cₓ − pₓ, RZ₂₄(Cᵧ) − pᵧ, RZ₂₄(C_z) − p_z), double precision)
    let u = [
        f(c.x).sub(p[0], mode),
        n(f(c.y)).sub(p[1], mode),
        n(f(c.z)).sub(p[2], mode),
    ];
    let m = |a: F64, b: F64| a.mul(b, mode);
    // spec: S-0306 claim 27 (a = (uᵧd_z − u_zdᵧ, u_zdₓ − d_zuₓ, dᵧuₓ − uᵧdₓ), double precision)
    let a = [
        m(u[1], d[2]).sub(m(u[2], d[1]), mode),
        m(u[2], d[0]).sub(m(d[2], u[0]), mode),
        m(d[1], u[0]).sub(m(u[1], d[0]), mode),
    ];
    // spec: S-0306 claim 27 (hₓ = RZ₂₄(aᵧd_z − a_zdᵧ), hᵧ = RZ₂₄(a_zdₓ − d_zaₓ), h_z = dᵧaₓ − aᵧdₓ in double)
    let h = [
        n(m(a[1], d[2]).sub(m(a[2], d[1]), mode)),
        n(m(a[2], d[0]).sub(m(d[2], a[0]), mode)),
        m(d[1], a[0]).sub(m(a[1], d[0]), mode),
    ];
    // spec: S-0306 claim 27 (N = RZ₂₄((h_z u_z + hᵧuᵧ) + hₓuₓ))
    let num = n(m(h[2], u[2]).add(m(h[1], u[1]), mode).add(m(h[0], u[0]), mode));
    // spec: S-0306 claim 27 (L = (hᵧhᵧ + h_zh_z) + hₓhₓ, double precision)
    let len = m(h[1], h[1]).add(m(h[2], h[2]), mode).add(m(h[0], h[0]), mode);
    // spec: S-0306 claim 27 (D = |N / r(L)|, r the real-input table root of S-0204 claim 5, used only on L)
    let root = glasnost_num::geom::table_root_real(len, table, mode);
    num.div(root, mode).abs()
}

/// The finite distance of `quirks.railgun_collinear_hits` off (S-0306
/// Edge cases): the perpendicular Euclidean distance of `c` from the line
/// through `p` and `q`, from `p` for a zero-length beam, in double
/// precision with the table root of S-0204 claim 5.
pub fn euclidean_line_distance(p: Pos, q: Pos, c: Pos, table: bool, mode: Rounding) -> F64 {
    let f = |v: i32| F64::from_i64(i64::from(v), mode);
    let d = [q.x - p.x, q.y - p.y, q.z - p.z].map(f);
    let e = [c.x - p.x, c.y - p.y, c.z - p.z].map(f);
    type V3 = [F64; 3]; // nocite: vector and matrix dimensions (three axes)
    let sq = |v: V3| {
        v.iter()
            .fold(F64::ZERO, |a, x| a.add(x.mul(*x, mode), mode))
    };
    let root = |x: F64| glasnost_num::geom::table_root_real(x, table, mode);
    let dd = sq(d);
    if !dd.gt(F64::ZERO) {
        // spec: S-0306 Edge cases (quirks.railgun_collinear_hits off: a zero-length beam uses the Euclidean distance to its start)
        return root(sq(e));
    }
    // spec: S-0306 Edge cases (quirks.railgun_collinear_hits off: for a nonzero beam the perpendicular Euclidean distance to its infinite line)
    let cross = [
        e[1].mul(d[2], mode).sub(e[2].mul(d[1], mode), mode),
        e[2].mul(d[0], mode).sub(e[0].mul(d[2], mode), mode),
        e[0].mul(d[1], mode).sub(e[1].mul(d[0], mode), mode),
    ];
    root(sq(cross)).div(root(dd), mode)
}

/// Whether a non-building candidate at `c` passes claim 27's line-distance
/// test against the hit distance `radius` (signed integer leptons).
pub fn near_line(w: &World, p: Pos, q: Pos, c: Pos, radius: i32) -> bool {
    let rules = w.rules();
    let (mode, table) = (rules.mode, rules.quirks.table_root_distance);
    let r = F64::from_i64(i64::from(radius), mode);
    if rules.quirks.railgun_collinear_hits {
        let d = line_distance(p, q, c, table, mode);
        // spec: S-0306 claim 27 (Degenerate comparisons: a distance that is not a number is accepted as a hit, even for a hit distance of 0 or below; quirks.railgun_collinear_hits)
        if d.is_nan() {
            return true;
        }
        // spec: S-0306 claim 27 (for a finite distance the comparison is strictly less than)
        return d.lt(r);
    }
    // spec: S-0306 Edge cases (quirks.railgun_collinear_hits off: the finite distance strictly below the threshold)
    euclidean_line_distance(p, q, c, table, mode).lt(r)
}

/// S-0306 claim 27 and S-0616 claim 1: the railgun shot of `firer` with
/// `weapon` at `target`, from the emitter's position `p` to the target's
/// position `q`. Returns the beam's end point (F14), the railgun emitter's
/// target point.
pub fn shot(
    w: &mut World,
    firer: ObjectId,
    weapon: WeaponId,
    target: Target,
    p: Pos,
    q: Pos,
) -> Pos {
    let walk = walk(w, p, q);
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    if let Some(c) = walk.stop {
        // spec: S-0306 claim 27 (a terrain stop cancels every hit; only on a destroyable-cliff stop cell one draw R(0, 99), the cliff collapsed when it is strictly below the chance, also for chances at or below 0 or at least 100)
        if crate::combat::range::destroyable_cliff(w, c) {
            let r = w.rng.ranged(0, COLLAPSE_TOP, half);
            if r < rules.combat.general.collapse_chance {
                cliff_collapse(w, c);
            }
        }
        return walk.end;
    }
    let target_object = match target {
        Target::Object(x) => Some(x),
        Target::Cell(_) => None,
    };
    // spec: S-0306 claim 27 and Import mapping (the game-wide railgun hit distance: signed integer leptons, no clamp at 0)
    let radius = rules.combat.general.railgun_radius;
    let mut hits: Vec<ObjectId> = Vec::new();
    for (c, z) in &walk.visited {
        let level = w.cells.cell(*c).map_or(0, |x| i32::from(x.level));
        // spec: S-0306 claim 27 (in a bridge cell the deck layer when the sample's world height is at least (level + 4) × 104 leptons, else the ground layer; in other cells the ground layer)
        let layer = if w.high_bridge(*c) && *z >= (level + DECK_LEVELS) * LEVEL {
            Layer::Deck
        } else {
            Layer::Ground
        };
        // spec: S-0306 claim 27 and S-0209 claim 13 (the cell's listing order)
        let list: Vec<ObjectId> = w.cells.list(*c, layer).to_vec();
        for x in list {
            if x == firer || hits.contains(&x) {
                continue;
            }
            let Some(o) = w.obj(x) else { continue };
            // spec: S-0306 claim 27 (infantry, vehicles, aircraft and buildings other than the firer qualify)
            if !matches!(
                o.kind,
                Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
            ) {
                continue;
            }
            // spec: S-0306 claim 27 (the shot target bypasses both comparisons; buildings count as finite distance 0 and qualify only for a positive hit distance; others by the line-distance test; no alliance test)
            let near = if o.kind == Kind::Building {
                0 < radius
            } else {
                near_line(w, p, q, o.pos, radius)
            };
            if Some(x) == target_object || near {
                hits.push(x);
            }
        }
    }
    // spec: S-0306 claim 27 (an object target not encountered is included at the end)
    if let Some(x) = target_object
        && x != firer
        && !hits.contains(&x)
        && w.obj(x).is_some()
    {
        hits.push(x);
    }
    let wr = rules.combat.weapon(weapon).clone();
    for x in hits {
        if w.obj(x).is_none() {
            continue;
        }
        // spec: S-0306 claim 27 (one ordinary hit of S-0302 claim 1: the weapon's integer damage field without the firer's bonuses, distance 0, the weapon's warhead, the firer as attacker, no explicit attacking house, defences not ignored, passenger escape allowed; not an area detonation)
        let hit = crate::combat::damage::Hit {
            amount: wr.damage,
            warhead: wr.warhead,
            distance: 0,
            attacker: Some(firer),
            house: None,
            ignore_defences: false,
            no_escape: false,
        };
        crate::combat::damage::apply(w, x, hit);
    }
    if let Target::Cell(c) = target
        && crate::combat::range::destroyable_cliff(w, c)
    {
        // spec: S-0306 claim 27 (without a terrain stop, the one-draw test after the ordinary hits only when the shot target is a destroyable-cliff cell)
        let r = w.rng.ranged(0, COLLAPSE_TOP, half);
        if r < rules.combat.general.collapse_chance {
            cliff_collapse(w, c);
        }
    }
    walk.end
}

/// Seam `cliff_collapse` (S-0306 claim 27: "the cliff is collapsed"): no
/// approved spec gives what a collapse changes (the tiles, heights and
/// passability of the cliff's cells), so the draw is taken and nothing
/// changes.
pub fn cliff_collapse(_w: &mut World, _c: CellXY) {}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0616 Constants and F14 (δ is the single-precision value nearest 1/49)
    #[test]
    fn step_is_the_single_nearest_one_49th() {
        let d = step().to_f64();
        let m = Rounding::Nearest;
        let n = F64::from_i64(1, m).div(F64::from_i64(49, m), m);
        // Within half a single-precision ulp at this exponent (2⁻³⁰).
        let half_ulp = F64::from_parts(false, 1, -30, m);
        assert!(d.sub(n, m).abs().lt(half_ulp));
        assert!(d.lt(n), "the nearest single is just below 1/49");
    }

    /// spec: S-0616 F14 (Last sample: t₀ = 0, so s₀ = P; t₄₉ follows the rounding rule, 1 − 2⁻²⁴ under round-toward-zero and 1.0 to nearest)
    #[test]
    fn last_sample_follows_the_mode() {
        let one = F64::from_i64(1, Rounding::Nearest);
        assert!(fraction(49, Rounding::Nearest).eq_num(one));
        let below = one.sub(F64::from_parts(false, 1, -24, Rounding::Nearest), Rounding::Nearest);
        assert!(fraction(49, Rounding::TowardZero).eq_num(below));
        // The spec's example: P = (2688, 2688, 60), Q = (3840, 2688, 0).
        let (p, q) = (Pos::new(2688, 2688, 60), Pos::new(3840, 2688, 0));
        for m in [Rounding::Nearest, Rounding::TowardZero] {
            assert_eq!(sample(p, q, 0, m), p);
        }
        assert_eq!(sample(p, q, 49, Rounding::TowardZero), Pos::new(3839, 2688, 0));
        assert_eq!(sample(p, q, 49, Rounding::Nearest), q);
    }

    /// spec: S-0616 F14 (t₃ is 0.06122448667883873 in the original mode and 0.06122449040412903 with quirks.chop_rounding off)
    #[test]
    fn third_parameter_differs_by_mode() {
        // The decimals denote single-precision values: the nearest single of each.
        let single = |t: &str| {
            crate::rules::nearest_double(t)
                .to_f32(Rounding::Nearest)
                .to_f64()
        };
        let (rz, rn) = (single("0.06122448667883873"), single("0.06122449040412903"));
        assert!(fraction(3, Rounding::TowardZero).eq_num(rz));
        assert!(fraction(3, Rounding::Nearest).eq_num(rn));
    }

    /// spec: S-0616 scenario P22 part d (P = (2560, 2688, 60), Q = (15104, 2688, 0): sample 3 at x = 3327 in cell 12, sample 49 at x = 15103 in cell 58 in the original mode)
    #[test]
    fn p22_part_d_samples() {
        let (p, q) = (Pos::new(2560, 2688, 60), Pos::new(15104, 2688, 0));
        let m = Rounding::TowardZero;
        assert_eq!(sample(p, q, 3, m).x, 3327);
        assert_eq!(cell_of(sample(p, q, 3, m)).x, 12);
        assert_eq!(sample(p, q, 49, m).x, 15103);
        assert_eq!(cell_of(sample(p, q, 49, m)).x, 58);
        // Rounding to nearest skips cell 12 at sample 3.
        assert_ne!(cell_of(sample(p, q, 3, Rounding::Nearest)).x, 12);
    }

    /// spec: S-0306 claim 27 (Degenerate comparisons: a candidate exactly on an axis-aligned beam gives h = 0, so D is not a number; one lepton off it a finite distance)
    #[test]
    fn collinear_distance_is_not_a_number() {
        let m = Rounding::TowardZero;
        let (p, q) = (Pos::new(1000, 1000, 0), Pos::new(5000, 1000, 0));
        assert!(line_distance(p, q, Pos::new(3000, 1000, 0), true, m).is_nan());
        let d = line_distance(p, q, Pos::new(3000, 1001, 0), true, m);
        assert!(!d.is_nan());
        assert!(d.gt(F64::ZERO) && d.lt(F64::from_i64(2, m)));
        // The quirk-off form is finite on the line.
        assert!(euclidean_line_distance(p, q, Pos::new(3000, 1000, 0), true, m).eq_num(F64::ZERO));
    }

    /// spec: S-0306 claim 27 (D = |N / r(L)| from the double cross product: a candidate 100 leptons off a diagonal beam is about 100 leptons from it)
    #[test]
    fn diagonal_distance_is_close_to_euclidean() {
        let m = Rounding::TowardZero;
        let (p, q) = (Pos::new(0, 0, 0), Pos::new(3000, 4000, 0));
        // (400, −300) is perpendicular to (3, 4) with length 500; scaled to 100 leptons: (80, −60).
        let c = Pos::new(1500 + 80, 2000 - 60, 0);
        let d = line_distance(p, q, c, true, m);
        let lo = F64::from_i64(99, m);
        let hi = F64::from_i64(101, m);
        assert!(d.gt(lo) && d.lt(hi), "{d:?}");
    }
}
