//! Audio: sound effects, voices and announcements (M5b), and the asset
//! viewer's previews (M1).
//!
//! The pipeline, front to back:
//!
//! 1. [`bank::SoundBank`]: samples by name, decoded from the player's files
//!    (IDX/BAG banks, AUD, plain PCM WAVE) through `glasnost-fmt` and
//!    `glasnost-vfs`; [`bank::discover`] builds one from the mounted
//!    archives.
//! 2. [`catalog::SoundCatalog`]: the sound entries of `[SoundList]`
//!    (S-0701 claims 1 to 4); [`speech::SpeechCatalog`]: the announcement
//!    entries of `[DialogList]` (S-0701 claim 26).
//! 3. [`names::SoundNames`]: the rules data's sound names by role, from the
//!    keys the specs' Import mappings give.
//! 4. [`mixer::Mixer`]: the pure-Rust, integer software mixer of S-0701
//!    (placement, F2 to F4 levels and balance, per-entry limits, the 16
//!    channels and their victims, audio passes, the announcement voice),
//!    rendered offline in tests.
//! 5. [`output::Output`]: streams the mix to a device: WebAudio on wasm32,
//!    a command-line player's pipe natively with feature `native-output`,
//!    or nothing.
//! 6. [`Audio`]: what the game client calls every frame
//!    ([`Audio::tick`], [`Audio::selected`], [`Audio::ordered`],
//!    [`Audio::key`], [`Audio::cue`], [`Audio::set_view`],
//!    [`Audio::frame`], [`Audio::resume`]).
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
pub mod speech;
#[cfg(target_arch = "wasm32")]
pub mod web;

pub use events::{Audio, UiCue};
pub use mixer::{View, Where};
pub use names::{Announcement, SoundNames};
pub use preview::{Player, wav_bytes};
