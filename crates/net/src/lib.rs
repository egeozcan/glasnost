//! Lockstep networking (milestone M9): the protocol and the lockstep session,
//! with no I/O.
//!
//! Peers exchange only the persons' orders, their delay proposals and state
//! hashes, never rules data or game state. Every peer runs the same game
//! from the same start values; our AI plays the computer slots on every
//! peer inside the tick, so its orders never cross the network.
//!
//! - [`start`]: the session start values and the refusal of a mismatch.
//! - [`wire`]: the versioned message format.
//! - [`session`]: one peer's lockstep state machine (frames, the tick rule,
//!   the agreed delay, state-hash checkpoints, events).
//! - [`delay`]: round-trip estimates and the delay proposal.
//! - [`sim`]: what the session drives ([`sim::Simulation`], implemented for
//!   [`glasnost_ai::game::Game`]).
//! - [`transport`]: the seam to a network ([`transport::Transport`],
//!   [`transport::pump`]).
//! - [`simnet`]: a seeded in-memory network for tests.
//! - [`lobby`]: the network lobby's protocol and room state machine (N2),
//!   which the relay keeps and the client drives.
//! - [`desync`]: the evidence every peer keeps and the desync report it
//!   makes on a mismatch (N4).
//!
//! Nothing here reads a clock, opens a socket or starts a thread: the
//! caller passes its clock into every call (`now_ms`) and moves the bytes.
//! The relay (N2), the client's transports (N3) and reconnects (N5) build on
//! this crate. The protocol, the frame rule and the
//! delay rule are our own design (docs/development.md, *Multiplayer (M9)*).

pub mod delay;
pub mod desync;
pub mod lobby;
pub mod rejoin;
pub mod session;
pub mod sim;
pub mod simnet;
pub mod start;
pub mod transport;
pub mod wire;

pub use session::{
    Applied, Catchup, Config, Event, Life, Phase, Session, SlotCatchup, SlotHash, Tick,
};
pub use sim::Simulation;
pub use start::{Field, Params, Slot, SlotKind, StartInfo};
pub use transport::{Transport, pump};
