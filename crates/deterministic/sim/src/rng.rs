//! The random streams (S-0202): the generator, its seeding, ranged draws and
//! their derived forms.
//!
//! The simulation owns one stream, the *scenario random stream*, which is
//! part of the saved state (claim 9). The *unsynchronised random stream*
//! (claim 11) uses the same generator outside the simulation and never
//! reaches game state; [`Rng`] serves both.

use glasnost_num::{F64, Rounding};
use serde::{Deserialize, Serialize};

/// Values in a stream's state (S-0202 claim 1).
// spec: S-0202 Constants (state: the 250 most recent values)
pub const STATE: usize = 250;

/// The short lag of the recurrence (S-0202 claim 2).
// spec: S-0202 Constants (short lag 147)
const SHORT_LAG: usize = 147;

/// The mixing constants P₀ … P₃ and Q₀ … Q₃ of S-0202 F (claim 3).
// spec: S-0202 Formulas (mixing function F, constants table)
const P: [u32; 4] = [3131664519, 504877868, 62708796, 255054258];
// spec: S-0202 Formulas (mixing function F, constants table)
const Q: [u32; 4] = [1259289432, 3899977923, 1767228838, 1437059654];

/// One stream of the generator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng {
    /// The 250 most recent values x_{N−250} … x_{N−1}, as a ring.
    state: Vec<u32>,
    /// Position of x_{N−250} in the ring.
    next: usize,
    /// Raw values taken since seeding (diagnostics; not used by the rules).
    taken: u64,
}

/// The round function G_i of S-0202's mixing function.
// spec: S-0202 Formulas (round function G_i: 16-bit halves)
fn g(i: usize, y: u32) -> u32 {
    // spec: S-0202 Formulas (A = y ⊕ P_i)
    let a = y ^ P[i];
    // spec: S-0202 Formulas (h = ⌊σ(A) / 65536⌋, ℓ = A mod 65536)
    let h = (a as i32) >> 16;
    let l = (a & 0xFFFF) as i32;
    // spec: S-0202 Formulas (B = (4294967295 − h² + ℓ²) mod 2³²)
    let b = u32::MAX
        .wrapping_sub((h.wrapping_mul(h)) as u32)
        .wrapping_add((l.wrapping_mul(l)) as u32);
    // spec: S-0202 Formulas (T(B): halves swapped below 2³¹; upper half all ones from 2³¹)
    let t = if b < 0x8000_0000 {
        b.rotate_left(16)
    } else {
        0xFFFF_0000 | (b >> 16)
    };
    // spec: S-0202 Formulas (G = ((T(B) ⊕ Q_i) + h × ℓ) mod 2³², signed product)
    (t ^ Q[i]).wrapping_add(h.wrapping_mul(l) as u32)
}

/// The seeding function F(s, k) of S-0202 claim 3.
pub fn mix(s: u32, k: u32) -> u32 {
    let (mut l, mut r) = (s, k);
    // spec: S-0202 Formulas (four rounds: L ← R, R ← L ⊕ G_i(R))
    for i in 0..4 {
        let next_r = l ^ g(i, r);
        l = r;
        r = next_r;
    }
    r
}

impl Rng {
    /// A stream seeded with `seed` (S-0202 claims 3 and 8).
    pub fn new(seed: u32) -> Self {
        Self {
            state: (0..STATE as u32).map(|k| mix(seed, k)).collect(),
            next: 0,
            taken: 0,
        }
    }

    /// Whether the stream holds together, as [`Rng::new`] makes it: its
    /// state has the generator's length and the position lies within it. A
    /// stream decoded from bytes may not (#157).
    pub fn is_consistent(&self) -> bool {
        // spec: S-0202 claim 2 (250 words of state)
        self.state.len() == STATE && self.next < STATE
    }

    /// The next raw value (S-0202 claims 1 and 2).
    pub fn raw(&mut self) -> u32 {
        // spec: S-0202 claim 2 (x_n = x_{n−250} ⊕ x_{n−147})
        let other = (self.next + (STATE - SHORT_LAG)) % STATE;
        let x = self.state[self.next] ^ self.state[other];
        self.state[self.next] = x;
        self.next = (self.next + 1) % STATE;
        self.taken += 1;
        x
    }

    /// Raw values taken since seeding.
    pub fn taken(&self) -> u64 {
        self.taken
    }

    // spec: S-0202 claims 4 and 5 (ranged draws, spans up to 2³² − 1)
    /// The ranged draw R(a, b) of S-0202 claims 4 and 5. `half_span_quirk` is
    /// `quirks.ranged_draw_half_span`.
    pub fn ranged(&mut self, a: i32, b: i32, half_span_quirk: bool) -> i32 {
        // spec: S-0202 claim 4 (a = b: no raw value)
        if a == b {
            return a;
        }
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let d = (i64::from(hi) - i64::from(lo)) as u64;
        // spec: S-0202 claim 5 (d = 2147483647: no raw value, lo + 2147483648 wrapped)
        if d == 2147483647 && half_span_quirk {
            return lo.wrapping_add(i32::MIN);
        }
        if d >= 2147483648 {
            // spec: S-0202 claim 5 (departure: all 32 bits kept, with rejection)
            loop {
                let x = self.raw();
                if u64::from(x) <= d {
                    return lo.wrapping_add(x as i32);
                }
            }
        }
        // spec: S-0202 claim 4 (m = binary digits of d; keep the lowest m bits, reject above d)
        let m = 64 - d.leading_zeros();
        let mask = (1u64 << m) - 1;
        loop {
            let v = u64::from(self.raw()) & mask;
            if v <= d {
                return lo.wrapping_add(v as i32);
            }
        }
    }

    /// The percent draw R(0, 99) (S-0202 claim 6).
    pub fn percent(&mut self) -> i32 {
        // spec: S-0202 claim 6 (percent draw R(0, 99))
        self.ranged(0, 99, true)
    }

    /// The unit-interval value u of S-0202 claim 6: R(0, 2147483646) × c,
    /// c = (2³⁰ + 1) / 2⁶¹, rounded by `mode` (S-0205 claim 5).
    pub fn unit(&mut self, mode: Rounding) -> F64 {
        // spec: S-0202 Formulas (unit-interval value: r = R(0, 2147483646))
        let r = self.ranged(0, 2147483646, true);
        unit_value(r, mode)
    }

    /// A list pick: one raw value modulo `n` (S-0202 claim 6).
    pub fn list_pick(&mut self, n: u32) -> u32 {
        self.raw() % n.max(1)
    }
}

/// u = r × c for a drawn r (S-0202 Formulas).
pub fn unit_value(r: i32, mode: Rounding) -> F64 {
    // spec: S-0202 Formulas (c = (2³⁰ + 1) / 2⁶¹)
    let c = F64::from_parts(false, (1u128 << 30) + 1, -61, Rounding::TowardZero);
    F64::from_i64(i64::from(r), mode).mul(c, mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0202 Formulas (test values for seeds 0, 1 and 123456789; x₉₉₉ for seed 0)
    #[test]
    fn test_values() {
        let cases: [(u32, [u32; 3], u32, [u32; 4]); 3] = [
            (
                0,
                [2905516568, 1718636987, 3199744093],
                3610121526,
                [1229352179, 1781224790, 2424954917, 2632964923],
            ),
            (
                1,
                [4099924131, 2128042508, 2801846478],
                1868701209,
                [2025287381, 660436142, 3663968561, 2901129938],
            ),
            (
                123456789,
                [1236345482, 1460399687, 2538562447],
                461485239,
                [3323095506, 1961208251, 3793762349, 793961667],
            ),
        ];
        for (seed, first, last, x) in cases {
            assert_eq!([mix(seed, 0), mix(seed, 1), mix(seed, 2)], first);
            assert_eq!(mix(seed, 249), last);
            let mut r = Rng::new(seed);
            let got: Vec<u32> = (0..4).map(|_| r.raw()).collect();
            assert_eq!(got, x);
        }
        let mut r = Rng::new(0);
        let x999 = (0..1000).map(|_| r.raw()).last();
        assert_eq!(x999, Some(2255550017));
    }

    // spec: S-0202 claim 2 (every bit position obeys the recurrence; checked on the low two bits)
    #[test]
    fn recurrence_on_low_bits() {
        let mut r = Rng::new(42);
        let xs: Vec<u32> = (0..600).map(|_| r.raw() & 3).collect();
        for k in 250..600 {
            assert_eq!(xs[k], xs[k - 250] ^ xs[k - 147]);
        }
    }

    // spec: S-0202 claim 4 (range rule: low m bits, rejection consumes raw values)
    #[test]
    fn ranged_draw_rule() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..1000 {
            let v = a.ranged(0, 10, true);
            // Reproduce by hand.
            let want = loop {
                let x = b.raw() & 15;
                if x <= 10 {
                    break x as i32;
                }
            };
            assert_eq!(v, want);
            assert_eq!(a.taken(), b.taken());
        }
        // Bounds in either order; a = b takes nothing.
        let mut c = Rng::new(3);
        assert_eq!(c.ranged(5, 5, true), 5);
        assert_eq!(c.taken(), 0);
        let v = c.ranged(10, -10, true);
        assert!((-10..=10).contains(&v));
        // A power-of-two size takes exactly one value.
        let before = c.taken();
        c.ranged(0, 7, true);
        assert_eq!(c.taken(), before + 1);
    }

    // spec: S-0202 claim 5 (half span: no value, wrapped result; switch off: inside the range)
    #[test]
    fn half_span() {
        let mut r = Rng::new(9);
        assert_eq!(r.ranged(0, 2147483647, true), i32::MIN);
        assert_eq!(r.taken(), 0);
        let v = r.ranged(0, 2147483647, false);
        assert!(v >= 0);
        assert_eq!(r.taken(), 1);
        let w = r.ranged(i32::MIN, i32::MAX, true);
        let _ = w;
    }

    // spec: S-0202 Formulas (largest u is 1 − 2⁻⁵³ under RZ, exactly 1 under RN)
    #[test]
    fn unit_interval_extremes() {
        let rz = unit_value(2147483646, Rounding::TowardZero);
        let rn = unit_value(2147483646, Rounding::Nearest);
        assert_eq!(rz.0, 0x3FEF_FFFF_FFFF_FFFF);
        assert_eq!(rn.0, 0x3FF0_0000_0000_0000);
        assert_eq!(unit_value(0, Rounding::TowardZero).0, 0);
    }
}
