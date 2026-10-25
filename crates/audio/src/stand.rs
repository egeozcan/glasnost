//! The stand-stills of a loaded skirmish and the two audio clocks that
//! leave them out (S-0701 claim 30, *Clock and passes* and *Music time*).
//!
//! Our audio passes still run on host time (the logical cycle-end clock of
//! claim 30 is a seam), so each clock here is host time with the
//! stand-stills it does not count left out: the sound clock stops in the
//! options menu and without focus, music time only without focus.

/// What a drawn frame of a loaded game is, for its audio.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Stand {
    /// Play goes on (also a network game's options menu and lost focus,
    /// which stand nothing still, S-0201 claims 15 and 16): an audio pass
    /// when one is due, each with its music update.
    #[default]
    Play,
    /// The options menu of a skirmish, or our settings screen opened from a
    /// game, with focus: a stand-still for sound, while music time runs on
    /// and music-only passes run (S-0701 claim 30, S-0703 claim 13).
    Menu,
    /// The window has lost focus in a skirmish, menu open or not: a full
    /// stand-still, for music as for sound (S-0701 claim 30).
    Unfocused,
}

/// A host-time clock that does not count its stand-stills: from its last
/// counted reading to the first reading after it runs again nothing is
/// added, so resuming causes no catch-up.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeldClock {
    /// The host time left out so far.
    off: i64,
    /// The last counted reading.
    last: Option<i64>,
    /// Stopped since the last counted reading.
    stopped: bool,
}

impl HeldClock {
    /// Reads the host clock at `now` (whole milliseconds of a monotonic
    /// clock) while the clock runs; returns the clock's time.
    pub fn run(&mut self, now: i64) -> i64 {
        if let Some(last) = self.last {
            // spec: S-0701 claim 30 (resumption causes no catch-up: the reading at the return becomes the previous reading, so the stretch adds nothing)
            if self.stopped {
                self.off = self.off.saturating_add(now.saturating_sub(last).max(0));
            }
            // A host clock that went back adds nothing.
            if now < last {
                self.off = self.off.saturating_sub(last - now);
            }
        }
        self.stopped = false;
        self.last = Some(now);
        self.time()
    }

    /// The clock stands still: no reading is taken until it runs again.
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    /// The clock's time at its last counted reading (0 before any).
    pub fn time(&self) -> i64 {
        self.last.map_or(0, |l| l.saturating_sub(self.off))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0701 claim 30 (Music time: each later reading adds the whole milliseconds elapsed since the previous one; a stopped stretch adds nothing, the reading at the return becoming the previous reading)
    #[test]
    fn a_stopped_stretch_adds_nothing() {
        let mut c = HeldClock::default();
        assert_eq!(c.time(), 0);
        assert_eq!(c.run(1000), 1000);
        assert_eq!(c.run(1017), 1017);
        c.stop();
        assert_eq!(c.time(), 1017, "stopped: no reading");
        c.stop();
        assert_eq!(c.run(9000), 1017, "the return adds nothing");
        assert_eq!(c.run(9033), 1050);
        // A host clock that goes back adds nothing either.
        assert_eq!(c.run(9020), 1050);
        assert_eq!(c.run(9030), 1060);
    }
}
