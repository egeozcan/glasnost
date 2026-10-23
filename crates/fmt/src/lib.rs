//! File-format parsers, each paired with a writer for synthetic fixtures.
//!
//! Every parser here reads the public, documented container and asset
//! formats of the user's install. Layouts follow OpenRA and XCC (both
//! GPL-3.0-or-later, credited in each module) and public format notes. None
//! of this code describes game behaviour; it only turns bytes into
//! structures.
//!
//! **Hostile input.** Parsers never panic: every read is bounds-checked
//! ([`bytes`]), allocations are bounded by the input or by [`limits`], and
//! failures are [`Error`] values. Fuzz targets live in `fuzz/` (cargo-fuzz)
//! and a stable smoke test (`tests/hostile.rs`) runs mutated fixtures in CI.
//!
//! Each parser lands together with a [`fixture`] writer so that tests and CI
//! run on bytes we generated ourselves, never on game data.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod adpcm;
pub mod aud;
pub mod bytes;
pub mod csf;
pub mod error;
pub mod fixture;
pub mod fnt;
pub mod hostile;
pub mod hva;
pub mod idx;
pub mod image;
pub mod lcw;
pub mod limits;
pub(crate) mod lz;
pub mod lzo;
pub mod mappack;
pub mod mix;
pub mod mix_key;
pub mod pal;
pub mod pcm;
pub mod pcx;
pub mod shp;
pub mod sniff;
pub mod synth;
pub mod tmp;
pub mod vpl;
pub mod vxl;
pub mod wav;

pub use error::{Error, Result};
