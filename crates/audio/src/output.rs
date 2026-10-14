//! Output backends. Every backend streams the software mix of
//! [`Mixer`]: the game calls [`Output::pump`] once per frame, and the
//! backend renders as much as it needs to stay a short lead ahead of the
//! device. One mixing path for every platform, so the offline tests cover
//! what is heard.
//!
//! - [`NullOutput`]: no device (tests, servers, a build without output).
//! - `WebOutput` (wasm32): WebAudio, scheduling rendered chunks as
//!   `AudioBuffer`s back to back on the `AudioContext`'s clock.
//! - `PipeOutput` (native, feature `native-output`): streams to a
//!   command-line player's standard input.

use crate::mixer::Mixer;

/// Default output rate where the device does not choose one (our design).
pub const DEFAULT_RATE: u32 = 48_000;

/// What an output is doing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputState {
    /// Playing.
    Running,
    /// Waiting for a user gesture before the device may start (browsers).
    WaitingForGesture,
    /// No device; the mix is discarded.
    Silent(String),
}

/// An audio device fed from the mixer.
pub trait Output {
    /// The device's frame rate; the mixer renders at this rate.
    fn rate(&self) -> u32;
    /// Renders and queues the mix to stay ahead of the device. Called once
    /// per frame.
    fn pump(&mut self, mixer: &mut Mixer);
    /// Starts or resumes the device; call it from a user-input handler
    /// (browsers refuse to start audio otherwise).
    fn resume(&mut self) {}
    /// The state.
    fn state(&self) -> OutputState;
}

/// No device: every pump drops the voices, so nothing piles up.
#[derive(Clone, Debug)]
pub struct NullOutput {
    rate: u32,
    reason: String,
}

impl NullOutput {
    /// A silent output at `rate`, with the reason it is silent.
    pub fn new(rate: u32, reason: &str) -> Self {
        Self {
            rate,
            reason: reason.to_owned(),
        }
    }
}

impl Output for NullOutput {
    fn rate(&self) -> u32 {
        self.rate
    }
    fn pump(&mut self, mixer: &mut Mixer) {
        mixer.clear();
    }
    fn state(&self) -> OutputState {
        OutputState::Silent(self.reason.clone())
    }
}

/// The platform's default output: WebAudio on the web; natively the pipe
/// output with feature `native-output`, otherwise [`NullOutput`].
pub fn platform_default() -> Box<dyn Output> {
    #[cfg(target_arch = "wasm32")]
    {
        match crate::web::WebOutput::new() {
            Ok(o) => Box::new(o),
            Err(e) => Box::new(NullOutput::new(DEFAULT_RATE, &e)),
        }
    }
    #[cfg(all(not(target_arch = "wasm32"), feature = "native-output"))]
    {
        match crate::pipe::PipeOutput::new(DEFAULT_RATE) {
            Ok(o) => Box::new(o),
            Err(e) => Box::new(NullOutput::new(DEFAULT_RATE, &e)),
        }
    }
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native-output")))]
    {
        Box::new(NullOutput::new(
            DEFAULT_RATE,
            "native audio output is off (build with feature native-output)",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{EntryId, SoundEntry};
    use crate::mixer::{Clip, MixQuirks, Prepared, Where};
    use std::sync::Arc;

    #[test]
    fn null_output_drops_voices() {
        let mut o = NullOutput::new(8000, "test");
        let mut m = Mixer::new(o.rate(), 1, MixQuirks::default());
        let prep = Prepared {
            id: EntryId(0),
            entry: Arc::new(SoundEntry::new("k", &["k"])),
            clips: vec![Arc::new(Clip {
                sample_rate: 8000,
                channels: 1,
                samples: vec![1; 10],
            })],
        };
        m.request(prep, Where::Screen, crate::catalog::FULL)
            .unwrap();
        o.pump(&mut m);
        assert_eq!(m.alive(), 0);
        assert!(matches!(o.state(), OutputState::Silent(_)));
    }
}
