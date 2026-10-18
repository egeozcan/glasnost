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
//! Interim (Q-1405): F14's step products are formed in the program's
//! rounding mode, with *t*₄₉ = 1.0 as F14 states; the line distance is
//! |(*C* − *P*) × (*Q* − *P*)| / |*Q* − *P*| with both lengths by the table
//! root of S-0204, and a zero-length beam measures |*C* − *P*|.

use glasnost_map::CellXY;
use glasnost_num::{F32, F64, Rounding};

use crate::combat::data::WeaponId;
use crate::rules::Kind;
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// The number of samples along the beam (F14: *i* = 0 … 49).
const SAMPLES: i32 = 50; // spec: S-0616 Constants (railgun end-point samples 50, i = 0 … 49)

/// The significand of the step δ, the single-precision value nearest 1/49:
/// δ = 10956549 × 2⁻²⁹.
const STEP_SIGNIFICAND: u128 = 10_956_549; // spec: S-0616 Constants and F14 (the railgun end-point step: the single-precision value nearest 1/49, about 0.020408163)

/// The binary exponent of δ.
const STEP_EXPONENT: i32 = -29; // spec: S-0616 Constants and F14 (the single-precision value nearest 1/49)

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

/// *t*ᵢ = fl(*i* × δ) of F14, in the program's mode, with *t*₄₉ = 1.0
/// exactly as F14 states (interim, Q-1405: under the round-toward-zero
/// mode the product 49 × δ narrows to the single below 1.0).
pub fn fraction(i: i32, mode: Rounding) -> F64 {
    if i == SAMPLES - 1 {
        // spec: S-0616 F14 (t₄₉ = 1.0 exactly, so s₄₉ = Q; interim Q-1405 for the round-toward-zero mode)
        return F64::from_i64(1, mode);
    }
    // spec: S-0616 F14 (tᵢ = fl(i × δ), single precision; the product of i ≤ 49 and δ is exact in double, so one narrowing gives it)
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
    // spec: S-0616 F14 (sᵢ = trunc(Q × tᵢ + P × (1 − tᵢ)) per component, the sum formed with no single-precision rounding of its intermediate results, truncated toward zero)
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
    // spec: S-0616 F14 (when no tested sample is below the terrain, the end point is Q)
    Walk {
        end: q,
        stop: None,
        visited,
    }
}

/// The distance of `c` from the infinite line through `p` and `q`
/// (S-0306 claim 27; interim formula, Q-1405).
fn line_distance(w: &World, p: Pos, q: Pos, c: Pos) -> F64 {
    let mode = w.rules().mode;
    let table = w.rules().quirks.table_root_distance;
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
        // Interim (Q-1405): a zero-length beam measures the distance from its start.
        return root(sq(e));
    }
    // spec: S-0306 claim 27 (the distance from the infinite line through the start and target positions, in floating point with the table square root, no integer truncation; interim Q-1405 for the exact form)
    let cross = [
        e[1].mul(d[2], mode).sub(e[2].mul(d[1], mode), mode),
        e[2].mul(d[0], mode).sub(e[0].mul(d[2], mode), mode),
        e[0].mul(d[1], mode).sub(e[1].mul(d[0], mode), mode),
    ];
    root(sq(cross)).div(root(dd), mode)
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
    let radius = F64::from_i64(i64::from(rules.combat.general.railgun_radius), rules.mode);
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
            // spec: S-0306 claim 27 (buildings count as distance 0; the shot target qualifies; others when strictly below the railgun hit distance; no alliance test)
            let near = if o.kind == Kind::Building {
                F64::ZERO.lt(radius)
            } else {
                line_distance(w, p, q, o.pos).lt(radius)
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

    /// spec: S-0616 F14 (t₀ = 0 and t₄₉ = 1.0 exactly, so s₀ = P and s₄₉ = Q)
    #[test]
    fn ends_are_exact() {
        for m in [Rounding::Nearest, Rounding::TowardZero] {
            let p = Pos::new(1000, 2000, 300);
            let q = Pos::new(7001, -5003, 17);
            assert_eq!(sample(p, q, 0, m), p);
            assert_eq!(sample(p, q, 49, m), q);
        }
        // Under round to nearest 49 × δ narrows to 1.0 without the pin.
        let m = Rounding::Nearest;
        let t = F64::from_i64(49, m)
            .mul(step().to_f64(), m)
            .to_f32(m)
            .to_f64();
        assert!(t.eq_num(F64::from_i64(1, m)));
    }
}
