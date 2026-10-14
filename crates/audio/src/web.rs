//! WebAudio output (wasm32): renders the mix in chunks and schedules each
//! as an `AudioBuffer` on the `AudioContext`'s clock, back to back, a short
//! lead ahead of the playback position. No worklet, no threads.
//!
//! Browsers create an `AudioContext` suspended until a user gesture.
//! [`WebOutput::new`] installs one-shot `pointerdown`, `keydown` and
//! `touchend` listeners on the window that resume it, and
//! [`Output::resume`] does the same for a caller's own input handler. While
//! the context is not running, pumps drop the voices so that nothing plays
//! late.

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{AudioContext, AudioContextState};

use crate::mixer::Mixer;
use crate::output::{Output, OutputState};

/// Frames per scheduled chunk (our design).
const CHUNK: usize = 1024;
/// How far ahead of the playback position to keep scheduled, seconds (our
/// design).
const LEAD: f64 = 0.1;

/// The WebAudio output.
pub struct WebOutput {
    ctx: AudioContext,
    rate: u32,
    until: f64,
    buf: Vec<i16>,
    left: Vec<f32>,
    right: Vec<f32>,
    _gesture: Vec<Closure<dyn FnMut()>>,
}

impl WebOutput {
    /// Creates the context (suspended until a gesture in most browsers) and
    /// the gesture listeners.
    pub fn new() -> Result<Self, String> {
        let js = |e: wasm_bindgen::JsValue| format!("{e:?}");
        let ctx = AudioContext::new().map_err(js)?;
        let rate = ctx.sample_rate() as u32;
        let mut gesture = Vec::new();
        if let Some(window) = web_sys::window() {
            for ev in ["pointerdown", "keydown", "touchend"] {
                let c = ctx.clone();
                let cb = Closure::<dyn FnMut()>::new(move || {
                    let _ = c.resume();
                });
                let opts = web_sys::AddEventListenerOptions::new();
                opts.set_once(true);
                let _ = window.add_event_listener_with_callback_and_add_event_listener_options(
                    ev,
                    cb.as_ref().unchecked_ref(),
                    &opts,
                );
                gesture.push(cb);
            }
        }
        Ok(Self {
            ctx,
            rate: rate.max(1),
            until: 0.0,
            buf: Vec::new(),
            left: Vec::new(),
            right: Vec::new(),
            _gesture: gesture,
        })
    }

    fn schedule_chunk(&mut self, mixer: &mut Mixer) -> Result<(), wasm_bindgen::JsValue> {
        self.buf.resize(CHUNK * 2, 0);
        mixer.render(&mut self.buf);
        self.left.clear();
        self.right.clear();
        for f in self.buf.as_chunks::<2>().0 {
            self.left.push(f32::from(f[0]) / 32768.0);
            self.right.push(f32::from(f[1]) / 32768.0);
        }
        let buffer = self.ctx.create_buffer(2, CHUNK as u32, self.rate as f32)?;
        buffer.copy_to_channel(&self.left, 0)?;
        buffer.copy_to_channel(&self.right, 1)?;
        let source = self.ctx.create_buffer_source()?;
        source.set_buffer(Some(&buffer));
        source.connect_with_audio_node(&self.ctx.destination())?;
        #[allow(deprecated)]
        source.start_with_when(self.until)?;
        self.until += CHUNK as f64 / f64::from(self.rate);
        Ok(())
    }
}

impl Output for WebOutput {
    fn rate(&self) -> u32 {
        self.rate
    }

    fn pump(&mut self, mixer: &mut Mixer) {
        if self.ctx.state() != AudioContextState::Running {
            mixer.clear();
            return;
        }
        let now = self.ctx.current_time();
        if self.until < now {
            // Underrun (a stalled frame) or the first pump: restart a
            // little ahead of now.
            self.until = now + 0.01;
        }
        while self.until < now + LEAD {
            if self.schedule_chunk(mixer).is_err() {
                mixer.clear();
                return;
            }
        }
    }

    fn resume(&mut self) {
        let _ = self.ctx.resume();
    }

    fn state(&self) -> OutputState {
        match self.ctx.state() {
            AudioContextState::Running => OutputState::Running,
            AudioContextState::Suspended => OutputState::WaitingForGesture,
            _ => OutputState::Silent("audio context closed".into()),
        }
    }
}
