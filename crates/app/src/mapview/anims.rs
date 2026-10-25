//! Animation playback for the drawing (S-0109 claims 1 to 16). The stage
//! clock itself is the simulation's (`glasnost_sim::anim_clock`), shared so
//! that the drawing and the simulation's infantry-making animations (S-0611)
//! play alike.
//!
//! The drawing's playback is presentation state and never feeds the
//! simulation. The draws the simulation does not take for the animations it
//! does not play (a follow-up type's random delay, the random loop delays
//! between passes) come from a cosmetic stream of the drawing's own. S-0109
//! claim 28 makes them scenario-stream simulation effects; for animations
//! other than infantry-making ones that is a seam (docs/development.md).
//!
//! Damage, start and middle events (claims 13, 14 and 17) are the
//! simulation's and are not played here. Held animations (claim 23) are
//! building slot animations, which `Live::update_slots` holds through
//! `Player::update_held`; translucency levels above 0 (claim 20) belong to
//! building slot animations too.

pub use glasnost_sim::anim_clock::{AnimQuirks, Env, Fields, Playback, Player};

use super::slots::AnimType;

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

/// The playback fields of art type `t` as given.
pub fn fields(t: &AnimType) -> Fields {
    Fields {
        start: t.start,
        end: t.end,
        loop_end: t.loop_end,
        loop_start: t.loop_start,
        loop_count: t.loop_count,
        reverse: t.reverse,
        ping_pong: t.ping_pong,
        shadow: t.shadow,
        new_theater: t.new_theater,
        normalized: t.normalized,
        random_loop_delay: t.random_loop_delay,
        next: t.next.clone(),
    }
}

/// The fields in play of type `t` whose sprite holds `n` image frames
/// (F2's assumed case: the first sprite found is the one drawn), with
/// claim 2's delay `delay` and claim 3's range `random_delay`.
pub fn playback_of(
    t: &AnimType,
    n: i32,
    delay: i32,
    random_delay: (i32, i32),
    q: &AnimQuirks,
) -> Playback {
    Playback::of_fields(&fields(t), n, delay, random_delay, q)
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
        playback_of(&t, n, d, random_delay, &AnimQuirks::default())
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

    /// [`run`] for an animation that replaced one at stage `k` (S-0108
    /// claim 25).
    fn run_kept(p: Playback, k: i32, env: &mut TestEnv, limit: i32) -> Vec<Option<i32>> {
        let q = AnimQuirks::default();
        let mut a = Player::new(b"A", p, 0, None, env, &q);
        a.keep_stage(k);
        let mut out = Vec::new();
        for t in 0..limit {
            if a.update(t, env, &q) {
                out.push(Some(a.frame()));
            } else {
                out.push(None);
                break;
            }
        }
        out
    }

    // spec: S-0109 claim 23 (a held animation does not advance; its first-update exclusion and its loop delay still run, and its clock keeps running, so it advances at its first eligible update after its release)
    #[test]
    fn a_held_animation_keeps_its_clock() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let mut a = Player::new(b"A", ty("", 10, (0, 0)), 0, None, &mut env, &q);
        for t in 0..5 {
            assert!(a.update_held(t, &mut env, &q, true));
            assert_eq!(a.frame(), 0, "held (tick {t})");
        }
        assert!(a.update_held(5, &mut env, &q, false));
        assert_eq!(
            a.frame(),
            1,
            "it advances at the first update after its release"
        );
        let c = glasnost_sim::anim_clock::Creation {
            loop_delay: 3,
            ..Default::default()
        };
        let mut b = Player::create(b"B", ty("", 10, (0, 0)), 0, None, c, &mut env, &q);
        for t in 0..4 {
            assert!(b.update_held(t, &mut env, &q, true));
        }
        assert_eq!(b.loop_delay(), 0, "the loop delay counted down while held");
        assert_eq!(b.frame(), 0);
        assert!(b.update_held(4, &mut env, &q, false));
        assert_eq!(b.frame(), 1);
    }

    // spec: S-0108 claim 25 (the stage is kept, not the image frame: the new animation shows its own start field plus the kept stage) and S-0109 claim 1 (image frame a + k)
    #[test]
    fn keep_stage_shows_its_own_start_plus_the_stage() {
        let q = AnimQuirks::default();
        let mut env = TestEnv::new();
        let mut a = Player::new(b"A", ty("Start=2", 10, (0, 0)), 0, None, &mut env, &q);
        assert_eq!(a.frame(), 2);
        a.keep_stage(4);
        assert_eq!((a.frame(), a.stage()), (6, 4));
    }

    // spec: S-0108 claim 25 (the passes are not kept: they take their creation value; the partial pass it starts in uses up one of them like any other pass)
    #[test]
    fn kept_stage_partial_pass_counts_whole() {
        let mut env = TestEnv::new();
        let shown = run_kept(ty("LoopCount=2", 10, (0, 0)), 7, &mut env, 1000);
        let mut want = vec![7, 8, 9];
        want.extend(0..10);
        assert_eq!(
            sequence(&shown),
            want,
            "the partial pass, then one whole pass"
        );
        assert_eq!(shown.last(), Some(&None), "then it ends");
    }

    // spec: S-0108 claim 25 (a kept stage beyond the new type's range: a forward type that is not ping-pong shows a + k until its first advance, which ends the pass; a forward ping-pong type alternates between k and k + 1 and never ends a pass) and S-0109 claims 1, 6 and 9
    #[test]
    fn kept_stage_beyond_the_range() {
        let mut env = TestEnv::new();
        let shown = run_kept(ty("", 10, (0, 0)), 12, &mut env, 1000);
        assert_eq!(shown, vec![Some(12), None], "one advance ends its one pass");
        let shown = run_kept(
            ty("PingPong=yes\nLoopCount=-1", 10, (0, 0)),
            12,
            &mut env,
            40,
        );
        assert_eq!(sequence(&shown).len(), shown.len(), "a new stage each tick");
        assert!(
            shown.iter().all(|f| matches!(f, Some(12 | 13))),
            "{shown:?}"
        );
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
        assert_eq!(a.remaining_passes(), Some(256));
        let art = glasnost_ini::Document::parse(
            b"[A]\nEnd=6\nLoopCount=2\nRate=300\n",
            Default::default(),
            &mut Vec::new(),
        );
        let t = super::super::slots::read_anim(&art, b"A");
        let p = playback_of(&t, 10, 3, (0, 0), &q);
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
