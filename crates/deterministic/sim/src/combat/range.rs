//! The range test (S-0307): effective range, the elevation bonus, the
//! measured and source points, the two comparisons, the high-bridge
//! refusals and the line of fire; and the firer's weapon range.

use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;
use glasnost_num::trig::{cos_t, sin_t};
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

/// The outcome of the approach-position search (S-0307 claims 15 to 17).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Approach {
    /// A position was found: the destination becomes this cell and the
    /// target is kept (claims 15 and 16).
    Cell(CellXY),
    /// No position was found by a hunter-seeker or a vehicle thief: the
    /// destination becomes the target object itself and the target is kept
    /// (claim 17).
    TargetItself,
    /// No position was found by any other actor: the target is dropped and
    /// the destination becomes the nearby cell, or is cleared when there is
    /// none (claim 17).
    Drop(Option<CellXY>),
}

/// The bearing differences of F4, in 1/256-turn units, in preference order.
// spec: S-0307 F4 (bearing differences 0, +1, −1, … +6, −6, +8, −8, +16, −16, +24, −24, +32, −32, +48, −48, +64, −64)
const BEARINGS: [i32; 25] = [
    0, 1, -1, 2, -2, 3, -3, 4, -4, 5, -5, 6, -6, 8, -8, 16, -16, 24, -24, 32, -32, 48, -48, 64, -64,
];

/// A vehicle's alternative cells of an obstructed primary cell, in
/// preference order (S-0307 claim 16).
// spec: S-0307 claim 16 (displacements (0, −1), (1, −2), (2, −2), (3, −1), (3, 0), (2, 1), (1, 1), (0, 0))
const ALTERNATIVES: [(i32, i32); 8] = [
    (0, -1),
    (1, -2),
    (2, -2),
    (3, -1),
    (3, 0),
    (2, 1),
    (1, 1),
    (0, 0),
];

/// The radii of S-0307 F4 for the adjusted reach `r_prime` (ρ′), the
/// building lengthening `beta` and whether the actor is infantry.
pub fn approach_radii(w: &World, r_prime: i32, beta: i32, infantry: bool) -> Vec<i32> {
    let mode = w.rules().mode;
    // spec: S-0307 F4 and Constants (margin a: 281.6 leptons for infantry, 179.2 for vehicles, the nearest doubles)
    let margin = if infantry {
        crate::rules::nearest_double("281.6")
    } else {
        crate::rules::nearest_double("179.2")
    };
    // spec: S-0307 F4 (u = max(0, trunc(RZ₅₃(ρ′ + β − a))); ρ′ + β wraps as a 32-bit sum)
    let sum = wrap(i64::from(r_prime) + i64::from(beta));
    let u = F64::from_i64(i64::from(sum), mode)
        .sub(margin, mode)
        .to_i32_common()
        .max(0);
    // spec: S-0307 F4 (u′ = 256 when 0 < u ≤ 256, else u)
    let first = if u > 0 && u <= 256 { 256 } else { u };
    // spec: S-0307 F4 and Constants (radii u′, u′ − 256, …, strictly greater than 204.8 leptons)
    let cut = crate::rules::nearest_double("204.8");
    (0..)
        .map(|k: i32| first.wrapping_sub(k.wrapping_mul(256)))
        .take_while(|&rad| F64::from_i64(i64::from(rad), mode).gt(cut))
        .collect()
}

/// The approach-position search of S-0307 claims 15 to 17, for the actor's
/// current target, started with the adjusted reach `r_prime` (ρ′ of F4,
/// which is S-0215 F10's r′, so the caller passes the r′ it has worked
/// out, including that spec's deploy-to-fire replacement).
///
/// It is meant to be called by S-0215 claim 37's approach step when that
/// step asks for a search, and only for an infantry unit or vehicle that
/// has a target. It changes nothing; the caller applies the outcome:
/// `Cell(c)` sets the destination to `c` and keeps the target,
/// `TargetItself` sets the destination to the target object and keeps it,
/// and `Drop(n)` clears the target and sets the destination to `n` (none
/// clears it). `None`: the actor has no target or no type.
///
/// Claims 15 and 16 cover ground units on flat height-0 terrain without
/// bridges, a target that is an object, and types without the close-range
/// property; the same rules are applied to every other case meanwhile
/// (request Q-0072, which the revision answers only in part). The regions
/// must be up to date ([`crate::regions::ensure`]).
pub fn approach_search(w: &World, id: ObjectId, r_prime: i32) -> Option<Approach> {
    let r = w.rules();
    let mode = r.mode;
    let o = w.obj(id)?;
    let t = o.target?;
    let u = r.unit(o.kind, o.type_index)?;
    let (_, weapon) = super::weapon::would_use(w, id, Some(t));
    // spec: S-0307 F4 (T: the target's reference point; the attacker's reference point)
    let tp = target_point(w, t);
    let ap = w.reference_point(id);
    let beta = match t {
        Target::Object(x) => w
            .obj(x)
            .filter(|x| x.kind == Kind::Building)
            .map_or(0, |x| 64 * (x.foundation.0 + x.foundation.1)), // spec: S-0307 F4 and Constants (β: the building lengthening of claim 9, 64 leptons per foundation cell of width plus height)
        Target::Cell(_) => 0,
    };
    let radii = approach_radii(w, r_prime, beta, o.kind == Kind::Infantry);
    // spec: S-0307 F4 (b: S-0204's 256-step facing of the direction from T to the attacker's reference point)
    let f = glasnost_num::geom::direction(
        i64::from(ap.x - tp.x),
        i64::from(ap.y - tp.y),
        r.quirks.table_direction,
    );
    let b = glasnost_num::geom::n_step(f, 256) as i32; // spec: S-0307 F4 (a 256-step facing)
    // spec: S-0307 F4 (c = the double nearest −0.00009587672516830327 radians per facing unit)
    let c = crate::rules::nearest_double("-0.00009587672516830327");
    let a_cell = ap.cell();
    let t_cell = tp.cell();
    let ground = Layer::Ground;
    let clear = |cell: CellXY| -> bool {
        // spec: S-0307 claim 15 (a clear cell: enterable by class for its class and zone, an occupancy record empty as a whole, in the actor's reach region)
        crate::passability::enterable_by_class(r, w, cell, u.class, u.zone, false)
            && w.cells.occ(cell, ground).is_none_or(|x| x.empty())
            && crate::regions::same_region(w, u.zone, o.pos.cell(), o.layer, cell, ground)
    };
    let fits = |cell: CellXY| -> bool {
        // spec: S-0307 claim 15 (a point outside the usable area never qualifies; the range test from the cell centre at world height 0 with the weapon chosen for the target)
        w.cells.usable(cell) && in_range(w, id, weapon, Some(t), Some(Pos::centre(cell, 0)))
    };
    let cheb = |p: CellXY, q: CellXY| (p.x - q.x).abs().max((p.y - q.y).abs());
    let detour = |cell: CellXY| -> bool {
        // spec: S-0307 claim 16 and Constants (E(c, t) ≤ C(c, t) + 8 or E(a, c) ≤ C(a, c) + 8 cells; a failed estimate never satisfies)
        let e = |p: CellXY, q: CellXY| crate::path::coarse_estimate(w, u.zone, p, q);
        e(cell, t_cell).is_some_and(|x| x <= cheb(cell, t_cell) + 8)
            || e(a_cell, cell).is_some_and(|x| x <= cheb(a_cell, cell) + 8)
    };
    let alternatives: &[(i32, i32)] = if r.quirks.approach_alternative_walk {
        &ALTERNATIVES
    } else {
        // spec: S-0307 Edge cases (approach_alternative_walk off: the eight neighbours in step-direction order 0 to 7)
        &crate::regions::STEPS
    };
    for &radius in &radii {
        let rd = F64::from_i64(i64::from(radius), mode);
        for d in BEARINGS {
            // spec: S-0307 F4 (p: the signed representative of 256(b + δ) modulo 65536)
            let p = i64::from((256 * (b + d)).rem_euclid(65536) as u16 as i16);
            // spec: S-0307 F4 (θ = RZ₅₃((p − 16383) × c))
            let theta = F64::from_i64(p - 16383, mode).mul(c, mode);
            // spec: S-0307 F4 (x = trunc(RZ₅₃(T.x + RZ₅₃(cosₜ(θ) × radius))), y = trunc(RZ₅₃(T.y − RZ₅₃(sinₜ(θ) × radius))))
            let x = F64::from_i64(i64::from(tp.x), mode)
                .add(cos_t(theta, mode).mul(rd, mode), mode)
                .to_i32_common();
            let y = F64::from_i64(i64::from(tp.y), mode)
                .sub(sin_t(theta, mode).mul(rd, mode), mode)
                .to_i32_common();
            // spec: S-0307 F4 (candidate cell (trunc(x / 256), trunc(y / 256)))
            let cell = CellXY::new(x / 256, y / 256);
            if !fits(cell) {
                continue;
            }
            let chosen = if clear(cell) {
                Some(cell)
            } else if o.kind == Kind::Vehicle {
                // spec: S-0307 claim 16 (a vehicle: the first alternative that passes the range and clear-cell tests)
                alternatives
                    .iter()
                    .map(|&(dx, dy)| CellXY::new(cell.x + dx, cell.y + dy))
                    .find(|&a| fits(a) && clear(a))
            } else {
                // spec: S-0307 claim 16 (an infantry unit skips a primary cell that is not clear)
                None
            };
            // spec: S-0307 claim 16 (only the first qualifying alternative meets the detour limit; when it fails the next primary has preference)
            if let Some(ch) = chosen
                && detour(ch)
            {
                return Some(Approach::Cell(ch));
            }
        }
    }
    // spec: S-0307 claim 17 (no position: a hunter-seeker or a vehicle thief takes the target object as its destination and keeps the target)
    let hunter = u.raw.hunter_seeker.unwrap_or(false);
    if hunter || (o.kind == Kind::Infantry && u.vehicle_thief) {
        return Some(Approach::TargetItself);
    }
    // spec: S-0307 claim 17 (any other actor drops the target; with empty radii the nearby-cell search is centred on the target's reference cell; its centre after candidates is not specified, request Q-0072, interim: the same)
    let nearby = crate::path::nearby(w, t_cell, crate::path::NearbyUse::Approach { unit: id });
    Some(Approach::Drop(nearby))
}

/// Whether a balloon-hover vehicle's approach step takes the target object
/// as its destination and keeps the target (S-0307 claim 18): its weapon
/// chosen for the target exists and has a vertical projectile. The
/// approach step then does nothing else.
pub fn balloon_hover_approach(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let r = w.rules();
    // spec: S-0307 claim 18 (a vehicle with the balloon-hover property)
    let hover = o.kind == Kind::Vehicle
        && r.unit(o.kind, o.type_index)
            .is_some_and(|u| u.raw.balloon_hover.unwrap_or(false));
    if !hover || o.target.is_none() {
        return false;
    }
    let (_, weapon) = super::weapon::would_use(w, id, o.target);
    // spec: S-0307 claim 18 (the weapon and its projectile must both exist, and the projectile be vertical)
    weapon
        .and_then(|x| r.combat.projectile_of(x))
        .is_some_and(|p| p.vertical)
}
