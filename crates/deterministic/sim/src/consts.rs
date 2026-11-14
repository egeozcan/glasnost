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

/// The threat coefficient a unit must exceed for the threat term and the
/// threat count (S-0212 claims 13 and 18, F7).
// spec: S-0212 F7 (the binary64 value nearest 0.00001)
pub const THREAT_COEFFICIENT_MIN: F64 = F64(0x3ee4_f8b5_88e3_68f1);

/// The threat a cell must reach to count in claim 18's limit (S-0212 F7).
// spec: S-0212 F7 (≥ the binary64 value nearest 0.01)
pub const THREAT_COUNTED: F64 = F64(0x3f84_7ae1_47ae_147b);

/// 1.0 (exact in binary).
// spec: S-0212 F7 (claim 17's threat test: below 1.0)
pub const ONE: F64 = F64(0x3ff0_0000_0000_0000);

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

/// 2π of S-0503 F6's bob term: the binary64 nearest exact 2π.
// spec: S-0503 F6 (sin(ϱ × 2π / Pᵈ); 2π the binary64 number nearest exact mathematical 2π, not a widened single; Q-0690 answered)
pub const TWO_PI: F64 = F64(0x4019_21fb_5444_2d18);

/// *m* = 1.1 of S-0503 F6 for an even object number: the binary64 nearest
/// exact decimal 1.1.
// spec: S-0503 F6 (m = 1.1 for an even object number, the binary64 number nearest exact decimal 1.1; Q-0690 answered)
pub const EVEN_BOB_FACTOR: F64 = F64(0x3ff1_9999_9999_999a);

// The decimals below are read by per-tick code (#406). Each is the double
// nearest the decimal named in its comment; the tests check every pattern.

/// The double nearest 4.656612877414201 × 10⁻¹⁰, the scale that turns a
/// raw draw into a fraction.
// spec: S-0306 F5 (k_ξ, the double nearest 1/2147483646) and S-0616 Constants (scale and spark chance scale 4.656612877414201 × 10⁻¹⁰)
pub const DRAW_SCALE: F64 = F64(0x3e00_0000_0040_0000);

/// The double nearest 4.656613 × 10⁻¹⁰, before its narrowing to single
/// precision.
// spec: S-0616 F16 (the single-precision scale 4.656613 × 10⁻¹⁰)
pub const COLOUR_START_SCALE: F64 = F64(0x3e00_0000_0751_0c0e);

/// The double nearest 0.05.
// spec: S-0306 F13 (max(d′, 0.05)) and S-0616 Constants (colour progress jitter 0.05)
pub const TWENTIETH: F64 = F64(0x3fa9_9999_9999_999a);

/// The double nearest 0.9.
// spec: S-0306 F6 (w × 0.9)
pub const NINE_TENTHS: F64 = F64(0x3fec_cccc_cccc_cccd);

/// The double nearest 0.06666666666666667 (1/15).
// spec: S-0306 F7 (j × 0.06666666666666667 × 2π)
pub const FIFTEENTH: F64 = F64(0x3fb1_1111_1111_1111);

/// 1.5 (exact in binary).
// spec: S-0306 F7 (q* = trunc(q × 1.5)) and S-0616 claim 1 and Constants (natural-smoke yellow factor 1.5)
pub const THREE_HALVES: F64 = F64(0x3ff8_0000_0000_0000);

/// *a* of S-0409 F2: the double nearest 0.14350360082660085.
// spec: S-0409 F2 (a, the binary64 constant nearest 0.14350360082660085 pixels per lepton)
pub const SIGHT_LIFT_SCALE: F64 = F64(0x3fc2_5e53_7434_4960);

/// The double nearest 0.8.
// spec: S-0107 claim 10 (a speed fraction of at most 0.8; the double nearest 0.8)
pub const FOUR_FIFTHS: F64 = F64(0x3fe9_9999_9999_999a);

/// The double nearest 0.3 (not the single-precision [`BRAKE_FLOOR`]).
// spec: S-0401 F3 and claim 4 (k = the double nearest 0.3)
pub const THREE_TENTHS: F64 = F64(0x3fd3_3333_3333_3333);

/// The double nearest 10⁻⁶.
// spec: S-0401 F8 (the double nearest 10⁻⁶) and S-0616 Constants (normalised speed threshold 10⁻⁶)
pub const MILLIONTH: F64 = F64(0x3eb0_c6f7_a0b5_ed8d);

/// The double nearest 0.1.
// spec: S-0310 claim 8 and Constants (the infantry speed fraction above 0.1, compared in double precision)
pub const TENTH: F64 = F64(0x3fb9_9999_9999_999a);

/// The double nearest 0.01.
// spec: S-0409 F1 (b′ × 0.01), S-0504 Constants (sinking pitch step 0.01 radians, double precision)
pub const HUNDREDTH: F64 = F64(0x3f84_7ae1_47ae_147b);

/// The double nearest 0.001.
// spec: S-0306 F3 (d = 0 counts as 0.001)
pub const THOUSANDTH: F64 = F64(0x3f50_624d_d2f1_a9fc);

/// The double nearest 0.02.
// spec: S-0616 F8 (φ = k × 0.02 + 1.0 in double precision)
pub const FIFTIETH: F64 = F64(0x3f94_7ae1_47ae_147b);

/// 0.25 (exact in binary).
// spec: S-0302 F5 (H′ = max(trunc(RZ₅₃(S × 0.25)), 1))
pub const QUARTER: F64 = F64(0x3fd0_0000_0000_0000);

/// 0.125 (exact in binary).
// spec: S-0616 F6 (V[i] = trunc(sinₜ(0.125 × i) × 12.0 + 0.49))
pub const EIGHTH: F64 = F64(0x3fc0_0000_0000_0000);

/// The double nearest 0.49.
// spec: S-0616 F6 (V[i] = trunc(sinₜ(0.125 × i) × 12.0 + 0.49))
pub const SWING_BIAS: F64 = F64(0x3fdf_5c28_f5c2_8f5c);

/// The double nearest 0.35.
// spec: S-0616 F4 and Constants (S = S − (T − T₀) × 0.35 in double precision)
pub const SMOKE_SPEED_LOSS: F64 = F64(0x3fd6_6666_6666_6666);

/// *c* of S-0303 F2: the double nearest 0.99.
// spec: S-0303 F2 (c = the double nearest 0.99)
pub const AREA_C99: F64 = F64(0x3fef_ae14_7ae1_47ae);

/// The arc speed margin: the double nearest 1.2.
// spec: S-0306 Constants (arc speed margin 1.2, the double nearest it)
pub const ARC_SPEED_MARGIN: F64 = F64(0x3ff3_3333_3333_3333);

/// Margin *a* of S-0307 F4 for infantry: the double nearest 281.6.
// spec: S-0307 F4 and Constants (margin a: 281.6 leptons for infantry, the nearest double)
pub const APPROACH_MARGIN_INFANTRY: F64 = F64(0x4071_9999_9999_999a);

/// Margin *a* of S-0307 F4 for vehicles: the double nearest 179.2.
// spec: S-0307 F4 and Constants (margin a: 179.2 leptons for vehicles, the nearest double)
pub const APPROACH_MARGIN_VEHICLE: F64 = F64(0x4066_6666_6666_6666);

/// The smallest radius of S-0307 F4, excluded: the double nearest 204.8.
// spec: S-0307 F4 and Constants (radii strictly greater than 204.8 leptons)
pub const APPROACH_RADIUS_CUT: F64 = F64(0x4069_9999_9999_999a);

/// *c* of S-0313 F2: the double nearest −π/16.
// spec: S-0313 F2 (c = −0.19634954084936207, the double nearest −π/16)
pub const FIRE_C16: F64 = F64(0xbfc9_21fb_5444_2d18);

/// α of S-0313 F3, a single-precision constant held as its double.
// spec: S-0313 F3 (α = 4.26669979095459, a single-precision constant)
pub const PIXEL_ALPHA: F64 = F64(0x4011_1119_c000_0000);

/// β of S-0313 F3, a single-precision constant held as its double.
// spec: S-0313 F3 (β = 8.53339958190918, a single-precision constant)
pub const PIXEL_BETA: F64 = F64(0x4021_1119_c000_0000);

/// The cliff speed line: the double nearest −0.0002.
// spec: S-0317 claim 10 and S-0319 claim 6 and Constants (the cliff speed line −0.0002, compared in double precision)
pub const CLIFF_SPEED_LINE: F64 = F64(0xbf2a_36e2_eb1c_432d);

/// *k* of S-0317 F7: the double nearest 6.967.
// spec: S-0317 F7 and Constants (k = 6.967, our engine's value)
pub const PIECE_TRAILER_K: F64 = F64(0x401b_de35_3f7c_ed91);

/// The sinking pitch limit *L*: π/4 in single precision, exact as a
/// decimal.
// spec: S-0504 F4 and Constants (L = 0.785398185253143310546875, π/4 in single precision)
pub const SINK_PITCH_LIMIT: F64 = F64(0x3fe9_21fb_6000_0000);

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

    // spec: S-0212 F1, F7 and F9, S-0210 F4, S-0207 F4, S-0211 Formulas, S-0208 F3 (the doubles nearest the decimals)
    #[test]
    fn nearest_doubles() {
        assert_eq!(PATH_MARGIN, nearest_double("1.009"));
        assert_eq!(DIAGONAL_CONNECTION, nearest_double("0.001"));
        assert_eq!(CRUSH_CAP, nearest_double("0.2"));
        assert_eq!(SEVENTH, nearest_double("0.142857142857142857142857142857"));
        assert_eq!(LOW_MULTIPLIER, nearest_double("0.01"));
        assert_eq!(THREAT_COEFFICIENT_MIN, nearest_double("0.00001"));
        assert_eq!(THREAT_COUNTED, nearest_double("0.01"));
        assert_eq!(ONE, dec("1"));
        assert_eq!(WALK_C2, nearest_double("-0.00009587672516830327"));
        assert_eq!(WALK_C, nearest_double("-0.00009587379924285257"));
        assert_eq!(HALF, dec("0.5"));
        assert_eq!(THREE_QUARTERS, dec("0.75"));
    }

    // spec: S-0503 F6 (2π and m = 1.1 as the binary64 numbers nearest the exact values; Q-0690 answered)
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

    // spec: S-0306 F3, F5, F6, F7 and F13, S-0616 F4, F6, F8, F16 and Constants, S-0409 F1 and F2, S-0107 claim 10, S-0401 F3 and F8, S-0310 Constants, S-0302 F5, S-0303 F2, S-0307 F4, S-0313 F2 and F3, S-0317 F7 and claim 10, S-0319 claim 6, S-0504 F4 (the doubles nearest the decimals the per-tick code used to parse, #406)
    #[test]
    fn per_tick_decimals() {
        let t: [(F64, &str); 31] = [
            (DRAW_SCALE, "4.656612877414201e-10"),
            (COLOUR_START_SCALE, "4.656613e-10"),
            (TWENTIETH, "0.05"),
            (NINE_TENTHS, "0.9"),
            (FIFTEENTH, "0.06666666666666667"),
            (THREE_HALVES, "1.5"),
            (SIGHT_LIFT_SCALE, "0.14350360082660085"),
            (FOUR_FIFTHS, "0.8"),
            (THREE_TENTHS, "0.3"),
            (MILLIONTH, "0.000001"),
            (TENTH, "0.1"),
            (HUNDREDTH, "0.01"),
            (THOUSANDTH, "0.001"),
            (FIFTIETH, "0.02"),
            (QUARTER, "0.25"),
            (EIGHTH, "0.125"),
            (SWING_BIAS, "0.49"),
            (SMOKE_SPEED_LOSS, "0.35"),
            (AREA_C99, "0.99"),
            (ARC_SPEED_MARGIN, "1.2"),
            (APPROACH_MARGIN_INFANTRY, "281.6"),
            (APPROACH_MARGIN_VEHICLE, "179.2"),
            (APPROACH_RADIUS_CUT, "204.8"),
            (FIRE_C16, "-0.19634954084936207"),
            (PIXEL_ALPHA, "4.26669979095459"),
            (PIXEL_BETA, "8.53339958190918"),
            (CLIFF_SPEED_LINE, "-0.0002"),
            (PIECE_TRAILER_K, "6.967"),
            (SINK_PITCH_LIMIT, "0.785398185253143310546875"),
            (HALF, "0.5"),
            (THREAT_COEFFICIENT_MIN, "0.00001"),
        ];
        for (c, s) in t {
            assert_eq!(c, nearest_double(s), "{s}");
        }
        // The literals the per-tick code parsed for constants already here.
        assert_eq!(TWO_PI, nearest_double("6.283185307179586"));
        assert_eq!(WALK_C2, nearest_double("-0.00009587672516830327"));
        // α and β are singles: narrowing them changes nothing.
        for c in [PIXEL_ALPHA, PIXEL_BETA, SINK_PITCH_LIMIT] {
            assert_eq!(c, c.to_f32(Rounding::Nearest).to_f64());
        }
    }
}
