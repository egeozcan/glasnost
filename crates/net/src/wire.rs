//! The wire format: what one peer's message holds and how it is written.
//!
//! A message is a four-byte header, then the body in postcard (the
//! workspace's serializer, also used by saves and replays):
//!
//! | Bytes | Value |
//! |---|---|
//! | 0, 1 | the magic `G`, `N` |
//! | 2 | [`WIRE_VERSION`] |
//! | 3 | the sender's slot, or [`RELAY`] for the relay's own messages |
//! | 4 … | the [`Body`] in postcard |
//!
//! The header is read before the body, so a peer of another wire version is
//! refused by name ([`WireError::Version`]) even when its body cannot be
//! read. Every message is sent to every other peer of the session (the
//! relay of N2 forwards it, a mesh transport copies it to each link).
//!
//! Only the relay speaks as [`RELAY`], and only the relay sends
//! [`Body::Dropped`]: it checks the sender byte of every message it forwards
//! against the sender's slot and never forwards a `Dropped` from a client,
//! and a session takes a `Dropped` only from [`RELAY`] (N2). Version 2 added
//! that rule and the lobby's messages ([`crate::lobby::wire`]), which share
//! this version; version 3 the lobby's chat (N3); version 4 the desync
//! report's digest in [`Frames::report`] (N4).
//!
//! This is our own protocol (docs/development.md, *Multiplayer (M9)*); it is
//! not compatible with the original game's clients.

use glasnost_sim::orders::Order;
use serde::{Deserialize, Serialize};

use crate::start::StartInfo;

/// The first two bytes of every message.
// nocite: our own netcode
pub const MAGIC: [u8; 2] = *b"GN";

/// The wire format's version. Bump it whenever a message's layout changes.
// nocite: our own netcode
pub const WIRE_VERSION: u8 = 4;

/// The sender byte of the relay's own messages. No slot table has this many
/// slots (a game has at most eight players), so no peer has this slot.
// nocite: our own netcode
pub const RELAY: u8 = 255;

/// Length of the header before the body.
// nocite: our own netcode (magic, version, slot)
pub const HEADER_LEN: usize = 4;

/// A state hash: the 32 bytes of the SHA-256 digest of the game state.
pub type Hash = [u8; 32];

/// One message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    /// The sender's slot.
    pub from: u8,
    /// What it says.
    pub body: Body,
}

/// The kinds of message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Body {
    /// The session start values (the handshake). Sent until every other
    /// peer is seen running.
    Start(StartInfo),
    /// Frames, acknowledgements, round-trip echoes and state hashes.
    Frames(Frames),
    /// A slot has left the session: its frames from number `kept` on are
    /// empty and no longer awaited. Only the relay sends it, as [`RELAY`],
    /// after the last frame of that slot it forwarded to every other peer;
    /// N5 adds the surrender order.
    Dropped {
        /// The slot that left.
        slot: u8,
        /// How many of its frames (from frame 0) count.
        kept: u32,
    },
}

/// The lockstep traffic of one peer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frames {
    /// The sender's clock when it sent this message, in milliseconds
    /// (wrapping). Only the sender reads it back, from an [`Echo`].
    pub stamp: u32,
    /// How many ticks the sender has run.
    pub ran: u32,
    /// For each other peer the sender has heard from: that peer's latest
    /// stamp and how long the sender held it.
    pub echoes: Vec<Echo>,
    /// Per slot of the slot table: how many of that slot's frames the
    /// sender holds, counted from frame 0 without a gap.
    pub acks: Vec<u32>,
    /// Per slot of the slot table: how many of that slot's state-hash
    /// checkpoints the sender holds, counted from checkpoint 0 without a
    /// gap.
    pub hash_acks: Vec<u32>,
    /// The number of the first frame in `frames`.
    pub first: u32,
    /// The sender's frames `first`, `first + 1`, … in order.
    pub frames: Vec<Frame>,
    /// The number of the first checkpoint in `hashes`.
    pub first_hash: u32,
    /// The sender's state hashes at checkpoints `first_hash`, … in order.
    pub hashes: Vec<Hash>,
    /// The sender's desync report, once it has written one (N4): sent in
    /// every later message, so it arrives over a lossy link too.
    pub report: Option<Report>,
}

/// A peer's desync report as the others hear of it: the tick of the
/// mismatch it reports and the SHA-256 of its bundle's bytes
/// ([`crate::desync::Bundle::digest`]). The bundle itself never crosses the
/// network: it is far larger than a message may be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// The mismatch's tick (the last tick the differing hashes cover).
    pub tick: u32,
    /// The digest of the bundle's bytes.
    pub digest: Hash,
}

/// A round-trip echo: the stamp of another peer's message, given back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Echo {
    /// The peer whose stamp this is.
    pub slot: u8,
    /// Its stamp.
    pub stamp: u32,
    /// Milliseconds between the echoing peer receiving the stamp and
    /// sending this echo.
    pub hold: u32,
}

/// One peer's input for one tick.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    /// The sender's delay proposal (see `delay`); the session's delay from
    /// the next tick on is the largest proposal in the tick's frames.
    pub delay: u8,
    /// The person's orders for this tick, in the order given.
    pub orders: Vec<Order>,
}

/// Why bytes are not a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WireError {
    /// Shorter than the header.
    Short,
    /// Not our protocol.
    Magic,
    /// Our protocol in another version.
    Version {
        /// The sender's slot, from the header.
        from: u8,
        /// The version it speaks.
        version: u8,
    },
    /// The body cannot be read.
    Body(String),
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::Short => write!(f, "message shorter than its header"),
            WireError::Magic => write!(f, "not a Glasnost message"),
            WireError::Version { from, version } => write!(
                f,
                "slot {from} speaks wire version {version}, we speak {WIRE_VERSION}"
            ),
            WireError::Body(e) => write!(f, "unreadable message body: {e}"),
        }
    }
}

/// Writes a message.
pub fn encode(env: &Envelope) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + 64); // nocite: our own netcode (a first guess at a message's size)
    out.extend_from_slice(&MAGIC);
    out.push(WIRE_VERSION);
    out.push(env.from);
    postcard::to_extend(&env.body, out).expect("a message body serializes")
}

/// Reads a message. Never panics on hostile bytes.
pub fn decode(bytes: &[u8]) -> Result<Envelope, WireError> {
    if bytes.len() < HEADER_LEN {
        return Err(WireError::Short);
    }
    if bytes[..2] != MAGIC {
        return Err(WireError::Magic);
    }
    let from = bytes[3];
    if bytes[2] != WIRE_VERSION {
        return Err(WireError::Version {
            from,
            version: bytes[2],
        });
    }
    let (body, rest): (Body, &[u8]) = postcard::take_from_bytes(&bytes[HEADER_LEN..])
        .map_err(|e| WireError::Body(e.to_string()))?;
    if !rest.is_empty() {
        return Err(WireError::Body(format!("{} trailing bytes", rest.len())));
    }
    Ok(Envelope { from, body })
}
