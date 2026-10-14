//! Deterministic simulation: integer and fixed-point only, single-threaded,
//! tick-based and fully serializable.
//!
//! **Milestone M3** fills the core (number domains, simulation RNG, tick
//! pipeline, locomotion, Orders, save/load, replays); **M4** combat; **M5**
//! economy and production; **M6–M7** the remaining mechanics. Every
//! behavioural constant, formula and ordering rule here cites an approved spec
//! (`// spec: S-xxxx`) or measurement (`// measure: M-xxxx`); `cargo xtask
//! check` enforces it.
//!
//! This crate depends only on the typed ruleset (`rules`), the typed map
//! (`map`) and the number domains (`num`), never on rendering, audio,
//! networking, file formats or the importer. `cargo xtask check` and the
//! `xtask` tests enforce that rule.

pub mod approach;
pub mod behaviour;
pub mod bridges;
pub mod cells;
pub mod combat;
pub mod consts;
pub mod corridor;
pub mod crush;
pub mod drive;
pub mod econ;
pub mod infantry;
pub mod load;
pub mod objects;
pub mod occupancy;
pub mod orders;
pub mod overlays;
pub mod passability;
pub mod path;
pub mod process;
pub mod regions;
pub mod rng;
pub mod rules;
pub mod save;
pub mod scatter;
pub mod seams;
pub mod starting;
pub mod starts;
pub mod surface;
pub mod targeting;
pub mod teleport;
pub mod testkit;
pub mod tick;
pub mod tracks;
pub mod triggers;
pub mod turning;
pub mod turret;
pub mod update;
pub mod walk;
pub mod walls;
pub mod world;
