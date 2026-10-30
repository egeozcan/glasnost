//! The network pump (fixed after the M9 exit run): a network game keeps
//! its link and its lockstep going while its window draws no frames.
//!
//! The window updates the client once per drawn frame. A window that is
//! covered or minimised (KDE on Wayland stops its redraws under a
//! screenshot overlay) or a background browser tab (no animation frames)
//! then updated it never: no pings, no acknowledgements and no frames went
//! to the relay, which closes a connection silent for 15 s, and the house
//! surrendered after the grace period. So while a network link is open the
//! window also wakes on a timer ([`Pump::wait_ms`]), and when no frame has
//! updated the client for [`STALE_MS`] it updates the client itself
//! ([`Pump::between_frames`]): the same [`Client::update`] a frame runs,
//! without the drawing. While frames come the pump does nothing, so no
//! per-frame work runs twice.
//!
//! The update runs the lockstep's ticks too. A peer makes its frames as it
//! runs ticks, so a peer that stopped ticking would stop the game for
//! everyone, and in a network game neither the options menu nor lost focus
//! stops the ticks (S-0201 claims 15 and 16); a window nobody sees is the
//! same case. The pacer still runs at most one tick per update with no
//! catch-up (S-0201 claim 17), so a browser that throttles a background
//! tab's timers to one a second runs one tick a second there, and the
//! others wait for it; the connection lives on either way.
//!
//! Without a network link the pump never wakes the window, so a
//! single-player window that draws no frames stands still as before.
//! Our own netcode; no game rule is involved.

use super::client::Client;

/// The window has drawn no frame for this long: it is covered, minimised
/// or a background tab, and the pump updates the client itself.
// nocite: our own netcode (several frames at any display rate the game is played at, far inside the relay's 15 s silence limit)
pub const STALE_MS: i64 = 200;

/// When the window last drew a frame, for the network pump.
#[derive(Clone, Debug, Default)]
pub struct Pump {
    drawn: Option<i64>,
}

impl Pump {
    /// The window drew a frame (and updated the client) at host time
    /// `now_ms`.
    pub fn drew(&mut self, now_ms: i64) {
        self.drawn = Some(now_ms);
    }

    /// Whether the window has drawn no frame for [`STALE_MS`] at `now_ms`.
    fn stale(&self, now_ms: i64) -> bool {
        self.drawn.is_none_or(|t| now_ms - t >= STALE_MS)
    }

    /// Between frames, at host time `now_ms`: updates `client` when it has
    /// a network link and no frame has updated it for [`STALE_MS`].
    /// Whether it did.
    pub fn between_frames(&mut self, client: &mut Client, now_ms: i64) -> bool {
        if !client.networked() || !self.stale(now_ms) {
            return false;
        }
        client.update(now_ms);
        true
    }

    /// How long the window may sleep at `now_ms` before the pump must look
    /// again, in milliseconds; `None` (wait for events only) without a
    /// network link. While frames come, until they would be stale.
    pub fn wait_ms(&self, client: &Client, now_ms: i64) -> Option<u64> {
        let wait = client.net_wait_ms(now_ms)?;
        Some(match self.drawn {
            Some(t) if !self.stale(now_ms) => (t + STALE_MS - now_ms).max(1) as u64,
            _ => wait,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_a_network_link_the_pump_never_wakes_nor_updates() {
        let mut c = Client::new();
        let mut p = Pump::default();
        assert_eq!(p.wait_ms(&c, 0), None);
        assert!(!p.between_frames(&mut c, 0));
        assert!(!p.between_frames(&mut c, 10_000));
        p.drew(10_000);
        assert_eq!(p.wait_ms(&c, 20_000), None);
        assert!(!p.between_frames(&mut c, 20_000));
    }
}
