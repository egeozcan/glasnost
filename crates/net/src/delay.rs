//! The adaptive delay: round-trip estimates and each peer's proposal.
//!
//! Each peer measures the round trip to every other peer from the echoes of
//! its own stamps (times passed in by the caller) and keeps a smoothed
//! estimate with its variation, in integer milliseconds (the estimator of
//! RFC 6298, scaled by 8 and 4, with faster samples weighing less in the
//! variation). From the largest estimate it works out the
//! delay it needs and writes it into every frame it sends as its proposal.
//! The session's delay is never chosen locally: from tick T + 1 on it is the
//! largest proposal in the frames of tick T, which every peer holds before
//! it runs T, so every peer switches on the same tick.
//!
//! A peer's proposal:
//!
//! 1. *Need* = ⌈RTO / (2 × tick period)⌉ + 1, where RTO = smoothed round
//!    trip + 4 × its variation: the frame for a tick is sent one delay ahead
//!    and must cross one way in that time, with a tick to spare.
//! 2. *Late frames raise it*: if the peer had to wait for a frame since the
//!    last change took effect, it proposes at least the current delay + 1.
//! 3. *A better link lowers it, in steps*: a need below the peer's own last
//!    proposal lowers the proposal half the way to the need (at least one)
//!    only after `calm_ticks` ticks with no wait, no change of the delay and
//!    no lowering of its own; otherwise the peer repeats its last proposal.
//!    It repeats its own proposal, not the session's delay: a peer that held
//!    at the delay would echo another peer's one-off proposal back one delay
//!    later, and the echo would change the delay again and again and never
//!    let it fall.
//! 4. The proposal is clamped to the session's smallest and largest delay.
//!
//! A wait counts as a late frame only when the waiting peer is not ahead of
//! the others in time (`Session::ahead`): a peer that runs ahead always
//! waits, at any delay.

use crate::start::Params;

/// Round-trip estimate to one peer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rtt {
    /// Smoothed round trip × 8, in milliseconds; `None` before the first
    /// sample.
    srtt8: Option<u32>,
    /// Round-trip variation × 4, in milliseconds.
    var4: u32,
    /// Samples taken.
    samples: u32,
}

/// Round trips longer than this are discarded as nonsense (a wrapped or
/// forged stamp).
// nocite: our own netcode
pub const MAX_RTT_MS: u32 = 60_000;

impl Rtt {
    /// Takes one round-trip sample, in milliseconds.
    pub fn sample(&mut self, r: u32) {
        if r > MAX_RTT_MS {
            return;
        }
        self.samples = self.samples.saturating_add(1);
        match self.srtt8 {
            None => {
                // RFC 6298 2.2: SRTT = R, RTTVAR = R / 2.
                self.srtt8 = Some(r << 3); // nocite: our own netcode (×8 scaling)
                self.var4 = r << 1; // nocite: our own netcode (R/2 × 4)
            }
            Some(s8) => {
                // RFC 6298 2.3 with alpha = 1/8 and beta = 1/4, in the
                // scaled integer form, except that a sample below the
                // estimate moves the variation an eighth as much: a link
                // that gets faster is no less steady, and the full weight
                // would raise the RTO, and the delay, just as the round
                // trip falls.
                let srtt = s8 >> 3; // nocite: our own netcode (×8 scaling)
                let err = i64::from(r) - i64::from(srtt);
                let s8 = (i64::from(s8) + err).max(0);
                let dev = if err < 0 { -err >> 3 } else { err }; // nocite: our own netcode (an eighth for faster samples)
                let var4 = i64::from(self.var4);
                let var4 = (var4 + dev - (var4 >> 2)).max(0); // nocite: our own netcode (beta = 1/4)
                self.srtt8 = Some(s8.min(i64::from(u32::MAX)) as u32);
                self.var4 = var4.min(i64::from(u32::MAX)) as u32;
            }
        }
    }

    /// The smoothed round trip, in milliseconds.
    pub fn smoothed(&self) -> Option<u32> {
        self.srtt8.map(|s| s >> 3) // nocite: our own netcode (×8 scaling)
    }

    /// Smoothed round trip plus four times its variation, in milliseconds.
    pub fn rto(&self) -> Option<u32> {
        self.srtt8.map(|s| (s >> 3).saturating_add(self.var4)) // nocite: our own netcode (×8 scaling; var4 is 4 × the variation)
    }

    /// How many samples were taken.
    pub fn samples(&self) -> u32 {
        self.samples
    }
}

/// The delay a round trip needs: ⌈rto / (2 × tick_ms)⌉ + 1.
pub fn need(rto_ms: u32, tick_ms: u32) -> u32 {
    let two_ticks = tick_ms.max(1).saturating_mul(2); // nocite: our own netcode (one way is half the round trip)
    rto_ms.div_ceil(two_ticks).saturating_add(1)
}

/// One peer's delay state: when the delay last changed, when this peer
/// last had to wait for a frame, and its own last proposal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Control {
    /// The tick whose frames last changed the delay.
    pub last_change: u32,
    /// The last tick this peer had to wait for, if any.
    pub last_wait: Option<u32>,
    /// The tick this peer last lowered its proposal.
    pub last_lower: u32,
    /// Its last proposal, if it made one.
    pub held: Option<u8>,
}

impl Control {
    /// Notes that the peer had to wait for a frame of tick `tick`.
    pub fn waited(&mut self, tick: u32) {
        self.last_wait = Some(tick);
    }

    /// Notes that the frames of tick `tick` changed the delay.
    pub fn changed(&mut self, tick: u32) {
        self.last_change = tick;
    }

    /// The next proposal, with `ran` ticks run, the session's delay
    /// `current` and the largest RTO to the other peers (`None` before any
    /// sample); it becomes the peer's last proposal.
    pub fn propose(
        &mut self,
        ran: u32,
        current: u8,
        rto_ms: Option<u32>,
        tick_ms: u32,
        calm_ticks: u32,
        params: &Params,
    ) -> u8 {
        let current32 = u32::from(current);
        let held = self.held.map_or(current32, u32::from);
        let mut p = rto_ms.map_or(held, |r| need(r, tick_ms));
        // A change takes a delay's worth of ticks to show in the frames
        // that arrive; a wait before that says nothing about the new delay.
        let effective = self.last_change.saturating_add(current32);
        let waited_since = self.last_wait.is_some_and(|w| w > effective);
        if waited_since {
            p = p.max(current32 + 1);
        }
        if p < held {
            let calm_since = self
                .last_change
                .max(self.last_lower)
                .max(self.last_wait.unwrap_or(0));
            p = if ran >= calm_since.saturating_add(calm_ticks) {
                self.last_lower = ran;
                // Half the way down to the need, at least one step.
                held - ((held - p) / 2).max(1) // nocite: our own netcode (halve the gap)
            } else {
                held
            };
        }
        let p = p.clamp(u32::from(params.min_delay), u32::from(params.max_delay)) as u8;
        self.held = Some(p);
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtt_follows_samples() {
        let mut r = Rtt::default();
        assert_eq!(r.rto(), None);
        r.sample(100);
        assert_eq!(r.smoothed(), Some(100));
        assert_eq!(r.rto(), Some(300));
        for _ in 0..200 {
            r.sample(40);
        }
        assert_eq!(r.smoothed(), Some(40));
        assert!(r.rto().unwrap() <= 45, "variation decays: {:?}", r.rto());
        // A sudden drop barely moves the RTO up; a sudden rise does.
        let mut slow = Rtt::default();
        for _ in 0..50 {
            slow.sample(500);
        }
        let before = slow.rto().unwrap();
        slow.sample(10);
        assert!(
            slow.rto().unwrap() <= before + 60,
            "{before} -> {:?}",
            slow.rto()
        );
        let mut fast = Rtt::default();
        for _ in 0..50 {
            fast.sample(10);
        }
        fast.sample(500);
        assert!(fast.rto().unwrap() >= 450, "{:?}", fast.rto());
        r.sample(MAX_RTT_MS + 1);
        assert_eq!(r.samples(), 201, "nonsense samples are dropped");
    }

    #[test]
    fn need_is_one_way_plus_a_tick() {
        assert_eq!(need(0, 33), 1);
        assert_eq!(need(66, 33), 2);
        assert_eq!(need(67, 33), 3);
        assert_eq!(need(400, 33), 8);
    }

    #[test]
    fn proposal_rises_on_waits_and_falls_after_calm() {
        let params = Params {
            initial_delay: 3,
            min_delay: 1,
            max_delay: 10,
            hash_every: 1,
        };
        let mut c = Control::default();
        // No sample: keep the delay.
        assert_eq!(c.propose(5, 3, None, 33, 20, &params), 3);
        // A wait right after a change does not count yet.
        c.changed(10);
        c.waited(12);
        assert_eq!(c.propose(12, 3, Some(30), 33, 20, &params), 3);
        // A wait after the change took effect raises by one.
        c.waited(14);
        assert_eq!(c.propose(14, 3, Some(30), 33, 20, &params), 4);
        // A large need raises at once, clamped to the largest.
        assert_eq!(c.propose(14, 3, Some(600), 33, 20, &params), 10);
        // The raise to 10 is agreed at tick 15; a low need lowers nothing
        // before a calm stretch, and the peer repeats its own proposal even
        // when the session's delay is another peer's.
        c.changed(15);
        assert_eq!(c.propose(30, 10, Some(30), 33, 20, &params), 10);
        assert_eq!(c.propose(31, 7, Some(30), 33, 20, &params), 10);
        // After the calm stretch, half the way down to the need (1).
        assert_eq!(c.propose(36, 10, Some(30), 33, 20, &params), 6);
        // The lowering starts a new calm stretch.
        assert_eq!(c.propose(37, 10, Some(30), 33, 20, &params), 6);
        c.changed(46);
        assert_eq!(c.propose(60, 6, Some(30), 33, 20, &params), 6);
        assert_eq!(c.propose(66, 6, Some(30), 33, 20, &params), 4);
        // At least one step, down to the need.
        c.changed(70);
        assert_eq!(c.propose(90, 4, Some(66), 33, 20, &params), 3);
        c.changed(94);
        assert_eq!(c.propose(114, 3, Some(66), 33, 20, &params), 2);
        // A new wait raises again, above the session's delay.
        c.waited(120);
        assert_eq!(c.propose(121, 2, Some(30), 33, 20, &params), 3);
        // Clamped to the smallest.
        let mut calm = Control::default();
        assert_eq!(calm.propose(100, 1, Some(0), 33, 20, &params), 1);
    }
}
