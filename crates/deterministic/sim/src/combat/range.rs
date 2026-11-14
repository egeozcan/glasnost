//! The range test (S-0307): effective range, the elevation bonus, the
//! measured and source points, the two comparisons, the high-bridge
//! refusals and the line of fire; and the firer's weapon range.

use crate::table_trig::{cos_q, sin_q};
use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;
use glasnost_num::{F64, int};

use crate::combat::ballistics;
use crate::combat::data::WeaponId;
use crate::combat::lof;
use crate::combat::weapon::{range_airborne, range_floor_bound};
use crate::rules::Kind;
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// The unlimited-range sentinel R₀ = −512 (S-0307 claim 2).
// spec: S-0307 Constants (unlimited-range sentinel −512 leptons, a field of −2)
pub const UNLIMITED: i32 = -512;

/// Floor-bound (S-0307 Terms, claim 26): on the map and not airborne; the
/// set-A and set-B rocket types by their flight phase (#253).
fn floor_bound(w: &World, id: ObjectId) -> bool {
    // spec: S-0307 claim 26 (elevation eligibility, claim 6, and target-height replacement, claim 7)
    range_floor_bound(w, id)
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
    if range_airborne(w, firer) {
        // spec: S-0307 claim 8 (b) (an airborne firer: the world height of the target's reference point as it is); claim 26 (the set-A and set-B rocket types by their in-the-air answer)
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
    // spec: S-0307 Edge cases (quirks.negative_elevation_cap: the cap as it is; off: max(0, c) in F2)
    let cap = if r.quirks.negative_elevation_cap {
        g.elevation_cap
    } else {
        g.elevation_cap.max(F64::ZERO)
    };
    // spec: S-0307 F2 (f′ = min(f, c))
    let fp = f.min(cap);
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
    // spec: S-0307 claim 3 and claim 26 (the air bonus: the set-A and set-B rocket types by their in-the-air answer)
    let target_air = matches!(t, Target::Object(x) if range_airborne(w, x));
    let air_bonus = super::veterancy::type_combat(w, firer).map_or(0, |tc| tc.air_range_bonus);
    // spec: S-0307 F1 (R_b = R₀ + a · [target airborne]); the open-topped bonus is a seam
    let rr = if crate::structures::garrison::garrisoned(w, firer) {
        // spec: S-0307 claim 4 (a garrisoned building's range is 256 × (⌊min(W, H) / 2⌋ + G) leptons: the weapon's own range is not used, nor, with quirks.garrison_range_drops_air_bonus, the air bonus)
        let (fw, fh) = w.obj(firer).map_or((0, 0), |o| o.foundation);
        let cells = fw.min(fh) / 2 + r.combat.general.occupy_weapon_range;
        wrap(
            256 * i64::from(cells)
                + if target_air && !r.quirks.garrison_range_drops_air_bonus {
                    i64::from(air_bonus)
                } else {
                    0
                },
        )
    } else {
        wrap(i64::from(wr.range) + if target_air { i64::from(air_bonus) } else { 0 })
    };
    // spec: S-0307 claim 5 and F1 (R = R_b + 256 B_k when the firer is bunkered and is not a building; signed 32-bit wrapping, S-0205 claim 1)
    let rr = if crate::structures::bunker::bunkered(w, firer)
        && w.obj(firer)
            .is_some_and(|o| o.kind != crate::rules::Kind::Building)
    {
        wrap(i64::from(rr) + 256 * i64::from(r.combat.general.bunker_range_bonus))
    } else {
        rr
    };
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
/// (the shortest range of their passengers' current weapons) are a seam.
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
    /// The close-range shortcut (claim 22 b): the movement is sent to this
    /// standing position in the attacker's own cell, the destination becomes
    /// this cell and the target is kept.
    Shortcut(Pos, CellXY),
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

/// The single radius of a close-range infantry search, leptons.
// spec: S-0307 claim 22 (a) (332 leptons; u′ compared with 332.8, the same as u′ ≤ 332)
const CLOSE_RADIUS: i32 = 332;
/// The largest d₀ of the close-range shortcut, leptons.
// spec: S-0307 claim 22 (b) (d₀ ≤ 384 leptons)
const SHORTCUT_REACH: i64 = 384;
/// The range bypass of a close-range search, leptons.
// spec: S-0307 claim 22 (d) (at most 307 leptons; compared with 307.2)
const BYPASS_REACH: i64 = 307;

/// F4's first radius u′ for the adjusted reach `r_prime` (ρ′), the
/// building lengthening `beta` and whether the actor is infantry.
fn approach_u_prime(w: &World, r_prime: i32, beta: i32, infantry: bool) -> i32 {
    let mode = w.rules().mode;
    // spec: S-0307 F4 and Constants (margin a: 281.6 leptons for infantry, 179.2 for vehicles, the nearest doubles)
    let margin = if infantry {
        crate::consts::APPROACH_MARGIN_INFANTRY
    } else {
        crate::consts::APPROACH_MARGIN_VEHICLE
    };
    // spec: S-0307 F4 (u = max(0, trunc(RZ₅₃(ρ′ + β − a))); ρ′ + β wraps as a 32-bit sum)
    let sum = wrap(i64::from(r_prime) + i64::from(beta));
    let u = F64::from_i64(i64::from(sum), mode)
        .sub(margin, mode)
        .to_i32_common()
        .max(0);
    // spec: S-0307 F4 (u′ = 256 when 0 < u ≤ 256, else u)
    if u > 0 && u <= 256 { 256 } else { u }
}

/// The radii of S-0307 F4 for the adjusted reach `r_prime` (ρ′), the
/// building lengthening `beta` and whether the actor is infantry.
pub fn approach_radii(w: &World, r_prime: i32, beta: i32, infantry: bool) -> Vec<i32> {
    let mode = w.rules().mode;
    let first = approach_u_prime(w, r_prime, beta, infantry);
    // spec: S-0307 F4 and Constants (radii u′, u′ − 256, …, strictly greater than 204.8 leptons)
    let cut = crate::consts::APPROACH_RADIUS_CUT;
    (0..)
        .map(|k: i32| first.wrapping_sub(k.wrapping_mul(256))) // spec: S-0307 F4 (steps of 256 leptons)
        .take_while(|&rad| F64::from_i64(i64::from(rad), mode).gt(cut))
        .collect()
}

/// The approach-position search of S-0307 claims 15 to 17 and 19 to 24,
/// for the actor's current target, started with the adjusted reach
/// `r_prime` (ρ′ of F4, which is S-0215 F10's r′, so the caller passes the
/// r′ it has worked out, including that spec's deploy-to-fire replacement).
///
/// It is meant to be called by S-0215 claim 37's approach step when that
/// step asks for a search, and only for an infantry unit or vehicle that
/// has a target. It takes the position-choice draws of claim 22 (f) and
/// replaces a destroyable-cliff cell target (claim 23); the caller applies
/// the outcome: `Cell(c)` sets the destination to `c` and keeps the target,
/// `Shortcut(p, c)` sends the movement to `p`, sets the destination to `c`
/// and keeps the target, `TargetItself` sets the destination to the target
/// object and keeps it, and `Drop(n)` clears the target and sets the
/// destination to `n` (none clears it). `None`: the actor has no target or
/// no type.
///
/// Every search in our engine may change the actor (S-0307 claim 17): no
/// rule of ours asks for one that may not. A jumpjet's detour exemption
/// (claim 24) is the early return of the detour test.
/// The regions must be up to date ([`crate::regions::ensure`]).
pub fn approach_search(w: &mut World, id: ObjectId, r_prime: i32) -> Option<Approach> {
    let rules = w.rules_arc();
    let r = &*rules;
    let mode = r.mode;
    let o = w.obj(id)?.clone();
    let mut t = o.target?;
    let u = r.unit(o.kind, o.type_index)?;
    if let Target::Cell(c) = t
        && destroyable_cliff(w, c)
    {
        // spec: S-0307 claim 23 (a destroyable-cliff cell target is first replaced by one cell of its tile)
        t = Target::Cell(cliff_replacement(w, id, c));
        if let Some(ob) = w.obj_mut(id) {
            ob.target = Some(t);
            // spec: S-0309 claim 24 (every setting of a target leaves a target that is not an opportunity target)
            ob.combat.opportunity = false;
        }
        // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
        crate::emitters::retarget_let_go(w, id);
    }
    // spec: S-0307 claim 22 (the close-range property)
    let close = u.raw.close_range.unwrap_or(false);
    let infantry = o.kind == Kind::Infantry;
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
    let radii = if close && infantry && approach_u_prime(w, r_prime, beta, true) <= CLOSE_RADIUS {
        // spec: S-0307 claim 22 (a) (close-range infantry with u′ ≤ 332: the single radius 332)
        vec![CLOSE_RADIUS]
    } else {
        approach_radii(w, r_prime, beta, infantry)
    };
    // spec: S-0307 claim 22 (f) (the class of T is the desired class of every position choice)
    let t_class = crate::walk::position_class(
        tp.x.rem_euclid(256), // spec: S-0204 claim 2 (the position within the cell)
        tp.y.rem_euclid(256), // spec: S-0204 claim 2 (the position within the cell)
        true,
    );
    let planar = |w: &World, a: Pos, b: Pos| -> i64 {
        let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
        // spec: S-0204 claim 6 (the planar distance: ⌊r(Δx² + Δy²)⌋, the table square root)
        root_floor(
            (dx * dx + dy * dy) as u64,
            w.rules().quirks.table_root_distance,
            w.rules().mode,
        ) as i64
    };
    if close {
        // spec: S-0307 claim 22 (b) (q: the centre of the target's cell for a vehicle, infantry unit, aircraft or building, else T)
        let q = match t {
            Target::Object(x) => w
                .obj(x)
                .filter(|x| {
                    matches!(
                        x.kind,
                        Kind::Vehicle | Kind::Infantry | Kind::Aircraft | Kind::Building
                    )
                })
                .map_or(tp, |x| Pos::centre(x.pos.cell(), 0)),
            Target::Cell(_) => tp,
        };
        let qc = Pos::centre(q.cell(), 0);
        let own = o.pos.cell();
        // spec: S-0307 claim 22 (b) (d₀: from the centre of the attacker's cell to the centre of the cell holding q)
        let d0 = planar(w, Pos::centre(own, 0), qc);
        if d0 <= SHORTCUT_REACH {
            // spec: S-0307 claim 22 (b) (P: the position choice in the attacker's own cell, desired point T, its ground record whatever its surface; its own entry counts as taken)
            if let Some(p) = crate::walk::choose_position(w, own, Layer::Ground, t_class, false) {
                let (su, sv) = crate::walk::standing_point(p);
                let (x, y) = (256 * own.x + su, 256 * own.y + sv); // spec: S-0208 table P (a standing position within the cell)
                // spec: S-0208 claim 1 (the surface height there, 416 above on a bridge deck)
                let z = w.surface_at(x, y) + if o.layer == Layer::Deck { 416 } else { 0 };
                let pt = Pos::new(x, y, z);
                // spec: S-0307 claim 22 (b) (d₁ < d₀: the search ends at once, sent to P with the cell of T as destination; the target is kept)
                if planar(w, pt, qc) < d0 {
                    return Some(Approach::Shortcut(pt, tp.cell()));
                }
            }
        }
    }
    // spec: S-0307 F4 (b: S-0204's 256-step facing of the direction from T to the attacker's reference point)
    let f = glasnost_num::geom::direction(
        i64::from(ap.x - tp.x),
        i64::from(ap.y - tp.y),
        r.quirks.table_direction,
    );
    let b = glasnost_num::geom::n_step(f, 256) as i32; // spec: S-0307 F4 (a 256-step facing)
    // spec: S-0307 F4 (c = the double nearest −0.00009587672516830327 radians per facing unit)
    let c = crate::consts::WALK_C2;
    let a_cell = ap.cell();
    let t_cell = tp.cell();
    let q = &r.quirks;
    // spec: S-0307 claim 22 (e) (an attacker in the ground layer has its own occupancy entry withdrawn during each clear-cell test)
    let withdraw = close && o.layer == Layer::Ground;
    let own_vehicle_cells: Vec<CellXY> = match o.drive.as_ref() {
        // spec: S-0307 claim 22 (e) (the entry withdrawn is the one in the ground record: a claim the vehicle holds there, S-0209 claim 1)
        Some(d) if o.kind == Kind::Vehicle => d
            .claims
            .iter()
            .filter(|x| x.1 == Layer::Ground)
            .map(|x| x.0)
            .collect(),
        _ => crate::teleport::mark_cell(w, id).into_iter().collect(),
    };
    let clear = |w: &World, cell: CellXY| -> bool {
        // spec: S-0307 claims 15 and 21 (winged class: the clear-cell test accepts without the region, occupancy, height, land or wall tests; the range, detour and primary usable-area tests still apply)
        if u.class == Some(glasnost_rules::sim::LocomotionClass::Winged) {
            return true;
        }
        // spec: S-0307 claim 19 (a high-bridge cell reads its deck occupancy, the deck bypassing the land multiplier; other cells the ground)
        let deck = w.high_bridge(cell);
        let layer = if deck { Layer::Deck } else { Layer::Ground };
        let mut rec = w.cells.occ(cell, layer).copied().unwrap_or_default();
        if withdraw {
            // The withdrawal is a test on a copy of the record, not a change of the
            // cell's occupants and their order (open in S-0307, interim, request Q-0172).
            for m in rec.marks.iter_mut() {
                if matches!(m, Some(crate::cells::Mark { by: crate::cells::Marker::Infantry(who, _), .. }) if *who == id)
                {
                    *m = None;
                }
            }
            if o.kind == Kind::Vehicle
                && layer == Layer::Ground
                && own_vehicle_cells.contains(&cell)
            {
                rec.vehicles = rec.vehicles.saturating_sub(1);
            }
        }
        // spec: S-0307 claims 15 and 19 (revision 20, Q-2171: a clear cell: enterable by class for its class and zone, an occupancy record empty as a whole, and the candidate's reach region, read for its deck when its cell carries a high bridge and its own otherwise, the actor's: from its actual cell, read for its deck when it stands on one, S-0212 claim 4)
        crate::passability::enterable_by_class(r, w, cell, u.class, u.zone, deck)
            && rec.empty()
            && crate::regions::same_region_read(
                w,
                u.zone,
                o.pos.cell(),
                o.layer == Layer::Deck,
                cell,
                deck,
            )
    };
    // The candidate's height (claim 19): a primary at the terrain height of
    // its centre, 416 leptons up on a high bridge; a vehicle's alternative
    // at world height 0 under `approach_alternative_zero_height`.
    let height = |w: &World, cell: CellXY, alternative: bool| -> i32 {
        if alternative && q.approach_alternative_zero_height {
            // spec: S-0307 claim 19 (a vehicle's alternatives keep world height 0; quirks.approach_alternative_zero_height)
            return 0;
        }
        let c = Pos::centre(cell, 0);
        // spec: S-0307 claim 19 and Edge cases (the terrain height at the cell centre, plus 416 leptons on a high bridge)
        w.surface_at(c.x, c.y) + if w.high_bridge(cell) { 416 } else { 0 }
    };
    let ranged = |w: &World, point: Pos| -> bool {
        // spec: S-0307 claims 15 and 19 (the range test from the candidate's point with the weapon chosen for the target; the elevation bonus still reads the firer's own cell)
        in_range(w, id, weapon, Some(t), Some(point))
            // spec: S-0307 claim 22 (d) (a close-range attacker's point within 307 leptons of T passes)
            || (close && planar(w, point, tp) <= BYPASS_REACH)
    };
    let usable = |w: &World, cell: CellXY, alternative: bool| -> bool {
        // spec: S-0307 claim 21 (the usable-area test rejects primaries, not a vehicle's alternatives; quirks.approach_alternative_outside_area)
        (alternative && q.approach_alternative_outside_area) || w.cells.usable(cell)
    };
    let cheb = |p: CellXY, q: CellXY| (p.x - q.x).abs().max((p.y - q.y).abs());
    // spec: S-0307 claim 19 (the target end on the deck for a deck-standing mobile target or a high-bridge cell target)
    let t_deck = match t {
        Target::Object(x) => w.obj(x).is_some_and(|x| x.layer == Layer::Deck),
        Target::Cell(c) => w.high_bridge(c),
    };
    // spec: S-0307 claim 19 and S-0212 claim 7 (the attacker end on its movement layer)
    let a_deck = o.layer == Layer::Deck;
    let detour = |w: &World, cell: CellXY| -> bool {
        // spec: S-0307 claim 24 (a jumpjet: a clear primary, or the first qualifying alternative, is accepted without either inequality of claim 16)
        if crate::jump::is_jumpjet(w, id) {
            return true;
        }
        // spec: S-0307 claim 19 (the candidate end on the deck when its cell carries a high bridge)
        let c_deck = w.high_bridge(cell);
        let e = |p: CellXY, pd: bool, q: CellXY, qd: bool| {
            crate::path::coarse_estimate_layers(w, u.zone, p, pd, q, qd, None)
        };
        // spec: S-0307 claim 16 and Constants (E(c, t) <= C(c, t) + 8 or E(a, c) <= C(a, c) + 8 cells; a failed estimate never satisfies)
        e(cell, c_deck, t_cell, t_deck).is_some_and(|x| x <= cheb(cell, t_cell) + 8) // spec: S-0307 claim 16 (8 cells)
            || e(a_cell, a_deck, cell, c_deck).is_some_and(|x| x <= cheb(a_cell, cell) + 8) // spec: S-0307 claim 16 (8 cells)
    };
    let deploy_to_fire_actor = o.kind == Kind::Vehicle && u.vehicle.deploy_to_fire;
    let alternatives: &[(i32, i32)] = if q.approach_alternative_walk {
        &ALTERNATIVES
    } else {
        // spec: S-0307 Edge cases (approach_alternative_walk off: the eight neighbours in step-direction order 0 to 7)
        &crate::regions::STEPS
    };
    // spec: S-0307 claim 17 (the failure centre: the last primary passing the range test, or its alternative that failed only the detour limit)
    let mut anchor: Option<CellXY> = None;
    for &radius in &radii {
        let rd = F64::from_i64(i64::from(radius), mode);
        for d in BEARINGS {
            // spec: S-0307 F4 (p: the signed representative of 256(b + δ) modulo 65536)
            let p = i64::from((256 * (b + d)).rem_euclid(65536) as u16 as i16);
            // spec: S-0307 F4 (θ = RZ₅₃((p − 16383) × c))
            let theta = F64::from_i64(p - 16383, mode).mul(c, mode);
            // spec: S-0307 F4 (x = trunc(RZ₅₃(T.x + RZ₅₃(cosₜ(θ) × radius))), y = trunc(RZ₅₃(T.y − RZ₅₃(sinₜ(θ) × radius))))
            let x = F64::from_i64(i64::from(tp.x), mode)
                .add(cos_q(w, theta, mode).mul(rd, mode), mode)
                .to_i32_common();
            let y = F64::from_i64(i64::from(tp.y), mode)
                .sub(sin_q(w, theta, mode).mul(rd, mode), mode)
                .to_i32_common();
            // spec: S-0307 F4 (candidate cell (trunc(x / 256), trunc(y / 256)))
            let mut cell = CellXY::new(x / 256, y / 256);
            let point = if close && infantry {
                // spec: S-0307 claim 22 (c) and (f) (an infantry primary's point: the position choice in the primary cell, desired point T, its ground record; its draw comes before the range test)
                match crate::walk::choose_position(w, cell, Layer::Ground, t_class, false) {
                    Some(sp) => {
                        let (su, sv) = crate::walk::standing_point(sp);
                        let (px, py) = (256 * cell.x + su, 256 * cell.y + sv); // spec: S-0208 table P (a standing position within the cell)
                        // spec: S-0307 claim 22 (c) (the surface height at the point, plus 416 leptons on a high bridge)
                        let z = w.surface_at(px, py) + if w.high_bridge(cell) { 416 } else { 0 };
                        Pos::new(px, py, z)
                    }
                    None => {
                        // spec: S-0307 claim 22 (c) (no position: the map origin at the surface height there, its cell the cell at (0, 0))
                        cell = CellXY::new(0, 0);
                        Pos::new(0, 0, w.surface_at(0, 0))
                    }
                }
            } else {
                Pos::centre(cell, height(w, cell, false))
            };
            // spec: S-0517 claim 14 (in the approach search of a deploy-to-fire vehicle a primary cell is considered only when it passes the deploy-to-fire cell test, and a skipped primary never anchors the failure outcome)
            if deploy_to_fire_actor && !crate::econ::inplace::deploy_test_cell(w, cell) {
                continue;
            }
            if !ranged(w, point) {
                continue;
            }
            // spec: S-0307 claim 17 (a primary passing the range test is the failure anchor, whether or not it is usable or clear)
            anchor = Some(cell);
            // spec: S-0307 claim 15 (a primary outside the usable area never qualifies)
            if !usable(w, cell, false) {
                continue;
            }
            let chosen = if clear(w, cell) {
                Some(cell)
            } else if o.kind == Kind::Vehicle {
                // spec: S-0307 claim 16 (a vehicle: the first alternative that passes the range and clear-cell tests)
                alternatives
                    .iter()
                    .map(|&(dx, dy)| CellXY::new(cell.x + dx, cell.y + dy))
                    .find(|&a| {
                        usable(w, a, true)
                            && ranged(w, Pos::centre(a, height(w, a, true)))
                            && clear(w, a)
                    })
            } else {
                // spec: S-0307 claim 16 (an infantry unit skips a primary cell that is not clear)
                None
            };
            // spec: S-0307 claim 16 (only the first qualifying alternative meets the detour limit; when it fails the next primary has preference)
            if let Some(ch) = chosen {
                if detour(w, ch) {
                    return Some(Approach::Cell(ch));
                }
                // spec: S-0307 claim 17 (an alternative that failed only the detour limit is the centre; a clear primary that failed it leaves the primary's cell)
                anchor = Some(ch);
            }
        }
    }
    // spec: S-0307 claim 17 (no position: a hunter-seeker or a vehicle thief takes the target object as its destination and keeps the target)
    let hunter = u.raw.hunter_seeker.unwrap_or(false);
    if hunter || (infantry && u.vehicle_thief) {
        return Some(Approach::TargetItself);
    }
    // spec: S-0307 claim 17 (any other actor drops the target; the nearby-cell search is centred on the failure anchor's cell, else on the target's reference cell, with the empty-radius parameters)
    let centre = anchor.unwrap_or(t_cell);
    let nearby = crate::path::nearby(w, centre, crate::path::NearbyUse::Approach { unit: id });
    Some(Approach::Drop(nearby))
}

/// Whether cell `c` is a destroyable-cliff cell (S-0214 Terms).
pub(crate) fn destroyable_cliff(w: &World, c: CellXY) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    let cat = w.catalogue.as_deref();
    let (s, n) = cat.map_or((0, 0), |k| (k.roles.destroyable_cliff, k.tile_count()));
    // spec: S-0214 Terms (destroyable-cliff cell: s ≤ t ≤ s + 1 and 0 ≤ t < N)
    cell.tile >= s && cell.tile <= s + 1 && cell.tile >= 0 && cell.tile < n
}

/// The cell of a destroyable-cliff target's tile that replaces it (S-0307
/// claim 23): the qualifying tile cell nearest the cell of the attacker's
/// scatter reference point, ties to the earliest in row-major order, or
/// the cell at map coordinates (0, 0) when none qualifies.
fn cliff_replacement(w: &World, id: ObjectId, c: CellXY) -> CellXY {
    let r = w.rules();
    // spec: S-0307 claim 23 (no qualifying cell: the cell at map coordinates (0, 0))
    let origin = CellXY::new(0, 0);
    let Some(cell) = w.cells.cell(c) else {
        return origin;
    };
    let Some(info) = w
        .catalogue
        .as_deref()
        .and_then(|k| k.tiles.get(usize::try_from(cell.tile).ok()?))
    else {
        return origin;
    };
    let (width, h) = (i32::from(info.width), i32::from(info.height));
    if width == 0 {
        return origin;
    }
    let s = i32::from(cell.sub_tile);
    // spec: S-0307 claim 23 (x₀ = x − (s mod W), y₀ = y − ⌊s / W⌋)
    let (x0, y0) = (c.x - s % width, c.y - s / width);
    // spec: S-0307 claim 23 (c: the cell of the attacker's scatter reference point; S-0208 and S-0207 Terms)
    let from = crate::teleport::heading_cell(w, id).unwrap_or(c);
    let mut best: Option<(i64, CellXY)> = None;
    // spec: S-0307 claim 23 (row-major order: increasing j, then increasing i)
    for j in 0..h {
        for i in 0..width {
            let k = usize::try_from(j * width + i).unwrap_or(usize::MAX);
            let tile_cell = info.has_image && info.cells.get(k).copied().flatten().is_some();
            let at = CellXY::new(x0 + i, y0 + j);
            // spec: S-0307 claim 23 (a tile cell inside the usable area qualifies; the map cell is not checked)
            if !tile_cell || !w.cells.usable(at) {
                continue;
            }
            let (dx, dy) = (i64::from(at.x - from.x), i64::from(at.y - from.y));
            // spec: S-0307 claim 23 (r((Δx)² + (Δy)²), the table square root of S-0204, in cells)
            let d = root_floor(
                (dx * dx + dy * dy) as u64,
                r.quirks.table_root_distance,
                r.mode,
            ) as i64;
            // spec: S-0307 claim 23 (ties: the earliest in row-major order)
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, at));
            }
        }
    }
    best.map_or(origin, |(_, at)| at)
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
