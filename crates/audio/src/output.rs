//! Output backends. Every backend streams the software mix of
//! [`Mixer`]: the game calls [`Output::pump`] once per frame, and the
//! backend renders as much as it needs to stay a short lead ahead of the
//! device. One mixing path for every platform, so the offline tests cover
//! what is heard.
//!
//! - [`NullOutput`]: no device (tests, servers, a build without output).
//! - `WebOutput` (wasm32): WebAudio, scheduling rendered chunks as
//!   `AudioBuffer`s back to back on the `AudioContext`'s clock.
//! - `DeviceOutput` (native, feature `device`): the system's default
//!   output device through cpal, fed through a lock-free ring.
//!
//! Natively the device is opened only in a program that calls
//! [`allow_device`] (the desktop game does), so tests and tools built with
//! the feature never open one.

use std::sync::atomic::{AtomicBool, Ordering};

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

/// Whether this build has the native device output (feature `device`).
pub const DEVICE_BUILT: bool = cfg!(all(not(target_arch = "wasm32"), feature = "device"));

static DEVICE_ALLOWED: AtomicBool = AtomicBool::new(false);

/// Lets [`platform_default`] open the system's audio device. The desktop
/// game calls it at start; nothing else does, so tests never open a device.
pub fn allow_device() {
    DEVICE_ALLOWED.store(true, Ordering::Relaxed);
}

/// Why the platform's output will be silent, when that is known before
/// opening it: natively, a build without the device output or a program
/// that has not called [`allow_device`]. `None` when it may play.
pub fn known_silent() -> Option<&'static str> {
    if cfg!(target_arch = "wasm32") {
        None
    } else if !DEVICE_BUILT {
        Some("this build has no audio output (built without the audio feature)")
    } else if !DEVICE_ALLOWED.load(Ordering::Relaxed) {
        Some("this program does not open the audio device")
    } else {
        None
    }
}

/// The output `open` gives, or, when it fails, a [`NullOutput`] naming why
/// (logged once, here).
pub fn open_or_silent(open: impl FnOnce() -> Result<Box<dyn Output>, String>) -> Box<dyn Output> {
    match open() {
        Ok(o) => o,
        Err(e) => {
            log::warn!("no sound: {e}");
            Box::new(NullOutput::new(DEFAULT_RATE, &e))
        }
    }
}

/// The platform's default output: WebAudio on the web; natively the
/// default device with feature `device` once [`allow_device`] was called;
/// otherwise, or when that fails, [`NullOutput`] with the reason.
pub fn platform_default() -> Box<dyn Output> {
    #[cfg(target_arch = "wasm32")]
    {
        open_or_silent(|| crate::web::WebOutput::new().map(|o| Box::new(o) as Box<dyn Output>))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        open_or_silent(native_device)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn native_device() -> Result<Box<dyn Output>, String> {
    if let Some(why) = known_silent() {
        return Err(why.to_owned());
    }
    #[cfg(feature = "device")]
    {
        crate::device::DeviceOutput::open().map(|o| Box::new(o) as Box<dyn Output>)
    }
    #[cfg(not(feature = "device"))]
    {
        Err("this build has no audio output".into())
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

    #[test]
    fn a_device_that_fails_to_open_falls_back_to_silence_with_its_reason() {
        let o = open_or_silent(|| Err("no audio output device (test)".into()));
        assert_eq!(o.rate(), DEFAULT_RATE);
        assert_eq!(
            o.state(),
            OutputState::Silent("no audio output device (test)".into())
        );
    }

    #[test]
    fn an_output_that_opens_is_used() {
        let o = open_or_silent(|| Ok(Box::new(NullOutput::new(44_100, "stand-in"))));
        assert_eq!(o.rate(), 44_100);
        assert_eq!(o.state(), OutputState::Silent("stand-in".into()));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn tests_never_open_the_device() {
        // No test calls allow_device, so the platform's output is silent
        // whether or not the build has the device output.
        let why = known_silent().expect("silent before allow_device");
        assert_eq!(platform_default().state(), OutputState::Silent(why.into()));
        if DEVICE_BUILT {
            assert!(why.contains("does not open"), "{why}");
        } else {
            assert!(why.contains("without the audio feature"), "{why}");
        }
    }
}
