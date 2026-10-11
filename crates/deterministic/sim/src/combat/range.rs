//! The range test (S-0307): effective range, the elevation bonus, the
//! measured and source points, the two comparisons, the high-bridge
//! refusals and the line of fire; and the firer's weapon range.

use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;
use glasnost_num::{F64, int};

use crate::combat::ballistics;
use crate::combat::data::WeaponId;
use crate::combat::lof;
use crate::combat::weapon::airborne_by_height;
use crate::rules::Kind;
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// The unlimited-range sentinel R₀ = −512 (S-0307 claim 2).
// spec: S-0307 Constants (unlimited-range sentinel −512 leptons, a field of −2)
pub const UNLIMITED: i32 = -512;

/// Floor-bound (S-0307 Terms): on the map and not airborne.
fn floor_bound(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.on_map) && !airborne_by_height(w, id)
}

/// A target's reference point (S-0204 claim 4); a cell's centre at its
/// terrain height.
pub fn target_point(w: &World, t: Target) -> Pos {
    match t {
        Target::Object(x) => w.reference_point(x),
        Target::Cell(c) => {
            let p = Pos::centre(c, 0);
            Pos::new(p.x, p.y, w.surface_at(p.x, p.y))
        }
    }
}

/// The measured point M_T (S-0307 claim 7).
pub fn measured_point(w: &World, t: Target) -> Pos {
    let mut m = target_point(w, t);
    let Target::Object(x) = t else { return m };
    if floor_bound(w, x) {
        let c = m.cell();
        let on_deck = w.obj(x).is_some_and(|o| o.layer == Layer::Deck);
        // spec: S-0307 claim 7 (a floor-bound target: terrain height at M_T, plus 416 in a high-bridge cell; quirks.range_target_on_deck)
        let deck = if w.rules().quirks.range_target_on_deck {
            w.high_bridge(c)
        } else {
            // spec: S-0307 Edge cases (range_target_on_deck off: the surface it is on)
            on_deck
        };
        m.z = w.surface_at(m.x, m.y) + if deck { 416 } else { 0 }; // spec: S-0307 Constants (deck 416 leptons)
    }
    m
}

/// The source point S of the firing check (S-0307 claim 8).
pub fn source_point(w: &World, firer: ObjectId, weapon: WeaponId, t: Target) -> Pos {
    let mut s = w.reference_point(firer);
    let r = w.rules();
    if r.combat.weapon(weapon).cell_rangefinding {
        // spec: S-0307 claim 8 (a) (cell rangefinding: the cell centre at the terrain height there, plus 416 on a deck)
        let c = Pos::centre(s.cell(), 0);
        let on_deck = w.obj(firer).is_some_and(|o| o.layer == Layer::Deck);
        s = Pos::new(
            c.x,
            c.y,
            w.surface_at(c.x, c.y) + if on_deck { 416 } else { 0 }, // spec: S-0307 Constants (deck 416 leptons)
        );
    }
    if airborne_by_height(w, firer) {
        // spec: S-0307 claim 8 (b) (an airborne firer: the world height of the target's reference point as it is)
        s.z = target_point(w, t).z;
    }
    s
}

/// ⌊r(n)⌋ of a squared distance (S-0204 claim 5).
fn root(w: &World, n: i64) -> i64 {
    let r = w.rules();
    root_floor(n as u64, r.quirks.table_root_distance, r.mode) as i64
}

/// The 32-bit wrap of S-0307 Formulas.
fn wrap(n: i64) -> i32 {
    // spec: S-0307 Formulas (w(n): the integer congruent to n modulo 2³² in [−2³¹, 2³¹ − 1])
    n as i32
}

/// The elevation bonuses (E₁, E₂) of S-0307 F2.
// spec: S-0307 F2 (a height level is 104 leptons)
fn elevation(w: &World, firer: ObjectId, t: Target, projectile_elevation: bool) -> (i32, i32) {
    let Target::Object(x) = t else { return (0, 0) };
    // spec: S-0307 claim 6 (subject to elevation, both floor-bound; else E = 0)
    if !projectile_elevation || !floor_bound(w, firer) || !floor_bound(w, x) {
        return (0, 0);
    }
    let r = w.rules();
    let g = &r.combat.general;
    let mode = r.mode;
    let level = |id: ObjectId| -> i32 {
        let c: CellXY = w.reference_point(id).cell();
        if r.quirks.elevation_deck_cell_level {
            lof::elevation_level(w, c)
        } else {
            // spec: S-0307 Edge cases (elevation_deck_cell_level off: the 4 levels only for an object on the deck)
            let base = w.cells.cell(c).map_or(0, |cell| i32::from(cell.level));
            base + if w.obj(id).is_some_and(|o| o.layer == Layer::Deck) {
                4 // spec: S-0307 Terms (deck lift of an elevation level: 4 height levels)
            } else {
                0
            }
        }
    };
    // spec: S-0307 F2 (Δh = max(0, λ_A − λ_T))
    let dh = (level(firer) - level(x)).max(0);
    if g.elevation_increment == 0 {
        // spec: S-0307 Edge cases (elevation increment 0: E₁ = E₂ = 0, a departure)
        return (0, 0);
    }
    // spec: S-0307 F2 (k = trunc(Δh / I))
    let k = int::div(dh, g.elevation_increment);
    let f = F64::from_i64(i64::from(k), mode).mul(g.elevation_bonus, mode);
    let fp = f.min(g.elevation_cap);
    // spec: S-0307 F2 (t = trunc(f′) in range, −2⁶³ otherwise)
    let t = match fp.trunc_i128() {
        Some(v) if (i128::from(i64::MIN)..=i128::from(i64::MAX)).contains(&v) => v as i64,
        _ => i64::MIN,
    };
    // spec: S-0307 F2 (E₂ = w(256 t); σ = w(E₂² + (104 Δh)²); E₁ = ⌊r(|σ|)⌋)
    let e2 = wrap(t.wrapping_mul(256));
    let h = 104 * i64::from(dh);
    let sigma = wrap(i64::from(e2) * i64::from(e2) + h * h);
    let e1 = root(w, i64::from(sigma.unsigned_abs())) as i32;
    (e1, e2)
}

/// The range test of the firing check (S-0307 claims 1 to 11, 14). `source`
/// overrides the source point (the approach-position search of claim 12
/// passes a candidate position, without claim 8's substitutions).
pub fn in_range(
    w: &World,
    firer: ObjectId,
    weapon: Option<WeaponId>,
    target: Option<Target>,
    source: Option<Pos>,
) -> bool {
    // spec: S-0307 claim 14 (no target: in range; a target and no weapon: out of range)
    let Some(t) = target else { return true };
    let Some(weapon) = weapon else { return false };
    let r = w.rules();
    let wr = r.combat.weapon(weapon);
    // spec: S-0307 claim 2 (R₀ = −512 passes for every target, nothing else applied)
    if wr.range == UNLIMITED {
        return true;
    }
    let s = source.unwrap_or_else(|| source_point(w, firer, weapon, t));
    let m = measured_point(w, t);
    let target_air = matches!(t, Target::Object(x) if airborne_by_height(w, x));
    let air_bonus = super::veterancy::type_combat(w, firer).map_or(0, |tc| tc.air_range_bonus);
    // spec: S-0307 F1 (R_b = R₀ + a · [target airborne]); garrisons, bunkers and open-topped transports are seams
    let rr = wrap(i64::from(wr.range) + if target_air { i64::from(air_bonus) } else { 0 });
    let proj = r.combat.projectile_of(weapon);
    let (e1, e2) = elevation(w, firer, t, proj.is_some_and(|p| p.elevation));
    let (dx, dy, dz) = (
        i64::from(s.x - m.x),
        i64::from(s.y - m.y),
        i64::from(s.z - m.z),
    );
    let d3 = root(w, dx * dx + dy * dy + dz * dz);
    // spec: S-0307 claim 9 (minimum range, both branches: out when ⌊r⌋ < m)
    if wr.min_range != 0 && d3 < i64::from(wr.min_range) {
        return false;
    }
    let arcing = proj.is_some_and(|p| p.arcing);
    if arcing {
        // spec: S-0307 claim 9 (arcing: planar distance ≤ L = R + E₂ and reachable at v(L, g))
        let l = wrap(i64::from(rr) + i64::from(e2));
        let d2 = root(w, dx * dx + dy * dy);
        if d2 > i64::from(l) {
            return false;
        }
        let floater = proj.is_some_and(|p| p.floater);
        let g = ballistics::gravity(r.combat.general.gravity, floater, r.mode);
        let v = ballistics::arc_speed_g(l, g, r.mode);
        if !ballistics::reachable(
            v,
            d2 as i32,
            m.z - s.z,
            g,
            r.quirks.table_root_distance,
            r.mode,
        ) {
            return false;
        }
        // spec: S-0307 claim 10 (arcing: out when M_T's cell carries a high bridge and M_T.z − S.z ≥ 312)
        if w.high_bridge(m.cell()) && m.z - s.z >= 312 {
            return false;
        }
    } else {
        // spec: S-0307 claim 9 (not arcing: ⌊r⌋ ≤ R + E₁ + β, β = 64 (W + H) for a building target)
        let beta = match t {
            Target::Object(x) => w
                .obj(x)
                .filter(|o| o.kind == Kind::Building)
                .map_or(0, |o| 64 * (o.foundation.0 + o.foundation.1)), // spec: S-0307 Constants (building lengthening 64)
            Target::Cell(_) => 0,
        };
        let limit = wrap(i64::from(rr) + i64::from(e1) + i64::from(beta));
        if d3 > i64::from(limit) {
            return false;
        }
        // spec: S-0307 claim 10 (not arcing: a source under a deck cannot reach a target at or above that deck)
        let sc = s.cell();
        let deck = w.surface_at(s.x, s.y) + 416; // spec: S-0307 Constants (deck 416 leptons)
        if w.high_bridge(sc) && s.z < deck && m.z >= deck {
            return false;
        }
    }
    // spec: S-0307 claim 11 (the line of fire; a wall-damaging warhead passes the wall overlays 2, 26 and 243)
    if let Some(p) = proj {
        let house = w.obj(firer).and_then(|o| o.owner);
        if let Some(c) = lof::walk(w, p, house, s, m) {
            let wall_warhead = r.combat.warhead_of(weapon).is_some_and(|h| h.wall);
            // spec: S-0307 claim 11 and Constants (wall overlay indices 2, 26, 243, engine constants)
            let excepted = lof::wall_overlay(w, c).is_some_and(|o| [2u8, 26, 243].contains(&o));
            if !(wall_warhead && excepted) {
                return false;
            }
        }
    }
    true
}

/// The firer's weapon range ρ(A, i) (S-0307 claim 13): 0 without a weapon
/// in the slot, else the weapon's range field; open-topped transports
/// (their passengers' shortest range) are a seam.
pub fn firer_weapon_range(w: &World, firer: ObjectId, slot: usize) -> i32 {
    // spec: S-0307 claim 13 (0 without a weapon, else R₀ with no bonus)
    super::weapon::slot_weapon(w, firer, slot).map_or(0, |x| w.rules().combat.weapon(x).range)
}
