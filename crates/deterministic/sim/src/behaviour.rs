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
    /// The elapsed-tick count (S-0609 Terms): grows by 1 every tick just
    /// before the behaviour may be processed and returns to 0 at every
    /// behaviour start; only gattling buildings read it.
    #[serde(default)]
    pub elapsed: i32,
    /// A gate's door and close countdown (S-0215 claim 35); unused by
    /// every other object.
    pub gate: Gate,
}

/// A gate's door (S-0215 claim 35): at rest closed, at rest open, opening
/// or closing. A moving door reaches rest at the gate's update on the first
/// tick on which `ticks` ticks have passed since `since` (claim 35 (d)).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Door {
    /// At rest closed; every gate starts so (S-0215 claim 35 (a)).
    #[default]
    Closed,
    /// At rest open.
    Open,
    /// Opening.
    Opening {
        /// Tick on which the motion began.
        since: i32,
        /// Ticks the motion takes (0 or more).
        ticks: i32,
    },
    /// Closing.
    Closing {
        /// Tick on which the motion began.
        since: i32,
        /// Ticks the motion takes (0 or more).
        ticks: i32,
    },
}

/// A gate's state besides its behaviour slots (S-0215 claim 35).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gate {
    /// The door.
    pub door: Door,
    /// Tick of the close countdown's last restart.
    pub countdown: i32,
}

impl Behaviours {
    /// The behaviour a rule sees when it asks for the actor's behaviour
    /// (S-0215 claim 1): the current one, or the queued one while none is
    /// current. Seeing it neither starts the queued behaviour nor fills the
    /// current slot; processing and the slot mechanics read the raw slots.
    pub fn seen(&self) -> Option<Behaviour> {
        // spec: S-0215 claim 1 (with no current behaviour, rules that ask for its behaviour see the queued one)
        self.current.or(self.queued)
    }

    /// The behaviour start of S-0215 claim 3: timer expired now, start tick
    /// now, progress count 0.
    fn start(&mut self, tick: i32) {
        // spec: S-0215 claim 3 (a behaviour start sets the timer to "expired now")
        self.expired = true;
        self.start_tick = tick;
        self.progress = 0;
        self.allowance = false;
        // spec: S-0609 Terms (the elapsed-tick count returns to 0 whenever a behaviour starts, queued and started or forced; an override does not reset it)
        self.elapsed = 0;
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

    /// What an effective start-next or force does to the suspended slot.
    /// `stale` is `quirks.stale_suspended_revert`: on (the original), the
    /// slot and the put-aside target and destination stay as they are; off,
    /// both are emptied, so a later revert finds nothing (S-0215 Edge cases).
    fn on_start(&mut self, stale: bool) {
        // spec: S-0215 claims 8 and 9, Edge cases (quirks.stale_suspended_revert)
        if !stale {
            self.suspended = None;
            self.aside = None;
        }
    }

    /// Start-next (S-0215 claim 8). Returns whether a behaviour started.
    /// `stale` is `quirks.stale_suspended_revert`.
    pub fn start_next(&mut self, tick: i32, stale: bool) -> bool {
        match self.queued.take() {
            Some(b) => {
                // spec: S-0215 claim 8 (the queued behaviour becomes current; suspended untouched, quirks.stale_suspended_revert)
                self.current = Some(b);
                self.start(tick);
                self.on_start(stale);
                true
            }
            None => false,
        }
    }

    /// A queue request with the readiness-dependent immediate start (S-0215
    /// claim 7).
    pub fn queue_and_start(&mut self, b: Option<Behaviour>, ready: bool, tick: i32, stale: bool) {
        if self.queue(b) && ready {
            self.start_next(tick, stale);
        }
    }

    /// Queues and starts at once, whether or not ready (S-0215 claim 7).
    pub fn queue_and_start_at_once(&mut self, b: Behaviour, tick: i32, stale: bool) {
        if self.queue(Some(b)) {
            self.start_next(tick, stale);
        }
    }

    /// Force (S-0215 claim 9). `stale` is `quirks.stale_suspended_revert`.
    pub fn force(&mut self, b: Behaviour, tick: i32, stale: bool) {
        // spec: S-0215 claim 9 (wait refuses a forced guard; selling does not refuse)
        if self.current == Some(Behaviour::Wait) && b == Behaviour::Guard {
            return;
        }
        self.current = Some(b);
        self.queued = None;
        self.start(tick);
        self.on_start(stale);
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

    // spec: S-0215 claim 1 (with no current behaviour the queued one is seen; seeing it fills no slot)
    #[test]
    fn seen_is_current_else_queued() {
        let mut s = Behaviours {
            queued: Some(B::Guard),
            ..Default::default()
        };
        assert_eq!(s.seen(), Some(B::Guard));
        assert_eq!((s.current, s.queued), (None, Some(B::Guard)));
        s.current = Some(B::Attack);
        assert_eq!(s.seen(), Some(B::Attack));
        s.current = None;
        s.queued = None;
        assert_eq!(s.seen(), None);
    }

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
        assert!(s.start_next(10, true));
        assert_eq!(
            (s.current, s.queued, s.suspended),
            (Some(B::Move), None, Some(B::Hunt))
        );
        assert!(s.expired && s.start_tick == 10 && s.progress == 0);
        s.force(B::Selling, 11, true);
        s.force(B::Guard, 12, true);
        assert_eq!((s.current, s.suspended), (Some(B::Guard), Some(B::Hunt)));
        let mut w = Behaviours {
            current: Some(B::Wait),
            ..Default::default()
        };
        w.force(B::Guard, 1, true);
        assert_eq!(w.current, Some(B::Wait));
    }

    // spec: S-0215 Edge cases (quirks.stale_suspended_revert off: an effective start-next or force empties the suspended slot and the put-aside pair; a refused force or an empty start-next does nothing)
    #[test]
    fn stale_suspended_revert_off() {
        let base = Behaviours {
            current: Some(B::Attack),
            queued: Some(B::Move),
            suspended: Some(B::Guard),
            aside: Some((None, None)),
            ..Default::default()
        };
        let mut s = base.clone();
        assert!(s.start_next(3, false));
        assert_eq!((s.suspended, s.aside), (None, None));
        assert!(s.revert().is_none());
        let mut s = base.clone();
        s.force(B::Hunt, 3, false);
        assert_eq!(
            (s.current, s.suspended, s.aside),
            (Some(B::Hunt), None, None)
        );
        // Nothing queued: start-next does nothing.
        let mut s = Behaviours {
            queued: None,
            ..base.clone()
        };
        assert!(!s.start_next(3, false));
        assert_eq!(s.suspended, Some(B::Guard));
        // A refused force (wait refusing guard) does nothing.
        let mut s = Behaviours {
            current: Some(B::Wait),
            ..base
        };
        s.force(B::Guard, 3, false);
        assert_eq!(s.suspended, Some(B::Guard));
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
