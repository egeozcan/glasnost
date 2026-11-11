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

pub mod air;
pub mod anim_clock;
pub mod anims;
pub mod approach;
pub mod base;
pub mod behaviour;
pub mod bomb;
pub mod bounce;
pub mod bridges;
pub mod cells;
pub mod cloak;
pub mod combat;
pub mod computer;
pub mod consts;
pub mod corridor;
pub mod crates;
pub mod crush;
pub mod depot;
pub mod depot_anim;
pub mod disc;
pub mod domination;
pub mod drain;
pub mod drive;
pub mod econ;
pub mod emitters;
pub mod fall;
pub mod gas;
pub mod gattling;
pub mod hover;
pub mod infantry;
pub mod jump;
pub mod leftover;
pub mod lift;
pub mod lighting;
pub mod linked;
pub mod load;
pub mod mark;
pub mod mind;
pub mod mp_triggers;
pub mod mutation;
pub mod naval;
pub mod objects;
pub mod occupancy;
pub mod orders;
pub mod overlays;
pub mod paradrop;
pub mod parasite;
pub mod particles;
pub mod passability;
pub mod path;
pub mod pieces;
pub mod probe;
pub mod process;
pub mod psychic;
pub mod radiation;
pub mod refresh;
pub mod regions;
pub mod rng;
pub mod rocket;
pub mod routes;
pub mod rules;
pub mod save;
pub mod scatter;
pub mod seams;
pub mod spawn;
pub mod starting;
pub mod starts;
pub mod strike;
pub mod structures;
pub mod superweapons;
pub mod surface;
pub mod targeting;
pub mod teleport;
pub mod temporal;
pub mod testkit;
pub mod threat;
pub mod tick;
pub mod trace;
pub mod tracks;
pub mod transport;
pub mod triggers;
pub mod turning;
pub mod turret;
pub mod update;
pub mod walk;
pub mod walker;
pub mod walls;
pub mod world;
