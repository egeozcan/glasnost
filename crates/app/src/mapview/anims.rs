//! Animation playback for the drawing (S-0109 claims 1 to 16): the stage,
//! step, remaining passes, stage delay, loop delay and follow-up switch of
//! one playing animation, advanced update by update with the shown game
//! ticks (never the wall clock).
//!
//! This is presentation state. The simulation keeps no playback clock: it
//! takes an animation's creation draws (S-0202 claim 10) and reports the
//! animation in `World::effects`, with the stage delay its creation draw
//! gave. Playback here never feeds the simulation. The draws the
//! simulation does not take (a follow-up type's random delay, the random
//! loop delays between passes) come from a cosmetic stream of the
//! drawing's own: an interim rule (docs/development.md, request Q-0770).
//!
//! Damage, start and middle events (claims 13, 14 and 17) are the
//! simulation's and are not played here; held animations (claim 23) and
//! translucency levels above 0 (claim 20) belong to building slot
//! animations, which this player does not run.

use super::slots::{AnimType, end_in_play, sprite_end};

/// The switches of S-0109's playback that the drawing implements, each
/// defaulting to the original's behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl AnimQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 6] {
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

/// The stage delay of claim 2 from the rate field as given: an absent
/// field, or −1 with no earlier value, gives 1; otherwise F1, with
/// `quirks.anim_fast_rate_freezes` as `fast_freezes`.
pub fn rate_delay(rate: Option<i32>, fast_freezes: bool) -> i32 {
    match rate {
        // spec: S-0109 claim 2, F1 (no rate field: d = 1; −1 keeps the held value, 1 when none was given)
        None | Some(-1) => 1,
        Some(r) => glasnost_sim::rules::anim_delay(r, fast_freezes),
    }
}

/// One animation type's playback fields in play (S-0109 Terms, F2).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

impl Playback {
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

    /// The fields in play of type `t` whose sprite holds `n` image frames
    /// (F2's assumed case: the first sprite found is the one drawn), with
    /// claim 2's delay `delay` and claim 3's range `random_delay`.
    pub fn of(t: &AnimType, n: i32, delay: i32, random_delay: (i32, i32), q: &AnimQuirks) -> Self {
        let end = end_in_play(t, n);
        // spec: S-0109 F2 (LoopEnd: absent E_s (quirks.anim_loop_end_from_sprite; off: E); −1 E;
        // 0 kept except for a new-theater type)
        let loop_end = match t.loop_end {
            None if q.anim_loop_end_from_sprite => sprite_end(t, n),
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
        glasnost_sim::infantry::normalised(d, env.speed())
    } else {
        d
    }
}

/// The remaining passes *r* (claim 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    fn more(self) -> bool {
        // spec: S-0109 claim 7, F3 (a pass that ends with r ≥ 2 is followed by a new pass)
        match self {
            Remaining::Count(r) => r >= 2,
            Remaining::Endless => true,
        }
    }
}

/// One playing animation.
#[derive(Clone, Debug, PartialEq, Eq)]
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
        let delay = choose_delay(&play, drawn, env);
        let r = remaining_at_creation(play.loop_count, 1, q);
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
            loop_delay: 0,
            ended: false,
        }
    }

    /// The image frame shown, *a* + *k* (claim 1); outside 0 … N − 1 it
    /// draws nothing.
    pub fn frame(&self) -> i32 {
        // spec: S-0109 claim 1 (image frame a + k)
        self.play.start.saturating_add(self.k)
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

    /// The animation's update in tick `tick` (S-0109 claims 5, 6, 9, 11, 15
    /// and 16). Returns whether it is still playing.
    pub fn update(&mut self, tick: i32, env: &mut dyn Env, q: &AnimQuirks) -> bool {
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
        let p = env.follow_up(next);
        // spec: S-0109 Terms (reversed is decided afresh from the current type; no creator here
        // asks for reverse play)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A test surrounding: follow-up types by name, a scripted draw list,
    /// a speed.
    struct TestEnv {
        types: Vec<(Vec<u8>, Playback)>,
        draws: Vec<i32>,
        taken: Vec<(i32, i32)>,
        speed: u8,
        normalise: bool,
    }

    impl TestEnv {
        fn new() -> Self {
            Self {
                types: Vec::new(),
                draws: Vec::new(),
                taken: Vec::new(),
                speed: 4,
                normalise: true,
            }
        }
    }

    impl Env for TestEnv {
        fn follow_up(&mut self, name: &[u8]) -> Playback {
            self.types
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, p)| p.clone())
                .unwrap_or_else(Playback::missing)
        }
        fn draw(&mut self, lo: i32, hi: i32) -> i32 {
            self.taken.push((lo, hi));
            if self.draws.is_empty() {
                lo
            } else {
                self.draws.remove(0)
            }
        }
        fn speed(&self) -> u8 {
            self.speed
        }
        fn normalise(&self) -> bool {
            self.normalise
        }
    }

    /// A type from art text `fields` (section `[A]`) on a sprite of `n`
    /// image frames, with the given rate and random delay range.
    fn ty(fields: &str, n: i32, random_delay: (i32, i32)) -> Playback {
        let art = glasnost_ini::Document::parse(
            format!("[A]\n{fields}\n").as_bytes(),
            Default::default(),
            &mut Vec::new(),
        );
        let t = super::super::slots::read_anim(&art, b"A");
        let d = rate_delay(t.rate, true);
        Playback::of(&t, n, d, random_delay, &AnimQuirks::default())
    }

    /// Plays `p` created in tick 0 with its first update in tick 0 (an
    /// animation created before or during the turn pass, claim 5) until it
    /// ends or `limit` ticks pass: the frame shown at the end of each tick,
    /// `None` once removed.
    fn run(p: Playback, env: &mut TestEnv, q: &AnimQuirks, limit: i32) -> Vec<Option<i32>> {
        let mut a = Player::new(b"A", p, 0, None, env, q);
        let mut out = Vec::new();
        for t in 0..limit {
            if a.update(t, env, q) {
                out.push(Some(a.frame()));
            } else {
                out.push(None);
                break;
            }
        }
        out
    }

    /// The distinct frames in order of showing (runs collapsed).
    fn sequence(shown: &[Option<i32>]) -> Vec<i32> {
        let mut out: Vec<i32> = Vec::new();
        for f in shown.iter().flatten() {
            if out.last() != Some(f) {
                out.push(*f);
            }
        }
        out
    }

    /// The hold of each run of equal frames.
    fn holds(shown: &[Option<i32>]) -> Vec<usize> {
        let mut out: Vec<usize> = Vec::new();
        let mut prev = None;
        for f in shown.iter().flatten() {
            if Some(f) == prev {
                *out.last_mut().expect("a run") += 1;
            } else {
                out.push(1);
                prev = Some(f);
            }
        }
        out
    }

    // spec: S-0109 claims 1, 2, 5, 6 and 16, scenario AN1 (T1: rate 100, every frame held 9 ticks,
    // lifetime 9N; T2: rate 300, end 6, frames 0 … 5, 3 ticks each)
    #[test]
    fn plain_playback_an1() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let n = 7;
        let shown = run(ty("Rate=100", n, (0, 0)), &mut env, &q, 1000);
        assert_eq!(sequence(&shown), (0..n).collect::<Vec<_>>());
        assert!(holds(&shown).iter().all(|&h| h == 9));
        // Removed in the update of tick 9N: lifetime 9N ticks.
        assert_eq!(shown.len(), 9 * n as usize + 1);
        assert_eq!(shown.last(), Some(&None));
        let shown = run(ty("Rate=300\nEnd=6", 20, (0, 0)), &mut env, &q, 1000);
        assert_eq!(sequence(&shown), vec![0, 1, 2, 3, 4, 5]);
        assert!(holds(&shown).iter().all(|&h| h == 3));
    }

    // spec: S-0109 claims 1 and 6, AN1 T3 (end N + 2: the two frames past the sprite are shown, blank,
    // for 2 × 3 ticks before the end)
    #[test]
    fn end_past_the_sprite_an1_t3() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let shown = run(ty("Rate=300\nEnd=7", 5, (0, 0)), &mut env, &q, 1000);
        assert_eq!(sequence(&shown), vec![0, 1, 2, 3, 4, 5, 6]);
        let blank = shown.iter().flatten().filter(|&&f| f >= 5).count();
        assert_eq!(blank, 2 * 3);
    }

    // spec: S-0109 claims 5 and 16, AN1 T5 and T6 (d = 1, first update in the creation tick: the first
    // frame lasts 1 tick; T6 ends 6 ticks after creation)
    #[test]
    fn first_frame_and_lifetime_an1_t5_t6() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let t5 = run(ty("Rate=900", 4, (0, 0)), &mut env, &q, 1000);
        assert_eq!(&t5[..3], &[Some(0), Some(1), Some(2)]);
        let t6 = run(ty("End=6", 20, (0, 0)), &mut env, &q, 1000);
        assert_eq!(
            t6,
            vec![Some(0), Some(1), Some(2), Some(3), Some(4), Some(5), None]
        );
    }

    // spec: S-0109 claim 5 (first update in the tick after creation, u = c + 1, with d = 1: the first
    // frame lasts 2 ticks; with d ≥ 2 the first advance is still in tick c + d)
    #[test]
    fn first_update_after_creation() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let mut a = Player::new(b"A", ty("End=6", 20, (0, 0)), 0, None, &mut env, &q);
        // No update in tick 0 (created after the turn pass).
        let shown: Vec<i32> = (1..4)
            .map(|t| {
                a.update(t, &mut env, &q);
                a.frame()
            })
            .collect();
        assert_eq!(shown, vec![0, 1, 2]);
        let mut a = Player::new(b"A", ty("Rate=300", 20, (0, 0)), 0, None, &mut env, &q);
        let shown: Vec<i32> = (1..5)
            .map(|t| {
                a.update(t, &mut env, &q);
                a.frame()
            })
            .collect();
        assert_eq!(shown, vec![0, 0, 1, 1]);
    }

    // spec: S-0109 claim 2 and Edge cases (a rate above 900 freezes: d = 0, the animation never advances
    // or ends), AN2
    #[test]
    fn fast_rate_freezes_an2() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let shown = run(ty("Rate=1000", 4, (0, 0)), &mut env, &q, 600);
        assert_eq!(shown.len(), 600);
        assert!(shown.iter().all(|f| *f == Some(0)));
    }

    // spec: S-0109 claim 3, AN3 (the simulation's drawn delay is used; equal ends take no draw; no
    // reroll at a new pass)
    #[test]
    fn random_rate_an3() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let p = ty("Rate=300\nLoopCount=2\nEnd=3", 3, (2, 4));
        let a = Player::new(b"A", p.clone(), 0, Some(3), &mut env, &q);
        assert_eq!(a.delay(), 3);
        assert!(env.taken.is_empty());
        let shown = run(p, &mut env, &q, 1000);
        // Without a simulation draw the cosmetic stream draws once at creation, never at a pass.
        assert_eq!(env.taken, vec![(2, 4)]);
        assert!(holds(&shown).iter().all(|&h| h == 2));
        env.taken.clear();
        let a = Player::new(b"A", ty("", 3, (2, 2)), 0, None, &mut env, &q);
        assert_eq!(a.delay(), 2);
        assert!(env.taken.is_empty());
    }

    // spec: S-0109 claim 4, AN4 (U: rate 90, not normalised, holds 10 at every speed; V: normalised,
    // holds N(10, s): 80, 20 and 11 at speeds 0, 3 and 6; quirks.speed_normalised_delays off: 10)
    #[test]
    fn game_speed_an4() {
        let q = AnimQuirks::default();
        for (s, v) in [(0u8, 80), (3, 20), (6, 11)] {
            let mut env = TestEnv::new();
            env.speed = s;
            let u = Player::new(b"U", ty("Rate=90", 4, (0, 0)), 0, None, &mut env, &q);
            assert_eq!(u.delay(), 10, "U at speed {s}");
            let n = Player::new(
                b"V",
                ty("Rate=90\nNormalized=yes", 4, (0, 0)),
                0,
                None,
                &mut env,
                &q,
            );
            assert_eq!(n.delay(), v, "V at speed {s}");
            env.normalise = false;
            let n = Player::new(
                b"V",
                ty("Rate=90\nNormalized=yes", 4, (0, 0)),
                0,
                None,
                &mut env,
                &q,
            );
            assert_eq!(n.delay(), 10, "V unnormalised at speed {s}");
        }
    }

    // spec: S-0109 claims 6 and 7, AN5 (W1: 2 3 4 5 6, 4 5 6, 4 … 10; W2: loop count 256, one pass;
    // W3: loop count −1, endless; W4: absent loop end is N, 0 … N − 1 then 0 … 5)
    #[test]
    fn looped_an5() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let w1 = run(
            ty(
                "Rate=300\nStart=2\nLoopStart=4\nLoopEnd=7\nEnd=9\nLoopCount=3",
                12,
                (0, 0),
            ),
            &mut env,
            &q,
            1000,
        );
        assert_eq!(
            w1.iter().flatten().copied().collect::<Vec<_>>().len() % 3,
            0
        );
        let mut seq = Vec::new();
        let mut prev = None;
        for (i, f) in w1.iter().enumerate() {
            // Each stage lasts 3 ticks: sample one tick per stage.
            if i % 3 == 0
                && let Some(f) = f
            {
                seq.push(*f);
            }
            prev = Some(*f);
        }
        assert_eq!(prev, Some(None));
        assert_eq!(seq, vec![2, 3, 4, 5, 6, 4, 5, 6, 4, 5, 6, 7, 8, 9, 10]);
        let w2 = run(ty("Rate=300\nLoopCount=256", 4, (0, 0)), &mut env, &q, 1000);
        assert_eq!(sequence(&w2), vec![0, 1, 2, 3]);
        assert_eq!(w2.last(), Some(&None));
        let w3 = run(ty("Rate=300\nLoopCount=-1", 4, (0, 0)), &mut env, &q, 2000);
        assert_eq!(w3.len(), 2000);
        assert!(w3.iter().all(Option::is_some));
        let w4 = run(
            ty("Rate=300\nEnd=6\nLoopCount=2", 10, (0, 0)),
            &mut env,
            &q,
            1000,
        );
        let mut want: Vec<i32> = (0..10).collect();
        want.extend(0..6);
        assert_eq!(sequence(&w4), want);
    }

    // spec: S-0109 Edge cases (quirks.anim_loop_count_wrap off: a loop count of 256 plays 256 passes;
    // quirks.anim_loop_end_from_sprite off: an absent loop end takes E)
    #[test]
    fn loop_quirks_off() {
        let q = AnimQuirks {
            anim_loop_count_wrap: false,
            anim_loop_end_from_sprite: false,
            ..AnimQuirks::default()
        };
        let mut env = TestEnv::new();
        let a = Player::new(b"A", ty("LoopCount=256", 2, (0, 0)), 0, None, &mut env, &q);
        assert_eq!(a.r, Remaining::Count(256));
        let art = glasnost_ini::Document::parse(
            b"[A]\nEnd=6\nLoopCount=2\nRate=300\n",
            Default::default(),
            &mut Vec::new(),
        );
        let t = super::super::slots::read_anim(&art, b"A");
        let p = Playback::of(&t, 10, 3, (0, 0), &q);
        assert_eq!(p.loop_end, 6);
        let shown = run(p, &mut env, &q, 1000);
        let mut want: Vec<i32> = (0..6).collect();
        want.extend(0..6);
        assert_eq!(
            shown
                .iter()
                .enumerate()
                .filter(|(i, f)| i % 3 == 0 && f.is_some())
                .map(|(_, f)| f.expect("frame"))
                .collect::<Vec<_>>(),
            want
        );
    }

    // spec: S-0109 claim 8, AN6 (Q1: 7 … 1 then 8 … 1, frame 0 never shown; Q2: 10, then 11, 10 … 4)
    #[test]
    fn reversed_an6() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let q1 = run(
            ty(
                "Reverse=yes\nEnd=8\nLoopEnd=8\nLoopCount=2\nRate=300",
                12,
                (0, 0),
            ),
            &mut env,
            &q,
            1000,
        );
        let mut want: Vec<i32> = (1..8).rev().collect();
        want.extend((1..9).rev());
        assert_eq!(sequence(&q1), want);
        assert_eq!(q1.last(), Some(&None));
        let q2 = run(
            ty(
                "Reverse=yes\nStart=3\nEnd=8\nLoopEnd=8\nLoopCount=2\nRate=300",
                12,
                (0, 0),
            ),
            &mut env,
            &q,
            1000,
        );
        let mut want = vec![10];
        want.extend((4..12).rev());
        assert_eq!(sequence(&q2), want);
        // quirks.anim_reverse_raw_loop_end off: λₑ − 1 down to a.
        let off = AnimQuirks {
            anim_reverse_raw_loop_end: false,
            ..AnimQuirks::default()
        };
        let shown = run(
            ty(
                "Reverse=yes\nStart=3\nEnd=8\nLoopEnd=8\nRate=300",
                12,
                (0, 0),
            ),
            &mut env,
            &off,
            1000,
        );
        assert_eq!(sequence(&shown), vec![7, 6, 5, 4, 3]);
    }

    // spec: S-0109 claim 9, AN7 (P1 cycles 0 … 6, 5 … 0, 1 …; P2 cycles 0 … 5 … 0; P3 shows 4 … 8, then
    // 7 … 0, then nothing; P4 shows 0, 1, 2 and is removed)
    #[test]
    fn ping_pong_an7() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let p1 = run(
            ty("PingPong=yes\nLoopCount=1\nEnd=6\nRate=300", 12, (0, 0)),
            &mut env,
            &q,
            2000,
        );
        assert_eq!(p1.len(), 2000);
        assert_eq!(
            &sequence(&p1)[..15],
            &[0, 1, 2, 3, 4, 5, 6, 5, 4, 3, 2, 1, 0, 1, 2]
        );
        let p2 = run(
            ty(
                "PingPong=yes\nLoopCount=-1\nLoopEnd=5\nRate=300",
                12,
                (0, 0),
            ),
            &mut env,
            &q,
            2000,
        );
        assert_eq!(p2.len(), 2000);
        assert_eq!(
            &sequence(&p2)[..13],
            &[0, 1, 2, 3, 4, 5, 4, 3, 2, 1, 0, 1, 2]
        );
        let p3 = run(
            ty(
                "PingPong=yes\nLoopCount=-1\nStart=4\nLoopStart=4\nLoopEnd=8\nRate=300",
                12,
                (0, 0),
            ),
            &mut env,
            &q,
            2000,
        );
        assert_eq!(p3.len(), 2000);
        let seq = sequence(&p3);
        assert_eq!(&seq[..13], &[4, 5, 6, 7, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
        assert!(seq[13..].iter().all(|&f| f < 0), "then nothing");
        let p4 = run(
            ty(
                "PingPong=yes\nShadow=yes\nLoopCount=1\nEnd=6\nLoopEnd=3\nRate=300",
                24,
                (0, 0),
            ),
            &mut env,
            &q,
            2000,
        );
        assert_eq!(sequence(&p4), vec![0, 1, 2]);
        assert_eq!(p4.last(), Some(&None));
        // quirks.anim_pingpong_lower_turn off: P3 turns at λₛ − a coming down.
        let off = AnimQuirks {
            anim_pingpong_lower_turn: false,
            ..AnimQuirks::default()
        };
        let p3 = run(
            ty(
                "PingPong=yes\nLoopCount=-1\nStart=4\nLoopStart=4\nLoopEnd=8\nRate=300",
                12,
                (0, 0),
            ),
            &mut env,
            &off,
            200,
        );
        assert_eq!(&sequence(&p3)[..10], &[4, 5, 6, 7, 8, 7, 6, 5, 4, 5]);
    }

    // spec: S-0109 claim 10, AN8 (S1: shadow, loop end 3, end 6, one pass: 0, 1, 2, removed)
    #[test]
    fn shadow_pass_end_an8() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let s1 = run(
            ty(
                "Shadow=yes\nLoopEnd=3\nEnd=6\nLoopCount=1\nRate=300",
                24,
                (0, 0),
            ),
            &mut env,
            &q,
            1000,
        );
        assert_eq!(sequence(&s1), vec![0, 1, 2]);
        assert_eq!(s1.last(), Some(&None));
    }

    // spec: S-0109 claim 11, AN9 (loop count 3, random loop delay 30, 60, rate 300: every gap is
    // max(3, D + 1), the loop start frame shown during it; one draw per pass end)
    #[test]
    fn loop_delay_an9() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        env.draws = vec![30, 45];
        let shown = run(
            ty(
                "LoopCount=3\nRandomLoopDelay=60,30\nRate=300\nEnd=4",
                4,
                (0, 0),
            ),
            &mut env,
            &q,
            1000,
        );
        assert_eq!(env.taken, vec![(30, 60), (30, 60)]);
        // Holds: frames 0 … 3 for 3 ticks each; at each pass end the loop start frame 0 is held
        // max(3, D + 1) ticks.
        assert_eq!(holds(&shown), vec![3, 3, 3, 3, 31, 3, 3, 3, 46, 3, 3, 3]);
        // A drawn 0 gives no loop delay.
        let mut env = TestEnv::new();
        env.draws = vec![0];
        let shown = run(
            ty(
                "LoopCount=2\nRandomLoopDelay=0,5\nRate=300\nEnd=2",
                2,
                (0, 0),
            ),
            &mut env,
            &q,
            1000,
        );
        assert_eq!(holds(&shown), vec![3, 3, 3, 3]);
    }

    // spec: S-0109 claim 15, AN11 (F then G: 0 1 2 3, then G's frames 4, 5 … N_G + 1 at 6 ticks
    // each, one pass, removal; F2 then G2: 3 2 1, then 4 3 2 1 0 and nothing, never removed)
    #[test]
    fn follow_up_an11() {
        let q = AnimQuirks::default();
        let n_g = 6;
        let mut env = TestEnv::new();
        env.types.push((
            b"G".to_vec(),
            ty("Start=2\nLoopCount=0\nRate=150", n_g, (0, 0)),
        ));
        env.types
            .push((b"G2".to_vec(), ty("Start=2\nEnd=6\nRate=300", 12, (0, 0))));
        let f = run(
            ty("Rate=300\nEnd=4\nNext=G", 12, (0, 0)),
            &mut env,
            &q,
            2000,
        );
        let mut want: Vec<i32> = (0..4).collect();
        want.extend(4..n_g + 2);
        assert_eq!(sequence(&f), want);
        assert_eq!(holds(&f)[4..], vec![6; n_g as usize - 2][..]);
        assert_eq!(f.last(), Some(&None));
        let f2 = run(
            ty(
                "Reverse=yes\nEnd=4\nLoopEnd=4\nRate=300\nNext=G2",
                12,
                (0, 0),
            ),
            &mut env,
            &q,
            2000,
        );
        assert_eq!(f2.len(), 2000);
        let seq = sequence(&f2);
        assert_eq!(&seq[..8], &[3, 2, 1, 4, 3, 2, 1, 0]);
        assert!(seq[8..].iter().all(|&x| x < 0));
        // Switches off: G starts at k₀ = 0 (frame a′ = 2); G2's σ is set afresh (+1).
        let off = AnimQuirks {
            anim_followup_start_doubled: false,
            anim_followup_keeps_step: false,
            ..AnimQuirks::default()
        };
        let f = run(
            ty("Rate=300\nEnd=4\nNext=G", 12, (0, 0)),
            &mut env,
            &off,
            2000,
        );
        assert_eq!(sequence(&f)[4], 2);
        let f2 = run(
            ty(
                "Reverse=yes\nEnd=4\nLoopEnd=4\nRate=300\nNext=G2",
                12,
                (0, 0),
            ),
            &mut env,
            &off,
            2000,
        );
        assert_eq!(&sequence(&f2)[..6], &[3, 2, 1, 2, 3, 4]);
    }

    // spec: S-0109 claim 15 (a follow-up's random delay is chosen afresh: here the cosmetic draw)
    #[test]
    fn follow_up_redraws_its_delay() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        env.types.push((b"G".to_vec(), ty("", 4, (2, 5))));
        env.draws = vec![4];
        let mut a = Player::new(b"F", ty("End=1\nNext=G", 4, (0, 0)), 0, None, &mut env, &q);
        a.update(0, &mut env, &q);
        a.update(1, &mut env, &q);
        assert_eq!(a.name, b"G");
        assert_eq!(a.delay(), 4);
        assert_eq!(env.taken, vec![(2, 5)]);
    }

    // spec: S-0109 Edge cases (a follow-up name that names no type: absent-key defaults, end 0, so it
    // ends on its first advance)
    #[test]
    fn unknown_follow_up_ends() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let shown = run(ty("End=1\nNext=NOSUCH", 4, (0, 0)), &mut env, &q, 100);
        assert_eq!(shown, vec![Some(0), Some(0), None]);
    }
}
