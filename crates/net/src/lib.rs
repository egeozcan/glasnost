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
//! - [`rejoin`]: the game a live peer sends a rejoining peer (its session's
//!   catch-up and the game's save) and its parts (N5); the drop grace
//!   period and the surrender are in [`session`].
//!
//! Nothing here reads a clock, opens a socket or starts a thread: the
//! caller passes its clock into every call (`now_ms`) and moves the bytes.
//! The relay (N2) and the client's transports (N3) build on this crate. The
//! protocol, the frame rule, the delay rule, the rejoin and the grace
//! period are our own design (docs/development.md, *Multiplayer (M9)*).

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

/// The commit this build was made from: the short hash, `+dirty` when
/// tracked files differed from it, `unknown` outside git (M9 N8b).
pub const COMMIT: &str = env!("GLASNOST_COMMIT");

/// The build id a client names in a room and in the lockstep handshake, and
/// the relay and the tools print for `--version`: the version and the
/// commit, such as `glasnost 0.0.0 (a1b2c3d4e)`. Two builds of one commit
/// agree; a build of another commit, or of a dirty tree, does not.
pub const BUILD: &str = concat!(
    "glasnost ",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GLASNOST_COMMIT"),
    ")"
);

pub use session::{
    Applied, Catchup, Config, Event, Life, Paths, Phase, Session, SlotCatchup, SlotHash, Tick, Via,
};
pub use sim::Simulation;
pub use start::{Field, Params, Slot, SlotKind, StartInfo};
pub use transport::{Transport, pump};
