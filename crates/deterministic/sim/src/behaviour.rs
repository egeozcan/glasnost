//! Behaviours (standing orders): slots, queueing, starting, forcing,
//! overriding and reverting, and the processing timer (S-0215 claims 1 to
//! 3, 7 to 10).

use serde::{Deserialize, Serialize};

use crate::rules::Behaviour;
use crate::world::Target;

/// An actor's behaviour slots and processing state (S-0215 claims 1 to 3).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Behaviours {
    /// Current behaviour.
    pub current: Option<Behaviour>,
    /// Queued behaviour.
    pub queued: Option<Behaviour>,
    /// Suspended behaviour.
    pub suspended: Option<Behaviour>,
    /// Target and destination put aside by an override (S-0215 claim 10).
    pub aside: Option<(Option<Target>, Option<Target>)>,
    /// Tick of the last processing and the delay it chose.
    pub last: Option<(i32, i32)>,
    /// The timer is "expired now" (a behaviour start, S-0215 claim 3).
    pub expired: bool,
    /// Start tick of the current behaviour.
    pub start_tick: i32,
    /// Progress count (S-0215 Terms).
    pub progress: i32,
    /// A building's ready mark (S-0215 Terms).
    pub ready_mark: bool,
    /// The allowance to interrupt movement of a queued *guard* (S-0215
    /// claim 11).
    pub allowance: bool,
    /// Tick of the last fall-back to rest's common effects (S-0215 claim 13).
    pub last_common_effects: Option<i32>,
}

impl Behaviours {
    /// The behaviour start of S-0215 claim 3: timer expired now, start tick
    /// now, progress count 0.
    fn start(&mut self, tick: i32) {
        // spec: S-0215 claim 3 (a behaviour start sets the timer to "expired now")
        self.expired = true;
        self.start_tick = tick;
        self.progress = 0;
        self.allowance = false;
    }

    /// A queue request (S-0215 claim 7). Returns whether the request was not
    /// ignored outright (the first row).
    pub fn queue(&mut self, b: Option<Behaviour>) -> bool {
        // spec: S-0215 claim 7 (selling, or wait with guard: the request is ignored entirely)
        if self.current == Some(Behaviour::Selling)
            || (self.current == Some(Behaviour::Wait) && b == Some(Behaviour::Guard))
        {
            return false;
        }
        match b {
            // spec: S-0215 claim 7 (b is none: queue unchanged)
            None => {}
            // spec: S-0215 claim 7 (b is current and nothing or b queued: unchanged)
            Some(x)
                if Some(x) == self.current && (self.queued.is_none() || self.queued == Some(x)) => {
            }
            // spec: S-0215 claim 7 (any other case: b replaces any queued behaviour)
            Some(x) => {
                self.queued = Some(x);
                self.allowance = false;
            }
        }
        true
    }

    /// Start-next (S-0215 claim 8). Returns whether a behaviour started.
    pub fn start_next(&mut self, tick: i32) -> bool {
        match self.queued.take() {
            Some(b) => {
                // spec: S-0215 claim 8 (the queued behaviour becomes current; suspended untouched)
                self.current = Some(b);
                self.start(tick);
                true
            }
            None => false,
        }
    }

    /// A queue request with the readiness-dependent immediate start (S-0215
    /// claim 7).
    pub fn queue_and_start(&mut self, b: Option<Behaviour>, ready: bool, tick: i32) {
        if self.queue(b) && ready {
            self.start_next(tick);
        }
    }

    /// Queues and starts at once, whether or not ready (S-0215 claim 7).
    pub fn queue_and_start_at_once(&mut self, b: Behaviour, tick: i32) {
        if self.queue(Some(b)) {
            self.start_next(tick);
        }
    }

    /// Force (S-0215 claim 9).
    pub fn force(&mut self, b: Behaviour, tick: i32) {
        // spec: S-0215 claim 9 (wait refuses a forced guard; selling does not refuse)
        if self.current == Some(Behaviour::Wait) && b == Behaviour::Guard {
            return;
        }
        self.current = Some(b);
        self.queued = None;
        self.start(tick);
    }

    /// Override (S-0215 claim 10): returns the target and destination to put
    /// aside and replace (the caller swaps them in).
    pub fn override_with(
        &mut self,
        b: Behaviour,
        refused_retargets: bool,
        suspends_queued: bool,
    ) -> bool {
        let refused = self.current == Some(Behaviour::Selling)
            || (self.current == Some(Behaviour::Wait) && b == Behaviour::Guard);
        if refused {
            // spec: S-0215 claim 10 (refused: no slot change; target and destination replaced all the same)
            return refused_retargets;
        }
        // spec: S-0215 claim 10 (the suspended behaviour becomes the queued one if any, else the current one)
        self.suspended = if suspends_queued && self.queued.is_some() {
            self.queued
        } else {
            self.current
        };
        self.current = Some(b);
        self.allowance = false;
        true
    }

    /// Revert (S-0215 claim 10). Returns the target and destination to bring
    /// back, when a behaviour was suspended.
    pub fn revert(&mut self) -> Option<(Option<Target>, Option<Target>)> {
        let s = self.suspended.take()?;
        // spec: S-0215 claim 10 (the suspended behaviour becomes current; target and destination restored)
        self.current = Some(s);
        self.allowance = false;
        Some(self.aside.take().unwrap_or((None, None)))
    }

    /// Whether the processing timer has expired at tick `t` (S-0215 claim 2).
    pub fn due(&self, t: i32) -> bool {
        if self.expired {
            return true;
        }
        match self.last {
            None => true,
            // spec: S-0215 claim 2 (a second turn in the same tick processes again only after a delay of 0 or less)
            Some((at, d)) if t == at => d <= 0,
            // spec: S-0215 claim 2 (processed again on the first tick t′ > t with t′ − t ≥ d)
            Some((at, d)) => t.wrapping_sub(at) >= d,
        }
    }

    /// Records a processing at tick `t` that chose delay `d` (S-0215 claim 2).
    pub fn processed(&mut self, t: i32, d: i32) {
        self.last = Some((t, d));
        self.expired = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Behaviour as B;

    // spec: S-0215 claim 7 (queue rows: ignored under selling or wait/guard; current again keeps; replace)
    #[test]
    fn queue_rows() {
        let mut s = Behaviours {
            current: Some(B::Guard),
            ..Default::default()
        };
        s.queue(Some(B::Guard));
        assert_eq!(s.queued, None);
        s.queue(Some(B::Move));
        assert_eq!(s.queued, Some(B::Move));
        // Queueing the current behaviour while another is queued replaces it.
        s.queue(Some(B::Guard));
        assert_eq!(s.queued, Some(B::Guard));
        let mut w = Behaviours {
            current: Some(B::Wait),
            ..Default::default()
        };
        assert!(!w.queue(Some(B::Guard)));
        assert_eq!(w.queued, None);
    }

    // spec: S-0215 claims 8 and 9 (start-next and force reset the timer, start tick and progress; suspended untouched)
    #[test]
    fn start_and_force() {
        let mut s = Behaviours {
            current: Some(B::Guard),
            queued: Some(B::Move),
            suspended: Some(B::Hunt),
            progress: 5,
            ..Default::default()
        };
        assert!(s.start_next(10));
        assert_eq!(
            (s.current, s.queued, s.suspended),
            (Some(B::Move), None, Some(B::Hunt))
        );
        assert!(s.expired && s.start_tick == 10 && s.progress == 0);
        s.force(B::Selling, 11);
        s.force(B::Guard, 12);
        assert_eq!(s.current, Some(B::Guard));
        let mut w = Behaviours {
            current: Some(B::Wait),
            ..Default::default()
        };
        w.force(B::Guard, 1);
        assert_eq!(w.current, Some(B::Wait));
    }

    // spec: S-0215 claim 10 (override suspends the queued behaviour when one is queued; revert restores)
    #[test]
    fn override_and_revert() {
        let mut s = Behaviours {
            current: Some(B::Guard),
            queued: Some(B::Move),
            ..Default::default()
        };
        assert!(s.override_with(B::Attack, true, true));
        assert_eq!(
            (s.current, s.queued, s.suspended),
            (Some(B::Attack), Some(B::Move), Some(B::Move))
        );
        s.aside = Some((None, None));
        assert!(s.revert().is_some());
        assert_eq!((s.current, s.suspended), (Some(B::Move), None));
        // With the switch off the current behaviour is suspended.
        let mut t = Behaviours {
            current: Some(B::Guard),
            queued: Some(B::Move),
            ..Default::default()
        };
        t.override_with(B::Attack, true, false);
        assert_eq!(t.suspended, Some(B::Guard));
    }

    // spec: S-0215 claim 2 (a delay of 1 or less means again on the next tick)
    #[test]
    fn timer() {
        let mut s = Behaviours::default();
        assert!(s.due(5));
        s.processed(5, 14);
        assert!(!s.due(18));
        assert!(s.due(19));
        s.processed(19, 0);
        assert!(s.due(20));
        s.processed(20, 2);
        assert!(!s.due(21));
        assert!(s.due(22));
    }

    // spec: S-0215 claim 2 (a second turn in the same tick processes again only after a delay of 0 or less)
    #[test]
    fn second_turn() {
        let mut s = Behaviours::default();
        s.processed(7, 0);
        assert!(s.due(7));
        s.processed(7, -3);
        assert!(s.due(7));
        s.processed(7, 1);
        assert!(!s.due(7));
        assert!(s.due(8));
    }
}
