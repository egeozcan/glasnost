//! Redraw pacing of the windows while their surface cannot be drawn.
//!
//! design: render (#99): a hidden window (minimised, fully covered) gives
//! no surface texture at once, and asking for the next redraw straight away
//! spins the event loop. While hidden, a window asks for no redraw; it tries
//! again when it is shown, resized or focused, and otherwise every
//! [`RETRY_MS`] milliseconds.

/// Milliseconds between tries to draw while the window is hidden.
// design: render (#99): our own pacing, a tenth of a second
pub(crate) const RETRY_MS: i64 = 100;

/// Host milliseconds since the program started (pacing only; never game
/// state).
pub(crate) fn now_ms() -> i64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::OnceLock;
        static START: OnceLock<std::time::Instant> = OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_millis() as i64
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.performance())
            .map_or(0, |p| p.now() as i64)
    }
}

/// What a try to draw a frame came to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Drawn {
    /// The frame was presented, or the surface was set up again: draw on.
    Yes,
    /// The surface gave no texture this time (a timeout): try again at once.
    Again,
    /// The window is hidden: no texture until it is shown.
    Hidden,
}

/// Whether a window is hidden, and since when it last tried to draw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Hidden {
    /// The host time of the last try while hidden; `None` while shown.
    tried: Option<i64>,
}

impl Hidden {
    /// Records a try to draw at `now` that came to `drawn`. Returns whether
    /// the next redraw is to be asked for at once.
    pub(crate) fn after(&mut self, drawn: Drawn, now: i64) -> bool {
        match drawn {
            Drawn::Hidden => {
                self.tried = Some(now);
                false
            }
            Drawn::Yes | Drawn::Again => {
                self.tried = None;
                true
            }
        }
    }

    /// The window was shown, resized or focused. Returns whether it was
    /// hidden, so that a redraw is to be asked for.
    pub(crate) fn wake(&mut self) -> bool {
        self.tried.take().is_some()
    }

    /// Whether a hidden window's next try is due at `now`; a due try
    /// restarts the wait, so it is asked for once.
    pub(crate) fn due(&mut self, now: i64) -> bool {
        match self.tried {
            Some(t) if now - t >= RETRY_MS => {
                self.tried = Some(now);
                true
            }
            _ => false,
        }
    }

    /// Milliseconds until a hidden window's next try, `None` while shown.
    pub(crate) fn wait_ms(&self, now: i64) -> Option<u64> {
        self.tried
            .map(|t| u64::try_from(t + RETRY_MS - now).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// design: render (#99): a hidden window asks for no redraw, waits RETRY_MS between tries,
    /// and draws on at once when shown.
    #[test]
    fn a_hidden_window_waits_between_tries() {
        let mut h = Hidden::default();
        assert!(h.after(Drawn::Yes, 0));
        assert_eq!(h.wait_ms(0), None);
        // A timeout tries again at once.
        assert!(h.after(Drawn::Again, 5));
        // Hidden: no redraw now, the next try RETRY_MS later.
        assert!(!h.after(Drawn::Hidden, 10));
        assert_eq!(h.wait_ms(10), Some(100));
        assert_eq!(h.wait_ms(60), Some(50));
        assert!(!h.due(109));
        assert!(h.due(110));
        // Due once: the wait restarts.
        assert!(!h.due(111));
        assert_eq!(h.wait_ms(111), Some(99));
        // A late wake-up waits no more.
        assert_eq!(h.wait_ms(500), Some(0));
        // Shown again: one redraw, then nothing to wake.
        assert!(h.wake());
        assert!(!h.wake());
        assert_eq!(h.wait_ms(500), None);
    }
}
