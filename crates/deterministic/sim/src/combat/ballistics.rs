//! Ballistic arithmetic shared by the range test (S-0307) and projectiles
//! (S-0306): the arc speed, the reachability test and the gravity.

use glasnost_num::geom::table_root_real;
use glasnost_num::{F64, Rounding};

/// The double nearest 1.2, the arc speed margin (S-0306 F2).
fn margin() -> F64 {
    // spec: S-0306 Constants (arc speed margin 1.2, the double nearest it)
    crate::consts::ARC_SPEED_MARGIN
}

/// The gravity of a projectile (S-0306 Terms): g, or g / 2 for a floater.
pub fn gravity(g: i32, floater: bool, mode: Rounding) -> F64 {
    let gd = F64::from_i64(i64::from(g), mode);
    if floater {
        // spec: S-0306 Terms (g_f = g / 2, double precision)
        gd.div(F64::from_i64(2, mode), mode)
    } else {
        gd
    }
}

/// The arc speed v(L, g) of S-0306 F2: trunc(r(RZ₅₃(RZ₅₃(L × g) × 1.2))).
pub fn arc_speed(l: i32, g: i32, floater: bool, mode: Rounding) -> i32 {
    arc_speed_g(l, gravity(g, floater, mode), mode)
}

/// The arc speed with the gravity given as a double.
pub fn arc_speed_g(l: i32, g: F64, mode: Rounding) -> i32 {
    // spec: S-0306 F2 (v = trunc(r(RZ₅₃(RZ₅₃(L × g) × 1.2))))
    let lg = F64::from_i64(i64::from(l), mode).mul(g, mode);
    let x = lg.mul(margin(), mode);
    table_root_real(x, true, mode).to_i32_common()
}

/// The reachability test of S-0306 F3 (used by the range test, S-0307
/// claim 9): whether a projectile launched at speed `v` under gravity `g`
/// reaches a point `d` away horizontally and `z` higher.
pub fn reachable(v: i32, d: i32, z: i32, g: F64, table: bool, mode: Rounding) -> bool {
    let f = |n: i64| F64::from_i64(n, mode);
    let vd = f(i64::from(v));
    // spec: S-0306 F3 (d = 0 counts as 0.001)
    let dd = if d == 0 {
        crate::consts::THOUSANDTH
    } else {
        f(i64::from(d))
    };
    let zd = f(i64::from(z));
    let v2 = vd.mul(vd, mode);
    let v4 = v2.mul(v2, mode);
    // spec: S-0306 F3 (disc = v⁴ − 2gzv² − g²d²)
    let two = f(2);
    let disc = v4
        .sub(two.mul(g, mode).mul(zd, mode).mul(v2, mode), mode)
        .sub(g.mul(g, mode).mul(dd, mode).mul(dd, mode), mode);
    // spec: S-0306 F3 (ω = 2(1 + z²/d²))
    let omega = two.mul(
        f(1).add(zd.mul(zd, mode).div(dd.mul(dd, mode), mode), mode),
        mode,
    );
    if !disc.ge(F64::ZERO) {
        return false;
    }
    // spec: S-0306 F3 (reachable ⇔ disc ≥ 0 and (v² − gz + r(disc)) / ω ≥ 0)
    let num = v2
        .sub(g.mul(zd, mode), mode)
        .add(table_root_real(disc, table, mode), mode);
    num.div(omega, mode).ge(F64::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RZ: Rounding = Rounding::TowardZero;

    // spec: S-0306 F2 (v(L, g) = trunc(r(RZ₅₃(RZ₅₃(L × g) × 1.2))))
    #[test]
    fn arc_speed_of_flat_shots() {
        // 1280 × 3 × 1.2 = 4608 (just below under truncation), root about 67.88.
        assert_eq!(arc_speed(1280, 3, false, RZ), 67);
        // Floater: g/2 = 1.5, 1280 × 1.5 × 1.2 = 2304 (just below), root 47.99… → 47.
        assert_eq!(arc_speed(1280, 3, true, RZ), 47);
        assert_eq!(arc_speed(0, 3, false, RZ), 0);
    }

    // spec: S-0306 F3 (reachability: only targets far above fail)
    #[test]
    fn reachability_fails_only_high_above() {
        let g = gravity(3, false, RZ);
        let v = arc_speed(1280, 3, false, RZ);
        assert!(reachable(v, 1200, 0, g, true, RZ));
        assert!(reachable(v, 0, 0, g, true, RZ));
        assert!(!reachable(v, 1200, 4000, g, true, RZ));
    }
}
