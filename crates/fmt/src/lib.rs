//! File-format parsers, each paired with a writer for synthetic fixtures.
//!
//! **Milestone M1** fills the parsers listed in PLAN.md section 4.2 (archives,
//! sprites, tiles, voxels, palettes, string tables, sound banks, fonts and the
//! compression schemes). Each parser lands together with a [`fixture`] writer
//! so that tests and CI run on bytes we generated ourselves, never on game
//! data.
//!
//! What exists in M0:
//! - [`image`]: the format-neutral decoded forms (palette-indexed images and
//!   palettes) that parsers will produce and the renderer consumes;
//! - [`synth`]: deterministic generators of invented content for fixtures and
//!   demos;
//! - [`fixture`]: the writer trait, the fixture registry, the manifest that
//!   records what was generated, and the round-trip test harness.

pub mod fixture;
pub mod image;
pub mod synth;
