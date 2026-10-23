//! The software mixer of S-0701 claims 5 to 15: sound instances, their
//! placement against the battlefield view (Formulas F2), the per-entry limit
//! (claim 8), sixteen channels with priority (claim 9), the instance pool
//! (claim 10), audio passes on the audio clock (claim 12), the per-instance
//! draws and sample order (claims 13 and 14), and the level and pan law
//! (Formulas F3 and F4). Announcements play on a voice of their own
//! (claim 27).
//!
//! Pure Rust and integer arithmetic apart from F2's single-precision level,
//! which is formed exactly, so it renders offline and the tests check exact
//! samples. Sound never feeds back into the simulation (claim 29): the mixer
//! only reads the clips and positions it is given, and its draws come from
//! the stream it holds, which [`crate::Audio`] makes the client's
//! unsynchronised stream (claim 30), never the scenario random stream.

use std::collections::VecDeque;
use std::sync::Arc;

use glasnost_fmt::pcm::Pcm;
use glasnost_sim::rng::Rng;
use glasnost_sim::world::Pos;

use crate::catalog::{EntryId, FULL, Priority, SoundEntry};
use crate::music::MusicVoice;

/// Channels that sound instances play on (S-0701 Constants).
pub const CHANNELS: usize = 16; // spec: S-0701 claim 9 and Constants (16 channels)
/// Most sound instances at once (S-0701 Constants).
pub const POOL: usize = 300; // spec: S-0701 claim 10 and Constants (300 instances)
/// Most follow-placed positional instances at once (S-0701 Constants).
pub const FOLLOW_POOL: usize = 200; // spec: S-0701 claim 10 and Constants (200 positional one-shots)
/// Audio pass interval: a pass needs more than this many ms (S-0701
/// Constants).
pub const PASS_MS: i64 = 33; // spec: S-0701 claim 12 and Constants (more than 33 ms)
/// Smallest effective delay, and the ambient pre-delay minimum (S-0701
/// Constants).
pub const MIN_DELAY_MS: i32 = 33; // spec: S-0701 claims 13 and 14, Constants (33 ms)
/// Centre pan.
pub const CENTRE: i32 = 8192; // spec: S-0701 Terms (pan 0 … 16384; 8192 is centre)
/// Pixels per cell of range (S-0701 Constants).
const PIXELS_PER_CELL: i64 = 60; // spec: S-0701 F2 and Constants (R = 60 × range)

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

/// The battlefield view a positional sound is placed against (F2).
#[derive(Clone, Copy, Debug)]
pub struct View {
    /// World pixel at the view's top-left corner.
    pub origin: (i32, i32),
    /// Width *W* and height *H* in pixels.
    pub size: (i32, i32),
    /// World pixel of a map position (leptons, with its height).
    pub project: fn(i32, i32, i32) -> (i32, i32),
}

fn flat(x: i32, y: i32, _z: i32) -> (i32, i32) {
    (x, y)
}

impl Default for View {
    /// No view yet: every positional sound is out of range.
    fn default() -> Self {
        Self {
            origin: (0, 0),
            size: (0, 0),
            project: flat,
        }
    }
}

/// Where an instance plays (S-0701 claims 5 and 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Where {
    /// A screen sound (claim 5).
    Screen,
    /// A direct positional request: it keeps its start level and pan
    /// (claim 7).
    Fixed(Pos),
    /// A positional request that keeps its map position and is placed again
    /// at audio passes (claim 7).
    Follow(Pos),
    /// An attached sound, placed again when its anchor moves and at passes
    /// (claim 7).
    Attached {
        /// The anchor (an object, by the caller's numbering).
        anchor: u64,
        /// Its last position.
        pos: Pos,
    },
}

impl Where {
    fn pos(self) -> Option<Pos> {
        match self {
            Where::Screen => None,
            Where::Fixed(p) | Where::Follow(p) | Where::Attached { pos: p, .. } => Some(p),
        }
    }
}

/// F2: the request level ℓ and pan of a position, or `None` when its level
/// fraction is below 0.05 (claim 6).
pub fn place(e: &SoundEntry, view: &View, pos: Pos) -> Option<(i32, i32)> {
    let (w, h) = (i64::from(view.size.0), i64::from(view.size.1));
    if w <= 0 || h <= 0 {
        return None;
    }
    let (wx, wy) = (view.project)(pos.x, pos.y, pos.z);
    let px = i64::from(wx) - i64::from(view.origin.0);
    let py = i64::from(wy) - i64::from(view.origin.1);
    // Half pixels throughout, so W/2 and H/2 are exact.
    // spec: S-0701 F2 (δ = pₓ − W/2; uₓ = |trunc(δ)|, uᵧ = |trunc(pᵧ − H/2)|)
    let d2 = 2 * px - w;
    let ux = (d2 / 2).abs();
    let uy = ((2 * py - h) / 2).abs();
    let (ax2, ay2) = if e.placement.local {
        // spec: S-0701 F2 (with local: aₓ = uₓ, aᵧ = 2 × uᵧ)
        (2 * ux, 4 * uy)
    } else {
        // spec: S-0701 F2 (otherwise aₓ = max(0, uₓ − W/2), aᵧ = 2 × max(0, uᵧ − H/2))
        ((2 * ux - w).max(0), 2 * (2 * uy - h).max(0))
    };
    // spec: S-0701 F2 (R = 60 × range pixels)
    let r2 = 2 * PIXELS_PER_CELL * i64::from(e.range);
    // spec: S-0701 F2 (L₀ = (R − max(aₓ, aᵧ)) / R when aₓ < R, aᵧ < R and R > 0, else 0; stored in single precision toward zero)
    let l0 = if ax2 < r2 && ay2 < r2 && r2 > 0 {
        ratio_rz(r2 - ax2.max(ay2), r2)
    } else {
        0.0
    };
    // spec: S-0701 F2 (L = max(L₀, minimum level) with global)
    let l = if e.placement.global {
        l0.max(e.min_level)
    } else {
        l0
    };
    // spec: S-0701 F2 and claim 6 (not started when L < 0.05, compared in double precision)
    if f64::from(l) < 0.05 {
        return None;
    }
    // spec: S-0701 F2 (ℓ = min(trunc(L × 16384), 16384))
    let level = ((f64::from(l) * f64::from(FULL)) as i32).min(FULL);
    // spec: S-0701 F2 (pan = trunc(clamp(δ, −W, W) × 8192 / W + 8192))
    let dc = d2.clamp(-2 * w, 2 * w);
    let pan = (4096 * dc + 8192 * w).div_euclid(w);
    Some((level, pan as i32))
}

/// n / d (0 ≤ n ≤ d, d > 0) rounded toward zero to single precision, exactly.
fn ratio_rz(n: i64, d: i64) -> f32 {
    if n <= 0 {
        return 0.0;
    }
    if n >= d {
        return 1.0;
    }
    let (n, d) = (i128::from(n), i128::from(d));
    let mut k = 0u32;
    while (n << k) < (d << 23) {
        k += 1;
    }
    // 2²³ ≤ mantissa < 2²⁴: 24 significant bits, truncated.
    let m = (n << k) / d;
    (m as f64 / 2f64.powi(k as i32)) as f32
}

/// F4's attenuation T(k) in hundredths of a decibel, for k = 0 … 100.
pub fn attenuation(k: i32) -> i32 {
    // spec: S-0701 F4 (T(0) = −10000, silence)
    if k <= 0 {
        return -10000;
    }
    // spec: S-0701 F4 (T(k) = round-nearest(1000 × log₂(k / 100)); no argument is a tie)
    (1000.0 * (f64::from(k.min(100)) / 100.0).log2()).round() as i32
}

/// The amplitude of an attenuation in hundredths of a decibel, 65536 =
/// unity: 10^(A / 2000); silence at or below T(0).
pub fn amplitude(a: i32) -> i64 {
    if a <= -10000 {
        return 0;
    }
    (65536.0 * 10f64.powf(f64::from(a) / 2000.0)).round() as i64
}

/// F3's heard level h.
pub fn heard(e: i32, l: i32, s: i32, g: i32, o: i32) -> i32 {
    let (e, l, s, g, o) = (
        i64::from(e),
        i64::from(l),
        i64::from(s),
        i64::from(g),
        i64::from(o),
    );
    let full = i64::from(FULL);
    // spec: S-0701 F3 (σ = 16384 − ⌊s × 16384 / 100⌋)
    let sigma = full - s * full / 100;
    // spec: S-0701 F3 (h = ⌊⌊⌊⌊e × ℓ / 16384⌋ × σ / 16384⌋ × g / 16384⌋ × o / 16384⌋)
    (((e * l / full) * sigma / full) * g / full * o / full) as i32
}

/// F4: left and right amplitudes (65536 = unity) of a heard level and pan.
pub fn gains(h: i32, pan: i32) -> (i64, i64) {
    // spec: S-0701 F4 (k = ⌊h × 100 / 16384⌋)
    let k = (i64::from(h) * 100 / i64::from(FULL)) as i32;
    if k <= 0 {
        return (0, 0);
    }
    let t = attenuation(k);
    // spec: S-0701 F4 (q = ⌊pan × 200 / 16384⌋ − 100)
    let q = (i64::from(pan.clamp(0, FULL)) * 200 / i64::from(FULL)) as i32 - 100;
    // spec: S-0701 F4 (q < 0: the right side attenuated by −T(100 + q); q ≥ 0: the left side by −T(100 − q))
    let (bl, br) = if q < 0 {
        (0, attenuation(100 + q))
    } else {
        (attenuation(100 - q), 0)
    };
    (amplitude(t + bl), amplitude(t + br))
}

/// A sound entry ready to play: the entry and the clips of its samples, in
/// order.
#[derive(Clone, Debug)]
pub struct Prepared {
    /// The entry.
    pub id: EntryId,
    /// Its settings.
    pub entry: Arc<SoundEntry>,
    /// Its loaded samples.
    pub clips: Vec<Arc<Clip>>,
}

/// An instance, by number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceId(pub u64);

/// Why a request plays nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// Its placement is below 0.05 (claim 6).
    OutOfRange,
    /// The instance pool is full (claim 10).
    PoolFull,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    /// Waiting for a pass to get a channel.
    Requested,
    /// On a channel, waiting through its pre-delay until the audio clock
    /// reaches this time.
    Waiting(i64),
    /// On a channel, playing.
    Playing,
    /// An indefinite loop out of range: no channel, placed again at passes
    /// (claim 7).
    Dormant,
    /// Ended or cut.
    Dead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Attack,
    Body,
    Decay,
    Done,
}

/// The sample order of one instance (claim 14).
#[derive(Clone, Debug)]
struct Sequence {
    attack: Option<usize>,
    body: Vec<usize>,
    one: Option<usize>,
    all: bool,
    random: bool,
    order: VecDeque<usize>,
    passes_left: Option<i32>,
    decay: Option<usize>,
    stage: Stage,
}

#[derive(Clone, Debug)]
struct Voice {
    clip: Arc<Clip>,
    /// Position in the clip, frames in 32.32 fixed point.
    pos: u64,
    /// Clip frames per output frame, 32.32.
    step: u64,
}

#[derive(Clone, Debug)]
struct Instance {
    id: u64,
    prep: Prepared,
    at: Where,
    /// Request level ℓ.
    level: i32,
    pan: i32,
    phase: Phase,
    channel: Option<usize>,
    pitch: i32,
    shift: i32,
    seq: Option<Sequence>,
    voice: Option<Voice>,
}

impl Instance {
    fn alive(&self) -> bool {
        self.phase != Phase::Dead
    }

    /// The level used for ranking: ⌊e × ℓ / 16384⌋.
    fn rank_level(&self) -> i32 {
        (i64::from(self.prep.entry.level) * i64::from(self.level) / i64::from(FULL)) as i32
    }

    fn indefinite(&self) -> bool {
        let e = &self.prep.entry;
        e.control.looped && e.loop_count == 0
    }
}

/// Settings of the mixer that come from the rules' quirk switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MixQuirks {
    /// `quirks.sound_order_startup_draw` (S-0701 claim 14).
    pub sound_order_startup_draw: bool,
    /// `quirks.ranged_draw_half_span` (S-0202 claim 5).
    pub ranged_draw_half_span: bool,
}

impl Default for MixQuirks {
    fn default() -> Self {
        // spec: S-0701 Edge cases, S-0202 Edge cases (on by default)
        Self {
            sound_order_startup_draw: true,
            ranged_draw_half_span: true,
        }
    }
}

/// The announcement voice (claim 27).
#[derive(Clone, Debug)]
struct Speech {
    voice: Voice,
}

/// The mixer.
#[derive(Clone, Debug)]
pub struct Mixer {
    rate: u32,
    quirks: MixQuirks,
    view: View,
    instances: Vec<Instance>,
    channels: [Option<u64>; CHANNELS],
    next: u64,
    rng: Rng,
    now: i64,
    last_pass: Option<i64>,
    /// The master level g of F3.
    master: i32,
    /// The player's sound volume option o of F3.
    option: i32,
    /// The player's voice volume option, 0 … 16384 (announcements).
    voice_option: i32,
    speech: Option<Speech>,
    /// The music track playing (S-0703), and our own music volume,
    /// 0 … 16384.
    music: Option<MusicVoice>,
    music_option: i32,
    acc: Vec<i64>,
}

impl Mixer {
    /// A mixer rendering stereo at `output_rate` frames per second, drawing
    /// its per-instance choices from an S-0202 stream seeded with `seed`.
    /// [`crate::Audio`] lends it the client's unsynchronised stream instead
    /// while it services (claim 30).
    pub fn new(output_rate: u32, seed: u32, quirks: MixQuirks) -> Self {
        Self {
            rate: output_rate.max(1),
            quirks,
            view: View::default(),
            instances: Vec::new(),
            channels: [None; CHANNELS],
            next: 0,
            // spec: S-0701 claim 30 (per-instance draws on the unsynchronised stream, lent by the caller)
            rng: Rng::new(seed),
            now: 0,
            last_pass: None,
            master: FULL,
            // spec: S-0701 F3 (o = trunc(option fraction × 16384); full volume)
            option: FULL,
            // spec: S-0701 claim 35 (the voice option is at full unless the player sets it)
            voice_option: FULL,
            speech: None,
            music: None,
            // nocite: our own UI (the music volume is at full unless the person sets it)
            music_option: FULL,
            acc: Vec::new(),
        }
    }

    /// The output rate.
    pub fn output_rate(&self) -> u32 {
        self.rate
    }

    /// Restarts the presentation generator (a new game).
    pub fn reseed(&mut self, seed: u32) {
        self.rng = Rng::new(seed);
    }

    /// The stream the per-instance draws take (tests).
    pub fn rng(&self) -> &Rng {
        &self.rng
    }

    /// The stream the per-instance draws take, to lend it the shared
    /// unsynchronised stream (claim 30).
    pub fn rng_mut(&mut self) -> &mut Rng {
        &mut self.rng
    }

    /// Sets the battlefield view (call every frame).
    pub fn set_view(&mut self, v: View) {
        self.view = v;
    }

    /// The view.
    pub fn view(&self) -> &View {
        &self.view
    }

    /// Sets the player's sound volume option, 0 … 16384 (F3's o).
    pub fn set_volume(&mut self, o: i32) {
        self.option = o.clamp(0, FULL);
    }

    /// Sets the player's voice volume option for announcements, 0 … 16384
    /// (S-0701 claim 35 keeps it at full by default).
    pub fn set_voice_volume(&mut self, o: i32) {
        self.voice_option = o.clamp(0, FULL);
    }

    /// Sets our own music volume, 0 … 16384 (S-0703 leaves the original's
    /// scaling open). At 0 the track still plays, silently (S-0703 claim
    /// 3).
    pub fn set_music_volume(&mut self, o: i32) {
        self.music_option = o.clamp(0, FULL);
    }

    /// The music volume.
    pub fn music_volume(&self) -> i32 {
        self.music_option
    }

    /// Starts a music track on the music voice, replacing any, or stops the
    /// music (`None`).
    pub fn set_music(&mut self, v: Option<MusicVoice>) {
        self.music = v;
    }

    /// Whether a music track is sounding.
    pub fn has_music(&self) -> bool {
        self.music.is_some()
    }

    /// The master level g of F3.
    pub fn master(&self) -> i32 {
        self.master
    }

    /// Instances that exist (requested, waiting, playing or dormant).
    pub fn alive(&self) -> usize {
        self.instances.iter().filter(|i| i.alive()).count()
    }

    /// Whether an instance still exists.
    pub fn is_alive(&self, id: InstanceId) -> bool {
        self.instances.iter().any(|i| i.id == id.0 && i.alive())
    }

    /// Whether an instance is on a channel and audible now.
    pub fn is_playing(&self, id: InstanceId) -> bool {
        self.instances
            .iter()
            .any(|i| i.id == id.0 && i.phase == Phase::Playing)
    }

    /// Channels in use.
    pub fn busy_channels(&self) -> usize {
        self.channels.iter().filter(|c| c.is_some()).count()
    }

    /// The entries of the instances on channels, in channel order.
    pub fn channel_entries(&self) -> Vec<Option<EntryId>> {
        self.channels
            .iter()
            .map(|c| c.and_then(|id| self.get(id).map(|i| i.prep.id)))
            .collect()
    }

    /// The request level and pan of an instance.
    pub fn placement(&self, id: InstanceId) -> Option<(i32, i32)> {
        self.get(id.0).map(|i| (i.level, i.pan))
    }

    /// The pitch and volume shift an instance drew (claim 13).
    pub fn draws(&self, id: InstanceId) -> Option<(i32, i32)> {
        self.get(id.0).map(|i| (i.pitch, i.shift))
    }

    /// The sample order still to play of an instance, current body pass
    /// only (tests).
    pub fn sample_order(&self, id: InstanceId) -> Option<Vec<usize>> {
        let i = self.get(id.0)?;
        let s = i.seq.as_ref()?;
        let mut v = Vec::new();
        if s.stage == Stage::Attack {
            v.extend(s.attack);
        }
        if matches!(s.stage, Stage::Attack | Stage::Body) {
            v.extend(s.order.iter().copied());
        }
        if s.stage != Stage::Done {
            v.extend(s.decay);
        }
        Some(v)
    }

    fn get(&self, id: u64) -> Option<&Instance> {
        self.instances.iter().find(|i| i.id == id && i.alive())
    }

    /// Requests an instance (S-0701 claims 5, 6 and 10). `scale` is the
    /// request level of a screen sound (16384 = full); positional requests
    /// take theirs from F2. The instance gets a channel at the next pass.
    pub fn request(
        &mut self,
        prep: Prepared,
        at: Where,
        scale: i32,
    ) -> Result<InstanceId, Refused> {
        let (level, pan) = match at.pos() {
            // spec: S-0701 claim 5 and F2 (a screen sound: ℓ = 16384, pan 8192)
            None => (scale.clamp(0, FULL), CENTRE),
            // spec: S-0701 claim 6 (below 0.05: not started, no draws)
            Some(p) => place(&prep.entry, &self.view, p).ok_or(Refused::OutOfRange)?,
        };
        // spec: S-0701 claim 10 (a request when the pool is full, after dead instances are cleared, plays nothing)
        self.instances.retain(Instance::alive);
        if self.instances.len() >= POOL {
            return Err(Refused::PoolFull);
        }
        if matches!(at, Where::Follow(_))
            && self
                .instances
                .iter()
                .filter(|i| matches!(i.at, Where::Follow(_)))
                .count()
                >= FOLLOW_POOL
        {
            return Err(Refused::PoolFull);
        }
        let id = self.next;
        self.next += 1;
        self.instances.push(Instance {
            id,
            prep,
            at,
            level,
            pan,
            phase: Phase::Requested,
            channel: None,
            pitch: 100,
            shift: 0,
            seq: None,
            voice: None,
        });
        Ok(InstanceId(id))
    }

    /// Moves an anchor's attached sounds and places them again (claim 7).
    pub fn move_anchor(&mut self, anchor: u64, pos: Pos) {
        let view = self.view;
        for i in 0..self.instances.len() {
            if let Where::Attached { anchor: a, .. } = self.instances[i].at
                && a == anchor
                && self.instances[i].alive()
            {
                self.instances[i].at = Where::Attached { anchor, pos };
                self.replace_one(i, &view);
            }
        }
    }

    /// Asks an anchor's attached sounds to finish looping: the current body
    /// pass ends, then the decay plays (claim 23).
    pub fn finish_anchor(&mut self, anchor: u64) {
        for i in &mut self.instances {
            if let Where::Attached { anchor: a, .. } = i.at
                && a == anchor
                && i.alive()
            {
                match &mut i.seq {
                    Some(s) => s.passes_left = Some(0),
                    None => {
                        i.phase = Phase::Dead;
                        if let Some(c) = i.channel.take() {
                            self.channels[c] = None;
                        }
                    }
                }
            }
        }
    }

    /// Stops an instance at once.
    pub fn stop(&mut self, id: InstanceId) {
        if let Some(i) = self.instances.iter().position(|i| i.id == id.0) {
            self.kill(i);
        }
    }

    fn kill(&mut self, i: usize) {
        let inst = &mut self.instances[i];
        inst.phase = Phase::Dead;
        inst.voice = None;
        if let Some(c) = inst.channel.take() {
            self.channels[c] = None;
        }
    }

    /// Stops every instance, the announcement and the music.
    pub fn clear(&mut self) {
        self.instances.clear();
        self.channels = [None; CHANNELS];
        self.speech = None;
        self.music = None;
    }

    /// Places a follow-placed or attached instance again (claim 7).
    fn replace_one(&mut self, i: usize, view: &View) {
        let Some(p) = self.instances[i].at.pos() else {
            return;
        };
        if matches!(self.instances[i].at, Where::Fixed(_)) {
            return;
        }
        match place(&self.instances[i].prep.entry, view, p) {
            Some((level, pan)) => {
                let inst = &mut self.instances[i];
                inst.level = level;
                inst.pan = pan;
                if inst.phase == Phase::Dormant {
                    // spec: S-0701 claim 7 (an indefinitely looping entry can restart when audible again)
                    inst.phase = Phase::Requested;
                }
            }
            None => {
                // spec: S-0701 claim 7 (updated placement below 0.05 stops a playing instance; an indefinite loop can restart later)
                if self.instances[i].indefinite() {
                    let inst = &mut self.instances[i];
                    inst.phase = Phase::Dormant;
                    inst.voice = None;
                    inst.seq = None;
                    if let Some(c) = inst.channel.take() {
                        self.channels[c] = None;
                    }
                } else {
                    self.kill(i);
                }
            }
        }
    }

    /// Advances the audio clock to `now_ms` and runs an audio pass when more
    /// than 33 ms have passed since the last one (claim 12). Returns whether
    /// a pass ran.
    pub fn service(&mut self, now_ms: i64) -> bool {
        self.now = now_ms;
        self.start_waiting();
        // spec: S-0701 claim 12 and 30 (a pass is eligible only when more than 33 ms of the audio clock have elapsed since the preceding one; the interval is a lower bound)
        if self.last_pass.is_some_and(|t| now_ms - t <= PASS_MS) {
            return false;
        }
        self.last_pass = Some(now_ms);
        self.pass();
        true
    }

    /// Instances whose pre-delay is over start their first sample.
    fn start_waiting(&mut self) {
        for i in 0..self.instances.len() {
            if let Phase::Waiting(until) = self.instances[i].phase
                && self.now >= until
            {
                self.instances[i].phase = Phase::Playing;
                self.advance(i);
            }
        }
    }

    /// One audio pass (claims 7 to 13, F3).
    fn pass(&mut self) {
        self.instances.retain(Instance::alive);
        let view = self.view;
        // spec: S-0701 claim 7 (follow-placed and attached instances are placed again at passes)
        for i in 0..self.instances.len() {
            if matches!(
                self.instances[i].at,
                Where::Follow(_) | Where::Attached { .. }
            ) {
                self.replace_one(i, &view);
            }
        }
        // spec: S-0701 Ordering (the per-entry limit is applied before channels are given out)
        self.limit();
        let pending: Vec<usize> = (0..self.instances.len())
            .filter(|&i| self.instances[i].phase == Phase::Requested)
            .collect();
        for i in pending {
            self.allocate(i);
        }
        self.instances.retain(Instance::alive);
        // spec: S-0701 F3 (C = Σ ⌊20 × e / 16384⌋ over surviving instances; target 16384 when C ≤ 100, else ⌊1638400 / C⌋)
        let c: i64 = self
            .instances
            .iter()
            .filter(|i| matches!(i.phase, Phase::Playing | Phase::Waiting(_)))
            .map(|i| 20 * i64::from(i.prep.entry.level) / i64::from(FULL))
            .sum();
        // interim: Q-0333 (the master level reaches its target at once)
        self.master = if c <= 100 {
            FULL
        } else {
            (1_638_400 / c) as i32
        };
    }

    /// The per-entry limit (claim 8).
    fn limit(&mut self) {
        let mut entries: Vec<EntryId> = self
            .instances
            .iter()
            .filter(|i| i.alive())
            .map(|i| i.prep.id)
            .collect();
        entries.sort();
        entries.dedup();
        for e in entries {
            loop {
                let of: Vec<usize> = (0..self.instances.len())
                    .filter(|&i| self.instances[i].alive() && self.instances[i].prep.id == e)
                    .collect();
                let Some(&first) = of.first() else { break };
                let entry = self.instances[first].prep.entry.clone();
                // spec: S-0701 claim 8 (limit 0: no limit); negative limits are not established (Open questions), and are treated as none
                if entry.limit <= 0 || of.len() <= entry.limit as usize {
                    break;
                }
                // spec: S-0701 claim 8 (levels within ⌊e / 10⌋ count as equally loud)
                let band = entry.level / 10;
                let interrupt = entry.control.interrupt;
                let mut low = first;
                for &c in &of[1..] {
                    if self.ranks_below(c, low, band, interrupt) {
                        low = c;
                    }
                }
                // spec: S-0701 claim 8 (the lowest-ranked is cut, stopped at once if playing)
                self.kill(low);
            }
        }
    }

    /// Whether instance `a` ranks below `b` (claim 8).
    fn ranks_below(&self, a: usize, b: usize, band: i32, interrupt: bool) -> bool {
        let (ia, ib) = (&self.instances[a], &self.instances[b]);
        let (la, lb) = (ia.rank_level(), ib.rank_level());
        if (la - lb).abs() >= band.max(1) {
            // spec: S-0701 claim 8 (ranked loudest first by level)
            return la < lb;
        }
        let (na, nb) = (ia.phase == Phase::Requested, ib.phase == Phase::Requested);
        if na != nb {
            // spec: S-0701 claim 8 (equally loud: a new request ranks above the playing ones with interrupt, below them otherwise)
            return if interrupt { !na } else { na };
        }
        // Within one class: with interrupt the oldest is cut, otherwise the
        // newest (claim 8's two outcomes).
        if interrupt {
            ia.id < ib.id
        } else {
            ia.id > ib.id
        }
    }

    /// Gives a requested instance a channel, or discards it (claim 9).
    fn allocate(&mut self, i: usize) {
        let p = self.instances[i].prep.entry.priority;
        let ch = match self.channels.iter().position(Option::is_none) {
            // spec: S-0701 claim 9 (a free channel if there is one)
            Some(c) => Some(c),
            None => self.victim(p),
        };
        let Some(c) = ch else {
            // spec: S-0701 claim 9 (discarded; not retried)
            self.kill(i);
            return;
        };
        if let Some(old) = self.channels[c]
            && let Some(j) = self.instances.iter().position(|x| x.id == old)
        {
            // spec: S-0701 claim 9 (the chosen candidate is cut and replaced)
            self.kill(j);
        }
        self.channels[c] = Some(self.instances[i].id);
        self.instances[i].channel = Some(c);
        self.acquire(i);
    }

    /// The channel a new instance of priority `p` takes when all are busy,
    /// or `None` when it is discarded (claim 9).
    fn victim(&self, p: Priority) -> Option<usize> {
        let inst =
            |c: usize| self.channels[c].and_then(|id| self.instances.iter().find(|i| i.id == id));
        let mut playing: Option<(usize, Priority, i32)> = None;
        let mut waiting: Option<(usize, Priority)> = None;
        for c in 0..CHANNELS {
            let Some(i) = inst(c) else { continue };
            let pr = i.prep.entry.priority;
            if i.phase == Phase::Playing {
                let lv = heard(
                    i.prep.entry.level,
                    i.level,
                    i.shift,
                    self.master,
                    self.option,
                );
                playing = match playing {
                    None => Some((c, pr, lv)),
                    Some((_, bp, _)) if pr < bp => Some((c, pr, lv)),
                    // spec: S-0701 claim 9 (equal priority: a later channel replaces the candidate only if its level is lower by at least 1638/65536 of a unit; integer levels here, so strictly lower)
                    Some((_, bp, bl)) if pr == bp && lv < bl => Some((c, pr, lv)),
                    keep => keep,
                };
            } else {
                // spec: S-0701 claim 9 (among non-playing channels, equal priority keeps the first in channel order)
                waiting = match waiting {
                    Some((_, bp)) if bp <= pr => waiting,
                    _ => Some((c, pr)),
                };
            }
        }
        // spec: S-0701 claim 9 (the playing candidate only if its priority is strictly lower than the non-playing one's, or there is none)
        let (c, cp) = match (playing, waiting) {
            (Some((c, pp, _)), Some((_, wp))) if pp < wp => (c, pp),
            (_, Some((c, wp))) => (c, wp),
            (Some((c, pp, _)), None) => (c, pp),
            (None, None) => return None,
        };
        // spec: S-0701 claim 9 (a candidate of higher priority than the new instance: the new one is discarded)
        (cp <= p).then_some(c)
    }

    fn r(&mut self, a: i32, b: i32) -> i32 {
        self.rng.ranged(a, b, self.quirks.ranged_draw_half_span)
    }

    /// The draws of an instance that got its channel (claims 13 and 14,
    /// State and ordering), and its first sample or pre-delay.
    fn acquire(&mut self, i: usize) {
        let e = self.instances[i].prep.entry.clone();
        let n = self.instances[i].prep.clips.len();
        // spec: S-0701 claim 13 (pitch p = 100 + R(f₁, f₂))
        let pitch = 100 + self.r(e.fshift.0, e.fshift.1);
        // spec: S-0701 claim 13 (volume shift s = R(0, vs))
        let shift = self.r(0, e.vshift);
        // spec: S-0701 claim 13 (pre-delay R(a, d₂) only with pre-delay or ambient; a = 33 for ambient, else d₁)
        let predelay = if e.control.predelay || e.control.ambient {
            let a = if e.control.ambient {
                MIN_DELAY_MS
            } else {
                e.delay.0
            };
            Some(self.r(a, e.delay.1))
        } else {
            None
        };
        // The partition; oversized counts are not established (Open
        // questions) and are cut to the samples there are.
        let a = (e.attack.max(0) as usize).min(n);
        let d = (e.decay.max(0) as usize).min(n - a);
        let body: Vec<usize> = (a..n - d).collect();
        // spec: S-0701 claim 14 (one attack sample R(0, A − 1) if A > 0)
        let attack = (a > 0).then(|| self.r(0, a as i32 - 1) as usize);
        let random = e.control.random;
        let all = e.control.all;
        // spec: S-0701 claim 14 (random without all: one body sample R(A, n − D − 1); otherwise the first body sample)
        let one = if body.is_empty() {
            None
        } else if random && !all {
            Some(self.r(a as i32, (n - d) as i32 - 1) as usize)
        } else {
            Some(a)
        };
        // spec: S-0701 claim 14 (one decay sample R(n − D, n − 1) if D > 0)
        let decay = (d > 0).then(|| self.r((n - d) as i32, n as i32 - 1) as usize);
        // spec: S-0701 claim 14 (immediate mode: the delay's first number below 33)
        let immediate = e.delay.0 < MIN_DELAY_MS;
        // spec: S-0701 claim 14 and Edge cases (quirks.sound_order_startup_draw: random, immediate, no attack sample: one discarded R(0, m − 1))
        // interim: Q-0329 (delayed mode takes no startup draw)
        if random && immediate && a == 0 && self.quirks.sound_order_startup_draw && !body.is_empty()
        {
            let _ = self.r(0, body.len() as i32 - 1);
        }
        // spec: S-0701 claim 14 (with loop: forever when the loop count is 0, else that many passes in total)
        let passes_left = if e.control.looped {
            (e.loop_count > 0).then_some(e.loop_count - 1)
        } else {
            Some(0)
        };
        let mut seq = Sequence {
            attack,
            body,
            one,
            all,
            random,
            order: VecDeque::new(),
            passes_left,
            decay,
            stage: Stage::Attack,
        };
        seq.order = self.body_pass(&seq);
        let inst = &mut self.instances[i];
        inst.pitch = pitch;
        inst.shift = shift;
        inst.seq = Some(seq);
        // spec: S-0701 claim 13 (a pre-delay of 33 ms or more is waited; a shorter one is no wait)
        match predelay {
            Some(t) if t >= MIN_DELAY_MS => inst.phase = Phase::Waiting(self.now + i64::from(t)),
            _ => {
                inst.phase = Phase::Playing;
                self.advance(i);
            }
        }
    }

    /// The samples of one body pass (claim 14).
    fn body_pass(&mut self, s: &Sequence) -> VecDeque<usize> {
        if !s.all {
            // spec: S-0701 claim 14 (a body sample chosen once by random is the one repeated)
            return s.one.into_iter().collect();
        }
        if !s.random {
            // spec: S-0701 claim 14 (all: every body sample in list order)
            return s.body.iter().copied().collect();
        }
        // spec: S-0701 claim 14 (all and random: each next one is R(0, m − 1) among the m not yet played)
        let mut left = s.body.clone();
        let mut out = VecDeque::new();
        while !left.is_empty() {
            let k = self.r(0, left.len() as i32 - 1) as usize;
            out.push_back(left.remove(k));
        }
        out
    }

    /// Starts the instance's next sample, or ends it.
    fn advance(&mut self, i: usize) {
        let next = loop {
            let Some(mut s) = self.instances[i].seq.take() else {
                break None;
            };
            let got = match s.stage {
                Stage::Attack => {
                    s.stage = Stage::Body;
                    s.attack
                }
                Stage::Body => match s.order.pop_front() {
                    Some(x) => Some(x),
                    None => {
                        let more = s.passes_left.is_none_or(|p| p > 0) && !s.body.is_empty();
                        if more {
                            if let Some(p) = s.passes_left.as_mut() {
                                *p -= 1;
                            }
                            s.order = self.body_pass(&s);
                        } else {
                            s.stage = Stage::Decay;
                        }
                        None
                    }
                },
                Stage::Decay => {
                    s.stage = Stage::Done;
                    s.decay
                }
                Stage::Done => {
                    self.instances[i].seq = Some(s);
                    break None;
                }
            };
            self.instances[i].seq = Some(s);
            if let Some(x) = got {
                break Some(x);
            }
        };
        match next {
            Some(x) => {
                let inst = &mut self.instances[i];
                let clip = inst.prep.clips[x].clone();
                // spec: S-0701 F4 (playback rate = trunc(rate × pitch / 100)); F3 (pitches combine as trunc(p₁ × p₂ / 100))
                let rate = (u64::from(clip.sample_rate) * inst.pitch.max(1) as u64) / 100;
                let step = ((rate << 32) / u64::from(self.rate)).max(1);
                inst.voice = Some(Voice { clip, pos: 0, step });
            }
            None => self.kill(i),
        }
    }

    /// Starts an announcement's sample on the announcement voice, replacing
    /// any (claim 27).
    pub fn speak(&mut self, clip: Arc<Clip>) {
        let step = ((u64::from(clip.sample_rate) << 32) / u64::from(self.rate)).max(1);
        self.speech = Some(Speech {
            voice: Voice { clip, pos: 0, step },
        });
    }

    /// Whether an announcement is playing.
    pub fn speaking(&self) -> bool {
        self.speech.is_some()
    }

    /// Stops the announcement.
    pub fn stop_speech(&mut self) {
        self.speech = None;
    }

    /// Mixes the next `out.len() / 2` stereo frames (interleaved left,
    /// right) into `out`, overwriting it, and advances every playing
    /// instance and the announcement. The sum is clipped to 16 bits.
    pub fn render(&mut self, out: &mut [i16]) {
        let frames = out.len() / 2;
        self.acc.clear();
        self.acc.resize(frames * 2, 0);
        for c in 0..CHANNELS {
            let Some(id) = self.channels[c] else { continue };
            let Some(i) = self.instances.iter().position(|x| x.id == id) else {
                self.channels[c] = None;
                continue;
            };
            if self.instances[i].phase != Phase::Playing {
                continue;
            }
            let inst = &self.instances[i];
            let h = heard(
                inst.prep.entry.level,
                inst.level,
                inst.shift,
                self.master,
                self.option,
            );
            let (gl, gr) = gains(h, inst.pan);
            let mut f = 0;
            while f < frames {
                let Some(v) = self.instances[i].voice.as_mut() else {
                    break;
                };
                f = mix(v, &mut self.acc, f, frames, gl, gr);
                if f < frames || (v.pos >> 32) as usize >= v.clip.frames() {
                    // The sample ended: the next one (sample completion
                    // is serviced here, claim 12).
                    self.advance(i);
                    if !self.instances[i].alive() {
                        break;
                    }
                }
            }
        }
        if let Some(s) = self.speech.as_mut() {
            // spec: S-0701 claim 35 (announcements skip the master level and the sound volume option; the voice option is at full, centred)
            let unity = 1i64 << 16;
            // The player's voice option scales the announcement (a client setting; at full it is unity).
            let g = unity * i64::from(self.voice_option) / i64::from(FULL);
            mix(&mut s.voice, &mut self.acc, 0, frames, g, g);
            if (s.voice.pos >> 32) as usize >= s.voice.clip.frames() {
                self.speech = None;
            }
        }
        if let Some(m) = self.music.as_mut() {
            // nocite: our own UI (music skips the master level and the sound option; only our music volume scales it, Q-1700 question 5 open)
            m.mix(&mut self.acc, crate::music::music_gain(self.music_option));
            if m.finished() {
                self.music = None;
            }
        }
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

/// Mixes a voice into `acc` from frame `from` until the buffer or the clip
/// ends; returns the frame it stopped at.
fn mix(v: &mut Voice, acc: &mut [i64], from: usize, frames: usize, gl: i64, gr: i64) -> usize {
    let clip = &v.clip;
    let n = clip.frames();
    let mut f = from;
    while f < frames {
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
        acc[2 * f] += (sl * gl) >> 16;
        acc[2 * f + 1] += (sr * gr) >> 16;
        v.pos += v.step;
        f += 1;
    }
    f
}

#[cfg(test)]
#[path = "mixer_tests.rs"]
mod tests;
