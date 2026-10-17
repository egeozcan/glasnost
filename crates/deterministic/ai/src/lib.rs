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
//!   (M6c); [`tech_buildings`]: which tech buildings an engineer is worth
//!   sending to (S-0515).
//! - [`deployers`]: deployable units deployed near the enemy and packed up
//!   again (S-0517).
//! - [`jumpjets`]: jumpjet units sent over obstacles to reach the enemy (S-0502).
//! - [`memory`]: the orders a unit did not take, so that the brain tries
//!   another goal instead of the same click every think (PLAN §4.7).
//! - [`fixture`]: the synthetic skirmish (invented rules and map).

pub mod brain;
pub mod chrono;
pub mod crates;
pub mod deployers;
pub mod fixture;
pub mod fixture_m6c;
pub mod fixture_m7a;
pub mod game;
pub mod gattling;
pub mod jumpjets;
pub mod linked;
pub mod memory;
pub mod mind;
pub mod psychic;
pub mod radiation;
pub mod scripted;
pub mod skirmish;
pub mod special;
pub mod tech_buildings;
pub mod terror;
