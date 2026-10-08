//! Deterministic simulation: integer and fixed-point only, single-threaded,
//! tick-based and fully serializable.
//!
//! **Milestone M3** fills the core (fixed point, simulation RNG, tick pipeline,
//! locomotion, Orders, save/load, replays); **M4** combat; **M5** economy and
//! production; **M6–M7** the remaining mechanics. Every behavioural constant,
//! formula and ordering rule here cites an approved spec (`// spec: S-xxxx`)
//! or measurement (`// measure: M-xxxx`); `cargo xtask check` enforces it.
//!
//! This crate depends only on the typed ruleset (`rules`) and typed map
//! (`map`), never on rendering, audio, networking, file formats or the
//! importer. `cargo xtask check` and the `xtask` tests enforce that rule.
