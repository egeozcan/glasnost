//! Native device output (feature `device`): plays the mix on the system's
//! default output device through cpal (ALSA on Linux, which PipeWire and
//! PulseAudio serve through their ALSA plugins; WASAPI on Windows;
//! CoreAudio on macOS).
//!
//! The device's own thread pulls samples in its callback; the client's
//! frame loop renders the mix in [`Output::pump`]. A lock-free
//! [`Ring`] joins the two: each pump tops the ring up to [`LATENCY_MS`] of
//! audio plus the largest block the device has asked for, and the callback
//! takes what is there and plays silence for the rest, so it never waits.
//! The device's pull is the only clock: no wall-clock pacing.
//!
//! The mixer runs at the device's own rate ([`Output::rate`]): it already
//! resamples every clip and the music to its output rate, so no second
//! resampler is needed.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{ErrorKind, FromSample, Sample, SampleFormat, SizedSample, StreamConfig};

use crate::mixer::Mixer;
use crate::output::{Output, OutputState};
use crate::ring::Ring;

/// How much mixed audio the pump keeps queued ahead of the device, in
/// milliseconds, on top of the device's own block (our design: under 0.1 s
/// from an event to its sound, and room for a slow frame at 30 frames per
/// second).
// design: native audio (latency target)
pub const LATENCY_MS: u32 = 60;

/// The ring's size, in milliseconds (our design: the pump never queues
/// more than the target, so this only bounds a device with huge blocks).
// design: native audio (ring bound)
const RING_MS: u32 = 500;

/// Frames converted per step in the callback (our design: a fixed scratch
/// buffer, so the callback never allocates).
// design: native audio (callback scratch)
const CHUNK: usize = 256;

/// What the frame loop and the callback share.
struct Shared {
    ring: Ring,
    /// Why the stream stopped, once it has.
    lost: OnceLock<String>,
    /// Frames played from the ring.
    played: AtomicU64,
    /// Frames of silence played because the ring was empty (underruns).
    silent: AtomicU64,
    /// The largest block the device has asked for, in frames.
    largest: AtomicUsize,
}

impl Shared {
    fn new(capacity: usize) -> Self {
        Self {
            ring: Ring::new(capacity),
            lost: OnceLock::new(),
            played: AtomicU64::new(0),
            silent: AtomicU64::new(0),
            largest: AtomicUsize::new(0),
        }
    }
}

/// The device output.
pub struct DeviceOutput {
    rate: u32,
    shared: Arc<Shared>,
    /// The latency target in frames, without the device's block.
    lead: usize,
    buf: Vec<i16>,
    name: String,
    // Dropped last: stops the callback.
    _stream: cpal::Stream,
}

impl DeviceOutput {
    /// Opens the default output device at its default configuration and
    /// starts the stream (playing silence until the first pump).
    pub fn open() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| format!("no audio output device ({})", host.id()))?;
        let name = device
            .id()
            .map(|id| id.to_string())
            .unwrap_or_else(|_| "default".into());
        let supported = device
            .default_output_config()
            .map_err(|e| format!("the audio device {name} has no output format: {e}"))?;
        let format = supported.sample_format();
        let config: StreamConfig = supported.into();
        let rate = config.sample_rate;
        if rate == 0 || config.channels == 0 {
            return Err(format!(
                "the audio device {name} reports no rate or no channels"
            ));
        }
        let ms = |ms: u32| (u64::from(rate) * u64::from(ms) / 1000) as usize;
        let shared = Arc::new(Shared::new(ms(RING_MS)));
        let s = shared.clone();
        let stream = match format {
            SampleFormat::I8 => build::<i8>(&device, config, s),
            SampleFormat::I16 => build::<i16>(&device, config, s),
            SampleFormat::I32 => build::<i32>(&device, config, s),
            SampleFormat::I64 => build::<i64>(&device, config, s),
            SampleFormat::U8 => build::<u8>(&device, config, s),
            SampleFormat::U16 => build::<u16>(&device, config, s),
            SampleFormat::U32 => build::<u32>(&device, config, s),
            SampleFormat::U64 => build::<u64>(&device, config, s),
            SampleFormat::F32 => build::<f32>(&device, config, s),
            SampleFormat::F64 => build::<f64>(&device, config, s),
            other => Err(format!("unsupported sample format {other}")),
        }
        .map_err(|e| format!("the audio device {name} did not open: {e}"))?;
        stream
            .play()
            .map_err(|e| format!("the audio device {name} did not start: {e}"))?;
        log::info!("sound: playing on {name} at {rate} Hz, {format}");
        Ok(Self {
            rate,
            shared,
            lead: ms(LATENCY_MS),
            buf: Vec::new(),
            name,
            _stream: stream,
        })
    }

    /// Frames to keep queued: the latency target plus the device's largest
    /// block, within the ring.
    fn target(&self) -> usize {
        let block = self.shared.largest.load(Ordering::Relaxed);
        (self.lead + block).min(self.shared.ring.capacity())
    }
}

fn build<T>(
    device: &cpal::Device,
    config: StreamConfig,
    shared: Arc<Shared>,
) -> Result<cpal::Stream, String>
where
    T: SizedSample + FromSample<i16>,
{
    let channels = usize::from(config.channels);
    let data = shared.clone();
    let mut scratch = [0i16; 2 * CHUNK];
    device
        .build_output_stream(
            config,
            move |out: &mut [T], _: &cpal::OutputCallbackInfo| {
                fill(&data, channels, out, &mut scratch);
            },
            move |e: cpal::Error| match e.kind() {
                // Glitches the stream survives.
                ErrorKind::Xrun | ErrorKind::DeviceChanged | ErrorKind::RealtimeDenied => {}
                _ => {
                    if shared.lost.set(e.to_string()).is_ok() {
                        log::warn!("no sound: the audio device stopped: {e}");
                    }
                }
            },
            None,
        )
        .map_err(|e| e.to_string())
}

/// The callback: fills `out` (`channels` interleaved) from the ring, left
/// and right on the first two channels, their mean on a mono device,
/// silence on further channels and where the ring runs dry.
fn fill<T>(shared: &Shared, channels: usize, out: &mut [T], scratch: &mut [i16; 2 * CHUNK])
where
    T: Sample + FromSample<i16>,
{
    out.fill(T::EQUILIBRIUM);
    let channels = channels.max(1);
    shared
        .largest
        .fetch_max(out.len() / channels, Ordering::Relaxed);
    for block in out.chunks_mut(channels * CHUNK) {
        let frames = block.len() / channels;
        let got = shared.ring.pop_into(&mut scratch[..2 * frames]);
        shared.played.fetch_add(got as u64, Ordering::Relaxed);
        shared
            .silent
            .fetch_add((frames - got) as u64, Ordering::Relaxed);
        for (o, f) in block
            .chunks_exact_mut(channels)
            .zip(scratch.as_chunks::<2>().0)
        {
            if channels == 1 {
                let mean = (i32::from(f[0]) + i32::from(f[1])) / 2;
                o[0] = T::from_sample(mean as i16);
            } else {
                o[0] = T::from_sample(f[0]);
                o[1] = T::from_sample(f[1]);
            }
        }
    }
}

impl Output for DeviceOutput {
    fn rate(&self) -> u32 {
        self.rate
    }

    fn pump(&mut self, mixer: &mut Mixer) {
        if self.shared.lost.get().is_some() {
            mixer.clear();
            return;
        }
        let queued = self.shared.ring.len();
        let target = self.target();
        if queued >= target {
            return;
        }
        let frames = target - queued;
        self.buf.resize(frames * 2, 0);
        mixer.render(&mut self.buf);
        self.shared.ring.push(&self.buf);
    }

    fn state(&self) -> OutputState {
        match self.shared.lost.get() {
            None => OutputState::Running,
            Some(e) => OutputState::Silent(format!("the audio device stopped: {e}")),
        }
    }
}

impl Drop for DeviceOutput {
    fn drop(&mut self) {
        let played = self.shared.played.load(Ordering::Relaxed);
        let silent = self.shared.silent.load(Ordering::Relaxed);
        log::info!(
            "sound: {} closed after {played} frames played and {silent} frames of underrun silence",
            self.name
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared_with(frames: &[(i16, i16)]) -> Shared {
        let s = Shared::new(64);
        let samples: Vec<i16> = frames.iter().flat_map(|&(l, r)| [l, r]).collect();
        s.ring.push(&samples);
        s
    }

    #[test]
    fn stereo_frames_reach_a_stereo_device_and_then_silence() {
        let s = shared_with(&[(100, -100), (200, -200)]);
        let mut out = [7i16; 6];
        fill(&s, 2, &mut out, &mut [0; 2 * CHUNK]);
        assert_eq!(out, [100, -100, 200, -200, 0, 0]);
        assert_eq!(s.played.load(Ordering::Relaxed), 2);
        assert_eq!(s.silent.load(Ordering::Relaxed), 1);
        assert_eq!(s.largest.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn a_mono_device_gets_the_mean() {
        let s = shared_with(&[(100, 300), (-4, -2)]);
        let mut out = [7i16; 2];
        fill(&s, 1, &mut out, &mut [0; 2 * CHUNK]);
        assert_eq!(out, [200, -3]);
    }

    #[test]
    fn extra_channels_are_silent_and_float_samples_are_scaled() {
        let s = shared_with(&[(i16::MIN, 0)]);
        let mut out = [9.0f32; 6];
        fill(&s, 6, &mut out, &mut [0; 2 * CHUNK]);
        assert_eq!(out, [-1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn a_long_block_is_filled_in_chunks() {
        let s = Shared::new(4 * CHUNK);
        let samples: Vec<i16> = (0..3 * CHUNK as i16).flat_map(|k| [k, -k]).collect();
        s.ring.push(&samples);
        let mut out = vec![0i16; 2 * 3 * CHUNK + 2];
        fill(&s, 2, &mut out, &mut [0; 2 * CHUNK]);
        assert_eq!(out[..out.len() - 2], samples[..]);
        assert_eq!(out[out.len() - 2..], [0, 0]);
        assert_eq!(s.silent.load(Ordering::Relaxed), 1);
    }
}
