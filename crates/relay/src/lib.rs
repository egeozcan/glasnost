//! Glasnost's relay and lobby server (M9 step N2).
//!
//! Players connect over WebSocket, make or join a room by its code, agree
//! on the game setup in the room, and start; the relay then forwards each
//! person's lockstep messages to the others and reports a person whose
//! connection is lost as dropped. The relay never runs the game: it holds
//! no simulation state and decides nothing about the game but who is in it.
//!
//! [`hub`] is the core, with no I/O; [`server`] is the tokio shell around
//! it. The protocol types live in `glasnost-net` ([`glasnost_net::lobby`]
//! and [`glasnost_net::wire`]) so that clients share them. This crate is
//! native only: neither the simulation crates nor `glasnost-net` reach it
//! or its dependencies (`cargo xtask check` tests that).
//!
//! See docs/development.md, *The relay and lobby (N2)*.

pub mod hub;
pub mod server;

pub use hub::{ConnId, Entropy, Hub, Limits, Out, Stats};
pub use server::{OsEntropy, Relay, bind, bind_with, serve};
