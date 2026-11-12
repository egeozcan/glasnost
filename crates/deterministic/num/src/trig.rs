//! Table trigonometry (S-0105 F1), used by the walking step of S-0208 F3.

use crate::soft::{F32, F64, Rounding};
use crate::tables::{SINE_QUARTER, TAU_4096, TAU_8192};

/// τ(n) of S-0105 F1 for n = 0 … 10239, as a single.
// spec: S-0105 F1 (τ over 0 … 10239: quarter waves of 2048 steps, 8192 steps per turn)
pub fn tau(n: u32) -> F32 {
    // spec: S-0105 F1 (the two exceptions τ(4096) and τ(8192))
    if n == 4096 {
        return F32(TAU_4096);
    }
    if n == 8192 {
        return F32(TAU_8192);
    }
    // spec: S-0105 F1 (τ(n) = sin(2πn/8192) truncated toward zero: symmetric quarter waves)
    let m = n % 8192;
    let neg_sign = 1u32 << 31; // nocite: binary32 sign bit
    let (q, neg) = if m <= 2048 {
        (m, false)
    } else if m <= 4096 {
        (4096 - m, false)
    } else if m <= 6144 {
        (m - 4096, true)
    } else {
        (8192 - m, true)
    };
    let bits = SINE_QUARTER[q as usize];
    F32(if neg && bits != 0 {
        bits | neg_sign
    } else {
        bits
    })
}

/// The table index σ of an angle (S-0105 F1).
// spec: S-0105 F1 (σ from I: half steps, 8192 steps, capped at 8191)
pub fn table_index(x: F64, mode: Rounding) -> u32 {
    table_index_b(x, mode, true)
}

/// The table index σ of an angle with `quirks.table_trig_negative_bias`
/// at `bias` (S-0105 F1 and Quirks): on gives [`table_index`]; off rounds
/// a negative angle to the nearest step, halves away from zero.
// spec: S-0105 F1 and Quirks (quirks.table_trig_negative_bias; fixed: exact rounding)
pub fn table_index_b(x: F64, mode: Rounding, bias: bool) -> u32 {
    // spec: S-0105 F1 (I = trunc(x × 2607.594482421875), half table steps)
    let i = x
        .mul(half_steps_per_radian(), mode)
        .trunc_i128()
        .unwrap_or(0);
    let s = if i >= 0 || bias {
        // spec: S-0105 F1 (σ = min((trunc(I / 2) mod 8192) + (|I| mod 2), 8191))
        let half = i / 2;
        half.rem_euclid(8192) + (i.abs() % 2)
    } else {
        // spec: S-0105 F1 and Quirks (switch off: ŝ = −(m + f) gives −m for f < 1/2 and the nearest −(m + 1) for f ≥ 1/2; with k = |I|, m = ⌊k / 2⌋ and f ≥ 1/2 exactly when k is odd)
        let k = i.abs();
        (-(k / 2 + k % 2)).rem_euclid(8192)
    };
    s.min(8191) as u32
}

/// 2607.594482421875 = 16384/2π rounded to single precision (S-0105 F1),
/// an exact binary value: 10680707 / 4096.
fn half_steps_per_radian() -> F64 {
    // spec: S-0105 F1 (the factor 2607.594482421875, exactly 10680707 / 4096)
    F64::from_parts(false, 10680707, -12, Rounding::TowardZero)
}

/// sinT x (S-0105 F1).
pub fn sin_t(x: F64, mode: Rounding) -> F64 {
    tau(table_index(x, mode)).to_f64()
}

/// cosT x (S-0105 F1).
pub fn cos_t(x: F64, mode: Rounding) -> F64 {
    cos_tb(x, mode, true)
}

/// sinT x with `quirks.table_trig_negative_bias` at `bias` (S-0105 F1).
pub fn sin_tb(x: F64, mode: Rounding, bias: bool) -> F64 {
    tau(table_index_b(x, mode, bias)).to_f64()
}

/// cosT x with `quirks.table_trig_negative_bias` at `bias` (S-0105 F1).
pub fn cos_tb(x: F64, mode: Rounding, bias: bool) -> F64 {
    // spec: S-0105 F1 (cos x = τ(σ + 2048))
    tau(table_index_b(x, mode, bias) + 2048).to_f64()
}

/// The table arccosine of S-0306 F8: π/2 − A_s[trunc(2048 (x + 1))]; a
/// value that is not a number converts to the index 0 (S-0205 claim 7).
/// An index outside 0 … 4096 is clamped (the original's table has no
/// entries there).
pub fn acos_t(x: F64, mode: Rounding) -> F64 {
    // spec: S-0306 F8 (k = trunc(2048 (x + 1)))
    let k = x
        .add(F64::from_i64(1, mode), mode)
        .mul(F64::from_i64(2048, mode), mode)
        .trunc_i128()
        .unwrap_or(0)
        .clamp(0, 4096) as usize;
    let (c1, _) = crate::geom::direction_constants();
    c1.sub(F32(crate::tables::ASIN[k]).to_f64(), mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::disallowed_types)]
    #[test]
    fn factor_is_exact() {
        assert_eq!(
            f64::from_bits(half_steps_per_radian().0).to_bits(),
            2607.594482421875f64.to_bits()
        );
    }

    // spec: S-0105 F1 (non-negative angles round to the nearest step; negative ones two steps short)
    #[test]
    fn index_rounding() {
        let rz = Rounding::TowardZero;
        // One table step is 2π/8192; take exact multiples of the half step.
        let step = |k: i64| {
            // k half steps: k / 2607.594482421875 radians (slightly over, so I = k).
            F64::from_i64(k, rz)
                .div(half_steps_per_radian(), Rounding::Nearest)
                .add(F64::from_parts(false, 1, -40, rz), rz)
        };
        assert_eq!(table_index(step(0), rz), 0);
        assert_eq!(table_index(step(1), rz), 1); // half step rounds up
        assert_eq!(table_index(step(4), rz), 2);
        assert_eq!(table_index(step(5), rz), 3);
        assert_eq!(table_index(step(16383), rz), 8191); // never wraps to 0
        // Negative: −(m + f) steps with f ≥ 1/2, m ≥ 2 → −(m − 1).
        let neg = |k: i64| step(k).negated();
        // I = −5: m = 2, f = 1/2 → σ = −1 mod 8192 → 8191? trunc(−5/2) = −2, |I| mod 2 = 1.
        assert_eq!(table_index(neg(5), rz), (8192 - 2 + 1) as u32);
        assert_eq!(table_index(neg(1), rz), 1); // m = 0, f ≥ 1/2: +1 step
    }

    // spec: S-0105 F1 and Quirks (quirks.table_trig_negative_bias off: exact rounding of negative angles; non-negative ones unchanged)
    #[test]
    fn index_rounding_unbiased() {
        let rz = Rounding::TowardZero;
        let step = |k: i64| {
            F64::from_i64(k, rz)
                .div(half_steps_per_radian(), Rounding::Nearest)
                .add(F64::from_parts(false, 1, -40, rz), rz)
        };
        for k in [0, 1, 4, 5, 16383] {
            assert_eq!(table_index_b(step(k), rz, false), table_index(step(k), rz));
        }
        let neg = |k: i64| step(k).negated();
        // I = −5: m = 2, f = 1/2 → the nearest −3 steps.
        assert_eq!(table_index_b(neg(5), rz, false), 8192 - 3);
        // I = −4: m = 2, f < 1/2 → −2 steps, as with the switch on.
        assert_eq!(table_index_b(neg(4), rz, false), 8192 - 2);
        assert_eq!(table_index(neg(4), rz), 8192 - 2);
        // I = −1: m = 0, f ≥ 1/2 → −1 step instead of +1.
        assert_eq!(table_index_b(neg(1), rz, false), 8191);
    }
}
