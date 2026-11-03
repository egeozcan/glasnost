//! Projectiles (S-0306): launch, flight, homing, detonation and its point,
//! cluster detonations, and the projectile effects of a detonation
//! (S-0303 claims 21 and 22).

use glasnost_map::CellXY;
use glasnost_num::geom::{
    delta, direction, direction_constants, root_floor, table_angle_real, table_root,
    table_root_real,
};
use glasnost_num::trig::{acos_t, cos_t, sin_t};
use glasnost_num::{F64, Rounding};

use crate::combat::area::{self, Detonation, DetonationResult};
use crate::combat::ballistics;
use crate::combat::data::{ProjectileKind, ProjectileRules, WarheadId, WeaponId};
use crate::combat::effects::{self, Effect};
use crate::combat::range::target_point as ref_point;
use crate::combat::state::Projectile;
use crate::combat::weapon::airborne_by_height;
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, Pos, Target, World};

/// The double nearest −π/32767, the pitch and facing scale of S-0306 Terms.
pub fn kappa() -> F64 {
    // spec: S-0306 Constants (pitch scale: the double nearest −π/32767)
    crate::rules::nearest_double("-0.00009587672516830327")
}

/// θ(f) = (f − 16383) × κ for a facing read as a signed 16-bit value
/// (S-0306 Terms, Formulas).
pub fn theta(f: i32, mode: Rounding) -> F64 {
    let signed = i64::from(f as i16);
    // spec: S-0306 Terms (θ(p) = (p − 16383) × (−π / 32767), p read as signed 16-bit)
    F64::from_i64(signed - 16383, mode).mul(kappa(), mode)
}

/// The facing of an angle, trunc((θ − π/2) × c₂) (S-0306 Terms), as a
/// signed 16-bit value.
pub fn angle_facing(t: F64, mode: Rounding) -> i32 {
    let (c1, c2) = direction_constants();
    // spec: S-0306 Terms (the angle-to-facing map trunc((θ − π/2) × c₂))
    let v = t.sub(c1, mode).mul(c2, mode).trunc_i128().unwrap_or(0);
    i32::from(v as i64 as i16)
}

fn f(n: i64, mode: Rounding) -> F64 {
    F64::from_i64(n, mode)
}

/// ⌊r(n)⌋ of a squared distance formed exactly (S-0306 Terms, d₃ and d₂).
fn root_n(w: &World, n: i128) -> i32 {
    let r = w.rules();
    let n = n.max(0) as u64;
    root_floor(n, r.quirks.table_root_distance, r.mode) as i32
}

/// d₃ between two points (S-0306 Terms).
pub fn d3(w: &World, a: Pos, b: Pos) -> i32 {
    let (dx, dy, dz) = (
        i128::from(a.x - b.x),
        i128::from(a.y - b.y),
        i128::from(a.z - b.z),
    );
    root_n(w, dx * dx + dy * dy + dz * dz)
}

/// d₂ between two points (S-0306 Terms).
pub fn d2(w: &World, a: Pos, b: Pos) -> i32 {
    let (dx, dy) = (i128::from(a.x - b.x), i128::from(a.y - b.y));
    root_n(w, dx * dx + dy * dy)
}

/// |V| = r(RZ₅₃(Vₓ² + V_y² + V_z²)) (S-0306 Terms, F10).
// spec: S-0306 Terms (|V| over three components)
fn speed(w: &World, v: [F64; 3]) -> F64 {
    let mode = w.rules().mode;
    let s = v[0]
        .mul(v[0], mode)
        .add(v[1].mul(v[1], mode), mode)
        .add(v[2].mul(v[2], mode), mode);
    table_root_real(s, w.rules().quirks.table_root_distance, mode)
}

/// The horizontal length H = r(RZ₅₃(Vₓ² + V_y²)).
// spec: S-0306 F11 (the horizontal length of three components)
fn hlen(w: &World, v: [F64; 3]) -> F64 {
    let mode = w.rules().mode;
    let s = v[0].mul(v[0], mode).add(v[1].mul(v[1], mode), mode);
    table_root_real(s, w.rules().quirks.table_root_distance, mode)
}

/// V rescaled to length `l` (S-0306 F10).
// spec: S-0306 F10 (three velocity components)
fn rescale(w: &World, v: [F64; 3], l: F64) -> [F64; 3] {
    let mode = w.rules().mode;
    let zero = F64::ZERO;
    // spec: S-0306 F10 (Ṽ = (100, 0, 0) when V = (0, 0, 0))
    let vt = if v.iter().all(|c| c.eq_num(zero)) {
        [f(100, mode), zero, zero]
    } else {
        v
    };
    let mu = speed(w, vt);
    // spec: S-0306 F10 (f = RZ₅₃(L / μ); components RZ₅₃(f × Ṽᵢ))
    let k = l.div(mu, mode);
    [vt[0].mul(k, mode), vt[1].mul(k, mode), vt[2].mul(k, mode)]
}

/// Builds a velocity from a launch facing, a pitch facing and a start speed
/// (S-0306 claim 7, F1).
// spec: S-0306 F1 (three velocity components)
pub(crate) fn initial_velocity(w: &World, yaw: i32, pitch: i32, s: i32) -> [F64; 3] {
    let r = w.rules();
    let mode = r.mode;
    // spec: S-0306 F1 (s₀ = s when s > 0, 100 when s = 0; quirks.zero_speed_launch)
    let s0 = if s > 0 || !r.quirks.zero_speed_launch {
        s
    } else {
        100
    };
    // spec: S-0306 F1 (L = r(s₀²))
    let l = table_root((i64::from(s0) * i64::from(s0)) as u64, mode).to_f64();
    let ty = theta(yaw, mode);
    let chx = cos_t(ty, mode).mul(l, mode);
    let chy = sin_t(ty, mode).mul(l, mode).negated();
    // spec: S-0306 F1 (M = r(RZ₅₃(χₓ² + χ_y²)))
    let m = table_root_real(
        chx.mul(chx, mode).add(chy.mul(chy, mode), mode),
        r.quirks.table_root_distance,
        mode,
    );
    let tp = theta(pitch, mode);
    let cp = cos_t(tp, mode);
    [
        cp.mul(chx, mode),
        cp.mul(chy, mode),
        sin_t(tp, mode).mul(m, mode),
    ]
}

/// The shot speed of claim 1 (S-0306) for a weapon, from the fire point to
/// the shot target's target point.
fn shot_speed(w: &World, weapon: WeaponId, from: Pos, to: Pos) -> i32 {
    let r = w.rules();
    let mode = r.mode;
    let wr = r.combat.weapon(weapon);
    let Some(p) = r.combat.projectile_of(weapon) else {
        return wr.speed;
    };
    // spec: S-0306 claim 1 (ρ ≠ 0: the weapon's speed field)
    if p.rot != 0 {
        return wr.speed;
    }
    let (dx, dy) = (i64::from(to.x - from.x), i64::from(to.y - from.y));
    let n = dx * dx + dy * dy;
    // spec: S-0306 claim 1 (the sum of squares wraps in 32-bit arithmetic, quirks.shot_distance_wrap; d = trunc(r(|n|)))
    let n = if r.quirks.shot_distance_wrap {
        i64::from(n as i32).unsigned_abs()
    } else {
        n as u64
    };
    let d = root_floor(n, r.quirks.table_root_distance, mode) as i32;
    ballistics::arc_speed(d, r.combat.general.gravity, p.floater, mode)
}

/// The launch elevation of an arcing shot (S-0306 claim 6 a, F4): the pitch
/// facing, or `None` for a void shot.
// spec: S-0306 F4 (θ = π/4 for z > 0; ω = 2(1 + z²/d²))
fn arc_pitch(w: &World, s: i32, dist: i32, dz: i32, lob: bool, g: F64) -> Option<i32> {
    let r = w.rules();
    let mode = r.mode;
    let q = r.quirks;
    let (c1, _) = direction_constants();
    let quarter = c1.div(f(2, mode), mode);
    // spec: S-0306 F4 (z > 0: θ = π/4 whatever v, d and g, quirks.uphill_arc_45)
    if dz > 0 && q.uphill_arc_45 {
        return Some(angle_facing(quarter, mode));
    }
    let solve = |d: i32, steep: bool| -> Option<F64> {
        let v = f(i64::from(s), mode);
        let dd = f(i64::from(d), mode);
        let z = f(i64::from(dz), mode);
        let v2 = v.mul(v, mode);
        // spec: S-0306 F4 (disc = v⁴ − 2gzv² − g²d²; ω = 2(1 + z²/d²); U = (v² − gz ± r(disc)) / ω)
        let disc = v2
            .mul(v2, mode)
            .sub(f(2, mode).mul(g, mode).mul(z, mode).mul(v2, mode), mode)
            .sub(g.mul(g, mode).mul(dd, mode).mul(dd, mode), mode);
        let omega = f(2, mode).mul(
            f(1, mode).add(z.mul(z, mode).div(dd.mul(dd, mode), mode), mode),
            mode,
        );
        if !disc.ge(F64::ZERO) || omega.is_nan() || omega.eq_num(F64::ZERO) {
            return None;
        }
        let rd = table_root_real(disc, q.table_root_distance, mode);
        let num = if steep {
            v2.sub(g.mul(z, mode), mode).sub(rd, mode)
        } else {
            v2.sub(g.mul(z, mode), mode).add(rd, mode)
        };
        let u = num.div(omega, mode);
        if !u.ge(F64::ZERO) {
            return None;
        }
        // spec: S-0306 F4 (θ = acos_t(r(U) / v))
        Some(acos_t(
            table_root_real(u, q.table_root_distance, mode).div(v, mode),
            mode,
        ))
    };
    if dist == 0 && dz < 0 && !q.zero_distance_arc {
        // spec: S-0306 Edge cases (zero_distance_arc off: a lob straight up, the flatter shot straight down)
        return Some(if lob { 0 } else { 32766 });
    }
    let th = solve(dist, lob)?;
    let angle = if lob {
        th
    } else {
        // spec: S-0306 F4 (the flatter shot compares with the solve for d + 1: −θ when θ′ < θ; a failed d + 1 solve launches at θ, a departure)
        match solve(dist + 1, false) {
            Some(tp) if tp.lt(th) => th.negated(),
            _ => th,
        }
    };
    Some(angle_facing(angle, mode))
}

/// The straight launch pitch of S-0306 F13.
fn straight_pitch(w: &World, fire: Pos, q: Pos, target_building: bool, hb: i32) -> i32 {
    let r = w.rules();
    let mode = r.mode;
    let dz = q.z - fire.z;
    // spec: S-0306 F13 (16383 when |Δz| ≤ 200)
    if dz.abs() <= 200 {
        return 16383;
    }
    let dprime = d2(w, fire, q);
    // spec: S-0306 F13 (Z = Δz and τ = 20; a building: Z = 200 H_b − world height of F, τ = 20 when |Z| ≥ 20 else 0; quirks.building_pitch_term)
    let (z, tau) = if target_building && r.quirks.building_pitch_term {
        let z = 200 * hb - fire.z;
        (z, if z.abs() >= 20 { 20 } else { 0 })
    } else {
        (dz, 20)
    };
    let x = f(i64::from(dprime), mode).max(crate::rules::nearest_double("0.05"));
    let y = f(i64::from(z.abs() - tau), mode);
    let (a, i) = table_angle_real(x, y, mode);
    // spec: S-0306 F13 (θ_c = sgn(Z) × A(|Z| − τ, max(d′, 0.05)))
    let tc = if z < 0 { a.negated() } else { a };
    // spec: S-0204 claim 12 and F6 (the exceptions of F3 apply to the straight launch pitch, by the sign of θ_c after its negation)
    i32::from(glasnost_num::geom::pitch_of_angle(tc, i, mode) as i16)
}

/// The scatter of S-0306 F5: displacement `p` moved by radius `sigma` in the
/// direction of draw `xi`.
// spec: S-0306 F5 (2π, the double nearest it)
fn scatter(w: &World, p: (i32, i32), sigma: i32, xi: i32) -> (i32, i32) {
    let r = w.rules();
    let mode = r.mode;
    // spec: S-0306 F5 (k_ξ = the double nearest 1/2147483646; 2π and π/2 the doubles nearest them)
    let k = crate::rules::nearest_double("4.656612877414201e-10");
    let (c1, c2) = direction_constants();
    let two_pi = c1.mul(f(4, mode), mode);
    // spec: S-0306 F5 (f_ξ = trunc(RZ₅₃(RZ₅₃(RZ₅₃(RZ₅₃(ξ k_ξ) 2π) − π/2) c₂)) as signed 16-bit)
    let fx = f(i64::from(xi), mode)
        .mul(k, mode)
        .mul(two_pi, mode)
        .sub(c1, mode)
        .mul(c2, mode)
        .trunc_i128()
        .unwrap_or(0) as i64 as i16;
    let th = theta(i32::from(fx), mode);
    let s = f(i64::from(sigma), mode);
    // spec: S-0306 F5 (Qₓ = trunc(RZ₅₃(Pₓ + RZ₅₃(cos_t θ × σ))), Q_y = trunc(RZ₅₃(P_y − RZ₅₃(sin_t θ × σ))))
    let qx = f(i64::from(p.0), mode)
        .add(cos_t(th, mode).mul(s, mode), mode)
        .to_i32_common();
    let qy = f(i64::from(p.1), mode)
        .sub(sin_t(th, mode).mul(s, mode), mode)
        .to_i32_common();
    (qx, qy)
}

/// A projectile's target point P_T (S-0306 Terms): the reference point,
/// plus a building's aim shift.
// spec: S-0306 Terms (the aim shift, a lepton triple)
fn aim_target_point(w: &World, t: Target) -> Pos {
    let p = ref_point(w, t);
    if let Target::Object(x) = t
        && let Some(o) = w.obj(x)
        && o.kind == Kind::Building
    {
        let s = w.rules().buildings[o.type_index as usize]
            .building
            .aim_shift;
        return Pos::new(p.x + s[0], p.y + s[1], p.z + s[2]);
    }
    p
}

/// The start point of a projectile placed at `p` (S-0306 claim 8, revision
/// 45): `p` itself when it is above the terrain at its *x* and *y*, else 1
/// lepton above that terrain, at the same *x* and *y*. Every placement
/// raises by this rule (shots, fragments, the projectiles a burst releases,
/// superweapon payloads); the launch direction, pitch and velocity and the
/// line-of-fire walk's *X*₁ keep the unraised point, and the later move of
/// an invisible projectile (claim 9) is not raised.
pub fn start_point(w: &World, p: Pos) -> Pos {
    // spec: S-0306 claim 8 (the terrain height at its x and y; a bridge deck there is ignored)
    let terrain = w.surface_at(p.x, p.y);
    if p.z > terrain {
        // spec: S-0306 claim 8 (a position above the terrain is used as it is)
        p
    } else {
        // spec: S-0306 claim 8 (not above the terrain: 1 lepton above it, the same x and y)
        Pos::new(p.x, p.y, terrain + 1)
    }
}

/// Launches the projectile of a shot (S-0306 claims 1 to 9, 24). Returns
/// `None` for a void shot (S-0301 claim 15), whose launch draws stay taken.
#[allow(clippy::too_many_arguments)]
// spec: S-0306 claims 4, 6 and 9 (art height absent 2; direction draw R(0, 2147483646); an invisible projectile's velocity 0)
pub fn launch(
    w: &mut World,
    firer: ObjectId,
    weapon: WeaponId,
    shot_target: Target,
    fire_point: Pos,
    damage: i32,
) -> Option<ObjectId> {
    let rules = w.rules_arc();
    let mode = rules.mode;
    let q = rules.quirks;
    let half = q.ranged_draw_half_span;
    let wr = rules.combat.weapon(weapon).clone();
    let Some(pid) = wr.projectile else {
        // A weapon without a projectile launches nothing (the importer warns).
        return None;
    };
    let p = rules.combat.projectiles[pid as usize].clone();
    let o = w.obj(firer)?.clone();
    let tc = crate::combat::veterancy::type_combat(w, firer)
        .cloned()
        .unwrap_or_default();
    let g = ballistics::gravity(rules.combat.general.gravity, p.floater, mode);
    let shot_tp = ref_point(w, shot_target);
    // spec: S-0306 claim 1 (the shot speed towards the shot target's target point)
    let s1 = shot_speed(w, weapon, fire_point, aim_target_point(w, shot_target));
    // spec: S-0306 claim 3 (the aim point: the firer's target's reference point, no aim shift; no target: the origin, quirks.aim_origin_without_target)
    let firer_target = o.target;
    let redirected = wr.area_fire || tc_spray(w, firer);
    let mut aim = match firer_target {
        Some(t) => ref_point(w, t),
        None if q.aim_origin_without_target => Pos::default(),
        None => shot_tp,
    };
    if redirected && !q.redirected_shot_aim {
        // spec: S-0306 Edge cases (redirected_shot_aim off: the aim point is the redirected cell)
        aim = shot_tp;
    }
    // spec: S-0306 claim 3 (a moving vehicle target is led, F6)
    if let Some(Target::Object(t)) = firer_target
        && led(w, t)
    {
        aim = lead(w, firer, t, aim);
    }
    // spec: S-0306 claim 4 (an arcing, inaccurate projectile scatters at launch)
    let mut impact = aim;
    let b = rules.combat.general.ballistic_scatter;
    if p.arcing && p.inaccurate {
        let disp = (aim.x - fire_point.x, aim.y - fire_point.y);
        let sigma = if !p.flak_scatter || p.inviso {
            // spec: S-0306 claim 4 (σ = R(trunc(B / 2), B))
            w.rng.ranged(b / 2, b, half)
        } else {
            // spec: S-0306 claim 4 (σ = trunc(R(0, B) × D / R_W), D the spatial distance in single precision)
            let rr = w.rng.ranged(0, b, half);
            let dd = d3(w, fire_point, aim);
            // spec: S-0306 claim 4 (R_W: the firer's weapon range of S-0307 claim 13 for that weapon, the slot that holds it)
            let slot = (0..crate::combat::data::SLOTS)
                .find(|&i| crate::combat::weapon::slot_weapon(w, firer, i) == Some(weapon))
                .unwrap_or(0);
            let rw = crate::combat::range::firer_weapon_range(w, firer, slot);
            // spec: S-0306 Edge cases (a flak range of 0 gives a radius of 0, a departure)
            if rw == 0 { 0 } else { rr.wrapping_mul(dd) / rw }
        };
        // spec: S-0306 F5 (direction draw R(0, 2147483646))
        let xi = w.rng.ranged(0, 2147483646, half);
        let (qx, qy) = scatter(w, disp, sigma, xi);
        impact = Pos::new(fire_point.x + qx, fire_point.y + qy, aim.z);
    }
    // spec: S-0306 claim 2 (capped at half the spatial distance to the impact point)
    let capped = s1.min(d3(w, fire_point, impact) / 2);
    let radial = tc.radial_segments;
    let kind = p.kind();
    // spec: S-0306 claim 2 (a steering or vertical projectile whose firer type has no radial segments starts at 1)
    let start =
        if matches!(kind, ProjectileKind::Steering | ProjectileKind::Vertical) && radial <= 0 {
            1
        } else {
            capped
        };
    // spec: S-0306 claim 5 (ρ = 0 and not dropping: the direction towards the impact point; else the aiming facing)
    let mut yaw = if p.rot == 0 && !p.dropping {
        i32::from(direction(
            i64::from(impact.x - fire_point.x),
            i64::from(impact.y - fire_point.y),
            q.table_direction,
        ))
    } else {
        i32::from(aiming(w, firer))
    };
    if radial > 0 {
        yaw = radial_yaw(w, firer, radial, o.combat.radial_index);
        if let Some(ob) = w.obj_mut(firer) {
            ob.combat.radial_index = (ob.combat.radial_index + 1) % radial;
        }
    }
    // spec: S-0306 claim 6 (the pitch: (a) arcing flag, (b) voxel image, (c) otherwise)
    let target_building = matches!(firer_target, Some(Target::Object(t)) if w.obj(t).is_some_and(|x| x.kind == Kind::Building));
    let pitch = if p.arcing {
        let dz = impact.z - fire_point.z;
        let dist = d2(w, fire_point, impact);
        let tgt_ref = firer_target.map(|t| ref_point(w, t)).unwrap_or(aim);
        let horizontal = d2(w, o.pos, tgt_ref);
        // spec: S-0306 claim 6 a (a lob: the lobber flag, or the target higher above than it is far)
        let lob = wr.lobber || (tgt_ref.z - o.pos.z) > horizontal;
        arc_pitch(w, start, dist, dz, lob, g)?
    } else if p.voxel {
        let tgt_z = firer_target.map_or(aim.z, |t| ref_point(w, t).z);
        // spec: S-0306 claim 6 b (32768 when the target is lower than the firer, else 16384)
        if tgt_z < o.pos.z { 32768 } else { 16384 }
    } else {
        let hb = match firer_target {
            Some(Target::Object(t)) => w
                .obj(t)
                .filter(|x| x.kind == Kind::Building)
                .map_or(2, |x| {
                    rules.buildings[x.type_index as usize].building.art_height
                }),
            _ => 2, // spec: S-0306 Import mapping (Height absent: 2)
        };
        straight_pitch(w, fire_point, impact, target_building, hb)
    };
    let mut vel = initial_velocity(w, yaw, pitch, start);
    // spec: S-0306 claim 5 (only a dropping projectile starts from the firer's own position)
    let start_pos = if p.dropping { o.pos } else { fire_point };
    let id = w.new_id();
    // spec: S-0306 claim 9 (the stored target point K; the arm time: 0 against an aircraft)
    let mut stored = shot_tp;
    let arm = if matches!(shot_target, Target::Object(t) if w.obj(t).is_some_and(|x| x.kind == Kind::Aircraft))
    {
        0
    } else {
        p.arm
    };
    // spec: S-0306 claim 8 (revision 45: placed at its start point, the fire point or claim 5's start position raised 1 lepton above the terrain when not above it; claims 5 to 7 above keep the unraised fire point)
    let mut pos = start_point(w, start_pos);
    if p.inviso {
        if p.flak_scatter {
            // spec: S-0306 claim 9 (an invisible flak projectile: K scattered by trunc(R(0, 2B) × D / R₀))
            let rr = w.rng.ranged(0, 2 * b, half);
            let dd = d3(w, fire_point, stored);
            let r0 = wr.range;
            let sigma = if r0 == 0 { 0 } else { rr.wrapping_mul(dd) / r0 };
            let xi = w.rng.ranged(0, 2147483646, half);
            let (qx, qy) = scatter(w, (stored.x, stored.y), sigma, xi);
            stored = Pos::new(qx, qy, stored.z);
        }
        // spec: S-0306 claim 9 (an invisible projectile: moved to K, or the first blocking cell's centre)
        let house = o.owner;
        pos = match crate::combat::lof::walk(w, &p, house, fire_point, stored) {
            Some(c) => {
                let cc = Pos::centre(c, 0);
                Pos::new(cc.x, cc.y, w.surface_at(cc.x, cc.y))
            }
            None => stored,
        };
        vel = [F64::ZERO; 3];
    } else if kind == ProjectileKind::Steering {
        // spec: S-0306 claim 9 (a steering projectile's velocity is scaled to length 1)
        vel = rescale(w, vel, f(1, mode));
    }
    let top_speed = wr.speed;
    let proj = Projectile {
        id,
        weapon,
        attacker: Some(firer),
        house: o.owner,
        firer: Some(firer),
        firer_house: o.owner,
        pos,
        vel,
        target: Some(shot_target),
        stored,
        placed_at: pos,
        arm,
        placed: w.tick,
        damage,
        // spec: S-0306 T1 (revision 45: Ω starts as the full spatial distance from the start point of claim 8, or an invisible projectile's moved position, to K)
        omega: d3(w, pos, stored),
        top_speed,
        turns: 0,
        unlocked: 0,
        locked: p.course_lock != 1,
        closing: F64::ZERO,
        prev_cell: pos.cell(),
        // spec: S-0306 claims 8 and 22 (revision 45: the line-of-fire walk's X₁ is the fire point as it is, not raised)
        fire_point,
        nuke_wait: None,
        vehicle_scalable: o.kind == Kind::Vehicle && p.scalable,
        airburst_child: false,
        multiplier: 256, // spec: S-0518 claim 9 (every projectile starts with the multiplier 256)
        chain: 0, // spec: S-0318 Terms (a projectile that is not a fragment has chain level 0)
    };
    w.projectiles.insert(id, proj);
    if o.kind == Kind::Vehicle
        && p.scalable
        && let Some(pid) = wr.projectile
    {
        // spec: S-0306 claim 16 (N: projectiles of scalable types fired by vehicles in play, the new one included)
        let n = w
            .projectiles
            .values()
            .filter(|x| x.vehicle_scalable)
            .count() as i32;
        // spec: S-0306 claim 16 (spawn delay + trunc(max(N − 5, 0) / 3); a value of 0 means the spawn delay field)
        let v = p.spawn_delay + (n - 5).max(0) / 3;
        w.scaled_spawn_delay
            .insert(pid, if v == 0 { p.spawn_delay } else { v });
    }
    // spec: S-0306 claim 8 (the projectile enters play; its first turn is in the same tick)
    w.update.enter(id);
    let _ = mode;
    Some(id)
}

/// Whether a firer's type has the spray-attack flag (S-0313 Terms).
fn tc_spray(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| u.raw.spray_attack.unwrap_or(false))
}

/// The aiming facing of a firer (S-0310).
fn aiming(w: &World, id: ObjectId) -> u16 {
    let t = w.tick;
    let Some(o) = w.obj(id) else { return 0 };
    let turret = match o.kind {
        Kind::Vehicle => w
            .rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.turret),
        Kind::Infantry => false,
        _ => true,
    };
    if turret {
        o.turret.value(t)
    } else {
        o.body.value(t)
    }
}

/// The yaw of radial fire segment `k` of `n` (S-0306 F1).
// spec: S-0306 F1 (π, the double nearest it)
fn radial_yaw(w: &World, id: ObjectId, n: i32, k: i32) -> i32 {
    let r = w.rules();
    let mode = r.mode;
    let (c1, c2) = direction_constants();
    let pi = c1.mul(f(2, mode), mode);
    let body = w.obj(id).map_or(0, |o| i32::from(o.body.value(w.tick)));
    // spec: S-0306 F1 (φₖ = RZ₅₃(RZ₅₃(RZ₅₃(π / n) × k) − π/2); y = trunc(RZ₅₃(RZ₅₃(RZ₅₃(θ(b) + φₖ) − π/2) × c₂)))
    let phi = pi
        .div(f(i64::from(n), mode), mode)
        .mul(f(i64::from(k), mode), mode)
        .sub(c1, mode);
    let v = theta(body, mode)
        .add(phi, mode)
        .sub(c1, mode)
        .mul(c2, mode)
        .trunc_i128()
        .unwrap_or(0);
    i32::from(v as i64 as u16)
}

/// The aim point of S-0306 claim 3 towards target `t` of `firer`: its
/// reference point without an aim shift, led by F6 when it is a vehicle its
/// locomotor is moving. Read-only: the target line's end point (S-0301
/// claim 24 e), recomputed in every drawn frame.
pub fn led_aim_point(w: &World, firer: ObjectId, t: Target) -> Pos {
    // spec: S-0306 claim 3 (the target's reference point, no aim shift)
    let aim = ref_point(w, t);
    match t {
        // spec: S-0306 claim 3 (a moving vehicle target is led, F6)
        Target::Object(x) if led(w, x) => lead(w, firer, x, aim),
        _ => aim,
    }
}

/// Whether a target is a vehicle that claim 3 of S-0306 leads: one its
/// locomotor is moving, by the moving test of the spec that owns that
/// locomotor (S-0503 Terms for a hovering vehicle: a destination or a
/// heading point).
fn led(w: &World, t: ObjectId) -> bool {
    if !w.obj(t).is_some_and(|o| o.kind == Kind::Vehicle) {
        // spec: S-0306 claim 3 (infantry and aircraft targets are not led)
        return false;
    }
    if crate::hover::is_hover(w, t) {
        // spec: S-0306 claim 3 b and Edge cases (quirks.switched_off_hover_lead: a switched-off hovering vehicle that S-0503 counts as moving is led; off: a switched-off vehicle is not led)
        if crate::hover::switched_off(w, t) && !w.rules().quirks.switched_off_hover_lead {
            return false;
        }
        // spec: S-0306 claim 3 b and S-0503 Terms (a hovering vehicle is moving while it has a destination or a heading point)
        return crate::hover::moving(w, t);
    }
    // spec: S-0306 claim 3 (S-0206 Terms: the locomotor moving the vehicle)
    crate::occupancy::in_progress(w, t)
}

/// The 32-step drawn facing ψ and drawn pitch facing *e* of a projectile's
/// velocity `v` after its latest turn (S-0306 F14): presentation only.
pub fn drawn_facings(
    w: &World,
    v: [F64; 3], // nocite: three components x, y, z
) -> (u8, u8) {
    let mode = w.rules().mode;
    // spec: S-0306 F14 (φ: S-0204 F3 with X = Vₓ and Y = −V_y in single precision; both 0 give 16383)
    let phi = glasnost_num::geom::direction_real(v[0], v[1].negated(), mode);
    // spec: S-0306 F14 (p: S-0204 F6's pitch with X = r(Vₓ² + V_y²) and Y = V_z, read modulo 65536)
    let p = pitch_of(w, v) as u16;
    // spec: S-0306 F14 and glossary (f₃₂(φ) = ⌊(⌊φ / 1024⌋ + 1) / 2⌋ mod 32)
    #[allow(clippy::manual_div_ceil)] // written as the glossary's formula
    let f32_of = |f: u16| (((u32::from(f) / 1024 + 1) / 2) % 32) as u8;
    (f32_of(phi), f32_of(p))
}

/// The lead of S-0306 claim 3, F6.
fn lead(w: &World, firer: ObjectId, target: ObjectId, aim: Pos) -> Pos {
    let r = w.rules();
    let mode = r.mode;
    // spec: S-0306 claim 3 (the lead weapon: slot 0, or the slot of the weapon number for a multi-weapon type)
    let multi = crate::combat::veterancy::type_combat(w, firer).is_some_and(|tc| tc.multi_weapon);
    let slot = if multi {
        crate::combat::weapon::weapon_number(w, firer)
    } else {
        0
    };
    let Some(lw) = crate::combat::weapon::slot_weapon(w, firer, slot) else {
        return aim;
    };
    let dd3 = d3(w, w.reference_point(firer), w.reference_point(target));
    let p = r.combat.projectile_of(lw);
    let ws = match p {
        Some(p) if p.rot == 0 => {
            ballistics::arc_speed(dd3, r.combat.general.gravity, p.floater, mode)
        }
        _ => r.combat.weapon(lw).speed,
    };
    // spec: S-0306 claim 3 (u, the target's speed estimate: S-0207 F2 at the fraction it holds now, read as leptons per tick); S-0207 claim 18 and F2a
    let u = crate::drive::speed_estimate(w, target);
    let b = w.obj(target).map_or(0, |o| i32::from(o.body.value(w.tick)));
    let point9 = crate::rules::nearest_double("0.9");
    // spec: S-0306 F6 (ℓ = trunc(RZ₅₃(RZ₅₃(D₃ / RZ₅₃(w × 0.9)) × u)))
    let l = f(i64::from(dd3), mode)
        .div(f(i64::from(ws), mode).mul(point9, mode), mode)
        .mul(f(i64::from(u), mode), mode)
        .to_i32_common();
    let th = theta(b, mode);
    let lf = f(i64::from(l), mode);
    // spec: S-0306 F6 (aim = (trunc(RZ₅₃(x + RZ₅₃(cos_t θ(b) × ℓ))), trunc(RZ₅₃(y − RZ₅₃(sin_t θ(b) × ℓ))), z))
    Pos::new(
        f(i64::from(aim.x), mode)
            .add(cos_t(th, mode).mul(lf, mode), mode)
            .to_i32_common(),
        f(i64::from(aim.y), mode)
            .sub(sin_t(th, mode).mul(lf, mode), mode)
            .to_i32_common(),
        aim.z,
    )
}

/// The steering turn step t of S-0306 claim 13, F7.
// spec: S-0306 F7 (2π, the double nearest it)
fn turn_step(w: &World, pr: &Projectile, rot: i32, near: bool) -> i32 {
    let r = w.rules();
    let mode = r.mode;
    let m = r.combat.general.missile_rot_var;
    // spec: S-0306 claim 13 (j = (object number + tick number) mod 15)
    let j = (i64::from(pr.id.0) + i64::from(w.tick)).rem_euclid(15);
    let k15 = crate::rules::nearest_double("0.06666666666666667");
    let (c1, _) = direction_constants();
    let two_pi = c1.mul(f(4, mode), mode);
    let angle = f(j, mode).mul(k15, mode).mul(two_pi, mode);
    // spec: S-0306 F7 (q = trunc(RZ₅₃(RZ₅₃(RZ₅₃(sin_t(...) × m) + (m + 1)) × ρ)))
    let qv = sin_t(angle, mode)
        .mul(m, mode)
        .add(m.add(f(1, mode), mode), mode)
        .mul(f(i64::from(rot), mode), mode);
    let qi = qv.to_i32_common();
    // spec: S-0306 F7 (q* = trunc(q × 1.5) near the target)
    let qs = if near {
        f(i64::from(qi), mode)
            .mul(crate::rules::nearest_double("1.5"), mode)
            .to_i32_common()
    } else {
        qi
    };
    if r.quirks.missile_turn_wrap {
        // spec: S-0306 F7 (t = 256 × (q* mod 256), a signed 16-bit value)
        i32::from((256 * qs.rem_euclid(256)) as i16)
    } else {
        // spec: S-0306 Edge cases (missile_turn_wrap off: t = 256 × min(q*, 127))
        256 * qs.min(127)
    }
}

/// Turns facing `cur` towards `goal` by step `t` (S-0306 F7).
fn turn_towards(cur: i32, goal: i32, t: i32) -> i32 {
    let d = delta(cur as u16, goal as u16);
    // spec: S-0306 F7 (|t| ≥ |δ|: the goal; else f + t for δ ≥ 0, f − t for δ < 0, 16-bit)
    if t.abs() >= d.abs() {
        goal
    } else if d >= 0 {
        i32::from((cur + t) as u16)
    } else {
        i32::from((cur - t) as u16)
    }
}

/// The pitch facing of a velocity p(V) (S-0306 F12).
// spec: S-0306 F12 (the vertical component V_z)
fn pitch_of(w: &World, v: [F64; 3]) -> i32 {
    let mode = w.rules().mode;
    let h = hlen(w, v);
    let (a, i) = table_angle_real(h, v[2], mode);
    // spec: S-0306 F12 (p(V) = trunc((A(V_z, H) − π/2) × c₂), a 16-bit value); S-0204 claim 12 and F6 (the exceptions of F3, by the sign of the angle)
    i32::from(glasnost_num::geom::pitch_of_angle(a, i, mode) as i16)
}

/// The steering turn of one turn (S-0306 claims 12 to 15, F10 to F12).
/// Returns the displacement velocity, the velocity after the turn and the
/// cruise lift of claim 15 (leptons, applied to the position after the
/// move).
// spec: S-0306 claims 14 and 15 (the vertical component; direct steps 0 and 256; cruise look-ahead 6, cell centres, height levels of 104; steps t / 2; climb pitch 8192)
fn steer(w: &World, pr: &mut Projectile, p: &ProjectileRules) -> ([F64; 3], [F64; 3], i32) {
    let r = w.rules();
    let mode = r.mode;
    let q = r.quirks;
    let tick = w.tick;
    // Course lock (claim 12).
    let kappa_l = p.course_lock;
    let speed_before = speed(w, pr.vel);
    let s_top = f(i64::from(pr.top_speed), mode);
    if pr.locked {
        let ends = if kappa_l >= 1 {
            // spec: S-0306 claim 12 (κ ≥ 1: the κ-th turn is the first unlocked one)
            pr.turns + 1 >= kappa_l
        } else if kappa_l == 0 {
            // spec: S-0306 claim 12 (κ = 0: the first turn with S ≥ 40 or S ≤ |V| + 0.5)
            pr.top_speed >= 40
                || s_top.le(speed_before.add(crate::rules::nearest_double("0.5"), mode))
        } else {
            false
        };
        if ends {
            pr.locked = false;
        }
    }
    // spec: S-0306 claim 12 (during the lock with κ = 0 the acceleration is 1 in even ticks, 0 in odd ones)
    let a = if pr.locked && kappa_l == 0 {
        i32::from(tick % 2 == 0)
    } else {
        p.acceleration
    };
    // spec: S-0306 F10 (|V| < S: rescaled to min(|V| + a, S); |V| > S: max(|V| − trunc(a / 2), 0))
    let mut v = if speed_before.lt(s_top) {
        let l = speed_before.add(f(i64::from(a), mode), mode).min(s_top);
        rescale(w, pr.vel, l)
    } else if speed_before.gt(s_top) {
        let l = speed_before
            .sub(f(i64::from(a / 2), mode), mode)
            .max(F64::ZERO);
        rescale(w, pr.vel, l)
    } else {
        pr.vel
    };
    let target_pt = pr.target.map(|t| aim_target_point(w, t));
    // spec: S-0306 Terms (a target whose target point is the world origin counts as no target, quirks.origin_as_missing_target_point)
    let has_target =
        target_pt.is_some_and(|tp| !(q.origin_as_missing_target_point && tp == Pos::default()));
    let tp = target_pt.unwrap_or_default();
    let near = {
        let reference = if has_target || q.origin_as_missing_target_point {
            tp
        } else {
            Pos::default()
        };
        // spec: S-0306 claim 13 (near: d₃ from the start-of-turn position to the target point below 256)
        (has_target || q.origin_as_missing_target_point) && d3(w, pr.pos, reference) < 256
    };
    let t = if pr.locked {
        0 // spec: S-0306 claim 13 (the step is 0 during course lock)
    } else {
        turn_step(w, pr, p.rot, near)
    };
    if has_target {
        // spec: S-0306 claim 14 (the displacement is trunc(V) after the speed change, before the heading change)
        let disp = v;
        let pprime = add_trunc(pr.pos, disp);
        // F11: turn the horizontal direction towards the target point from P′.
        let yc = i32::from(glasnost_num::geom::direction_real(
            v[0],
            v[1].negated(),
            mode,
        ));
        let wdx = f(i64::from(tp.x - pprime.x), mode);
        let wdy = f(i64::from(tp.y - pprime.y), mode);
        let yg = i32::from(glasnost_num::geom::direction_real(wdx, wdy.negated(), mode));
        let yn = turn_towards(yc, yg, t);
        let zero = F64::ZERO;
        // spec: S-0306 F11 (H = r(RZ₅₃(Vₓ² + V_y²)), r(10000) when both are 0)
        let h = if v[0].eq_num(zero) && v[1].eq_num(zero) {
            table_root(10000, mode).to_f64()
        } else {
            hlen(w, v)
        };
        let th = theta(yn, mode);
        v = [
            cos_t(th, mode).mul(h, mode),
            sin_t(th, mode).mul(h, mode).negated(),
            v[2],
        ];
        // F12: the pitch.
        let wv = [wdx, wdy.negated(), f(i64::from(tp.z - pprime.z), mode)];
        let dp = d2(w, pprime, tp);
        let aircraft = matches!(pr.target, Some(Target::Object(x)) if w.obj(x).is_some_and(|o| o.kind == Kind::Aircraft));
        // spec: S-0306 claim 15 (direct mode: an aircraft target, within 3 cells (6 very high) and not airburst, or t of 0 or 256)
        let lim = if p.very_high { 6 * 256 } else { 3 * 256 };
        let direct = aircraft || (dp <= lim && !p.airburst) || t == 0 || t == 256;
        let mut lift = 0;
        let (goal, step, level_keep) = if p.level {
            // spec: S-0306 claim 15 (a level projectile with a target keeps its pitch)
            (0, 0, true)
        } else if direct {
            // spec: S-0306 claim 15 (direct: goal the pitch of the vector to the target point, step trunc((t + 256) / 2))
            (pitch_of(w, wv), i32::from(((t + 256) / 2) as i16), false)
        } else {
            // Cruise mode (claim 15): the cruising altitude.
            // spec: S-0306 claim 15 (the terrain height of the cell at P′ + 6 × trunc(V))
            let look = Pos::new(
                pprime.x.wrapping_add(v[0].to_i32_common().wrapping_mul(6)),
                pprime.y.wrapping_add(v[1].to_i32_common().wrapping_mul(6)),
                pprime.z,
            );
            let lc = look.cell();
            let mut e = w.surface_at(256 * lc.x + 128, 256 * lc.y + 128);
            if w.high_bridge(lc) {
                e += 416; // spec: S-0306 Constants (high-bridge allowance 416)
            }
            // spec: S-0306 claim 15 (n = 10 airburst or very high, else min(trunc(D / 256), 5))
            let n = if p.airburst || p.very_high {
                10
            } else {
                (d3(w, pprime, tp) / 256).min(5)
            };
            let cruise = e + n * 104;
            let ee = pprime.z - cruise;
            // spec: S-0306 claim 15 (lifted 18 leptons when e < −20, lowered 18 when e > 20, a direct change of position)
            lift = if ee < -20 {
                18
            } else if ee > 20 {
                -18
            } else {
                0
            };
            // spec: S-0306 claim 15 (goal 8192 when e < −52, 18432 when e > 52, else 16384; step trunc(t / 2))
            let goal = if ee < -52 {
                8192
            } else if ee > 52 {
                18432
            } else {
                16384
            };
            (goal, t / 2, false)
        };
        let pc = pitch_of(w, v);
        let m = speed(w, v);
        let pn = if level_keep {
            pc
        } else {
            let d = i32::from((goal - pc) as i16);
            if step.abs() >= d.abs() {
                goal
            } else if d >= 0 {
                pc + step
            } else {
                pc - step
            }
        };
        v = rebuild(w, v, pc, pn, m);
        (disp, v, lift)
    } else {
        // spec: S-0306 claim 15 (no target: goal 8192 with step t; the displacement already includes the pitch change)
        let pc = pitch_of(w, v);
        let m = speed(w, v);
        let d = i32::from((8192 - pc) as i16);
        let pn = if t.abs() >= d.abs() {
            8192
        } else if d >= 0 {
            pc + t
        } else {
            pc - t
        };
        v = rebuild(w, v, pc, pn, m);
        (v, v, 0)
    }
}

/// The F12 rebuild of a velocity from its pitch pc to pn, keeping length m.
// spec: S-0306 F12 (three components)
fn rebuild(w: &World, v: [F64; 3], pc: i32, pn: i32, m: F64) -> [F64; 3] {
    let mode = w.rules().mode;
    let cpc = cos_t(theta(pc, mode), mode);
    // spec: S-0306 F12 (Ĥᵢ = RZ₅₃(Vᵢ / cos_t θ(p_c)); rebuilt = (cos_t θ(p′) Ĥₓ, cos_t θ(p′) Ĥ_y, sin_t θ(p′) M))
    let hx = v[0].div(cpc, mode);
    let hy = v[1].div(cpc, mode);
    let tn = theta(pn, mode);
    let cn = cos_t(tn, mode);
    [
        cn.mul(hx, mode),
        cn.mul(hy, mode),
        sin_t(tn, mode).mul(m, mode),
    ]
}

/// P + trunc(V) componentwise.
// spec: S-0306 claim 11 (P + trunc(V), three components)
fn add_trunc(p: Pos, v: [F64; 3]) -> Pos {
    Pos::new(
        p.x.wrapping_add(v[0].to_i32_common()),
        p.y.wrapping_add(v[1].to_i32_common()),
        p.z.wrapping_add(v[2].to_i32_common()),
    )
}

/// trunc(P + V) componentwise, in double precision (S-0306 claim 10).
// spec: S-0306 claim 10 (trunc(P + V), three components)
fn add_exact(w: &World, p: Pos, v: [F64; 3]) -> Pos {
    let mode = w.rules().mode;
    Pos::new(
        f(i64::from(p.x), mode).add(v[0], mode).to_i32_common(),
        f(i64::from(p.y), mode).add(v[1], mode).to_i32_common(),
        f(i64::from(p.z), mode).add(v[2], mode).to_i32_common(),
    )
}

/// The rows of T1 that apply in a turn (S-0306 State and ordering).
#[derive(Default)]
struct Rows {
    first: Option<i32>,
    reached: bool,
    proximity: Option<Pos>,
    speed: bool,
    vertical: bool,
    steering_dist: bool,
    steer_ground: bool,
    safety: bool,
    orbit: bool,
    bridge: Option<i32>,
    arrival: bool,
    passed: bool,
    lof: bool,
    deep: bool,
    flak: bool,
    level: bool,
    aa: bool,
    outside: bool,
}

/// One turn of a projectile (S-0306 claims 10 to 21, T1, T2).
// spec: S-0306 T1 (deck height 416; half the speed; closing average over 60 turns; arrival below 32, pass-by below 256)
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(mut pr) = w.projectiles.get(&id).cloned() else {
        return;
    };
    let rules = w.rules_arc();
    let mode = rules.mode;
    let q = rules.quirks;
    let wr = rules.combat.weapon(pr.weapon).clone();
    let Some(pid) = wr.projectile else {
        remove(w, id);
        return;
    };
    let p = rules.combat.projectiles[pid as usize].clone();
    let g = ballistics::gravity(rules.combat.general.gravity, p.floater, mode);
    let tick = w.tick;
    if let Some(until) = pr.nuke_wait {
        // spec: S-0306 claim 25 (it does nothing while the fireball exists; in its first turn after, it detonates)
        if tick < until {
            return;
        }
        let pos = pr.pos;
        explode(w, &mut pr, &p, pos, pos, true);
        remove(w, id);
        crate::parasite::carrier_ended(w, id, pr.weapon, true);
        return;
    }
    // spec: S-0306 claim 17 (a lost target object: its cell if it was not airborne, else nothing)
    // A superweapon warp member's departure does not take the member from a
    // projectile (S-0513 claim 39). From departure until it appears the
    // projectile keeps it, and every rule here reads the position it had at
    // departure, which `superweapons::warp` leaves in place (S-0306 claim 17,
    // revision 57, answering Q-2212). Any other target object off the map is
    // dropped here.
    if let Some(Target::Object(t)) = pr.target
        && w.obj(t)
            .is_none_or(|o| !o.on_map && !crate::superweapons::warp::departed(w, t))
    {
        pr.target = None;
    }
    if !q.stored_target_point
        && let Some(t) = pr.target
    {
        // spec: S-0306 Edge cases (stored_target_point off: K follows the target while it is in play)
        pr.stored = ref_point(w, t);
    }
    let start = pr.pos;
    let h = start.z - w.surface_at(start.x, start.y);
    let speed_start = speed(w, pr.vel);
    // spec: S-0306 claim 16 (a trailer at the start-of-turn position, every turn whose tick number is a multiple of the effective spawn delay)
    // spec: S-0306 claim 16 (the effective spawn delay: the type's raised value after a vehicle's shot, else its field)
    let delay = w
        .scaled_spawn_delay
        .get(&pid)
        .copied()
        .unwrap_or(p.spawn_delay);
    if let Some(trailer) = p.trailer.clone()
        && delay != 0
        && tick.rem_euclid(delay) == 0
    {
        // spec: S-0109 claim 29 (a projectile trailer, created during its turn before its movement, with an initial loop delay of 1 update)
        effects::trailer(w, &trailer, start);
    }
    let kind = p.kind();
    let mut lift = 0;
    let (unlifted, disp_v) = match kind {
        ProjectileKind::Straight | ProjectileKind::Arcing => {
            // spec: S-0306 claim 10 (V_z falls by g, then trunc(P + V))
            pr.vel[2] = pr.vel[2].sub(g, mode);
            (add_exact(w, start, pr.vel), pr.vel)
        }
        ProjectileKind::Vertical => {
            let sp = speed(w, pr.vel).to_i32_common();
            let sw = pr.top_speed;
            // spec: S-0306 F10 (vertical: trunc(|V|) < S_w: rescaled to trunc(|V|) + α; quirks.vertical_speed_uncapped)
            if sp < sw {
                let mut l = sp + p.acceleration;
                if !q.vertical_speed_uncapped {
                    l = l.min(sw);
                }
                pr.vel = rescale(w, pr.vel, f(i64::from(l), mode));
            }
            (add_trunc(start, pr.vel), pr.vel)
        }
        ProjectileKind::Steering => {
            let (disp, v, l) = steer(w, &mut pr, &p);
            let np = add_trunc(start, disp);
            pr.vel = v;
            lift = l;
            (np, v)
        }
    };
    let _ = disp_v;
    // spec: S-0306 claim 15 and T1 (a steering projectile's new position P′ includes the cruise lift in every row of T1, the closing step, the deck test and T2; only the steering distance is measured before the lift)
    let newpos = Pos::new(unlifted.x, unlifted.y, unlifted.z.wrapping_add(lift));
    pr.turns += 1;
    let hp = newpos.z - w.surface_at(newpos.x, newpos.y);
    let mut rows = Rows::default();
    let nc = newpos.cell();
    let terrain_new = w.surface_at(newpos.x, newpos.y);
    let deck_old = w
        .high_bridge(start.cell())
        .then(|| w.surface_at(start.x, start.y) + 416);
    let deck_new = w.high_bridge(nc).then(|| terrain_new + 416);
    let crossing = |a: i32, b: i32, deck: Option<i32>| -> Option<i32> {
        let d = deck?;
        // spec: S-0306 T1 (crossing a bridge deck height going up (deck height) or down (20 below it))
        if a < d && b >= d {
            Some(d)
        } else if a >= d && b < d {
            Some(d - 20)
        } else {
            None
        }
    };
    let cross =
        crossing(start.z, newpos.z, deck_old).or_else(|| crossing(start.z, newpos.z, deck_new));
    // spec: S-0306 T1 (vertical: a high-bridge cell, old or new; deck height = terrain height of the cell of P′ + 416)
    let deck_here = Some(terrain_new + 416);
    let vertical_cross = (w.high_bridge(start.cell()) || w.high_bridge(nc))
        && crossing(start.z, newpos.z, deck_here).is_some();
    let in_map = w.cells.cell(nc).is_some();
    if matches!(kind, ProjectileKind::Straight | ProjectileKind::Arcing) {
        let owned = has_firer(w, &pr);
        // spec: S-0306 T1 row 1 and Edge cases (quirks.owned_shell_ground_pass: a projectile that has a firer is detonated only by the deck crossings; off: the whole row for every projectile)
        let ground = !owned || !w.rules().quirks.owned_shell_ground_pass;
        // spec: S-0306 T1 row 1 (below the terrain, only with no firer: z unchanged here, raised to the surface by claim 20 a; a deck crossing: the deck height going up, 20 below it going down; 0 to 149 above the terrain over an eligible building or one of the three wall overlays, only with no firer: z lowered to the terrain height)
        if ground && newpos.z < terrain_new {
            rows.first = Some(newpos.z);
        } else if let Some(z) = cross {
            rows.first = Some(z);
        } else if ground
            && (0..150).contains(&(newpos.z - terrain_new))
            && building_or_wall(w, &pr, nc, owned)
        {
            rows.first = Some(terrain_new);
        }
        // spec: S-0306 T1 (reached: the cell of K, or the same building, with h < 208)
        if h < 208 {
            let kc = pr.stored.cell();
            if nc == kc || same_building(w, nc, kc) {
                rows.reached = true;
            }
        }
    }
    if kind != ProjectileKind::Steering {
        // spec: S-0306 T1 (proximity: the nearest unit or building in the cell of P′ within 128, not the firer or allied)
        rows.proximity = proximity(w, &pr, newpos);
        // spec: S-0306 T1 (the speed row: |V| < 10 and h < 10)
        if speed(w, pr.vel).lt(f(10, mode)) && h < 10 {
            rows.speed = true;
        }
        // spec: S-0306 T1 (outside the legal map: removed)
        rows.outside = !in_map;
    }
    if kind == ProjectileKind::Vertical {
        // spec: S-0306 T1 (vertical: above the detonation altitude, h < 0, or a deck crossing)
        rows.vertical = newpos.z > p.detonation_altitude || h < 0 || vertical_cross;
    }
    if kind == ProjectileKind::Steering {
        let target_pt = pr.target.map(|t| aim_target_point(w, t));
        let has_target =
            target_pt.is_some_and(|tp| !(q.origin_as_missing_target_point && tp == Pos::default()));
        let sd = if has_target {
            let tp = target_pt.unwrap_or_default();
            // spec: S-0306 claim 15 (the steering distance is measured at P′ before the cruise lift)
            let (dx, dy) = (i128::from(tp.x - unlifted.x), i128::from(tp.y - unlifted.y));
            // spec: S-0306 Terms (steering distance: ζ = trunc(Δz / 4), 0 for airburst)
            let zeta = if p.airburst {
                0
            } else {
                i128::from((tp.z - unlifted.z) / 4)
            };
            root_n(w, dx * dx + dy * dy + zeta * zeta)
        } else {
            speed(w, pr.vel).to_i32_common()
        };
        let half_speed = speed(w, pr.vel).div(f(2, mode), mode);
        // spec: S-0306 T1 (steering distance row: half the speed ≥ the steering distance)
        rows.steering_dist = half_speed.ge(f(i64::from(sd), mode));
        rows.steer_ground = h <= 0;
        rows.safety = !has_target && h >= rules.combat.general.missile_safety_altitude;
        if !pr.locked {
            pr.unlocked += 1;
            let reference = target_pt.unwrap_or_default();
            // spec: S-0306 T1 (closing step c_k = d₃(P, target point) − d₃(P′, target point))
            let ck = d3(w, start, reference) - d3(w, newpos, reference);
            let ckd = f(i64::from(ck), mode);
            pr.closing = if pr.unlocked <= 60 {
                pr.closing.add(ckd, mode)
            } else {
                // spec: S-0306 T1 (C_k = RZ₅₃(RZ₅₃(C_{k−1} × f₆₀) + c_k), f₆₀ = 8857079267161975 / 2⁵³)
                let f60 = F64::from_parts(false, 8857079267161975, -53, Rounding::TowardZero);
                pr.closing.mul(f60, mode).add(ckd, mode)
            };
            if pr.unlocked > 60
                && !p.airburst
                && !p.very_high
                && (has_target || q.origin_as_missing_target_point)
                && pr.closing.ge(F64::ZERO)
                && pr.closing.lt(f(60, mode))
            {
                rows.orbit = true;
            }
        }
        if !(rows.steering_dist || rows.steer_ground || rows.safety || rows.orbit) {
            // spec: S-0306 T1 (steering: P′ strictly crosses the deck height of a high-bridge cell, old or new; z at the deck height)
            let strict = |deck: Option<i32>| {
                deck.filter(|&d| (start.z < d && newpos.z > d) || (start.z > d && newpos.z < d))
            };
            rows.bridge = strict(deck_old).or_else(|| strict(deck_new));
        }
    }
    if (kind == ProjectileKind::Steering || p.ranged) && !p.dropping {
        let past_arm = tick.wrapping_sub(pr.placed) >= pr.arm;
        if past_arm {
            // spec: S-0306 T1 (η = trunc(d₃(P′, K) / 2); arrival η < 32; passed by η < 256 and η > Ω)
            let eta = d3(w, newpos, pr.stored) / 2;
            if eta < 32 {
                rows.arrival = true;
            } else if eta < 256 && eta > pr.omega {
                rows.passed = true;
            } else {
                pr.omega = eta;
            }
        }
    }
    if p.cliffs || p.walls {
        // spec: S-0306 claim 22 (the line-of-fire test in flight for the single sample P′)
        let house = pr.firer_house;
        let x1 = pr.fire_point.cell();
        let x2 = pr.stored.cell();
        rows.lof = crate::combat::lof::sample_blocks(w, &p, house, nc, pr.prev_cell, x1, x2);
    }
    // spec: S-0306 T1 (deep fall: h′ ≤ −416)
    rows.deep = hp <= -416;
    if p.flak_scatter
        && let Some(t) = pr.target
    {
        // spec: S-0306 T1 (flak: P′ below the target's reference height and h′ < 0)
        rows.flak = newpos.z < ref_point(w, t).z && hp < 0;
    }
    if p.level {
        // spec: S-0306 T1 (level: the cell of P′ is not one of the 14 water tiles)
        let water = w.catalogue.as_deref().map_or(i32::MIN, |c| c.roles.water);
        let tile = w.cells.cell(nc).map_or(-1, |c| c.tile);
        rows.level = !(water >= 0 && (water..water + 14).contains(&tile)); // spec: S-0306 Constants (14 water tiles)
    }
    if p.aa
        && let Some(Target::Object(t)) = pr.target
        && target_airborne(w, t)
    {
        // spec: S-0306 T1 (anti-air with an airborne target: d₃ to its reference point below 128)
        rows.aa = d3(w, newpos, w.reference_point(t)) < 128;
    }
    pr.prev_cell = nc;
    // T2: the outcome.
    let outcome: Option<Option<Pos>> = if kind == ProjectileKind::Steering {
        let target_pt = pr.target.map(|t| aim_target_point(w, t));
        let has_target =
            target_pt.is_some_and(|tp| !(q.origin_as_missing_target_point && tp == Pos::default()));
        let any = rows.steering_dist
            || rows.steer_ground
            || rows.safety
            || rows.orbit
            || rows.bridge.is_some()
            || rows.arrival
            || rows.passed
            || rows.lof
            || rows.deep
            || rows.flak
            || rows.level
            || rows.aa;
        if rows.steering_dist && h > 0 && !p.airburst && has_target {
            // spec: S-0306 T2 (steering distance, h > 0, not airburst, a target: at the target point)
            Some(target_pt)
        } else if let Some(z) = rows.bridge {
            Some(Some(Pos::new(newpos.x, newpos.y, z)))
        } else if any {
            Some(Some(newpos))
        } else {
            None
        }
    } else if rows.reached {
        // spec: S-0306 T2 (a reached row: at P′ with the first row's vertical change)
        Some(Some(Pos::new(
            newpos.x,
            newpos.y,
            rows.first.unwrap_or(newpos.z),
        )))
    } else if let Some(op) = rows.proximity {
        // spec: S-0306 T2 (the proximity row: at the other object's position; inaccurate: at P′)
        if p.inaccurate {
            Some(Some(Pos::new(
                newpos.x,
                newpos.y,
                rows.first.unwrap_or(newpos.z),
            )))
        } else {
            Some(Some(op))
        }
    } else if rows.outside {
        // spec: S-0306 T2 (outside the legal map: removal without detonation)
        Some(None)
    } else if rows.first.is_some()
        || rows.speed
        || rows.vertical
        || rows.lof
        || rows.deep
        || rows.flak
        || rows.level
        || rows.aa
        || rows.arrival
        || rows.passed
    {
        Some(Some(Pos::new(
            newpos.x,
            newpos.y,
            rows.first.unwrap_or(newpos.z),
        )))
    } else {
        None
    };
    match outcome {
        None => {
            // spec: S-0306 claim 15 (the position kept when no row applies is the lifted one)
            pr.pos = newpos;
            // spec: S-0306 claim 18 (a degenerating projectile loses 1 per turn without a detonation, not below 5)
            if p.degenerates && pr.damage > 5 {
                pr.damage -= 1;
            }
            w.projectiles.insert(id, pr);
        }
        Some(None) => {
            remove(w, id);
            crate::parasite::carrier_ended(w, id, pr.weapon, false);
        }
        Some(Some(point)) => {
            pr.pos = point;
            // spec: S-0306 T1 and claim 20 b (the passed-by row is not a reached row; it counts as arrival when the firer's type is a jumpjet type)
            let jumpjet = pr
                .firer
                .and_then(|x| w.obj(x))
                .and_then(|o| w.rules().unit(o.kind, o.type_index))
                .is_some_and(|u| u.jumpjet);
            let arrival = rows.arrival || (rows.passed && jumpjet);
            let reached = rows.reached;
            let (burst, pt) = detonation_point(w, &pr, &p, point, arrival, reached, speed_start);
            // spec: S-0514 claim 20 (nuke-making is the last row: a higher selected row replaces it)
            if special_of(w, pr.weapon, pr.target) == Some(Special::NukeMake) {
                // spec: S-0513 claim 8 (a missile-maker warhead does no damage at its burst: it is replaced by the falling missile, claim 9)
                pr.pos = pt;
                crate::superweapons::nuke::burst(w, &pr);
                remove(w, id);
                crate::parasite::carrier_ended(w, id, pr.weapon, false);
                return;
            }
            if rules.combat.warhead_of(pr.weapon).is_some_and(|h| h.nuke) {
                // spec: S-0104 claim 5 (a nuclear missile detonating starts the nuke flash's brightening phase with a lighting re-evaluation)
                crate::lighting::nuke_detonated(w);
                // spec: S-0306 claim 25 (the nuclear warhead waits for its fireball animation)
                if rules.animation("NUKEBALL").is_some() {
                    effects::anim(w, "NUKEBALL", pt);
                    w.note("S-0306 claim 25: the fireball's length needs animation playback (S-0109), which is not implemented; the nuclear detonation follows at once");
                }
            }
            pr.pos = pt;
            explode(w, &mut pr, &p, pt, burst, false);
            remove(w, id);
            // spec: S-0614 claim 1 (when the projectile detonates, the parasite tries to latch on)
            crate::parasite::carrier_ended(w, id, pr.weapon, true);
        }
    }
}

/// Whether the cell holds a building that stops a straight or arcing
/// projectile (T1 row 1), or one of the three wall overlays.
// spec: S-0306 T1 and S-0307 claim 11 (the wall overlays 2, 26 and 243)
fn building_or_wall(w: &World, pr: &Projectile, c: CellXY, owned: bool) -> bool {
    let rules = w.rules();
    let buildings: Vec<ObjectId> = w
        .cells
        .list(c, crate::world::Layer::Ground)
        .iter()
        .copied()
        .filter(|&b| w.obj(b).is_some_and(|o| o.kind == Kind::Building))
        .collect();
    if buildings.is_empty() {
        // spec: S-0306 T1 row 1 and Constants (with no building present, one of the hard-coded wall overlays 2, 26, 243)
        return crate::combat::lof::wall_overlay(w, c).is_some_and(|o| [2u8, 26, 243].contains(&o));
    }
    buildings.iter().any(|&b| {
        let Some(o) = w.obj(b) else { return false };
        // spec: S-0306 T1 row 1 and S-0211 Terms (not an inactive laser-fence section; section states are not modelled, so every section counts as inactive, as for S-0317 claim 19)
        let fence = rules
            .buildings
            .get(o.type_index as usize)
            .is_some_and(|t| t.building.laser_fence);
        // spec: S-0306 T1 row 1 and S-0214 Terms (not a vehicle-like building)
        let vehicle_like = crate::targeting::vehicle_like(w, b);
        // spec: S-0306 T1 row 1 (any house's, an open gate included, scenario P32 f) and Edge cases (quirks.owned_shell_ground_pass off: a projectile that has a firer is not stopped by the firer itself or by a building allied with the firer's house)
        let own = owned && (Some(b) == pr.firer || w.allied(o.owner, pr.firer_house));
        !fence && !vehicle_like && !own
    })
}

/// Whether a projectile has a firer (S-0306 Terms): it was made with one,
/// and its detonation would still carry an attacker by S-0302 claim 1, so
/// its firer was neither killed (death, delayed kill: `death::mark_killed`
/// and `damage` clear the attacker) nor removed from the game since the
/// shot. A firer loaded into a closed transport stays in the game, so it is
/// still the firer. A firer that started to cloak is still counted as one:
/// S-0302 claim 1's cloaking case is not built (a seam, low).
pub fn has_firer(w: &World, pr: &Projectile) -> bool {
    // spec: S-0306 Terms (a projectile's firer; no firer: made without one, or its firer killed or removed from the game while it flies)
    pr.attacker.is_some() && pr.firer.is_some_and(|f| w.obj(f).is_some())
}

/// Whether two cells hold the same building.
fn same_building(w: &World, a: CellXY, b: CellXY) -> bool {
    let bl = |c: CellXY| {
        w.cells
            .list(c, crate::world::Layer::Ground)
            .iter()
            .copied()
            .find(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Building))
    };
    matches!((bl(a), bl(b)), (Some(x), Some(y)) if x == y)
}

/// The proximity row of T1: the nearest unit or building in the cell of P′.
// spec: S-0306 Constants (proximity to a unit in the cell: 128 leptons)
fn proximity(w: &World, pr: &Projectile, p: Pos) -> Option<Pos> {
    let c = p.cell();
    let mut best: Option<(i32, ObjectId)> = None;
    for &x in w.cells.list(c, crate::world::Layer::Ground) {
        let Some(o) = w.obj(x) else { continue };
        if o.kind == Kind::Terrain {
            continue;
        }
        let d = d3(w, p, o.pos);
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, x));
        }
    }
    let (d, x) = best?;
    let o = w.obj(x)?;
    // spec: S-0306 T1 (at d₃ < 128, neither the firer nor allied with the firer's house; with no firer any object counts)
    let excluded = has_firer(w, pr) && (Some(x) == pr.firer || w.allied(o.owner, pr.firer_house));
    (d < 128 && !excluded).then_some(o.pos)
}

/// The burst point of S-0318 Terms (the point that S-0306 claim 20 (a)
/// and (b) leave) and the detonation point of S-0306 claim 20 (a) to (c).
fn detonation_point(
    w: &World,
    pr: &Projectile,
    p: &ProjectileRules,
    point: Pos,
    arrival: bool,
    reached: bool,
    speed_start: F64,
) -> (Pos, Pos) {
    let mode = w.rules().mode;
    let mut pt = point;
    // spec: S-0306 claim 20 (a) (a negative height above the surface is raised to 0)
    let surf = w.surface_at(pt.x, pt.y);
    if pt.z < surf {
        pt.z = surf;
    }
    let Some(t) = pr.target else { return (pt, pt) };
    if p.inaccurate || p.airburst {
        return (pt, pt);
    }
    let rp = ref_point(w, t);
    // spec: S-0306 claim 20 (b) (arrival always; a reached row when ν′ ≤ max(2|V|, 128))
    let snap = if arrival {
        true
    } else if reached {
        let zeta = (rp.z + pt.z) / 2 - rp.z; // spec: S-0306 F9 (ζ = trunc((P_R.z + P.z) / 2) − P_R.z)
        let (dx, dy) = (i128::from(pt.x - rp.x), i128::from(pt.y - rp.y));
        let nu = root_n(w, dx * dx + dy * dy + i128::from(zeta) * i128::from(zeta));
        let nup = nu / 3; // spec: S-0306 F9 (ν′ = trunc(ν / 3))
        let thr = speed_start.mul(f(2, mode), mode).max(f(128, mode));
        f(i64::from(nup), mode).le(thr)
    } else {
        false
    };
    if snap {
        pt = rp;
    }
    // spec: S-0318 Terms (the burst point is the point after S-0306 claim 20 (a) and (b); (c) moves only the detonation point)
    let burst = pt;
    // spec: S-0306 claim 20 (c) (D_R and D_S from the point (a) and (b) leave)
    let dr = d3(w, pt, rp);
    let ds = match t {
        Target::Object(x) => w
            .obj(x)
            .filter(|o| o.kind == Kind::Building)
            .map_or(dr, |o| {
                (dr - 64 * (o.foundation.0 + o.foundation.1)).max(0) // spec: S-0306 Constants (building reduction 64 per cell)
            }),
        Target::Cell(_) => dr,
    };
    let em = w.rules().combat.warhead_of(pr.weapon).is_some_and(|h| h.em);
    // A departed warp member is airborne by claim 17's count.
    let airborne = |x| {
        if crate::superweapons::warp::departed(w, x) {
            // spec: S-0306 claim 17 (claim 20's question whether the target is airborne, for a departed warp member)
            target_airborne(w, x)
        } else {
            crate::combat::weapon::in_air(w, x)
        }
    };
    // spec: S-0306 r65 claim 17 (a warp member is in the ground render layer from its warp's launch until its own movement applies again after its arrival hold, whatever its height, and an aircraft member whose type cannot land while it flies, until a crash; so claim 20 (c) uses 42 for it, also when it departed from a deck and counts as airborne)
    let ground =
        |x| crate::temporal::ground_layer(w, x) || crate::superweapons::warp::in_ground_layer(w, x);
    // spec: S-0306 claim 20 (c) (threshold 42 for a target that is not airborne or is airborne in the ground render layer; 128 for a target airborne in another render layer)
    let threshold = match t {
        Target::Object(x) if airborne(x) && !ground(x) => 128,
        _ => 42,
    };
    // spec: S-0306 claim 20 (c) (below the threshold and no electromagnetic flag: the target point; else D_R below 32: the reference point)
    let det = if !em && ds < threshold {
        aim_target_point(w, t)
    } else if dr < 32 {
        rp
    } else {
        pt
    };
    (burst, det)
}

/// The detonations of a projectile (S-0306 claim 21) and their projectile
/// effects (S-0303 claims 21 and 22).
/// `burst` is the burst point of S-0318 Terms, from which every detonation
/// launches its fragments.
fn explode(
    w: &mut World,
    pr: &mut Projectile,
    p: &ProjectileRules,
    point: Pos,
    burst: Pos,
    _after_wait: bool,
) {
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    let wr = rules.combat.weapon(pr.weapon).clone();
    // spec: S-0303 claim 2, S-0306 claim 21 and S-0614 claim 1 (a parasite-flag warhead's projectile makes no area hits; its animation and debris stay)
    // spec: S-0303 claim 2 and S-0519 claim 16 (an air-strike-flag warhead's projectile makes no area hits, whether or not it calls or redirects a strike; a projectile-producer rule only)
    // spec: S-0303 claim 2 and S-0518 claim 11 (a detonation selecting the electric-charge effect bypasses area damage at any target, even when it registers no charger)
    // spec: S-0514 claim 20 (one selected special hit; a selected row replaces ordinary damage and suppresses every later row)
    let special = special_of(w, pr.weapon, pr.target);
    // spec: S-0516 claim 31 (a disguise-flag warhead's projectile does no damage to the target)
    let area = special.is_none();
    if special == Some(Special::Disguise) {
        // spec: S-0516 r13 claim 31 (detonating on its target, it disguises its firer; a departed warp member, off the map or phasing in, is an ordinary target and is copied as above) and S-0306 claim 17 (a departed warp member, off the map or phasing in, is copied as that claim gives)
        crate::cloak::disguise::disguise_hit(w, pr.attacker, pr.target);
    }
    if p.airburst {
        // spec: S-0525 claim 1 (the airburst projectile's own detonation leaves radiation with its own weapon; its nine children carry no weapon and leave none)
        // spec: S-0318 claim 5 and S-0525 claim 1 (revision 4: a fragment carries no weapon and leaves no radiation, so a fragment weapon's radiation level has no effect)
        if !pr.airburst_child && pr.chain == 0 {
            crate::radiation::detonation(w, pr.weapon, point.cell());
        }
        // spec: S-0318 claim 1 (an airburst projectile's one detonation bursts before its area damage and its nine children)
        crate::combat::shrapnel::burst(w, pr, p, burst, special);
        let res = detonation_inner(
            w,
            pr.weapon,
            crate::linked::scaled_damage(pr.damage, pr.multiplier), // spec: S-0518 claim 9 and F1 (the damage supplied to area damage is scaled by the projectile's multiplier)
            point,
            pr.attacker,
            pr.house,
            pr.pos,
            p.inviso,
            area,
            pr.target,
            None,
        );
        let _ = res;
        if let Some(aw) = p.airburst_weapon {
            airburst_children(w, pr, aw);
        }
        return;
    }
    // spec: S-0306 Edge cases (a cluster count of 0 or less: no detonation, no draw)
    let mut q = point;
    for _ in 0..p.cluster.max(0) {
        // spec: S-0525 claim 1 (each detonation of a projectile with a cluster count above 1 leaves radiation at its own point; a child of an airburst leaves none)
        // spec: S-0318 claim 5 and S-0525 claim 1 (revision 4: a fragment carries no weapon and leaves no radiation, so a fragment weapon's radiation level has no effect)
        if !pr.airburst_child && pr.chain == 0 {
            crate::radiation::detonation(w, pr.weapon, q.cell());
        }
        // spec: S-0318 claim 1 and State and ordering (each detonation of a cluster tests and bursts afresh from the same burst point, after the radiation and before the area damage)
        crate::combat::shrapnel::burst(w, pr, p, burst, special);
        detonation_inner(
            w,
            pr.weapon,
            crate::linked::scaled_damage(pr.damage, pr.multiplier), // spec: S-0518 claim 9 and F1 (the damage supplied to area damage is scaled by the projectile's multiplier)
            q,
            pr.attacker,
            pr.house,
            pr.pos,
            p.inviso,
            area,
            pr.target,
            None,
        );
        // spec: S-0306 claim 21 (after every detonation: R(256, 512), then one raw value for the nearby point)
        let lambda = w.rng.ranged(256, 512, half);
        let b = w.rng.raw();
        q = crate::combat::death::nearby_point(w, q, lambda, b, false);
    }
    let _ = wr;
}

/// The nine children of an airburst (S-0306 claim 21).
fn airburst_children(w: &mut World, pr: &Projectile, aw: WeaponId) {
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    let c = pr.pos.cell();
    let sw = rules.combat.weapon(aw).speed;
    let mode = rules.mode;
    // spec: S-0306 claim 21 (the eight neighbouring cells and its own cell, falling at a tenth of S_w)
    for (dx, dy) in [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
        (0, 0),
    ] {
        // spec: S-0306 claim 21 (each child takes one draw R(0, 32))
        let _ = w.rng.ranged(0, 32, half);
        let cell = CellXY::new(c.x + dx, c.y + dy);
        let cc = Pos::centre(cell, pr.pos.z);
        // spec: S-0306 claim 8 (revision 45: a projectile a burst releases is raised to 1 lepton above the terrain of its own x and y when not above it)
        let at = start_point(w, cc);
        let id = w.new_id();
        let child = Projectile {
            id,
            weapon: aw,
            attacker: pr.attacker,
            house: pr.house,
            firer: pr.firer,
            firer_house: pr.firer_house,
            pos: at,
            vel: [F64::ZERO, F64::ZERO, f(-i64::from(sw / 10), mode)],
            target: Some(Target::Cell(cell)),
            stored: Pos::new(cc.x, cc.y, w.surface_at(cc.x, cc.y)),
            placed_at: at,
            arm: 0,
            placed: w.tick,
            damage: rules.combat.weapon(aw).damage,
            omega: 0,
            top_speed: sw,
            turns: 0,
            unlocked: 0,
            locked: false,
            closing: F64::ZERO,
            prev_cell: cell,
            // spec: S-0306 claims 8 and 22 (revision 45: the line-of-fire walk's X₁ is not raised)
            fire_point: cc,
            nuke_wait: None,
            vehicle_scalable: false,
            airburst_child: true,
            multiplier: 256, // spec: S-0518 claim 9 (every projectile starts with the multiplier 256)
            chain: pr.chain, // spec: S-0318 Terms (only a fragment is a level below the projectile that launched it)
        };
        w.projectiles.insert(id, child);
        w.update.enter(id);
    }
}

/// One projectile detonation (S-0303 State and ordering, *Order of a
/// projectile's detonation*): area damage, then the invisible kind's
/// nearby point, the explosion animation (F5) and the debris (F6).
#[allow(clippy::too_many_arguments)]
// spec: S-0303 claim 21 (N(P, 32, u mod 256), the invisible-projectile animation scatter)
pub fn projectile_detonation(
    w: &mut World,
    weapon: WeaponId,
    damage: i32,
    point: Pos,
    attacker: Option<ObjectId>,
    house: Option<HouseId>,
    q: Pos,
    invisible: bool,
) -> DetonationResult {
    detonation_inner(
        w, weapon, damage, point, attacker, house, q, invisible, true, None, None,
    )
}

/// [`projectile_detonation`], without the area damage when `area` is false
/// (S-0303 claim 2: a parasite-flag, air-strike-flag or electric-charge
/// warhead's projectile).
#[allow(clippy::too_many_arguments)]
// spec: S-0303 claim 21 (N(P, 32, u mod 256), the invisible-projectile animation scatter)
fn detonation_inner(
    w: &mut World,
    weapon: WeaponId,
    damage: i32,
    point: Pos,
    attacker: Option<ObjectId>,
    house: Option<HouseId>,
    q: Pos,
    invisible: bool,
    area: bool,
    target: Option<Target>,
    warhead_as: Option<WarheadId>,
) -> DetonationResult {
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    // spec: S-0303 claim 14 and S-0302 claim 1 (a firer removed from the game by any means since the shot: neither attacker nor attacking house is carried; a loaded firer, which stays carried, is a seam: transports)
    let (attacker, house) = if attacker.is_some_and(|a| w.obj(a).is_none()) {
        (None, None)
    } else {
        (attacker, house)
    };
    // spec: S-0607 claim 18 (the impact detonations use the death weapon's projectile with the crushing or demolition warhead)
    let warhead = warhead_as.or(rules.combat.weapon(weapon).warhead);
    let Some(h) = warhead else {
        // spec: S-0303 Edge cases (a projectile without a warhead does nothing, a departure)
        return DetonationResult::Missed;
    };
    // spec: S-0514 claim 20 (one selected special hit: the first enabled row of the table wins, all later rows and ordinary damage are suppressed even when the selected effect cannot succeed; locomotor replacement is S-0607's seize)
    let special = if warhead_as.is_some() {
        None
    } else {
        special_of(w, weapon, target)
    };
    // spec: S-0306 claim 17 (these rows act on the projectile's target object, not through area damage, so a departed warp member, off the map or phasing in, receives them by each row's own tests; electric charging acts on buildings only; no locomotor seizing from departure until the arrival hold ends, `lift::seize_hit`)
    match special {
        // spec: S-0601 claim 5 (takeover replaces damage and every other special hit; only the projectile's target can change owner)
        Some(Special::Takeover) => crate::mind::detonation(w, attacker, target),
        // spec: S-0518 claim 11 (an electric-charge detonation registers an infantry firer on the building it hit, whatever the building)
        Some(Special::Charge) => crate::linked::charge_hit(w, attacker, target),
        // spec: S-0519 claims 2 and 7 (a bomb-planting warhead plants on its target instead of area damage; a disarm warhead removes the target's bomb; the distance is not tested)
        Some(Special::Bomb) => crate::bomb::plant(w, attacker, target),
        // spec: S-0519 claim 16 (an air-strike warhead calls or redirects the strike instead of area damage; a shot at a cell, a destroyed firer or a non-designator calls none)
        Some(Special::Airstrike) => crate::strike::designation(w, attacker, target),
        Some(Special::Disarm) => crate::bomb::disarm(w, target),
        // spec: S-0514 claims 1 and 21 (the erasing effect replaces ordinary damage; a firer with no beam makes no lock and no side effect)
        Some(Special::Erase) => crate::temporal::deliver(w, attacker, target),
        // spec: S-0607 claim 1 (a seizing warhead's detonation seizes in place of ordinary damage)
        Some(Special::Locomotor) => crate::lift::seize_hit(w, attacker, target, damage, h),
        _ => {}
    }
    // spec: S-0514 claim 20 (with no selected row ordinary damage applies; a selected row, built or not, replaces it)
    let area = area && special.is_none();
    let res = if area {
        area::detonate(
            w,
            Detonation {
                point,
                damage,
                warhead: Some(h),
                source: attacker,
                house,
                resources: true,
            },
        )
    } else {
        // spec: S-0303 claim 2 (no area hits for a parasite-flag, air-strike-flag or electric-charge warhead's projectile)
        DetonationResult::Missed
    };
    let wh = rules.combat.warhead(h).clone();
    let mut anim_at = point;
    if invisible {
        // spec: S-0303 claim 21 (an invisible projectile: the nearby point N(P, 32, u mod 256), one raw value after the area damage)
        let u = w.rng.raw();
        anim_at = crate::combat::death::nearby_point(w, point, 32, u & 255, false);
    }
    if let Some(name) = explosion_anim(w, h, damage, q, res == DetonationResult::Hit) {
        if res == DetonationResult::Nullified {
            // spec: S-0303 claim 21 (a nullified detonation shows the nullify animation instead)
            if let Some(n) = rules.combat.general.nullify_anim.clone() {
                effects::anim(w, &n, anim_at);
            }
        } else {
            effects::anim(w, &name, anim_at);
        }
    }
    // spec: S-0303 claim 22 (warhead debris whatever the damage amount, not when nullified)
    if res != DetonationResult::Nullified && wh.max_debris > 0 {
        // spec: S-0303 F6 (N = R(Y, X − 1))
        let n = w.rng.ranged(wh.min_debris, wh.max_debris - 1, half);
        if wh.debris_types.is_empty() {
            let list = rules.combat.general.metallic_debris.clone();
            let c = list.len() as i32;
            for _ in 0..n.max(0) {
                let k = w.rng.ranged(0, c - 1, half);
                if let Some(name) = list.get(k as usize).cloned() {
                    // spec: S-0303 F6 (metallic debris at Q raised by 20 leptons)
                    effects::anim(w, &name, Pos::new(q.x, q.y, q.z + 20));
                }
            }
        } else {
            // spec: S-0317 claim 1 (a detonation's pieces are taken to have no house)
            crate::combat::death::deal_debris(w, n, &wh.debris_types, &wh.debris_maximums, q, None);
        }
    }
    res
}

/// The special effect a projectile detonation selects (S-0514 claim 20), in
/// the order of the claim's table: the first enabled row wins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Special {
    /// Row 1, takeover (S-0601 claim 5).
    Takeover,
    /// Row 2, bomb planting (S-0519 claim 2).
    Bomb,
    /// Row 3, electric charging (S-0518 claim 11).
    Charge,
    /// Row 4, parasite attachment (S-0614 claims 1 to 4).
    Parasite,
    /// Row 5, temporal erasure (S-0514 claims 1 to 4 and 21).
    Erase,
    /// Row 6, locomotor replacement (S-0607 claim 1).
    Locomotor,
    /// Row 7, air-strike designation (S-0519 claim 16).
    Airstrike,
    /// Row 8, direct rocking of a vehicle target (selection only here).
    DirectRock,
    /// Row 9, bomb disarming (S-0519 claim 7).
    Disarm,
    /// Row 10, disguise-making (S-0516 claim 31).
    Disguise,
    /// Row 11, nuke-making (S-0513 claim 8).
    NukeMake,
}

/// The special effect `weapon`'s warhead selects against `target`; `None`
/// when no row is enabled, so ordinary damage applies.
pub fn special_of(w: &World, weapon: WeaponId, target: Option<Target>) -> Option<Special> {
    let h = w.rules().combat.warhead_of(weapon)?;
    // spec: S-0514 claim 20 (the first enabled row wins; every later row is suppressed even when the selected effect cannot succeed, has no suitable target or has lost its firer)
    let vehicle = || {
        // spec: S-0514 claim 20 (direct rocking counts only when the projectile target is a vehicle; its flag with another target does not suppress the later rows)
        matches!(target, Some(Target::Object(x)) if w.obj(x).is_some_and(|o| o.kind == Kind::Vehicle))
    };
    if h.mind_control {
        Some(Special::Takeover)
    } else if h.ivan_bomb {
        Some(Special::Bomb)
    } else if h.electric_assault {
        Some(Special::Charge)
    } else if h.parasite {
        Some(Special::Parasite)
    } else if h.temporal {
        Some(Special::Erase)
    } else if h.is_locomotor {
        Some(Special::Locomotor)
    } else if h.airstrike {
        Some(Special::Airstrike)
    } else if h.direct_rocker && vehicle() {
        Some(Special::DirectRock)
    } else if h.bomb_disarm {
        Some(Special::Disarm)
    } else if h.makes_disguise {
        Some(Special::Disguise)
    } else if h.nuke_maker {
        Some(Special::NukeMake)
    } else {
        None
    }
}

/// The explosion animation of S-0303 F5 (with the electromagnetic case's
/// draw when it applies).
pub(crate) fn explosion_anim(
    w: &mut World,
    h: WarheadId,
    d: i32,
    q: Pos,
    hit: bool,
) -> Option<String> {
    explosion_anim_with(w, h, d, q, hit, false, false)
}

/// [`explosion_anim`] chosen as if the land type were not water, so the
/// splash case is never selected (S-0520 claim 16).
pub(crate) fn explosion_anim_dry(w: &mut World, h: WarheadId, d: i32, q: Pos) -> Option<String> {
    explosion_anim_with(w, h, d, q, false, true, false)
}

/// `any_height`: the land type is that of Q's cell whatever its height
/// (a rocket detonation, S-0506 claim 19), with no "air" case.
fn explosion_anim_with(
    w: &mut World,
    h: WarheadId,
    d: i32,
    q: Pos,
    hit: bool,
    dry: bool,
    any_height: bool,
) -> Option<String> {
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    let g = &rules.combat.general;
    let wh = rules.combat.warhead(h);
    // spec: S-0303 F5 (D = 0: no animation and no draw)
    if d == 0 {
        return None;
    }
    let qc = q.cell();
    let terrain = w.surface_at(q.x, q.y);
    // spec: S-0303 claim 21 (the land type of Q's cell when its height above the terrain is below 208, else air)
    // spec: S-0506 claim 19 and F9 (a rocket detonation: the land type of the cell containing the nose point, whatever its height; the cell is the nose's x and y divided by 256, truncated toward zero, S-0204 claim 2)
    let mut land =
        (any_height || q.z - terrain < 208).then(|| w.cells.cell(qc).map_or(0, |c| c.land));
    // spec: S-0303 claim 21 (the area damage hit something and the first object in the ground layer of Q's cell is a naval vehicle, not underwater: L counts as air)
    let naval_first = w
        .cells
        .list(qc, crate::world::Layer::Ground)
        .first()
        .is_some_and(|&x| {
            w.obj(x)
                .is_some_and(|o| o.kind == crate::rules::Kind::Vehicle)
                && crate::combat::veterancy::type_combat(w, x)
                    .is_some_and(|tc| tc.naval && !tc.underwater)
        });
    if hit && naval_first {
        land = None;
    }
    let water = !dry && land == Some(glasnost_map::terrain::land::WATER);
    // spec: S-0303 F5 (splash: water, conventional, no bridge, q_z < terrain + 208)
    if water && wh.conventional && !w.high_bridge(qc) && q.z < terrain + 208 {
        let ns = g.splash_list.len() as i32;
        if ns == 0 {
            return None;
        }
        // spec: S-0303 F5 (entry trunc(min(D, 35 nₛ − 1) / 35))
        let e = d.min(35 * ns - 1) / 35;
        return if e < 0 {
            None
        } else {
            g.splash_list.get(e as usize).cloned()
        };
    }
    // spec: S-0303 F5 (the lightning warhead: the bolt explosion)
    if g.lightning_warhead == Some(h) {
        return g.bolt_explosion.clone();
    }
    let nw = wh.anim_list.len() as i32;
    if wh.em {
        // spec: S-0303 F5 (electromagnetic: entry R(0, n_W − 1), one draw when n_W > 0)
        if nw == 0 {
            return None;
        }
        let k = w.rng.ranged(0, nw - 1, half);
        return wh.anim_list.get(k as usize).cloned();
    }
    if nw == 0 {
        return None;
    }
    // spec: S-0303 F5 (graded: entry trunc(min(D, 25 n_W − 1) / 25)); Edge cases (a negative entry shows nothing)
    let e = d.min(25 * nw - 1) / 25;
    if e < 0 {
        None
    } else {
        wh.anim_list.get(e as usize).cloned()
    }
}

/// A death weapon's detonation (S-0312 claim 6): a projectile detonation
/// of the weapon at the object's position; its projectile is never placed,
/// so the land-type, height and water tests and the debris use the map
/// origin (`quirks.death_weapon_unplaced_projectile`).
pub fn detonate_weapon(
    w: &mut World,
    weapon: WeaponId,
    damage: i32,
    at: Pos,
    source: Option<ObjectId>,
    house: Option<HouseId>,
    _death: bool,
) {
    let unplaced = w.rules().quirks.death_weapon_unplaced_projectile;
    // spec: S-0312 claim 6 (the unplaced projectile's default position, believed the origin; off: the object's position)
    let q = if unplaced { Pos::default() } else { at };
    let inviso = w
        .rules()
        .combat
        .projectile_of(weapon)
        .is_some_and(|p| p.inviso);
    // spec: S-0525 claim 1 (a death weapon's detonation leaves radiation at the dying object's position)
    crate::radiation::detonation(w, weapon, at.cell());
    projectile_detonation(w, weapon, damage, at, source, house, q, inviso);
}

/// A death weapon's projectile detonation with another warhead (S-0607
/// claim 18): as [`detonate_weapon`], the projectile never placed.
pub fn detonate_weapon_as(
    w: &mut World,
    weapon: WeaponId,
    warhead: WarheadId,
    damage: i32,
    at: Pos,
    source: Option<ObjectId>,
    house: Option<HouseId>,
) {
    let unplaced = w.rules().quirks.death_weapon_unplaced_projectile;
    // spec: S-0312 claim 6 (the unplaced projectile's default position, believed the origin; off: the object's position)
    let q = if unplaced { Pos::default() } else { at };
    let inviso = w
        .rules()
        .combat
        .projectile_of(weapon)
        .is_some_and(|p| p.inviso);
    detonation_inner(
        w,
        weapon,
        damage,
        at,
        source,
        house,
        q,
        inviso,
        true,
        None,
        Some(warhead),
    );
}

/// A plain area detonation of a warhead (S-0303), with the explosion
/// animation of F5 chosen for its amount (S-0302 claim 20).
pub fn detonate_area(
    w: &mut World,
    warhead: WarheadId,
    damage: i32,
    at: Pos,
    source: Option<ObjectId>,
    house: Option<HouseId>,
    resources: bool,
) {
    if let Some(name) = explosion_anim(w, warhead, damage, at, false) {
        effects::anim(w, &name, at);
    }
    area::detonate(
        w,
        Detonation {
            point: at,
            damage,
            warhead: Some(warhead),
            source,
            house,
            resources,
        },
    );
}

/// The detonation of a rocket missile at its nose point `at` (S-0506 claim
/// 19, F9): the explosion animation chosen by S-0303 F5 with the land type
/// of the cell containing the nose point whatever the missile's height, at
/// the nose point exactly, with the depth adjustment −15 in place of its
/// type's; then the area damage at the same point.
pub fn detonate_rocket(
    w: &mut World,
    warhead: WarheadId,
    damage: i32,
    at: Pos,
    source: Option<ObjectId>,
) {
    if let Some(name) = explosion_anim_with(w, warhead, damage, at, false, false, true) {
        // spec: S-0506 claim 19 and Constants (detonation animation depth adjustment −15 depth units, replacing the type's; S-0102 claim 13)
        effects::anim_adjusted(w, &name, at, -15);
    }
    // spec: S-0506 claim 19 (source object: the missile; source house: none; resource reduction allowed; no height correction of the point)
    area::detonate(
        w,
        Detonation {
            point: at,
            damage,
            warhead: Some(warhead),
            source,
            house: None,
            resources: true,
        },
    );
}

/// The target object `id` leaves the map, by going inside or by being
/// removed (S-0306 claim 17): every projectile aimed at it now targets the
/// cell it stood in when it was not airborne, else nothing. Call it while
/// `id` is still on the map; an object already off the map changes nothing,
/// except a superweapon warp member between its departure and its
/// appearance, which projectiles keep (claim 17): its cell is the one it
/// stood in at departure.
pub(crate) fn target_leaves(w: &mut World, id: ObjectId) {
    let Some(pos) = w
        .obj(id)
        .filter(|o| o.on_map || crate::superweapons::warp::departed(w, id))
        .map(|o| o.pos)
    else {
        return;
    };
    // spec: S-0306 claim 17 (a target object that leaves the game or becomes untargetable: the cell the target stood in if it was not airborne, else nothing)
    let airborne = target_airborne(w, id);
    for p in w.projectiles.values_mut() {
        if p.target == Some(Target::Object(id)) {
            p.target = (!airborne).then_some(Target::Cell(pos.cell()));
        }
    }
}

/// Whether a projectile's target object counts as airborne where this
/// spec's flight and detonation ask (S-0306 T1, T2, claim 20 and claim
/// 17's loss). For a superweapon warp member between its departure and its
/// appearance (S-0513 claim 39), off the map then, S-0307's rule that an
/// object not on the map is neither airborne nor floor-bound does not
/// apply: it is counted by its height above the surface beneath the
/// position it kept, and as standing on a deck only while a landing check
/// that passed for a landing cell on a deck awaits its placement (S-0306
/// claim 17). After a refused placement it counts by its height again until
/// a later check passes for a deck cell (claim 17's design rule, Q-2232).
/// Every other object is S-0307's airborne. S-0307 r21 Terms states the
/// same exception for a member between its departure and its appearance.
pub fn target_airborne(w: &World, id: ObjectId) -> bool {
    use crate::superweapons::warp;
    let Some(o) = w.obj(id) else { return false };
    if !warp::departed(w, id) {
        return airborne_by_height(w, id);
    }
    // spec: S-0307 r21 Terms (a warp member between its departure and its appearance is counted by its height above the surface beneath its kept position, except while a passed landing check for a deck cell awaits its placement) and S-0306 claim 17 (a departed member counts as standing on a deck only while a landing check that passed for a landing cell on a deck awaits its placement; each passed check sets the count from its own landing cell; after a refused placement, engine design with no switch, it counts by its height again, Q-2232)
    if warp::passed_for_deck(w, id) {
        return false;
    }
    let surface = if w.rules().quirks.warp_deck_departure_airborne {
        // spec: S-0306 claim 17 and Edge cases (quirks.warp_deck_departure_airborne: counted by its height above the surface beneath its kept position; one that departed from a high-bridge deck no longer stands on the deck)
        w.surface_at(o.pos.x, o.pos.y)
    } else {
        // spec: S-0306 Edge cases (warp_deck_departure_airborne off: until it appears it counts as standing on the deck it departed from)
        w.surface_for(id)
    };
    // spec: S-0307 Terms (airborne: height above the surface at least 208 leptons)
    o.pos.z - surface >= 208
}

/// A projectile leaves play (S-0203 claim 5: a removal for claim 10).
pub fn remove(w: &mut World, id: ObjectId) {
    w.projectiles.remove(&id);
    let skip = w.rules().quirks.update_order_removal_skip;
    w.update.leave(id, skip);
}

/// The kind of a projectile type, for the drawing.
pub fn kind_of(w: &World, pr: &Projectile) -> Option<ProjectileKind> {
    w.rules().combat.projectile_of(pr.weapon).map(|p| p.kind())
}

/// Events of the drawing: a shot left a firer.
pub fn shot_effect(w: &mut World, firer: ObjectId, weapon: WeaponId, from: Pos) {
    // spec: S-0701 claims 21 and 32 (the report entry follows the firer's retained report value r)
    let report = w.obj(firer).map_or(0, |o| o.report_value);
    // spec: S-0609 claim 13 (a gattling-flagged firer never plays the ordinary per-shot report)
    let gattling = crate::gattling::flagged(w, firer);
    w.effects.push(Effect::Shot {
        firer,
        weapon,
        from,
        report,
        gattling,
    });
}
