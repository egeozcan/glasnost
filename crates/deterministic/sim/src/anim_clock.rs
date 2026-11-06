//! The stage clock of an animation in play (S-0109 claims 1 to 16): the
//! stage, step, remaining passes, stage delay, loop delay and follow-up
//! switch of one playing animation, advanced update by update with the game
//! ticks (never the wall clock).
//!
//! The simulation plays every animation in play with it, its draws from the
//! scenario stream (S-0109 claim 28): ordinary animations (`crate::anims`),
//! infantry-making animations (S-0611 claims 8 to 12, `crate::mutation`),
//! domination animations and depot slot animations. The drawing
//! (`glasnost_app::mapview::anims`) shows the simulation's stage for those
//! and plays with the same clock, and a cosmetic stream of its own, only the
//! pictures it makes without a simulated animation behind them.

use serde::{Deserialize, Serialize};

/// The switches of S-0109's playback that the drawing implements, each
/// defaulting to the original's behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimQuirks {
    /// `quirks.anim_loop_count_wrap` (claim 7): the remaining-pass count is
    /// taken modulo 256 and 255 means endless. Off: an unbounded count,
    /// endless for a negative loop count.
    pub anim_loop_count_wrap: bool,
    /// `quirks.anim_reverse_raw_loop_end` (claim 8): a reversed animation
    /// starts at λₑ − 1, restarts at λₑ and ends its pass at k ≤ 0. Off: it
    /// starts and restarts at λₑ − a − 1 and ends its pass at k < 0.
    pub anim_reverse_raw_loop_end: bool,
    /// `quirks.anim_pingpong_lower_turn` (claim 9): while r ≥ 2 the lower
    /// turn compares k with the start field a. Off: k ≤ λₛ − a.
    pub anim_pingpong_lower_turn: bool,
    /// `quirks.anim_loop_end_from_sprite` (claims 6 and 25): an absent loop
    /// end takes the sprite end Eₛ. Off: it takes the end field E.
    pub anim_loop_end_from_sprite: bool,
    /// `quirks.anim_followup_start_doubled` (claim 15): the follow-up type
    /// starts at k = a′. Off: at its k₀ of F3.
    pub anim_followup_start_doubled: bool,
    /// `quirks.anim_followup_keeps_step` (claim 15): σ keeps its sign at a
    /// follow-up switch. Off: σ is set afresh from the reversed test.
    pub anim_followup_keeps_step: bool,
}

impl Default for AnimQuirks {
    fn default() -> Self {
        // spec: S-0109 Edge cases and quirks (every switch defaults on)
        Self {
            anim_loop_count_wrap: true,
            anim_reverse_raw_loop_end: true,
            anim_pingpong_lower_turn: true,
            anim_loop_end_from_sprite: true,
            anim_followup_start_doubled: true,
            anim_followup_keeps_step: true,
        }
    }
}

impl From<&glasnost_rules::quirks::Quirks> for AnimQuirks {
    /// The clock's switches as the ruleset sets them (`quirks.anim_*`).
    fn from(q: &glasnost_rules::quirks::Quirks) -> Self {
        // spec: S-0109 Edge cases and quirks (the six clock switches, read from the rules' quirks)
        Self {
            anim_loop_count_wrap: q.anim_loop_count_wrap,
            anim_reverse_raw_loop_end: q.anim_reverse_raw_loop_end,
            anim_pingpong_lower_turn: q.anim_pingpong_lower_turn,
            anim_loop_end_from_sprite: q.anim_loop_end_from_sprite,
            anim_followup_start_doubled: q.anim_followup_start_doubled,
            anim_followup_keeps_step: q.anim_followup_keeps_step,
        }
    }
}

/// The number of animation quirk switches.
pub const ANIM_SWITCHES: usize = 6; // spec: S-0109 Edge cases and quirks (six switches)

impl AnimQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); ANIM_SWITCHES] {
        [
            ("anim_loop_count_wrap", self.anim_loop_count_wrap),
            ("anim_reverse_raw_loop_end", self.anim_reverse_raw_loop_end),
            ("anim_pingpong_lower_turn", self.anim_pingpong_lower_turn),
            ("anim_loop_end_from_sprite", self.anim_loop_end_from_sprite),
            (
                "anim_followup_start_doubled",
                self.anim_followup_start_doubled,
            ),
            ("anim_followup_keeps_step", self.anim_followup_keeps_step),
        ]
    }
}

/// One animation type's playback fields in play (S-0109 Terms, F2).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Playback {
    /// Start field *a*.
    pub start: i32,
    /// End field *E* in play.
    pub end: i32,
    /// Loop start λₛ.
    pub loop_start: i32,
    /// Loop end λₑ in play.
    pub loop_end: i32,
    /// Loop count *L*.
    pub loop_count: i32,
    /// Reverse flag.
    pub reverse: bool,
    /// Ping-pong flag.
    pub ping_pong: bool,
    /// Shadow type.
    pub shadow: bool,
    /// Stage delay of claim 2.
    pub delay: i32,
    /// The random delay range [d₁′, d₂′] of claim 3 ((0, 0): none).
    pub random_delay: (i32, i32),
    /// Normalised to the game speed (claim 4).
    pub normalized: bool,
    /// Random loop delay pair (claim 11).
    pub random_loop_delay: (i32, i32),
    /// Follow-up type name (claim 15).
    pub next: Option<Vec<u8>>,
}

/// One animation type's playback fields as given (S-0109 Import mapping),
/// before the sprite gives the end fields in play (F2).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Fields {
    /// Start field *a*.
    pub start: i32,
    /// End field as given.
    pub end: Option<i32>,
    /// Loop end as given.
    pub loop_end: Option<i32>,
    /// Loop start λₛ.
    pub loop_start: i32,
    /// Loop count *L*.
    pub loop_count: i32,
    /// Reverse flag.
    pub reverse: bool,
    /// Ping-pong flag.
    pub ping_pong: bool,
    /// Shadow type.
    pub shadow: bool,
    /// A new-theater type (S-0109 Terms).
    pub new_theater: bool,
    /// Normalised to the game speed.
    pub normalized: bool,
    /// Random loop delay pair.
    pub random_loop_delay: (i32, i32),
    /// Follow-up type name.
    pub next: Option<Vec<u8>>,
}

/// The sprite end *E*ₛ for a sprite of `n` image frames (S-0109 Terms,
/// claim 10).
pub fn sprite_end(shadow: bool, n: i32) -> i32 {
    // spec: S-0109 claim 10 (a shadow type's sprite end is ⌊N / 2⌋)
    if shadow { n / 2 } else { n }
}

impl Playback {
    /// The fields in play of a type with fields `t` whose sprite holds `n`
    /// image frames (F2's assumed case: the first sprite found is the one
    /// drawn; a type without a sprite has `n` = 0, S-0109 claim 28), with
    /// claim 2's delay `delay` and claim 3's range `random_delay`.
    pub fn of_fields(
        t: &Fields,
        n: i32,
        delay: i32,
        random_delay: (i32, i32),
        q: &AnimQuirks,
    ) -> Self {
        let es = sprite_end(t.shadow, n);
        // spec: S-0109 F2 (End: absent or −1 E_s; 0 kept except for a new-theater type found at the refresh)
        let end = match t.end {
            None | Some(-1) => es,
            Some(0) if t.new_theater => es,
            Some(v) => v,
        };
        // spec: S-0109 F2 (LoopEnd: absent E_s (quirks.anim_loop_end_from_sprite; off: E); −1 E; 0 kept except for a new-theater type)
        let loop_end = match t.loop_end {
            None if q.anim_loop_end_from_sprite => es,
            None | Some(-1) => end,
            Some(0) if t.new_theater => end,
            Some(v) => v,
        };
        Self {
            start: t.start,
            end,
            loop_start: t.loop_start,
            loop_end,
            loop_count: t.loop_count,
            reverse: t.reverse,
            ping_pong: t.ping_pong,
            shadow: t.shadow,
            delay,
            random_delay,
            normalized: t.normalized,
            random_loop_delay: t.random_loop_delay,
            next: t.next.clone(),
        }
    }

    /// The fields of a type that names no type in the list, or has no
    /// sprite: absent-key defaults, so *E* = λₑ = 0 and *d* = 1 (S-0109 Edge
    /// cases, claim 25, F1).
    pub fn missing() -> Self {
        Self {
            // spec: S-0109 Edge cases (absent-key defaults; no sprite: end 0), F1 (no rate: d = 1)
            delay: 1,
            ..Self::default()
        }
    }
}

/// What a playing animation asks of its surroundings.
pub trait Env {
    /// The fields in play of the follow-up type `name` (a name that names
    /// no type gives absent-key defaults with no sprite, S-0109 Edge
    /// cases).
    fn follow_up(&mut self, name: &[u8]) -> Playback;
    /// A ranged draw over `lo` … `hi` from the drawing's cosmetic stream.
    fn draw(&mut self, lo: i32, hi: i32) -> i32;
    /// The game speed index now.
    fn speed(&self) -> u8;
    /// `quirks.speed_normalised_delays`.
    fn normalise(&self) -> bool;
    /// An advance has just set stage `k` of a type with fields `p`, before
    /// any turn or pass test (State and ordering: the advance's damage, then
    /// its middle event of claim 14, both before either pass-boundary delay
    /// choice, claim 28). The drawing does nothing here; the simulation
    /// delivers the damage and plays the middle event.
    fn advanced(&mut self, p: &Playback, k: i32) {
        let _ = (p, k);
    }
    /// Whether the animation left play during this update's advance (in its
    /// own damage detonation): then nothing more of its update happens
    /// (State and ordering). The drawing's animations never do.
    fn left_play(&self) -> bool {
        false
    }
    /// A follow-up switch to type `name` has just chosen its delay; the new
    /// type's start event follows (claims 15 and 28). The drawing does
    /// nothing here; the simulation plays the start event.
    fn switched(&mut self, name: &[u8]) {
        let _ = name;
    }
    /// A loop delay has just ended on this update, with the current type's
    /// fields `p`: the start event happens (claims 11 and 13). The drawing
    /// does nothing here; the simulation plays the start event.
    fn delay_ended(&mut self, p: &Playback) {
        let _ = p;
    }
}

/// What a creator gives a new animation (S-0109 Terms; claims 7, 8, 11 and
/// 29): an initial loop delay *D* in updates, a repeat factor *m* and a
/// reverse request. The default is no loop delay, repeat factor 1 and no
/// reverse request.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Creation {
    /// The initial loop delay *D* (claim 11; 0 for none).
    pub loop_delay: i32,
    /// The repeat factor *m* (claim 7; below 1 counts as 1).
    pub repeat: i32,
    /// The creator asks for reverse play (claim 8, Terms).
    pub reverse: bool,
}

/// The stage delay chosen for `p` (claims 2 to 4): the random range's draw
/// (`drawn` when the simulation took it, else a cosmetic draw), its single
/// value, or claim 2's delay; then normalised.
fn choose_delay(p: &Playback, drawn: Option<i32>, env: &mut dyn Env) -> i32 {
    let (d1, d2) = p.random_delay;
    let d = if (d1, d2) == (0, 0) {
        // spec: S-0109 claim 3 (both ends 0: the value of claim 2)
        p.delay
    } else if d1 == d2 {
        // spec: S-0109 claim 3 (d₁′ = d₂′: that value, no draw)
        d1
    } else {
        // spec: S-0109 claim 3 (a ranged draw over [d₁′, d₂′])
        drawn.unwrap_or_else(|| env.draw(d1, d2))
    };
    if p.normalized && env.normalise() {
        // spec: S-0109 claim 4 (N(d, s) at the speed of the tick the delay is chosen)
        crate::infantry::normalised(d, env.speed())
    } else {
        d
    }
}

/// The remaining passes *r* (claim 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Remaining {
    /// A count.
    Count(i64),
    /// Endless (`quirks.anim_loop_count_wrap` off with a negative count).
    Endless,
}

/// The remaining-pass count that endlessly repeats with the wrap on.
const ENDLESS: i64 = 255; // spec: S-0109 claim 7, Constants (255: endless)
/// The remaining-pass modulus.
const MODULUS: i64 = 256; // spec: S-0109 claim 7, Constants (mod 256)

impl Remaining {
    /// The count, or `None` when endless.
    fn count(self) -> Option<i64> {
        match self {
            Remaining::Count(r) => Some(r),
            Remaining::Endless => None,
        }
    }

    fn more(self) -> bool {
        // spec: S-0109 claim 7, F3 (a pass that ends with r ≥ 2 is followed by a new pass)
        match self {
            Remaining::Count(r) => r >= 2,
            Remaining::Endless => true,
        }
    }
}

/// One playing animation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    /// The current type's fields.
    pub play: Playback,
    /// The current type's name (it changes at a follow-up switch).
    pub name: Vec<u8>,
    k: i32,
    step: i32,
    r: Remaining,
    delay: i32,
    /// Tick of the last advance, or of creation (or of the follow-up switch).
    last: i32,
    first_pending: bool,
    loop_delay: i32,
    ended: bool,
    /// The creator asked for reverse play (S-0109 Terms): the reversed test
    /// is the current type's flag or this request, decided afresh at a
    /// follow-up switch (claim 15).
    #[serde(default)]
    reverse_request: bool,
}

impl Player {
    /// A new animation of type `name` with fields `play`, created in tick
    /// `created` with no loop delay, repeat factor 1 and no reverse
    /// request (S-0109 claims 1, 7, 8; F3). `drawn` is the stage delay the
    /// simulation's creation draw gave, if it took one.
    pub fn new(
        name: &[u8],
        play: Playback,
        created: i32,
        drawn: Option<i32>,
        env: &mut dyn Env,
        q: &AnimQuirks,
    ) -> Self {
        Self::create(name, play, created, drawn, Creation::default(), env, q)
    }

    /// [`Player::new`] with what the creator gives (`c`): an initial loop
    /// delay, a repeat factor and a reverse request (S-0109 claims 7, 8, 11
    /// and 29).
    pub fn create(
        name: &[u8],
        mut play: Playback,
        created: i32,
        drawn: Option<i32>,
        c: Creation,
        env: &mut dyn Env,
        q: &AnimQuirks,
    ) -> Self {
        // spec: S-0109 Terms (reversed: the type's reverse flag or the creator's request)
        play.reverse |= c.reverse;
        let delay = choose_delay(&play, drawn, env);
        let r = remaining_at_creation(play.loop_count, c.repeat, q);
        let (k, step) = initial_stage(&play, q);
        Self {
            play,
            name: name.to_vec(),
            k,
            step,
            r,
            delay,
            last: created,
            first_pending: true,
            // spec: S-0109 claim 11 (a loop delay of D updates at creation when the creator asks for one, D > 0)
            loop_delay: c.loop_delay.max(0),
            ended: false,
            reverse_request: c.reverse,
        }
    }

    /// The loop delay still to count down (claim 11); 0 when none.
    pub fn loop_delay(&self) -> i32 {
        self.loop_delay
    }

    /// The image frame shown, *a* + *k* (claim 1); outside 0 … N − 1 it
    /// draws nothing.
    pub fn frame(&self) -> i32 {
        // spec: S-0109 claim 1 (image frame a + k)
        self.play.start.saturating_add(self.k)
    }

    /// The current type's name as text.
    pub fn name_text(&self) -> String {
        String::from_utf8_lossy(&self.name).into_owned()
    }

    /// The stage *k*.
    pub fn stage(&self) -> i32 {
        self.k
    }

    /// The stage delay in use.
    pub fn delay(&self) -> i32 {
        self.delay
    }

    /// Whether the animation has ended (claim 16).
    pub fn ended(&self) -> bool {
        self.ended
    }

    /// The remaining passes *r* (claim 7), `None` when endless.
    pub fn remaining_passes(&self) -> Option<i64> {
        self.r.count()
    }

    /// A refused end (S-0611 claim 11, S-0109 claim 16): the animation
    /// stays in play and its stage steps back by one, so the same end comes
    /// again when that frame is next due (the clock runs from the advance
    /// that ended it).
    pub fn step_back(&mut self) {
        if !self.ended {
            return;
        }
        // spec: S-0611 claim 11 (it steps back one frame and tries again when that frame is next due)
        self.ended = false;
        self.k = self.k.saturating_sub(self.step);
    }

    /// Keeps stage `k` of the animation this one replaces (S-0108 claim
    /// 25): the new animation's stage starts there, showing image frame
    /// *a* + *k* of its own type, its remaining passes and step direction
    /// its own creation values. A stage beyond its range is not limited:
    /// it plays on from there by S-0109's ordinary rules.
    pub fn keep_stage(&mut self, k: i32) {
        // spec: S-0108 claim 25 (the stage is kept, not the image frame; the passes are not kept; a kept stage beyond the new type's range is not limited)
        self.k = k;
    }

    /// A creator sets the stage to `k` in tick `tick` and restarts the
    /// stage clock with a stage delay of `delay` ticks, leaving the
    /// direction of play, the remaining passes and any loop delay as they
    /// are (S-0614 claim 25: a grab animation's setting). The delay is used
    /// as given, never normalised to the game speed.
    pub fn set_stage(&mut self, k: i32, tick: i32, delay: i32) {
        // spec: S-0614 claim 25 (k set, the direction of play unchanged, the clock restarted with the given stage delay in ticks at every game speed)
        self.k = k;
        self.last = tick;
        self.delay = delay;
    }

    /// Makes the current pass the last: the remaining passes become 0
    /// (S-0522 claim 13, a parachute still present when its paratrooper
    /// lands; S-0109 claim 7).
    pub fn last_pass(&mut self) {
        // spec: S-0522 claim 13 (its remaining passes become 0, making the current pass its last)
        self.r = Remaining::Count(0);
    }

    /// The animation's update in tick `tick` (S-0109 claims 5, 6, 9, 11, 15
    /// and 16). Returns whether it is still playing.
    pub fn update(&mut self, tick: i32, env: &mut dyn Env, q: &AnimQuirks) -> bool {
        self.update_held(tick, env, q, false)
    }

    /// [`Player::update`] for an animation that is `held` (S-0109 claim
    /// 23): a held one makes no advance, so nothing that follows from one
    /// happens, but its first update is still skipped, a loop delay still
    /// counts down with its start event, and its clock keeps running.
    pub fn update_held(
        &mut self,
        tick: i32,
        env: &mut dyn Env,
        q: &AnimQuirks,
        held: bool,
    ) -> bool {
        if self.ended {
            return false;
        }
        if self.first_pending {
            // spec: S-0109 claim 5 (the first update makes no advance, counts down no loop delay and
            // does not restart the clock)
            self.first_pending = false;
            return true;
        }
        if self.loop_delay > 0 {
            // spec: S-0109 claim 11 (each update counts D down; the update that brings it to 0
            // makes no advance)
            self.loop_delay -= 1;
            if self.loop_delay == 0 {
                // spec: S-0109 claims 11 and 13 (on the update that brings it to 0 the start event happens)
                env.delay_ended(&self.play);
            }
            return true;
        }
        if held {
            // spec: S-0109 claim 23 (a held animation does not advance; its clock keeps running, so it advances at its first eligible update after its release)
            return true;
        }
        // spec: S-0109 claim 5 (an advance when at least d ticks have passed since the last
        // advance, and d > 0)
        if self.delay <= 0 || tick - self.last < self.delay {
            return true;
        }
        self.advance(tick, env, q);
        !self.ended
    }

    /// One advance and what follows from it (F3).
    fn advance(&mut self, tick: i32, env: &mut dyn Env, q: &AnimQuirks) {
        // spec: S-0109 Terms (an advance: one change of k by σ)
        self.k = self.k.saturating_add(self.step);
        self.last = tick;
        // spec: S-0109 claims 14 and 28 and State and ordering (the damage and middle event of an advance precede the turn and pass tests and either pass-boundary delay choice)
        env.advanced(&self.play, self.k);
        if env.left_play() {
            // spec: S-0109 State and ordering (an animation that leaves play in its own detonation: no pass end, follow-up switch or end in that update; our engine's rule)
            return;
        }
        let p = &self.play;
        let a = p.start;
        let more = self.r.more();
        if p.ping_pong {
            let turn = if more {
                // spec: S-0109 claim 9 (while r ≥ 2: k ≥ λₑ − a, or k = a (quirks.anim_pingpong_lower_turn;
                // off: k ≤ λₛ − a))
                self.k >= p.loop_end - a
                    || if q.anim_pingpong_lower_turn {
                        self.k == a
                    } else {
                        self.k <= p.loop_start - a
                    }
            } else {
                // spec: S-0109 claim 9 (on the last pass: k ≥ E or k = 0)
                self.k >= p.end || self.k == 0
            };
            if turn {
                // spec: S-0109 claim 9 (σ negated; a turning advance never ends the pass)
                self.step = -self.step;
                return;
            }
        }
        // spec: S-0109 claim 6, F3 (limit λₑ − a while r ≥ 2, E on the last pass)
        let limit = if more { p.loop_end - a } else { p.end };
        // spec: S-0109 claim 8 (a reversed pass ends at k ≤ 0; quirks.anim_reverse_raw_loop_end off:
        // k < 0)
        let low = if q.anim_reverse_raw_loop_end {
            self.k <= 0
        } else {
            self.k < 0
        };
        // spec: S-0109 F3 (k ≥ limit, or reversed and at the low end, or a shadow type, not
        // reversed, at k ≥ λₑ − a, claim 10)
        let ends = self.k >= limit
            || (p.reverse && low)
            || (p.shadow && !p.reverse && self.k >= p.loop_end - a);
        if !ends {
            return;
        }
        if more {
            self.new_pass(env, q);
        } else if let Some(next) = p.next.clone() {
            self.follow_up(&next, tick, env, q);
        } else {
            // spec: S-0109 claim 16 (the last pass ends with no follow-up type: removed)
            self.ended = true;
        }
    }

    /// A new pass (F3, claim 11).
    fn new_pass(&mut self, env: &mut dyn Env, q: &AnimQuirks) {
        let p = &self.play;
        self.r = match self.r {
            // spec: S-0109 claim 7, F3 (r′ = r − 1, except 255 stays 255 with quirks.anim_loop_count_wrap)
            Remaining::Count(r) if q.anim_loop_count_wrap && r == ENDLESS => Remaining::Count(r),
            Remaining::Count(r) => Remaining::Count(r - 1),
            Remaining::Endless => Remaining::Endless,
        };
        self.k = if !p.reverse {
            // spec: S-0109 claim 6, F3 (k′ = λₛ − a)
            p.loop_start - p.start
        } else if q.anim_reverse_raw_loop_end {
            // spec: S-0109 claim 8, F3 (reversed: k′ = λₑ)
            p.loop_end
        } else {
            // spec: S-0109 Edge cases (quirks.anim_reverse_raw_loop_end off: λₑ − a − 1)
            p.loop_end - p.start - 1
        };
        let (d1, d2) = p.random_loop_delay;
        if (d1, d2) != (0, 0) {
            // spec: S-0109 claim 11 (a ranged draw over the pair, in either order; a drawn 0 gives no
            // loop delay)
            self.loop_delay = env.draw(d1.min(d2), d1.max(d2)).max(0);
        }
    }

    /// The follow-up switch to type `next` in tick `tick` (claim 15).
    fn follow_up(&mut self, next: &[u8], tick: i32, env: &mut dyn Env, q: &AnimQuirks) {
        let mut p = env.follow_up(next);
        // spec: S-0109 claim 15 and Terms (reversed is decided afresh from the follow-up type's flag and the creator's request)
        p.reverse |= self.reverse_request;
        let reversed = p.reverse;
        self.k = if q.anim_followup_start_doubled {
            // spec: S-0109 claim 15 (k = a′, showing 2a′)
            p.start
        } else {
            // spec: S-0109 Edge cases (quirks.anim_followup_start_doubled off: k₀ of F3)
            initial_stage(&p, q).0
        };
        if !q.anim_followup_keeps_step {
            // spec: S-0109 Edge cases (quirks.anim_followup_keeps_step off: σ afresh)
            self.step = if reversed { -1 } else { 1 };
        }
        self.r = if q.anim_loop_count_wrap {
            // spec: S-0109 claim 15 (r = L′ mod 256, no repeat factor, no raising to 1)
            Remaining::Count(i64::from(p.loop_count).rem_euclid(MODULUS))
        } else if p.loop_count < 0 {
            // spec: S-0109 Edge cases (quirks.anim_loop_count_wrap off: endless when L′ < 0)
            Remaining::Endless
        } else {
            Remaining::Count(i64::from(p.loop_count))
        };
        // spec: S-0109 claim 15 (d chosen afresh, claims 2 to 4)
        self.delay = choose_delay(&p, None, env);
        // spec: S-0109 claim 15 (the clock restarts at that tick, with no first-update exclusion)
        self.last = tick;
        self.name = next.to_vec();
        self.play = p;
        // spec: S-0109 claim 28 (the follow-up delay is chosen before the new type's start event and its effects)
        env.switched(next);
    }
}

/// k₀ and σ of a new animation (claims 1 and 8, F3).
fn initial_stage(p: &Playback, q: &AnimQuirks) -> (i32, i32) {
    if !p.reverse {
        // spec: S-0109 claim 1, F3 (k₀ = 0, σ = +1)
        (0, 1)
    } else if q.anim_reverse_raw_loop_end {
        // spec: S-0109 claim 8, F3 (k₀ = λₑ − 1, σ = −1)
        (p.loop_end - 1, -1)
    } else {
        // spec: S-0109 Edge cases (quirks.anim_reverse_raw_loop_end off: λₑ − a − 1)
        (p.loop_end - p.start - 1, -1)
    }
}

/// r₀ for loop count `l` and repeat factor `m` (claim 7, F3).
fn remaining_at_creation(l: i32, m: i32, q: &AnimQuirks) -> Remaining {
    // spec: S-0109 Terms (a repeat factor below 1 counts as 1)
    let m = i64::from(m.max(1));
    let l = i64::from(l);
    if q.anim_loop_count_wrap {
        // spec: S-0109 claim 7, F3 (r₀ = max(1, (L × max(m, 1)) mod 256))
        Remaining::Count((l * m).rem_euclid(MODULUS).max(1))
    } else if l < 0 {
        // spec: S-0109 Edge cases (quirks.anim_loop_count_wrap off: endless when L < 0)
        Remaining::Endless
    } else {
        // spec: S-0109 Edge cases (quirks.anim_loop_count_wrap off: max(1, L × max(m, 1)))
        Remaining::Count((l * m).max(1))
    }
}
