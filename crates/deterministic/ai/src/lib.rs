//! Our own computer opponent. It issues only Orders a human player could
//! issue and sees only its own player's shroud.
//!
//! **Milestone M5** brings a minimal opponent; later milestones deepen it.
//! It runs inside the simulation tick and uses the simulation RNG, so it is a
//! simulation crate: the determinism lints apply.
//!
//! - [`scripted`]: a fixed build plan played through orders (M5a's scripted
//!   skirmish).
//! - [`skirmish`]: the headless skirmish runner that records a replay.
//! - [`game`]: a game with its AI houses (world and AI state stepped, saved
//!   and hashed together); [`brain`]: one AI house's state and thinking.
//! - [`special`]: garrisoning, engineers capturing and spies infiltrating
//!   (M6c).
//! - [`fixture`]: the synthetic skirmish (invented rules and map).

pub mod brain;
pub mod fixture;
pub mod game;
pub mod scripted;
pub mod skirmish;
pub mod special;
