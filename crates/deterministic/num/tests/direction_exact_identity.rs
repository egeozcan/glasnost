//! The exact direction (`quirks.table_direction` off, S-0204 Edge cases) is
//! bit-identical to the big-integer routine it had before #393, which this
//! file keeps as the reference: for integer displacements (`direction`) and
//! for real ones narrowed to single precision (`direction_real_b`, S-0306
//! Formulas). The per-call figures of the report come from the ignored
//! `per_call_figures`.

// The ignored timing test reads the host clock and prints its figures; it is
// a measurement in a test, never simulation code.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::time::Instant;

use glasnost_num::bigmath;
use glasnost_num::geom::{direction, direction_real_b};
use glasnost_num::{F32, F64, Rounding};
use num_bigint::BigUint;

/// The exact angle as computed before #393: arctan by `bigmath` with
/// `bigmath::FRAC` fraction bits, × 65536 / (2π), halves up, mod 65536.
// spec: S-0204 Edge cases (exact direction with `quirks.table_direction` off)
fn reference_angle(neg_x: bool, ax: &BigUint, neg_y: bool, ay: &BigUint) -> u16 {
    let s = BigUint::from(1u32) << bigmath::FRAC;
    let pi = bigmath::pi();
    let beta = if *ay == BigUint::from(0u32) {
        &pi >> 1u32
    } else {
        bigmath::atan(&((ax * &s) / ay))
    };
    let alpha = match (!neg_x, !neg_y) {
        (true, true) => beta,
        (true, false) => &pi - beta,
        (false, false) => &pi + beta,
        (false, true) => (&pi << 1u32) - beta,
    };
    // spec: S-0204 Edge cases (× 65536 / (2π), halves up, mod 65536)
    let t = (alpha * 32768u32 * &s) / &pi;
    let rounded = (t + (&s >> 1u32)) >> bigmath::FRAC;
    u64::try_from(rounded % 65536u32).unwrap() as u16
}

/// `direction(dx, dy, false)` before #393, with the zero displacement of
/// S-0204 revision 14.
// spec: S-0204 F3 (Y = y_P − y_Q grows towards map north), S-0204 Edge cases (revision 14, Q-3103: two zero arguments give 16384)
fn reference_direction(dx: i64, dy: i64) -> u16 {
    let (x, y) = (dx, -dy);
    if x == 0 && y == 0 {
        return 16384;
    }
    reference_angle(
        x < 0,
        &BigUint::from(x.unsigned_abs()),
        y < 0,
        &BigUint::from(y.unsigned_abs()),
    )
}

/// A finite single as (negative, M, k) with value ±M × 2^k.
// nocite: binary32 field layout (sign, exponent and fraction fields, bias and hidden bit)
fn single_parts(s: F32) -> Option<(bool, u64, i32)> {
    let bits = s.0;
    let exp = (bits >> 23) & 0xff;
    let frac = bits & 0x7f_ffff;
    match exp {
        0xff => None,
        0 => Some((bits >> 31 == 1, u64::from(frac), -149)),
        _ => Some((
            bits >> 31 == 1,
            u64::from(frac | 0x80_0000),
            exp as i32 - 150,
        )),
    }
}

/// `direction_real_b(x, y, false, mode)` before #393, for arguments finite
/// in single precision (`None` otherwise: those keep the table path).
// spec: S-0204 Edge cases, S-0306 Formulas (narrowed to single precision; revision 90: a zero displacement is 16384)
fn reference_real(x: F64, y: F64, mode: Rounding) -> Option<u16> {
    let (nx, mx, kx) = single_parts(x.to_f32(mode))?;
    let (ny, my, ky) = single_parts(y.to_f32(mode))?;
    if mx == 0 && my == 0 {
        return Some(16384);
    }
    let k = kx.min(ky);
    let ax = BigUint::from(mx) << (kx - k) as u32;
    let ay = BigUint::from(my) << (ky - k) as u32;
    Some(reference_angle(nx && mx != 0, &ax, ny && my != 0, &ay))
}

/// SplitMix64, a fixed sequence for the synthetic inputs.
struct Mix(u64);

impl Mix {
    // nocite: SplitMix64 constants, a test input generator of our own
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A signed value of 1 to 62 random bits (magnitude below 2^62).
    fn signed(&mut self) -> i64 {
        let bits = 1 + self.next() % 62;
        let v = (self.next() >> (64 - bits)) as i64;
        if self.next() & 1 == 1 { -v } else { v }
    }
}

/// Displacements whose exact facing lies within 2^-60 of a half (found
/// offline from convergents of tan((n + 1/2) π / 32768)); synthetic.
// nocite: synthetic test inputs near a rounding half, not game data
const NEAR_HALF: [(i64, i64); 7] = [
    (22915627, -159345772067),
    (592068621, -61445856871),
    (9874122349, -18993446275),
    (95240465374, -95249596877),
    (757313546896, -648151647047),
    (175176187187, -6443712252),
    (159345772067, -22915627),
];

/// Integer displacements: a grid around the origin, the near-half pairs in
/// all eight symmetries, axes and diagonals and their neighbours at every
/// scale, and `random` random pairs of mixed magnitudes.
// nocite: synthetic test inputs
fn integer_cases(random: usize) -> Vec<(i64, i64)> {
    let mut v = Vec::new();
    for dx in -24i64..=24 {
        for dy in -24i64..=24 {
            v.push((dx, dy));
        }
    }
    for (a, b) in NEAR_HALF {
        for (p, q) in [(a, b), (b, a)] {
            for (sx, sy) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                v.push((sx * p, sy * q));
            }
        }
    }
    for k in 1..62 {
        let n = 1i64 << k;
        for d in [-1i64, 0, 1] {
            let m = n + d;
            for (p, q) in [(m, n), (m, 1), (1, m), (m, 0), (0, m), (m, -n), (-m, n)] {
                v.push((p, q));
                v.push((q, p));
            }
        }
    }
    let mut g = Mix(0x0393);
    for i in 0..random {
        let (a, b) = (g.signed(), g.signed());
        // Every fourth pair has both components at the same scale.
        let b = if i % 4 == 0 {
            let s = a.unsigned_abs().leading_zeros();
            let m = (g.next() >> 2) as i64 >> (s.saturating_sub(2).min(61));
            if g.next() & 1 == 1 { -m } else { m }
        } else {
            b
        };
        v.push((a, b));
    }
    v
}

/// Real displacements: singles of every class (zero, subnormal, normal,
/// near the overflow), doubles that narrow differently under the two
/// roundings, mixed signs and zeros, and the integer pairs as reals.
// nocite: synthetic test inputs
fn real_cases(random: usize) -> Vec<(F64, F64)> {
    let mut g = Mix(0x3930);
    let mut v = Vec::new();
    let single = |g: &mut Mix| {
        let sign = (g.next() & 1) as u32;
        let exp = (g.next() % 255) as u32;
        let frac = (g.next() & 0x7f_ffff) as u32;
        F32((sign << 31) | (exp << 23) | frac).to_f64()
    };
    let double = |g: &mut Mix| {
        // Exponents from below the single subnormals to above the single range.
        let sign = g.next() & 1;
        let exp = 0x340 + g.next() % 0x180;
        F64((sign << 63) | (exp << 52) | (g.next() >> 12))
    };
    let near = |g: &mut Mix, x: F64| {
        // A double at a similar scale to `x`.
        let e = (x.0 >> 52) & 0x7ff;
        let e = (e + g.next() % 40).saturating_sub(20).clamp(1, 0x7fe);
        F64(((g.next() & 1) << 63) | (e << 52) | (g.next() >> 12))
    };
    for i in 0..random {
        let x = if i % 2 == 0 {
            single(&mut g)
        } else {
            double(&mut g)
        };
        let y = match i % 5 {
            0 => near(&mut g, x),
            1 => single(&mut g),
            2 => double(&mut g),
            3 => F64(g.next() & (1 << 63)),
            _ => near(&mut g, x),
        };
        if i % 2 == 0 {
            v.push((x, y));
        } else {
            v.push((y, x));
        }
    }
    let rz = Rounding::TowardZero;
    for (dx, dy) in integer_cases(random / 8) {
        v.push((F64::from_i64(dx, rz), F64::from_i64(dy, rz)));
    }
    v
}

fn check(integers: usize, reals: usize) {
    let ints = integer_cases(integers);
    for &(dx, dy) in &ints {
        assert_eq!(
            direction(dx, dy, false),
            reference_direction(dx, dy),
            "({dx}, {dy})"
        );
    }
    let mut checked = 0;
    for (x, y) in real_cases(reals) {
        for mode in [Rounding::TowardZero, Rounding::Nearest] {
            if let Some(want) = reference_real(x, y, mode) {
                assert_eq!(
                    direction_real_b(x, y, false, mode),
                    want,
                    "({:#x}, {:#x}) {mode:?}",
                    x.0,
                    y.0
                );
                checked += 1;
            }
        }
    }
    assert!(checked > reals, "{checked}");
}

// spec: S-0204 Edge cases (exact direction with `quirks.table_direction` off), S-0306 Formulas
#[test]
fn exact_direction_is_identical_to_the_reference() {
    check(3000, 3000);
}

// spec: S-0204 Edge cases (exact direction with `quirks.table_direction` off), S-0306 Formulas
#[test]
#[ignore]
fn exact_direction_is_identical_to_the_reference_large() {
    check(400_000, 400_000);
}

/// Prints mean nanoseconds per call of the exact direction and of the
/// reference, for integer and real arguments. Asserts nothing.
// nocite: host timing for the report (#393), not game behaviour
#[test]
#[ignore]
fn per_call_figures() {
    let ints = integer_cases(100_000);
    let reals: Vec<_> = real_cases(100_000)
        .into_iter()
        .filter(|&(x, y)| reference_real(x, y, Rounding::Nearest).is_some())
        .collect();
    let per = |t: Instant, n: usize| t.elapsed().as_nanos() / n as u128;
    let t = Instant::now();
    let mut acc = 0u64;
    for &(dx, dy) in &ints {
        acc += u64::from(direction(dx, dy, false));
    }
    let a = per(t, ints.len());
    let t = Instant::now();
    for &(dx, dy) in &ints {
        acc += u64::from(reference_direction(dx, dy));
    }
    let b = per(t, ints.len());
    let t = Instant::now();
    for &(x, y) in &reals {
        acc += u64::from(direction_real_b(x, y, false, Rounding::Nearest));
    }
    let c = per(t, reals.len());
    let t = Instant::now();
    for &(x, y) in &reals {
        acc += u64::from(reference_real(x, y, Rounding::Nearest).unwrap());
    }
    let d = per(t, reals.len());
    println!(
        "direction(.., false): {a} ns per call (reference {b} ns), {} integer pairs",
        ints.len()
    );
    println!(
        "direction_real_b(.., false, Nearest): {c} ns per call (reference {d} ns), {} real pairs",
        reals.len()
    );
    std::hint::black_box(acc);
}
