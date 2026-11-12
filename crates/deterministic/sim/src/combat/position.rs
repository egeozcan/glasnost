//! Firing positions (S-0313): where a shot starts, burst alternation and
//! the pixel-shift conversion; the spray attack's shifts.

use glasnost_num::geom::n_step;
use glasnost_num::trig::{cos_tb, sin_tb};
use glasnost_num::{F64, Rounding};

use crate::combat::veterancy::{self, Rank};
use crate::rules::Kind;
use crate::world::{ObjectId, Pos, World};
use glasnost_rules::sim::Flh;

/// A single-precision constant from the decimal that identifies it.
fn single_const(text: &str) -> F64 {
    crate::rules::nearest_double(text)
        .to_f32(Rounding::Nearest)
        .to_f64()
}

/// A pixel shift in leptons (S-0313 F3): map x and map y.
pub fn pixel_to_leptons(w: &World, p: i32, q: i32) -> (i32, i32) {
    let mode = w.rules().mode;
    // spec: S-0313 F3 (α = 4.26669979095459 and β = 8.53339958190918, single-precision constants)
    let alpha = single_const("4.26669979095459");
    let beta = single_const("8.53339958190918");
    let pf = F64::from_i64(i64::from(p), mode).to_f32(mode).to_f64();
    let qf = F64::from_i64(i64::from(q), mode).to_f32(mode).to_f64();
    let bq = beta.mul(qf, mode);
    // spec: S-0313 F3 (Δx = trunc(s₂₄(s₅₃(s₅₃(β q) + s₅₃(α p)))), Δy with −α p)
    let dx = bq
        .add(alpha.mul(pf, mode), mode)
        .to_f32(mode)
        .to_f64()
        .to_i32_common();
    let dy = bq
        .add(alpha.negated().mul(pf, mode), mode)
        .to_f32(mode)
        .to_f64()
        .to_i32_common();
    (dx, dy)
}

/// A 3 × 4 matrix of single-precision values (held widened), composed by
/// multiplying on the right (S-0105 F2), each entry stored in single
/// precision (S-0105 F0).
#[derive(Clone, Copy)]
// spec: S-0313 F1 (3 × 4 single-precision matrices)
struct M34([[F64; 4]; 3]);

impl M34 {
    fn identity(mode: Rounding) -> Self {
        let o = F64::from_i64(1, mode);
        let z = F64::ZERO;
        M34([[o, z, z, z], [z, o, z, z], [z, z, o, z]])
    }

    /// Rz(θ) with table trigonometry (S-0105 F1, F2), the angle narrowed to
    /// single precision first (S-0313 F2).
    /// `bias` is `quirks.table_trig_negative_bias` (S-0105 F1).
    fn rz(theta: F64, mode: Rounding, bias: bool) -> Self {
        let a = theta.to_f32(mode).to_f64();
        let (c, s) = (cos_tb(a, mode, bias), sin_tb(a, mode, bias));
        let o = F64::from_i64(1, mode);
        let z = F64::ZERO;
        M34([[c, s.negated(), z, z], [s, c, z, z], [z, z, o, z]])
    }

    // spec: S-0313 F1 (the translation column of a 3 × 4 matrix)
    fn translation(x: i32, y: i32, z: i32, mode: Rounding) -> Self {
        let mut m = Self::identity(mode);
        // spec: S-0313 F1 (the translations are converted from integers to single precision first)
        let f = |v: i32| F64::from_i64(i64::from(v), mode).to_f32(mode).to_f64();
        m.0[0][3] = f(x);
        m.0[1][3] = f(y);
        m.0[2][3] = f(z);
        m
    }

    /// self · b, each product and sum rounded as S-0205 sets and stored in
    /// single precision.
    // spec: S-0313 F1 and S-0105 F2 (3 × 4 matrices multiplied on the right)
    fn mul(&self, b: &M34, mode: Rounding) -> M34 {
        let mut out = [[F64::ZERO; 4]; 3];
        for (i, row) in out.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                let mut acc = F64::ZERO;
                for k in 0..3 {
                    acc = acc.add(self.0[i][k].mul(b.0[k][j], mode), mode);
                }
                if j == 3 {
                    acc = acc.add(self.0[i][3], mode);
                }
                *cell = acc.to_f32(mode).to_f64();
            }
        }
        M34(out)
    }
}

/// The double nearest −π/16 (S-0313 F2).
fn c16() -> F64 {
    // spec: S-0313 F2 (c = −0.19634954084936207, the double nearest −π/16)
    crate::rules::nearest_double("-0.19634954084936207")
}

/// The fire point of a shot with the weapon in `slot`, made with burst
/// counter `k` (S-0313 claims 1 to 8).
// spec: S-0313 claim 1 and F1 (18 slots; the base point at the top corner, 256 leptons per cell; the translation column of a 3 × 4 matrix)
pub fn fire_point(w: &World, firer: ObjectId, slot: usize, k: i32) -> Pos {
    let Some(o) = w.obj(firer) else {
        return Pos::default();
    };
    let r = w.rules();
    let Some(tc) = veterancy::type_combat(w, firer) else {
        return o.pos;
    };
    let elite = veterancy::rank(w, firer) == Rank::Elite;
    let flh = if elite {
        tc.elite_flh[slot.min(17)]
    } else {
        tc.flh[slot.min(17)]
    };
    let base = if o.kind == Kind::Building {
        // spec: S-0313 Terms (a building's base point: the top corner of its foundation at its world height)
        let c = o.pos.cell();
        Pos::new(256 * c.x, 256 * c.y, o.pos.z)
    } else {
        // spec: S-0313 Terms (any other firer: its position)
        o.pos
    };
    let b = r
        .buildings
        .get(o.type_index as usize)
        .filter(|_| o.kind == Kind::Building);
    if o.kind == Kind::Building && crate::structures::garrison::garrisoned(w, firer) {
        // spec: S-0313 claim 6 (a garrisoned building fires from its base point shifted by its muzzle point k, F3, map x and y only; the height is the base point's)
        let k = crate::structures::garrison::current_index(w, firer).unwrap_or(0);
        let (dx, dy) = crate::structures::garrison::muzzle_shift(w, firer, k);
        return Pos::new(base.x + dx, base.y + dy, base.z);
    }
    if let Some(bt) = b
        && let Some([px, py]) = bt.building.fire_pixel
        && !bt.building.dual_offset
    {
        // spec: S-0313 claim 7 (a primary fire pixel shift without the dual-shift flag: the base point shifted, at its height)
        let (dx, dy) = pixel_to_leptons(w, px, py);
        return Pos::new(base.x + dx, base.y + dy, base.z);
    }
    let mut p = claim1_point(w, firer, base, flh, k);
    if let Some(bt) = b {
        if let Some([px, py]) = bt.building.fire_pixel {
            // spec: S-0313 claim 7 (with the dual-shift flag: the claim-1 fire point shifted)
            let (dx, dy) = pixel_to_leptons(w, px, py);
            p = Pos::new(p.x + dx, p.y + dy, p.z);
        } else if bt.building.voxel_turret && !bt.building.voxel_barrel {
            // spec: S-0313 claim 8 (a voxel turret and no voxel barrel: shifted by the turret anchor's pixel shift, x and y only)
            let [px, py] = bt.building.turret_anim;
            let (dx, dy) = pixel_to_leptons(w, px, py);
            p = Pos::new(p.x + dx, p.y + dy, p.z);
        }
        // A voxel barrel's own rule is an open question of S-0313 (claim 8): the claim-1 point is used.
    }
    p
}

/// The point S-0601 claim 17 anchors a link line at (F5): S-0313 claim 1's
/// firing-position transform of `firer` with the triple `flh` in place of
/// the weapon's muzzle triple and burst counter `k`.
pub fn transform_point(w: &World, firer: ObjectId, flh: Flh, k: i32) -> Pos {
    let Some(o) = w.obj(firer) else {
        return Pos::default();
    };
    let base = if o.kind == Kind::Building {
        // spec: S-0313 Terms (a building's base point: the top corner of its foundation at its world height)
        let c = o.pos.cell();
        Pos::new(256 * c.x, 256 * c.y, o.pos.z) // spec: S-0313 Terms (256 leptons per cell)
    } else {
        // spec: S-0313 Terms (any other firer: its position)
        o.pos
    };
    // spec: S-0601 F5 (anchor coordinates use the firing-position transform of S-0313 claim 1 with the imported anchor replacing the muzzle triple)
    claim1_point(w, firer, base, flh, k)
}

/// The turret pivot of `firer` (S-0313 Terms): the point a muzzle position
/// of (0, 0, 0) would fire from by claim 1, from its current position, body
/// transform and turret facing. Read-only (the target line of S-0301 claim
/// 24 d starts there).
pub fn turret_pivot(w: &World, firer: ObjectId) -> Pos {
    // spec: S-0313 Terms (turret pivot: claim 1 with the muzzle position (0, 0, 0)) and S-0301 claim 24 d
    transform_point(w, firer, [0, 0, 0], 0)
}

/// S-0313 claim 1's transform of the triple `flh` from base point `base`
/// (claims 1 to 3, F1 and F2).
fn claim1_point(w: &World, firer: ObjectId, base: Pos, flh: Flh, k: i32) -> Pos {
    let Some(o) = w.obj(firer) else {
        return base;
    };
    let r = w.rules();
    let mode = r.mode;
    let t = w.tick;
    let Some(tc) = veterancy::type_combat(w, firer) else {
        return base;
    };
    let b = r
        .buildings
        .get(o.type_index as usize)
        .filter(|_| o.kind == Kind::Building);
    // The angle θ (claim 3, F2).
    let f32_of = |f: u16| n_step(f, 32) as i64; // spec: S-0313 claim 3 (32-step facings)
    let body32 = f32_of(o.body.value(t));
    let turreted = match o.kind {
        Kind::Vehicle => r.unit(o.kind, o.type_index).is_some_and(|u| u.turret),
        Kind::Aircraft => true,
        Kind::Building => b.is_some_and(|b| b.turret),
        _ => false,
    };
    let aim = match o.kind {
        Kind::Building if !turreted => {
            // spec: S-0313 claim 3 (another building with a target: the direction of its target)
            match o.target {
                Some(tg) => {
                    let tp = crate::combat::range::target_point(w, tg);
                    let rp = w.reference_point(firer);
                    glasnost_num::geom::direction(
                        i64::from(tp.x - rp.x),
                        i64::from(tp.y - rp.y),
                        r.quirks.table_direction,
                    )
                }
                None => o.turret.value(t),
            }
        }
        _ if turreted => o.turret.value(t),
        _ => o.body.value(t),
    };
    let a32 = f32_of(aim);
    let c = c16();
    let bias = r.quirks.table_trig_negative_bias; // spec: S-0105 F1 (quirks.table_trig_negative_bias)
    let at = |n: i64| F64::from_i64(n - 8, mode).mul(c, mode); // spec: S-0313 F2 ((x₃₂ − 8) × c)
    let theta = match o.kind {
        // spec: S-0313 F2 (a building: s₅₃((a₃₂ − 8) × c))
        Kind::Building => at(a32),
        // spec: S-0313 claim 3 (a vehicle without the turret flag and infantry: 0)
        _ if !turreted => F64::ZERO,
        // spec: S-0313 F2 (a firer with a locomotor: s₅₃(s₅₃((a₃₂ − 8) c) − s₅₃((b₃₂ − 8) c)))
        _ => at(a32).sub(at(body32), mode),
    };
    // spec: S-0313 claim 1 (M: the body transform for a firer with a locomotor, the identity for any other)
    let m = if o.kind == Kind::Building {
        M34::identity(mode)
    } else {
        // spec: S-0105 claim 3 (θ_body = −(π/16)(f₃₂ − 8)); slope tilt (S-0106) is not applied
        M34::rz(at(body32), mode, bias)
    };
    // spec: S-0313 claim 2 (the lateral part negated for an odd burst counter)
    let sigma = if k.rem_euclid(2) == 1 { -1 } else { 1 };
    let delta = tc.turret_offset;
    let tr_flh = M34::translation(flh[0], sigma * flh[1], flh[2], mode); // spec: S-0313 claim 2 (odd burst counter: lateral part negated)
    let tm = if o.kind == Kind::Building && !r.quirks.building_fire_point_shift_fixed {
        // spec: S-0313 Edge cases (building_fire_point_shift_fixed off: T = Rz(θ) · Tr(δ) · Tr(F, σL, H))
        M34::rz(theta, mode, bias)
            .mul(&M34::translation(delta, 0, 0, mode), mode)
            .mul(&tr_flh, mode)
    } else {
        // spec: S-0313 F1 (T = M · Tr(δ, 0, 0) · Rz(θ) · Tr(F, σL, H))
        m.mul(&M34::translation(delta, 0, 0, mode), mode)
            .mul(&M34::rz(theta, mode, bias), mode)
            .mul(&tr_flh, mode)
    };
    let (u, v, wz) = (tm.0[0][3], tm.0[1][3], tm.0[2][3]); // spec: S-0313 F1 (the translation column of the 3 × 4 matrix)
    // spec: S-0313 F1 (shift = (trunc u, trunc(−v), trunc w))
    Pos::new(
        base.x + u.to_i32_common(),
        base.y + v.negated().to_i32_common(),
        base.z + wz.to_i32_common(),
    )
}

/// The eight spray shifts (S-0313 F4).
// spec: S-0313 F4 (spray shifts in leptons)
pub const SPRAY: [(i32, i32); 8] = [
    (256, 0),
    (180, 180),
    (0, 256),
    (-180, 180),
    (-256, 0),
    (-180, -180),
    (0, -256),
    (180, -180),
];
