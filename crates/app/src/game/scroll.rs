//! Edge and arrow-key scrolling of the battlefield view (S-0101 claims 12
//! to 15, F9), and the displacement of right-drag scrolling (claim 16,
//! F10), driven by S-0214 claim 27's gesture in the session.
//!
//! Scrolling is presentation state: it moves the view centre only, once per
//! unfrozen game cycle, and never reaches the simulation.

use glasnost_num::{F64, Rounding};

/// The magnitude table *T(i)* of F9, pixels before the multiplier.
// spec: S-0101 F9 (T(0) … T(8))
const MAGNITUDE: [i64; 9] = [448, 384, 320, 256, 192, 128, 64, 32, 16];

/// The highest table index (S-0101 F9: *i* = 8 − *aₑ*).
const LAST_INDEX: i32 = 8; // spec: S-0101 F9

/// The momentum ceiling's base (S-0101 F9: *aₑ* = min(*a*, 7 − *r*)).
const MOMENTUM_BASE: i32 = 7; // spec: S-0101 F9

/// Pixels an unmodified held arrow requests per unfrozen cycle (S-0101
/// claim 14).
pub const ARROW_STEP: i32 = 21; // spec: S-0101 claim 14

/// The pacing clock's unit, milliseconds (S-0101 F9, as in S-0201).
const CLOCK_UNIT_MS: u64 = 16; // spec: S-0101 F9 (⌊host milliseconds / 16⌋)

/// The default scroll index *r* (S-0101 Import mapping: a missing saved
/// setting gives 3). Our engine has no settings file yet, so it always
/// applies.
pub const DEFAULT_SCROLL_INDEX: i32 = 3; // spec: S-0101 Import mapping (scroll-speed control default 3)

/// The direction vectors of F9 for *d* = 0 … 7 (screen x right, y down).
// spec: S-0101 F9 (direction vectors)
const DIRECTIONS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// Edge momentum and its retained direction (S-0101 Terms, F9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EdgeScroll {
    /// Edge momentum *a*, 0 at game start.
    pub momentum: i32,
    /// The direction last sampled at an edge.
    pub direction: Option<(i32, i32)>,
    /// The pacing clock at the last momentum change; `None` before any.
    pub changed_at: Option<u64>,
}

/// Whether a client pixel activates edge scrolling (S-0101 claim 12): only
/// the outermost full-client column or row, or beyond.
pub fn on_edge(p: (i32, i32), client: (i32, i32)) -> bool {
    let (b, c) = client;
    // spec: S-0101 claim 12 (client column 0 or B − 1, row 0 or C − 1, or beyond)
    p.0 <= 0 || p.0 >= b - 1 || p.1 <= 0 || p.1 >= c - 1
}

/// Whether edge scrolling may run this cycle: the window has focus and the
/// simulation tick is outside the order block ending at `blocked_through`
/// (S-0201 claim 14, S-0213 claim 1). Held arrows are not affected.
pub fn edge_enabled(focused: bool, tick: i32, blocked_through: i32) -> bool {
    // spec: S-0201 claim 14 (edge scrolling is suppressed during an order block)
    // spec: S-0213 claim 1 (the block covers ticks up to its last blocked tick)
    focused && tick > blocked_through
}

/// Whether a view-arrow press or release is accepted (S-0213 claim 23,
/// S-0201 claim 14): `blocked` says orders are blocked in this cycle, and
/// `discard` is `quirks.blocked_arrow_events`. A discarded event changes no
/// accepted contribution: a blocked release leaves a held arrow scrolling,
/// a blocked press adds nothing.
pub fn arrow_event_accepted(blocked: bool, discard: bool) -> bool {
    // spec: S-0213 claim 23 and S-0201 claim 14 (presses and releases made during an order block are discarded under quirks.blocked_arrow_events; off, they are accepted)
    !(blocked && discard)
}

fn dbl(v: i64) -> F64 {
    F64::from_i64(v, Rounding::TowardZero)
}

/// The F9 direction for a pointer on an edge: the snapped client point,
/// its bearing by S-0204 F3 in pixels, then the coarse direction.
pub fn edge_direction(p: (i32, i32), client: (i32, i32)) -> (i32, i32) {
    let rz = Rounding::TowardZero;
    let (b, c) = client;
    // spec: S-0101 F9 (α = 0.16 and β = 0.21 as the nearest doubles)
    let alpha = glasnost_num::decimal::nearest_double("0.16");
    let beta = glasnost_num::decimal::nearest_double("0.21");
    let one = dbl(1);
    // spec: S-0101 F9 (comparisons with double-precision products rounded toward zero)
    let snap = |v: i32, size: i32, k: F64| {
        let s = dbl(i64::from(size));
        let v64 = dbl(i64::from(v));
        if v64.lt(k.mul(s, rz)) {
            0
        } else if v64.gt(one.sub(k, rz).mul(s, rz)) {
            size - 1
        } else {
            // spec: S-0101 F9 (otherwise trunc₀(B/2))
            size / 2
        }
    };
    let s = snap(p.0, b, alpha);
    let t = snap(p.1, c, beta);
    // spec: S-0101 F9 (Δx = s − trunc₀(B/2), Δy = t − trunc₀(C/2), in pixels; zero bearing is screen up)
    let dx = i64::from(s - b / 2);
    let dy = i64::from(t - c / 2);
    let f = i32::from(glasnost_num::geom::direction(dx, dy, true));
    // spec: S-0101 F9 (u = ⌊(f + 128)/256⌋ mod 256, d = ⌊(u + 16)/32⌋ mod 8)
    let u = ((f + 128) / 256).rem_euclid(256);
    let d = ((u + 16) / 32).rem_euclid(8);
    DIRECTIONS[d as usize]
}

/// The F9 magnitude for effective momentum `ae`: trunc₀(rz(T(8 − aₑ) × μ)).
pub fn magnitude(ae: i32, mu: F64) -> i32 {
    let i = (LAST_INDEX - ae).clamp(0, LAST_INDEX) as usize;
    // spec: S-0101 F9 (floating-point product rounded toward zero, then truncation toward zero, S-0205)
    let m = dbl(MAGNITUDE[i]).mul(mu, Rounding::TowardZero);
    m.trunc_i128().map_or(0, |v| {
        v.clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32
    })
}

impl EdgeScroll {
    /// One unfrozen cycle (S-0101 claim 13, F9): the displacement the edge
    /// requests, with the momentum transition. `pointer` is the client
    /// pixel (none when the pointer is outside the window), `r` the scroll
    /// index, `mu` the multiplier, `now_ms` the host clock, and `blocked`
    /// whether F5 blocks a one-pixel move in a direction.
    pub fn cycle(
        &mut self,
        pointer: Option<(i32, i32)>,
        client: (i32, i32),
        r: i32,
        mu: F64,
        now_ms: u64,
        blocked: impl Fn((i32, i32)) -> bool,
    ) -> (i32, i32) {
        // spec: S-0101 F9 (aₑ = min(a, 7 − r))
        let ae = self.momentum.min(MOMENTUM_BASE - r).max(0);
        let clock = now_ms / CLOCK_UNIT_MS;
        // spec: S-0101 claim 13 (a stage changes at most once per cycle, and only when the 16 ms clock has advanced since the last change)
        let advanced = self.changed_at.is_none_or(|t| clock > t);
        let at_edge = pointer.filter(|&p| on_edge(p, client));
        if let Some(p) = at_edge {
            // spec: S-0101 claim 12 (the direction is sampled again at an edge)
            let dir = edge_direction(p, client);
            self.direction = Some(dir);
            // spec: S-0101 F9 (a one-pixel request blocked by F5: no request and no acceleration)
            if blocked(dir) {
                self.momentum = ae;
                return (0, 0);
            }
            let m = magnitude(ae, mu);
            self.momentum = if advanced {
                self.changed_at = Some(clock);
                ae + 1
            } else {
                ae
            };
            // spec: S-0101 claim 13 (a diagonal's components have the cardinal magnitude, not normalised)
            return (m * dir.0, m * dir.1);
        }
        match self.direction {
            // spec: S-0101 F9 (away from an edge a nonzero momentum still requests in the retained direction, and decays when the interval expires, even when blocked)
            Some(dir) if ae > 0 => {
                let m = magnitude(ae, mu);
                self.momentum = if advanced {
                    self.changed_at = Some(clock);
                    ae - 1
                } else {
                    ae
                };
                (m * dir.0, m * dir.1)
            }
            // spec: S-0101 F9 (no request away from an edge with momentum 0)
            _ => {
                self.momentum = ae;
                (0, 0)
            }
        }
    }

    /// The requested screen direction the edge-scroll cursor shows (S-0213
    /// claim 24): the pointer's edge direction on the outer client edge,
    /// otherwise the retained direction while momentum decays, otherwise
    /// none. `r` is the scroll index.
    pub fn shown_direction(
        &self,
        pointer: Option<(i32, i32)>,
        client: (i32, i32),
        r: i32,
    ) -> Option<(i32, i32)> {
        // spec: S-0213 claim 24 (the outer edge selects the presentation; S-0101 F9 owns the direction)
        if let Some(p) = pointer.filter(|&p| on_edge(p, client)) {
            return Some(edge_direction(p, client));
        }
        // spec: S-0213 claim 24 (the retained direction while edge momentum decays)
        // spec: S-0101 F9 (aₑ = min(a, 7 − r); a run-on request needs aₑ > 0)
        let ae = self.momentum.min(MOMENTUM_BASE - r);
        self.direction.filter(|_| ae > 0)
    }
}

/// The edge region of right-drag scrolling, pixels (S-0101 F10, strict
/// comparisons).
const DRAG_EDGE_REGION: i32 = 10; // spec: S-0101 Constants (drag edge region)

/// The least outward displacement before the boost (S-0101 F10).
const DRAG_MIN_OUTWARD: i32 = 5; // spec: S-0101 Constants (drag minimum outward displacement)

/// The outward boost of right-drag scrolling (S-0101 F10).
const DRAG_EDGE_BOOST: i32 = 4; // spec: S-0101 Constants (drag edge boost)

/// A right-drag scrolling method (S-0101 claim 16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragMethod {
    /// Method 0: the pointer stays displaced; scrolls toward the
    /// displacement.
    Hold,
    /// Method 1: the same direction; the pointer returns to the anchor.
    Recentre,
    /// Method 2: the opposite direction; the pointer returns to the anchor.
    Reversed,
}

/// What one unfrozen cycle of right-drag scrolling requests (S-0101 claim
/// 16, F10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DragRequest {
    /// The requested displacement of the view centre, pixels, before F5.
    pub displacement: (i32, i32),
    /// Whether the pointer returns to the anchor (methods 1 and 2, when the
    /// two magnitudes sum to more than 0), even if F5 then blocks the move.
    pub recentre: bool,
}

/// The right-drag request of one unfrozen cycle (S-0101 claim 16, F10):
/// `anchor` and `pointer` are view pixels, `client` the full client size
/// (*B*, *C*), `r` the scroll index 0 … 6. Activation, the anchor and the
/// release are S-0214 claim 27's (`session::scroll_cycle`).
pub fn drag_request(
    anchor: (i32, i32),
    pointer: (i32, i32),
    client: (i32, i32),
    r: i32,
    method: DragMethod,
) -> DragRequest {
    let rz = Rounding::TowardZero;
    let (b, c) = client;
    // spec: S-0101 F10 (a zero difference becomes −1 at bₓ = 0 or +1 at bₓ ≥ B − 1; vertically −1 at
    // bᵧ = 0 or +1 at bᵧ ≥ C)
    let substitute = |delta: i32, at: i32, far: i32| match delta {
        0 if at == 0 => -1,
        0 if at >= far => 1,
        d => d,
    };
    let dx = substitute(pointer.0 - anchor.0, anchor.0, b - 1);
    let dy = substitute(pointer.1 - anchor.1, anchor.1, c);
    // spec: S-0101 F10 (e = −4·max(5, |δ|) for b < 10 and δ < 0; 4·max(5, δ) for b > N − 10 and δ > 0;
    // else δ; exact integers)
    let boost = |delta: i32, at: i32, n: i32| {
        if at < DRAG_EDGE_REGION && delta < 0 {
            -DRAG_EDGE_BOOST * DRAG_MIN_OUTWARD.max(delta.abs())
        } else if at > n - DRAG_EDGE_REGION && delta > 0 {
            DRAG_EDGE_BOOST * DRAG_MIN_OUTWARD.max(delta)
        } else {
            delta
        }
    };
    let ex = boost(dx, anchor.0, b);
    let ey = boost(dy, anchor.1, c);
    // spec: S-0101 F10 (K = 1 for method 0, 12 for methods 1 and 2)
    let k = match method {
        DragMethod::Hold => 1,
        DragMethod::Recentre | DragMethod::Reversed => 12,
    };
    // spec: S-0101 F10 (g = rz_d(K / (r + 1)))
    let g = dbl(k).div(dbl(i64::from(r) + 1), rz);
    // spec: S-0101 F10 (magnitude |trunc₀(rz_d(e·g))|; direction the sign of e, opposite for
    // method 2; a zero component requests zero)
    let sign = match method {
        DragMethod::Reversed => -1,
        DragMethod::Hold | DragMethod::Recentre => 1,
    };
    let axis = |e: i32| -> i32 {
        let m = dbl(i64::from(e))
            .mul(g, rz)
            .trunc_i128()
            .map_or(0, |v| v.unsigned_abs().min(i32::MAX as u128) as i32);
        sign * e.signum() * m
    };
    let displacement = (axis(ex), axis(ey));
    // spec: S-0101 F10 (methods 1 and 2 recentre when the two magnitudes sum to more than 0, before F5)
    let recentre = method != DragMethod::Hold && (displacement.0 != 0 || displacement.1 != 0);
    DragRequest {
        displacement,
        recentre,
    }
}

/// The arrow-key request of one unfrozen cycle (S-0101 claim 14): 21 pixels
/// per held arrow along its axis; opposed arrows cancel. `held` is left,
/// right, up, down.
pub fn arrows(held: [bool; 4]) -> (i32, i32) {
    let [l, r, u, d] = held;
    (
        ARROW_STEP * (i32::from(r) - i32::from(l)),
        ARROW_STEP * (i32::from(d) - i32::from(u)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one() -> F64 {
        dbl(1)
    }

    /// spec: S-0101 claim 12 (only the outermost full-client edge activates edge scrolling)
    #[test]
    fn claim_12_outermost_edge_only() {
        let c = (800, 600);
        assert!(on_edge((0, 300), c));
        assert!(on_edge((799, 300), c));
        assert!(on_edge((400, 0), c));
        assert!(on_edge((400, 599), c));
        assert!(!on_edge((1, 300), c));
        assert!(!on_edge((798, 300), c));
        assert!(!on_edge((400, 598), c));
    }

    /// spec: S-0213 claim 24 (the edge cursor follows the requested direction at the outer edge and the retained direction during decay)
    #[test]
    fn s0213_claim_24_shown_direction() {
        let c = (800, 600);
        let mut e = EdgeScroll::default();
        assert_eq!(e.shown_direction(Some((400, 300)), c, 3), None);
        assert_eq!(e.shown_direction(Some((799, 300)), c, 3), Some((1, 0)));
        assert_eq!(e.shown_direction(Some((0, 0)), c, 3), Some((-1, -1)));
        assert_eq!(e.shown_direction(None, c, 3), None);
        for k in 0..3u64 {
            e.cycle(Some((799, 300)), c, 3, one(), 16 * k, |_| false);
        }
        // Off the edge, the retained direction while momentum decays.
        assert_eq!(e.shown_direction(Some((400, 300)), c, 3), Some((1, 0)));
        for k in 3..10u64 {
            e.cycle(Some((400, 300)), c, 3, one(), 16 * k, |_| false);
        }
        assert_eq!(e.momentum, 0);
        assert_eq!(e.shown_direction(Some((400, 300)), c, 3), None);
    }

    /// spec: S-0201 claim 14 (edge scrolling is off during an order block and without focus)
    #[test]
    fn s0201_claim_14_edge_off_during_order_block() {
        assert!(!edge_enabled(true, 3, 3));
        assert!(!edge_enabled(true, 0, 3));
        assert!(edge_enabled(true, 4, 3));
        assert!(!edge_enabled(false, 4, 3));
    }

    /// spec: S-0201 claim 14 and S-0213 claim 23 (arrow events during an order block are discarded under quirks.blocked_arrow_events, accepted with it off or outside the block)
    #[test]
    fn s0201_claim_14_arrow_events_during_order_block() {
        assert!(!arrow_event_accepted(true, true));
        assert!(arrow_event_accepted(true, false));
        assert!(arrow_event_accepted(false, true));
        assert!(arrow_event_accepted(false, false));
    }

    /// spec: S-0101 claim 12 and F9 (corners scroll diagonally, edge midpoints along one screen axis)
    #[test]
    fn claim_12_direction() {
        let c = (800, 600);
        assert_eq!(edge_direction((0, 0), c), (-1, -1));
        assert_eq!(edge_direction((799, 599), c), (1, 1));
        assert_eq!(edge_direction((799, 0), c), (1, -1));
        assert_eq!(edge_direction((400, 0), c), (0, -1));
        assert_eq!(edge_direction((0, 300), c), (-1, 0));
        assert_eq!(edge_direction((799, 300), c), (1, 0));
        assert_eq!(edge_direction((400, 599), c), (0, 1));
        // Along the top edge, the snap band is 0.16 · 800 = 128 wide.
        assert_eq!(edge_direction((127, 0), c), (-1, -1));
        assert_eq!(edge_direction((128, 0), c), (0, -1));
    }

    /// spec: S-0101 claim 13 and F9 (the table sequence 8, 7, …, r + 1, then r + 1 repeated; one stage per advanced 16 ms clock unit; truncation of T × μ)
    #[test]
    fn claim_13_acceleration_and_magnitude() {
        let client = (800, 600);
        for r in 0..=6 {
            let mut e = EdgeScroll::default();
            let mut seen = Vec::new();
            for k in 0..16u64 {
                let d = e.cycle(Some((799, 300)), client, r, one(), 16 * k, |_| false);
                assert_eq!(d.1, 0);
                seen.push(d.0);
            }
            let mut expect: Vec<i32> = (r + 1..=8)
                .rev()
                .map(|i| MAGNITUDE[i as usize] as i32)
                .collect();
            expect.resize(16, MAGNITUDE[(r + 1) as usize] as i32);
            assert_eq!(seen, expect, "r = {r}");
        }
        // μ = 0.1: T(7) gives 3 and T(8) gives 1 (truncation, not nearest).
        let tenth = glasnost_num::decimal::nearest_double("0.1");
        assert_eq!(magnitude(0, tenth), 1);
        assert_eq!(magnitude(1, tenth), 3);
        // No stage change within one clock unit.
        let mut e = EdgeScroll::default();
        e.cycle(Some((799, 300)), client, 3, one(), 0, |_| false);
        assert_eq!(e.momentum, 1);
        e.cycle(Some((799, 300)), client, 3, one(), 15, |_| false);
        assert_eq!(e.momentum, 1);
        // A diagonal has the cardinal magnitude on both axes.
        let mut e = EdgeScroll::default();
        assert_eq!(
            e.cycle(Some((0, 0)), client, 3, one(), 0, |_| false),
            (-16, -16)
        );
    }

    /// spec: S-0101 claims 12 and 13, F9 (leaving the edge keeps the direction while momentum decays; a blocked one-pixel move gives no request or acceleration)
    #[test]
    fn claim_13_run_on_and_blocking() {
        let client = (800, 600);
        let mut e = EdgeScroll::default();
        for k in 0..6u64 {
            e.cycle(Some((799, 300)), client, 3, one(), 16 * k, |_| false);
        }
        // The stored momentum may exceed 7 − r by one; aₑ caps it.
        assert_eq!(e.momentum, 5);
        let mut run_on = Vec::new();
        for k in 6..12u64 {
            run_on.push(e.cycle(Some((400, 300)), client, 3, one(), 16 * k, |_| false));
        }
        assert_eq!(
            run_on,
            [(192, 0), (128, 0), (64, 0), (32, 0), (0, 0), (0, 0)]
        );
        let mut e = EdgeScroll::default();
        assert_eq!(
            e.cycle(Some((799, 300)), client, 3, one(), 0, |_| true),
            (0, 0)
        );
        assert_eq!(e.momentum, 0);
    }

    /// spec: S-0101 claim 15 (a stand-still or freeze runs no cycle, so it performs no transition and clears no momentum: scrolling resumes without a new ramp and without a backlog jump)
    #[test]
    fn claim_15_stand_still_keeps_momentum() {
        let client = (800, 600);
        let mut e = EdgeScroll::default();
        for k in 0..3u64 {
            e.cycle(Some((799, 300)), client, 3, one(), 16 * k, |_| false);
        }
        let held = e;
        // No cycle runs for ten seconds; the next one continues the ramp.
        let d = e.cycle(Some((799, 300)), client, 3, one(), 10_000, |_| false);
        assert_eq!(held.momentum, 3);
        assert_eq!(d, (MAGNITUDE[5] as i32, 0));
    }

    /// spec: S-0101 claim 14 (21 pixels per held arrow, no acceleration, perpendicular arrows add, opposed arrows cancel)
    #[test]
    fn claim_14_arrows() {
        assert_eq!(arrows([false, true, false, false]), (21, 0));
        assert_eq!(arrows([true, false, true, false]), (-21, -21));
        assert_eq!(arrows([true, true, false, true]), (0, 21));
    }

    /// spec: S-0101 claim 16, F10 (g = rz(K/(r + 1)), magnitude |trunc₀(rz(e·g))|, K = 1 or 12,
    /// rounding toward zero at each step)
    #[test]
    fn claim_16_drag_magnitude() {
        let client = (800, 600);
        let anchor = (400, 300);
        let req = |dx: i32, r: i32, m: DragMethod| {
            drag_request(anchor, (400 + dx, 300), client, r, m).displacement
        };
        // 12/7 rounds down, so 7 × g falls just short of 12.
        assert_eq!(req(7, 6, DragMethod::Recentre), (11, 0));
        // 12/5 = 2.4 is not a double; rounded down, 5 × g falls short of 12.
        assert_eq!(req(5, 4, DragMethod::Recentre), (11, 0));
        // Method 0: K = 1; 1/3 rounded down, times 3, truncates to 0.
        assert_eq!(req(3, 2, DragMethod::Hold), (0, 0));
        assert_eq!(req(30, 0, DragMethod::Hold), (30, 0));
        // Negative differences keep their sign.
        assert_eq!(req(-7, 6, DragMethod::Recentre), (-11, 0));
        // Diagonals keep their separate full components.
        assert_eq!(
            drag_request(anchor, (410, 290), client, 0, DragMethod::Recentre).displacement,
            (120, -120)
        );
    }

    /// spec: S-0101 claim 16, F10 (method 2 reverses the direction; methods 1 and 2 recentre when
    /// a magnitude is positive, method 0 never)
    #[test]
    fn claim_16_drag_methods() {
        let client = (800, 600);
        let anchor = (400, 300);
        let at = |p: (i32, i32), m| drag_request(anchor, p, client, 0, m);
        let hold = at((420, 300), DragMethod::Hold);
        assert_eq!(hold.displacement, (20, 0));
        assert!(!hold.recentre);
        let one = at((420, 300), DragMethod::Recentre);
        assert_eq!(one.displacement, (240, 0));
        assert!(one.recentre);
        let two = at((420, 300), DragMethod::Reversed);
        assert_eq!(two.displacement, (-240, 0));
        assert!(two.recentre);
        // Nothing requested: no recentring.
        let still = at(anchor, DragMethod::Recentre);
        assert_eq!(still.displacement, (0, 0));
        assert!(!still.recentre);
        // A one-pixel difference at r = 6 still requests 1 (trunc₀(−12/7)) and recentres.
        let small = drag_request(anchor, (400, 299), client, 6, DragMethod::Recentre);
        assert_eq!(small.displacement, (0, -1));
        assert!(small.recentre);
    }

    /// spec: S-0101 F10 (zero differences at the anchor's edges become ∓1; outward components near
    /// the edges get the minimum 5 and the fourfold boost; strict comparisons)
    #[test]
    fn claim_16_drag_edges() {
        let client = (800, 600);
        let d = |a: (i32, i32), p: (i32, i32)| {
            drag_request(a, p, client, 0, DragMethod::Hold).displacement
        };
        // Anchor at column 0, no horizontal difference: −1, boosted to −4 × 5.
        assert_eq!(d((0, 300), (0, 300)), (-20, 0));
        // Anchor at column B − 1: +1, boosted to 4 × 5.
        assert_eq!(d((799, 300), (799, 300)), (20, 0));
        // Anchor at row 0: −1, boosted; at row C − 1 (below C) the zero stays zero.
        assert_eq!(d((400, 0), (400, 0)), (0, -20));
        assert_eq!(d((400, 599), (400, 599)), (0, 0));
        // Anchor at row C (reachable only in principle): +1, boosted.
        assert_eq!(d((400, 600), (400, 600)), (0, 20));
        // Outward differences above 5 keep their size, times 4.
        assert_eq!(d((5, 300), (-3, 300)), (-32, 0));
        // b < 10 is strict: at column 10 no boost.
        assert_eq!(d((10, 300), (8, 300)), (-2, 0));
        assert_eq!(d((9, 300), (7, 300)), (-20, 0));
        // b > N − 10 is strict: column 790 no boost, 791 boosted.
        assert_eq!(d((790, 300), (792, 300)), (2, 0));
        assert_eq!(d((791, 300), (793, 300)), (20, 0));
        // An inward difference near an edge is not boosted.
        assert_eq!(d((5, 300), (8, 300)), (3, 0));
    }
}
