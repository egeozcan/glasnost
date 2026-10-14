//! The software mixer: voices with gain, distance attenuation and stereo
//! panning against the view, voice limits and priority stealing, and
//! resampling to the output rate. Pure Rust and integer arithmetic, so it
//! renders offline and every test checks exact samples.
//!
//! How loud a sound is at a distance, how it is panned, how many play at
//! once and which one gives way are not specified (Q-0262). Every such
//! parameter is in [`MixParams`], marked as our own interim design.
//!
//! Sound never feeds back into the simulation: the mixer only reads the
//! [`Clip`]s and positions it is given.

use std::sync::Arc;

use glasnost_fmt::pcm::Pcm;

/// Unity gain in the mixer's fixed point (16 fractional bits).
pub const UNITY: u32 = 1 << 16;

/// Leptons per cell side, used only to state the interim distances in
/// cells.
const CELL: i32 = 256; // spec: S-0204 claim 2 (position to cell: 256 leptons to a cell side)

/// A decoded sound, ready to mix: interleaved signed 16-bit samples, one or
/// two channels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clip {
    /// Frames per second.
    pub sample_rate: u32,
    /// 1 (mono) or 2 (stereo).
    pub channels: u16,
    /// Samples, `channels` per frame.
    pub samples: Vec<i16>,
}

impl Clip {
    /// A clip from decoded PCM, or `None` when it has no sample rate, a
    /// channel count other than 1 or 2, or no whole frame.
    pub fn from_pcm(pcm: Pcm) -> Option<Self> {
        if pcm.sample_rate == 0 || !(1..=2).contains(&pcm.channels) || pcm.frames() == 0 {
            return None;
        }
        let whole = pcm.frames() * usize::from(pcm.channels);
        let mut samples = pcm.samples;
        samples.truncate(whole);
        Some(Self {
            sample_rate: pcm.sample_rate,
            channels: pcm.channels,
            samples,
        })
    }

    /// Whole frames.
    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels.max(1))
    }

    /// The sample of channel `c` in frame `f` (the last channel stands in
    /// for a missing one).
    fn at(&self, f: usize, c: usize) -> i64 {
        let ch = usize::from(self.channels);
        i64::from(self.samples[f * ch + c.min(ch - 1)])
    }
}

/// The mixer's tunable parameters. **Every default is our own interim
/// design (Q-0262)**, not the game's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MixParams {
    /// Voices that can play at once (interim: 16, Q-0262).
    pub max_voices: usize,
    /// Distance from the listener, in leptons, within which a positional
    /// sound plays at full gain (interim: 10 cells, Q-0262).
    pub full_radius: i32,
    /// Distance in leptons at and beyond which a positional sound is
    /// silent; a sound that starts there is not started (interim: 30
    /// cells, Q-0262). Between the two radii the gain falls linearly.
    pub silent_radius: i32,
    /// Screen-x offset (x − y, leptons) at which a sound is panned fully to
    /// one side (interim: 20 cells, Q-0262). Panning is a balance: the far
    /// side's gain falls linearly, the near side stays at full gain.
    pub pan_width: i32,
}

impl Default for MixParams {
    fn default() -> Self {
        Self {
            max_voices: 16,           // interim, Q-0262
            full_radius: 10 * CELL,   // interim, Q-0262
            silent_radius: 30 * CELL, // interim, Q-0262
            pan_width: 20 * CELL,     // interim, Q-0262
        }
    }
}

/// The listener: the centre of the view, in map leptons (given by the
/// caller every frame).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Listener {
    /// Map x, leptons.
    pub x: i32,
    /// Map y, leptons.
    pub y: i32,
}

/// Where a sound plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    /// Everywhere: no attenuation, centred (interface sounds,
    /// announcements, a sound the specs give no position).
    Everywhere,
    /// At a map position (leptons): attenuated by distance from the
    /// listener and panned.
    At {
        /// Map x, leptons.
        x: i32,
        /// Map y, leptons.
        y: i32,
    },
}

/// A request to start a voice.
#[derive(Clone, Debug)]
pub struct VoiceRequest {
    /// The samples.
    pub clip: Arc<Clip>,
    /// The sound it belongs to (per-sound limits count voices by key).
    pub key: String,
    /// Gain, [`UNITY`] = full.
    pub gain: u32,
    /// Where it plays.
    pub placement: Placement,
    /// Priority for voice stealing: higher wins.
    pub priority: i32,
    /// At most this many voices of the same key at once; `None`: no limit.
    pub limit: Option<u16>,
}

/// Why a voice was not started.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The clip has no frames.
    Empty,
    /// A positional sound at or beyond the silent radius.
    OutOfRange,
    /// Its sound already plays as often as its limit allows.
    Limit,
    /// Every voice is taken by one of higher priority.
    NoSlot,
}

/// A started voice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VoiceId(pub u64);

#[derive(Clone, Debug)]
struct Voice {
    id: u64,
    req: VoiceRequest,
    /// Position in the clip, frames in 32.32 fixed point.
    pos: u64,
    /// Clip frames per output frame, 32.32.
    step: u64,
}

/// The mixer.
#[derive(Clone, Debug)]
pub struct Mixer {
    rate: u32,
    params: MixParams,
    listener: Listener,
    master: u32,
    voices: Vec<Voice>,
    next: u64,
    acc: Vec<i64>,
}

impl Mixer {
    /// A mixer rendering stereo at `output_rate` frames per second.
    pub fn new(output_rate: u32, params: MixParams) -> Self {
        Self {
            rate: output_rate.max(1),
            params,
            listener: Listener::default(),
            master: UNITY,
            voices: Vec::new(),
            next: 0,
            acc: Vec::new(),
        }
    }

    /// The output rate.
    pub fn output_rate(&self) -> u32 {
        self.rate
    }

    /// The parameters.
    pub fn params(&self) -> &MixParams {
        &self.params
    }

    /// Moves the listener (the view centre). Playing voices follow it.
    pub fn set_listener(&mut self, l: Listener) {
        self.listener = l;
    }

    /// The listener.
    pub fn listener(&self) -> Listener {
        self.listener
    }

    /// Sets the master gain ([`UNITY`] = full).
    pub fn set_master(&mut self, gain: u32) {
        self.master = gain;
    }

    /// Voices playing.
    pub fn playing(&self) -> usize {
        self.voices.len()
    }

    /// Whether a voice still plays.
    pub fn is_playing(&self, id: VoiceId) -> bool {
        self.voices.iter().any(|v| v.id == id.0)
    }

    /// Keys of the playing voices, oldest first.
    pub fn playing_keys(&self) -> Vec<&str> {
        self.voices.iter().map(|v| v.req.key.as_str()).collect()
    }

    /// Stops a voice.
    pub fn stop(&mut self, id: VoiceId) {
        self.voices.retain(|v| v.id != id.0);
    }

    /// Stops every voice.
    pub fn clear(&mut self) {
        self.voices.clear();
    }

    /// Starts a voice, applying the range test, the per-sound limit and the
    /// voice limit, in that order. When every voice is taken, the new one
    /// replaces the playing voice of lowest priority (the oldest of those)
    /// if its own priority is at least as high, and is refused otherwise.
    /// All of this is our interim design (Q-0262).
    pub fn play(&mut self, req: VoiceRequest) -> Result<VoiceId, Refused> {
        if req.clip.frames() == 0 {
            return Err(Refused::Empty);
        }
        if self.spatial(req.placement).is_none() {
            return Err(Refused::OutOfRange);
        }
        if let Some(limit) = req.limit {
            let same = self.voices.iter().filter(|v| v.req.key == req.key).count();
            if same >= usize::from(limit) {
                return Err(Refused::Limit);
            }
        }
        if self.params.max_voices == 0 {
            return Err(Refused::NoSlot);
        }
        if self.voices.len() >= self.params.max_voices {
            // Lowest priority, then oldest (voices are kept oldest first).
            let victim = self
                .voices
                .iter()
                .enumerate()
                .min_by_key(|(_, v)| (v.req.priority, v.id))
                .map(|(i, v)| (i, v.req.priority));
            match victim {
                Some((i, p)) if p <= req.priority => {
                    self.voices.remove(i);
                }
                _ => return Err(Refused::NoSlot),
            }
        }
        let step = (u64::from(req.clip.sample_rate) << 32) / u64::from(self.rate);
        let id = self.next;
        self.next += 1;
        self.voices.push(Voice {
            id,
            req,
            pos: 0,
            step: step.max(1),
        });
        Ok(VoiceId(id))
    }

    /// Left and right gains for a placement, [`UNITY`] = full, or `None`
    /// when it is out of range (interim curve, Q-0262).
    fn spatial(&self, p: Placement) -> Option<(u32, u32)> {
        let Placement::At { x, y } = p else {
            return Some((UNITY, UNITY));
        };
        let dx = i64::from(x) - i64::from(self.listener.x);
        let dy = i64::from(y) - i64::from(self.listener.y);
        let d = (dx * dx + dy * dy).unsigned_abs().isqrt() as i64;
        let full = i64::from(self.params.full_radius.max(0));
        let silent = i64::from(self.params.silent_radius).max(full + 1);
        if d >= silent {
            return None;
        }
        let g = if d <= full {
            i64::from(UNITY)
        } else {
            i64::from(UNITY) * (silent - d) / (silent - full)
        };
        let w = i64::from(self.params.pan_width.max(1));
        let s = (dx - dy).clamp(-w, w);
        let near = i64::from(UNITY);
        let far = i64::from(UNITY) * (w - s.abs()) / w;
        let (l, r) = if s > 0 { (far, near) } else { (near, far) };
        Some((((g * l) >> 16) as u32, ((g * r) >> 16) as u32))
    }

    /// Mixes the next `out.len() / 2` stereo frames (interleaved left,
    /// right) into `out`, overwriting it, and advances every voice. Voices
    /// that reach their end are removed. The sum is clipped to 16 bits.
    pub fn render(&mut self, out: &mut [i16]) {
        let frames = out.len() / 2;
        self.acc.clear();
        self.acc.resize(frames * 2, 0);
        let master = i64::from(self.master);
        for vi in 0..self.voices.len() {
            let (l, r) = self
                .spatial(self.voices[vi].req.placement)
                .unwrap_or((0, 0));
            let v = &mut self.voices[vi];
            let g = (i64::from(v.req.gain) * master) >> 16;
            let gl = (g * i64::from(l)) >> 16;
            let gr = (g * i64::from(r)) >> 16;
            let clip = &v.req.clip;
            let n = clip.frames();
            for f in 0..frames {
                let i = (v.pos >> 32) as usize;
                if i >= n {
                    break;
                }
                let frac = (v.pos & 0xFFFF_FFFF) as i64;
                let j = (i + 1).min(n - 1);
                let lerp = |c: usize| {
                    let a = clip.at(i, c);
                    let b = clip.at(j, c);
                    a + (((b - a) * frac) >> 32)
                };
                let (sl, sr) = if clip.channels == 1 {
                    let s = lerp(0);
                    (s, s)
                } else {
                    (lerp(0), lerp(1))
                };
                self.acc[2 * f] += (sl * gl) >> 16;
                self.acc[2 * f + 1] += (sr * gr) >> 16;
                v.pos += v.step;
            }
        }
        self.voices
            .retain(|v| ((v.pos >> 32) as usize) < v.req.clip.frames());
        for (o, a) in out.iter_mut().zip(&self.acc) {
            *o = (*a).clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16;
        }
        if out.len() % 2 == 1
            && let Some(last) = out.last_mut()
        {
            *last = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(rate: u32, channels: u16, samples: Vec<i16>) -> Arc<Clip> {
        Arc::new(Clip {
            sample_rate: rate,
            channels,
            samples,
        })
    }

    fn req(c: &Arc<Clip>, key: &str) -> VoiceRequest {
        VoiceRequest {
            clip: c.clone(),
            key: key.into(),
            gain: UNITY,
            placement: Placement::Everywhere,
            priority: 0,
            limit: None,
        }
    }

    fn render(m: &mut Mixer, frames: usize) -> Vec<i16> {
        let mut out = vec![0; frames * 2];
        m.render(&mut out);
        out
    }

    #[test]
    fn one_mono_voice_plays_unchanged_on_both_sides_then_ends() {
        let c = clip(100, 1, vec![1000, -2000, 3000]);
        let mut m = Mixer::new(100, MixParams::default());
        m.play(req(&c, "a")).unwrap();
        assert_eq!(
            render(&mut m, 4),
            [1000, 1000, -2000, -2000, 3000, 3000, 0, 0]
        );
        assert_eq!(m.playing(), 0);
    }

    #[test]
    fn stereo_clip_keeps_its_channels() {
        let c = clip(100, 2, vec![10, 20, 30, 40]);
        let mut m = Mixer::new(100, MixParams::default());
        m.play(req(&c, "s")).unwrap();
        assert_eq!(render(&mut m, 2), [10, 20, 30, 40]);
    }

    #[test]
    fn voices_sum_and_the_sum_clips() {
        let a = clip(100, 1, vec![1000, 30000, -30000]);
        let b = clip(100, 1, vec![500, 10000, -10000]);
        let mut m = Mixer::new(100, MixParams::default());
        m.play(req(&a, "a")).unwrap();
        m.play(req(&b, "b")).unwrap();
        assert_eq!(
            render(&mut m, 3),
            [1500, 1500, i16::MAX, i16::MAX, i16::MIN, i16::MIN]
        );
    }

    #[test]
    fn gain_scales_linearly() {
        // S-0416 claim 4's "half volume" is applied as gain UNITY / 2 by
        // the event layer; here the mixer's linear gain.
        let c = clip(100, 1, vec![8000, -8000]);
        let mut m = Mixer::new(100, MixParams::default());
        m.play(VoiceRequest {
            gain: UNITY / 2,
            ..req(&c, "h")
        })
        .unwrap();
        assert_eq!(render(&mut m, 2), [4000, 4000, -4000, -4000]);
    }

    #[test]
    fn distance_attenuates_and_out_of_range_is_refused() {
        let p = MixParams {
            full_radius: 1000,
            silent_radius: 3000,
            pan_width: 1_000_000,
            ..MixParams::default()
        };
        let c = clip(100, 1, vec![10000; 4]);
        let at = |x, y| VoiceRequest {
            placement: Placement::At { x, y },
            ..req(&c, "p")
        };
        let mut m = Mixer::new(100, p);
        m.set_listener(Listener { x: 5000, y: 5000 });
        // Within the full radius: full gain (on the diagonal x = y the pan
        // offset x − y is 0).
        m.play(at(5500, 5500)).unwrap();
        assert_eq!(render(&mut m, 1), [10000, 10000]);
        m.clear();
        // Halfway between the radii: half gain.
        m.play(at(5000 + 1414, 5000 + 1414)).unwrap();
        let out = render(&mut m, 1);
        assert!((out[0] - 5000).abs() <= 10, "{out:?}");
        m.clear();
        // At the silent radius: not started.
        assert_eq!(m.play(at(5000 + 3000, 5000)), Err(Refused::OutOfRange));
    }

    #[test]
    fn playing_voices_follow_the_listener() {
        let p = MixParams {
            full_radius: 1000,
            silent_radius: 3000,
            pan_width: 1_000_000,
            ..MixParams::default()
        };
        let c = clip(100, 1, vec![10000; 4]);
        let mut m = Mixer::new(100, p);
        m.play(VoiceRequest {
            placement: Placement::At { x: 0, y: 0 },
            ..req(&c, "p")
        })
        .unwrap();
        assert_eq!(render(&mut m, 1), [10000, 10000]);
        // The view moves away: the voice keeps playing, silent.
        m.set_listener(Listener { x: 3000, y: 3000 });
        assert_eq!(render(&mut m, 1), [0, 0]);
        assert_eq!(m.playing(), 1);
    }

    #[test]
    fn panning_follows_the_screen_x_offset() {
        let p = MixParams {
            full_radius: 100_000,
            silent_radius: 200_000,
            pan_width: 1000,
            ..MixParams::default()
        };
        let c = clip(100, 1, vec![10000]);
        let mut m = Mixer::new(100, p);
        let mut at = |x: i32, y: i32| {
            m.play(VoiceRequest {
                placement: Placement::At { x, y },
                ..req(&c, "p")
            })
            .unwrap();
            render(&mut m, 1)
        };
        // Screen right (x − y > 0): left falls, right stays full.
        assert_eq!(at(500, 0), [5000, 10000]);
        assert_eq!(at(0, 2000), [10000, 0]);
        // On the screen's vertical through the listener: centred.
        assert_eq!(at(700, 700), [10000, 10000]);
    }

    #[test]
    fn a_full_mixer_steals_the_lowest_priority_oldest_voice() {
        let c = clip(100, 1, vec![1; 100]);
        let p = MixParams {
            max_voices: 2,
            ..MixParams::default()
        };
        let mut m = Mixer::new(100, p);
        let pri = |k: &str, priority| VoiceRequest {
            priority,
            ..req(&c, k)
        };
        m.play(pri("low1", 1)).unwrap();
        m.play(pri("low2", 1)).unwrap();
        // Higher priority replaces the oldest of the lowest.
        m.play(pri("high", 5)).unwrap();
        assert_eq!(m.playing_keys(), ["low2", "high"]);
        // Equal priority to the lowest still replaces it.
        m.play(pri("eq", 1)).unwrap();
        assert_eq!(m.playing_keys(), ["high", "eq"]);
        // Lower than every playing voice: refused.
        assert_eq!(m.play(pri("lower", 0)), Err(Refused::NoSlot));
        assert_eq!(m.playing_keys(), ["high", "eq"]);
    }

    #[test]
    fn per_sound_limit_refuses_extra_copies() {
        let c = clip(100, 1, vec![1; 100]);
        let mut m = Mixer::new(100, MixParams::default());
        let lim = VoiceRequest {
            limit: Some(2),
            ..req(&c, "tick")
        };
        m.play(lim.clone()).unwrap();
        m.play(lim.clone()).unwrap();
        assert_eq!(m.play(lim.clone()), Err(Refused::Limit));
        m.play(req(&c, "other")).unwrap();
        assert_eq!(m.playing(), 3);
    }

    #[test]
    fn resampling_interpolates_linearly() {
        // 50 Hz source into a 100 Hz output: every other output frame is a
        // midpoint; 3 source frames last 6 output frames (the last one
        // holds its value).
        let c = clip(50, 1, vec![0, 1000, -1000]);
        let mut m = Mixer::new(100, MixParams::default());
        m.play(req(&c, "r")).unwrap();
        let out = render(&mut m, 7);
        let left: Vec<i16> = out.iter().step_by(2).copied().collect();
        assert_eq!(left, [0, 500, 1000, 0, -1000, -1000, 0]);
        assert_eq!(m.playing(), 0);
        // Downsampling 200 Hz into 100 Hz takes every other frame.
        let c = clip(200, 1, vec![1, 2, 3, 4, 5, 6]);
        m.play(req(&c, "d")).unwrap();
        let left: Vec<i16> = render(&mut m, 4).iter().step_by(2).copied().collect();
        assert_eq!(left, [1, 3, 5, 0]);
    }

    #[test]
    fn rendering_in_pieces_equals_rendering_at_once() {
        let c = clip(
            22050,
            2,
            (0..2000).map(|i| (i * 13 % 4000 - 2000) as i16).collect(),
        );
        let mut a = Mixer::new(44100, MixParams::default());
        let mut b = a.clone();
        a.play(req(&c, "x")).unwrap();
        b.play(req(&c, "x")).unwrap();
        let whole = render(&mut a, 3000);
        let mut parts = Vec::new();
        for n in [1, 999, 1000, 1000] {
            parts.extend(render(&mut b, n));
        }
        assert_eq!(whole, parts);
    }

    #[test]
    fn clip_from_pcm_rejects_unplayable_shapes() {
        let ok = Pcm {
            sample_rate: 8000,
            channels: 2,
            samples: vec![1, 2, 3],
        };
        assert_eq!(Clip::from_pcm(ok).unwrap().samples, [1, 2]);
        for (rate, ch) in [(0, 1), (8000, 0), (8000, 3)] {
            let p = Pcm {
                sample_rate: rate,
                channels: ch,
                samples: vec![0; 12],
            };
            assert!(Clip::from_pcm(p).is_none());
        }
    }
}
