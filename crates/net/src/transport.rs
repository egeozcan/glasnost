//! The seam to the network: a transport moves a peer's messages to every
//! other peer and hands it what arrived. The session itself never touches a
//! socket or a clock; [`pump`] connects the two.
//!
//! N3 brings the native and browser WebSocket transports (through the relay
//! of N2), N7 WebRTC. [`crate::simnet`] is the in-memory one for tests.

use crate::session::Session;

/// A way to reach the other peers of a session.
pub trait Transport {
    /// Sends `bytes` to every other peer of the session.
    fn send(&mut self, now_ms: u64, bytes: &[u8]);
    /// The next message that has arrived by `now_ms`, if any.
    fn recv(&mut self, now_ms: u64) -> Option<Vec<u8>>;
}

/// Hands the session everything that arrived, then sends what it has to
/// send. Call it often (every drawn frame, say), with the host's clock.
pub fn pump<T: Transport + ?Sized>(session: &mut Session, transport: &mut T, now_ms: u64) {
    while let Some(bytes) = transport.recv(now_ms) {
        session.receive(now_ms, &bytes);
    }
    for m in session.poll(now_ms) {
        transport.send(now_ms, &m);
    }
}
