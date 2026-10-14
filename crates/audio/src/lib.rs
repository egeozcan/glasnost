//! Audio: sound effects, voices and announcements (M5b), and the asset
//! viewer's previews (M1).
//!
//! The pipeline, front to back:
//!
//! 1. [`bank::SoundBank`]: samples by name, decoded from the player's files
//!    (IDX/BAG banks, AUD, plain PCM WAVE) through `glasnost-fmt` and
//!    `glasnost-vfs`; [`bank::discover`] builds one from the mounted
//!    archives.
//! 2. [`catalog::SoundCatalog`]: what a sound name plays. Interim rule until
//!    a sound spec lands (Q-0260): the sample of the same name.
//! 3. [`names::SoundNames`]: the rules data's sound names by role, from the
//!    keys the specs' Import mappings give.
//! 4. [`mixer::Mixer`]: the pure-Rust, integer software mixer (gain,
//!    distance attenuation and panning against the view, voice limits and
//!    priority stealing, resampling), rendered offline in tests.
//! 5. [`output::Output`]: streams the mix to a device: WebAudio on wasm32,
//!    a command-line player's pipe natively with feature `native-output`,
//!    or nothing.
//! 6. [`Audio`]: what the game client calls every frame
//!    ([`Audio::play_effects`], [`Audio::acknowledge`], [`Audio::cue`],
//!    [`Audio::set_listener`], [`Audio::frame`], [`Audio::resume`]).
//!
//! Sound never feeds back into the simulation. Every interim rule is listed
//! in docs/development.md, "Audio (M5b)".

pub mod bank;
pub mod catalog;
pub mod events;
pub mod mixer;
pub mod names;
pub mod output;
#[cfg(all(not(target_arch = "wasm32"), feature = "native-output"))]
pub mod pipe;
mod preview;
#[cfg(target_arch = "wasm32")]
pub mod web;

pub use events::{Audio, PlayError, UiCue};
pub use mixer::{Listener, Placement};
pub use names::{Announcement, SoundNames};
pub use preview::{Player, wav_bytes};
