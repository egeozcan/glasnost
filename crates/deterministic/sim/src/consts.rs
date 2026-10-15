//! Engine constants that the specs give as decimals, held as their exact
//! binary64 bit patterns so that hot paths need no decimal parsing. The
//! tests check every pattern against its decimal: through S-0003's reading
//! for values the specs say are read as a decimal field, through the
//! nearest double for values the specs give as "the double nearest".

use glasnost_num::F64;

/// ε(d) of S-0212 table T4, by step direction d.
// spec: S-0212 table T4 (the single-precision values of 0.001, 0.005, 0.002, 0.006, 0.003, 0.007, 0.004, 0.008)
pub const EPSILON: [F64; 8] = [
    F64(0x3f50_624d_e000_0000),
    F64(0x3f74_7ae1_4000_0000),
    F64(0x3f60_624d_e000_0000),
    F64(0x3f78_9374_c000_0000),
    F64(0x3f68_9374_c000_0000),
    F64(0x3f7c_ac08_4000_0000),
    F64(0x3f70_624d_e000_0000),
    F64(0x3f80_624d_e000_0000),
];

/// The recorded-pair margin μ of S-0212 F1.
// spec: S-0212 F1 (μ: the double nearest 1.009)
pub const PATH_MARGIN: F64 = F64(0x3ff0_24dd_2f1a_9fbe);

/// The extra cost λ of entering a sector across a diagonal connection.
// spec: S-0212 F9 (λ: the double nearest 0.001)
pub const DIAGONAL_CONNECTION: F64 = F64(0x3f50_624d_d2f1_a9fc);

/// 0.5 (exact in binary).
// spec: S-0207 claim 9 (d) (0.5 where g × u = 0) and S-0213 Formulas (the half cell of the look line)
pub const HALF: F64 = F64(0x3fe0_0000_0000_0000);

/// 0.75 (exact in binary).
// spec: S-0207 claim 9 (e) (0.75 for a damaged vehicle)
pub const THREE_QUARTERS: F64 = F64(0x3fe8_0000_0000_0000);

/// The braking floor F₀.₃ of S-0207 claim 10.
// spec: S-0207 claim 10 (F₀.₃ = 0.300000011920928955078125, single-precision 0.3)
pub const BRAKE_FLOOR: F64 = F64(0x3fd3_3333_4000_0000);

/// The target-fraction cap in the crush-slowdown state.
// spec: S-0210 F4 (0.2, the double nearest)
pub const CRUSH_CAP: F64 = F64(0x3fc9_9999_9999_999a);

/// 1/7 for the sub-point interpolation of S-0207 F4.
// spec: S-0207 F4 (1/7, the double nearest)
pub const SEVENTH: F64 = F64(0x3fc2_4924_9249_2492);

/// The category rule 3 threshold of S-0211.
// spec: S-0211 Formulas (category rule 3 threshold: the double nearest 0.01)
pub const LOW_MULTIPLIER: F64 = F64(0x3f84_7ae1_47ae_147b);

/// c₂ = −2π / 65534 of S-0208 F3.
// spec: S-0208 F3 (c₂ = −2π / 65534 as a double)
pub const WALK_C2: F64 = F64(0xbf19_222d_989f_5e57);

/// −2π / 65536 of S-0208 F3 with `quirks.table_direction` off.
// spec: S-0208 F3 (`quirks.table_direction` off: −2π / 65536)
pub const WALK_C: F64 = F64(0xbf19_21fb_5444_2d18);

/// The fall-rate step of S-0511 F3: 1.4 in double precision.
// spec: S-0511 F3 and Constants (fall-rate step 1.4 leptons per update, double precision)
pub const FALL_STEP: F64 = F64(0x3ff6_6666_6666_6666);

/// The explosion chance threshold of S-0511 claim 9: 0.95 in double
/// precision.
// spec: S-0511 claim 9 and Constants (explosion chance threshold 0.95)
pub const BRIDGE_EXPLOSION_CHANCE: F64 = F64(0x3fee_6666_6666_6666);

/// *c* of S-0511 F4 (S-0202 claim 13): the double nearest 1 / (2³² − 1).
// spec: S-0511 F4 (c, the double-precision constant nearest 1 / (2³² − 1))
pub const MAP_GENERATOR_C: F64 = F64(0x3df0_0000_0010_0000);

/// 2π of S-0503 F6's bob term: the double nearest 2π (Q-0690 interim, as
/// S-0306 F7 forms it).
// spec: S-0503 F6 (sin(ϱ × 2π / Pᵈ)); Q-0690 (interim: the double nearest 2π)
pub const TWO_PI: F64 = F64(0x4019_21fb_5444_2d18);

/// *m* = 1.1 of S-0503 F6 for an even object number: the double nearest 1.1
/// (Q-0690 interim).
// spec: S-0503 F6 (m = 1.1 for an even object number); Q-0690 (interim: the double nearest 1.1)
pub const EVEN_BOB_FACTOR: F64 = F64(0x3ff1_9999_9999_999a);

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_num::Rounding;
    use glasnost_num::decimal::{Field, nearest_double, parse};

    fn dec(t: &str) -> F64 {
        parse(t, Field::Decimal, Rounding::TowardZero).unwrap()
    }

    // spec: S-0212 table T4 (ε values read as single-precision decimals)
    #[test]
    fn epsilons() {
        let t = [
            "0.001", "0.005", "0.002", "0.006", "0.003", "0.007", "0.004", "0.008",
        ];
        for (e, s) in EPSILON.iter().zip(t) {
            assert_eq!(*e, dec(s), "{s}");
            assert_eq!(
                *e,
                parse(s, Field::Decimal, Rounding::Nearest).unwrap(),
                "{s}"
            );
        }
    }

    // spec: S-0207 claim 10 (F₀.₃ = 0.300000011920928955078125)
    #[test]
    fn brake_floor() {
        assert_eq!(BRAKE_FLOOR, dec("0.3"));
        assert_eq!(
            glasnost_num::decimal::exact_decimal(BRAKE_FLOOR),
            "0.300000011920928955078125"
        );
    }

    // spec: S-0212 F1 and F9, S-0210 F4, S-0207 F4, S-0211 Formulas, S-0208 F3 (the doubles nearest the decimals)
    #[test]
    fn nearest_doubles() {
        assert_eq!(PATH_MARGIN, nearest_double("1.009"));
        assert_eq!(DIAGONAL_CONNECTION, nearest_double("0.001"));
        assert_eq!(CRUSH_CAP, nearest_double("0.2"));
        assert_eq!(SEVENTH, nearest_double("0.142857142857142857142857142857"));
        assert_eq!(LOW_MULTIPLIER, nearest_double("0.01"));
        assert_eq!(WALK_C2, nearest_double("-0.00009587672516830327"));
        assert_eq!(WALK_C, nearest_double("-0.00009587379924285257"));
        assert_eq!(HALF, dec("0.5"));
        assert_eq!(THREE_QUARTERS, dec("0.75"));
    }

    // spec: S-0503 F6 (2π and m = 1.1 as the doubles nearest them, Q-0690 interim)
    #[test]
    fn hover_bob_constants() {
        assert_eq!(
            TWO_PI,
            nearest_double("6.28318530717958647692528676655900576839")
        );
        assert_eq!(EVEN_BOB_FACTOR, nearest_double("1.1"));
        // The same 2π as S-0306 F7's: four times the double-precision π/2.
        let (c1, _) = glasnost_num::geom::direction_constants();
        assert_eq!(
            TWO_PI,
            c1.mul(F64::from_i64(4, Rounding::TowardZero), Rounding::TowardZero)
        );
    }

    // spec: S-0511 F3, claim 9 and F4 (1.4, 0.95 and the double nearest 1 / (2³² − 1))
    #[test]
    fn bridge_constants() {
        assert_eq!(FALL_STEP, nearest_double("1.4"));
        assert_eq!(BRIDGE_EXPLOSION_CHANCE, nearest_double("0.95"));
        let one = F64::from_i64(1, Rounding::Nearest);
        let d = F64::from_i64(4_294_967_295, Rounding::Nearest);
        assert_eq!(MAP_GENERATOR_C, one.div(d, Rounding::Nearest));
    }
}
