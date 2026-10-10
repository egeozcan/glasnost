//! Turning facings (S-0206 claims 1 to 5, F1).

use glasnost_num::geom::{Facing, delta};
use serde::{Deserialize, Serialize};

/// A facing that can turn over time (S-0206 Terms, *turning facing*).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turning {
    /// Start facing ψ₀ of the current turn.
    pub start: Facing,
    /// Target facing ψ₁.
    pub target: Facing,
    /// Tick number t₀ at which the turn began.
    pub t0: i32,
    /// Duration Θ in ticks.
    pub duration: i32,
    /// Step β in facing units per tick.
    pub step: i32,
}

impl Turning {
    /// A facing resting at `f`.
    pub fn at(f: Facing) -> Self {
        Self {
            start: f,
            target: f,
            t0: 0,
            duration: 0,
            step: 0,
        }
    }

    /// The facing's value at tick number `t` (S-0206 F1).
    pub fn value(&self, t: i32) -> Facing {
        let eps = t.wrapping_sub(self.t0);
        if self.duration <= 0 || eps >= self.duration || eps < 0 {
            return self.target;
        }
        // spec: S-0206 F1 (facing = (ψ₁ − β × (Θ − ε)) mod 65536 for 0 ≤ ε < Θ)
        let v = i64::from(self.target) - i64::from(self.step) * i64::from(self.duration - eps);
        v.rem_euclid(65536) as Facing // spec: S-0206 F1 (mod 65536)
    }

    /// Whether the facing is turning at tick `t` (S-0206 Terms).
    pub fn turning(&self, t: i32) -> bool {
        self.duration > 0 && t.wrapping_sub(self.t0) < self.duration && t >= self.t0
    }

    /// Gives a new target facing at tick `t` with turn rate `omega`
    /// (S-0206 claims 1 to 3). Returns whether the target changed.
    pub fn turn_to(&mut self, target: Facing, t: i32, omega: i32) -> bool {
        // spec: S-0206 claim 3 (the same target again changes nothing)
        if target == self.target {
            return false;
        }
        // spec: S-0206 claim 3 (with ω ≤ 0 the facing takes every new target at once)
        if omega <= 0 {
            *self = Self::at(target);
            return true;
        }
        let current = self.value(t);
        // spec: S-0206 F1 (Δ = δ(ψ₀, ψ₁), Θ = ⌊|Δ| / (256 ω)⌋, β = trunc(Δ / Θ))
        let d = delta(current, target);
        let theta = d.abs() / (256 * omega); // spec: S-0206 F1 (Θ = ⌊|Δ| / (256 ω)⌋)
        if theta == 0 {
            *self = Self::at(target);
            return true;
        }
        *self = Self {
            start: current,
            target,
            t0: t,
            duration: theta,
            step: d / theta,
        };
        true
    }

    /// Sets the facing at once (S-0206 claim 5), with
    /// `quirks.set_facing_keeps_target`.
    pub fn set(&mut self, f: Facing, t: i32, keeps_target_quirk: bool) {
        // spec: S-0206 claim 5 (exception: a turn in progress whose current value is already f ends at its target)
        if keeps_target_quirk && self.turning(t) && self.value(t) == f {
            *self = Self::at(self.target);
            return;
        }
        *self = Self::at(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0206 F1 (example: 16384 to 11264 at ω = 3)
    #[test]
    fn example_from_s0105() {
        let mut f = Turning::at(16384);
        assert!(f.turn_to(11264, 100, 3));
        let got: Vec<u16> = (100..107).map(|t| f.value(t)).collect();
        assert_eq!(got, vec![16382, 15529, 14676, 13823, 12970, 12117, 11264]);
        assert!(!f.turning(106));
    }

    // spec: S-0206 claim 2 and F1 (a half turn goes counter-clockwise: 0 to 32768 at ω = 3)
    #[test]
    fn half_turn() {
        let mut f = Turning::at(0);
        f.turn_to(32768, 0, 3);
        assert_eq!(f.duration, 42);
        assert_eq!(f.value(0), 65528);
        assert_eq!(f.value(1), 64748);
        assert_eq!(f.value(2), 63968);
        assert_eq!(f.value(42), 32768);
    }

    // spec: S-0206 claim 3 and scenario R7 (a new target 32 ticks into a turn starts from the interpolated value)
    #[test]
    fn retarget_mid_turn() {
        let mut f = Turning::at(0);
        f.turn_to(16383, 0, 1);
        assert_eq!(f.value(31), 8063);
        assert_eq!(f.value(32), 8323);
        f.turn_to(49153, 32, 1);
        assert_eq!(f.duration, 96);
        assert_eq!(f.value(32), 8289);
        assert_eq!(f.value(128), 49153);
        // The same target again keeps the turn's timing.
        let before = f;
        assert!(!f.turn_to(49153, 50, 1));
        assert_eq!(f, before);
    }

    // spec: S-0206 claim 1 (turn times: map facing values 0, 1, 4 turning to east at ω = 1 take 64, 63, 60 ticks)
    #[test]
    fn r2_turn_times() {
        for (v, ticks) in [(0u16, 64), (1, 63), (4, 60)] {
            let mut f = Turning::at(256 * v);
            f.turn_to(16384, 0, 1);
            assert_eq!(f.duration, ticks);
        }
        // ω = 0 and short turns complete at once.
        let mut g = Turning::at(0);
        g.turn_to(32768, 5, 0);
        assert_eq!(g.value(5), 32768);
        let mut h = Turning::at(1024);
        h.turn_to(16384, 5, 127);
        assert_eq!(h.value(5), 16384);
    }

    // spec: S-0206 claim 5 (setting at once; the keeps-target exception)
    #[test]
    fn set_at_once() {
        let mut f = Turning::at(0);
        f.turn_to(16383, 0, 1);
        let v = f.value(10);
        f.set(v, 10, true);
        assert_eq!(f.value(10), 16383);
        let mut g = Turning::at(0);
        g.turn_to(16383, 0, 1);
        let v = g.value(10);
        g.set(v, 10, false);
        assert_eq!(g.value(10), v);
        assert!(!g.turning(11));
    }
}
