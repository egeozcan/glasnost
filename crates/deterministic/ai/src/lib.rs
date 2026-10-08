//! Our own computer opponent. It issues only Orders a human player could
//! issue and sees only its own player's shroud.
//!
//! **Milestone M5** brings a minimal opponent; later milestones deepen it.
//! It runs inside the simulation tick and uses the simulation RNG, so it is a
//! simulation crate: the determinism lints apply.
