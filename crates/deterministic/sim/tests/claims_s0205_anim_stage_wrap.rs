//! The animation clock's stage arithmetic at the ends of the 32-bit range
//! (foundations F4, F3's seams): the stage *k* is an integer (S-0109 Terms),
//! and S-0205 claim 1 says an integer result outside the signed 32-bit
//! range wraps; S-0109 states no clamp. So an advance *k* + σ, a step back
//! *k* − σ and the image frame *a* + *k* wrap, where the clock used to
//! saturate. Synthetic fields only, every number our own.

use glasnost_sim::anim_clock::{AnimQuirks, Env, Playback, Player};

/// Surroundings that ask for nothing: no follow-up type, the low end of
/// every ranged draw, game speed 0, no normalisation.
struct Plain;

impl Env for Plain {
    fn follow_up(&mut self, _: &[u8]) -> Playback {
        Playback::missing()
    }
    fn draw(&mut self, lo: i32, _: i32) -> i32 {
        lo
    }
    fn speed(&self) -> u8 {
        0
    }
    fn normalise(&self) -> bool {
        false
    }
}

/// Fields with start `a`, end `e`, loop end 100, loop count `l` and stage
/// delay 1, played forwards.
fn fields(a: i32, e: i32, l: i32) -> Playback {
    Playback {
        start: a,
        end: e,
        loop_end: 100,
        loop_count: l,
        delay: 1,
        ..Playback::default()
    }
}

/// A player created in tick 0 with fields `p`.
fn player(p: Playback, q: &AnimQuirks) -> Player {
    Player::new(b"T", p, 0, None, &mut Plain, q)
}

/// The first update makes no advance (S-0109 claim 5); the second, one tick
/// later with delay 1, makes one.
fn first_advance(pl: &mut Player, q: &AnimQuirks) {
    pl.update(1, &mut Plain, q);
    pl.update(2, &mut Plain, q);
}

/// spec: S-0205 claim 1 (wrap); S-0109 Terms (an advance: one change of k by
/// σ) and F3 (k ≥ the limit λₑ − a ends the pass)
#[test]
fn an_advance_past_the_range_wraps() {
    let q = AnimQuirks::default();
    // Two passes, so the limit is λₑ − a = 100. From k = 2147483647 the
    // advance wraps to −2147483648, below the limit: the pass goes on.
    let mut pl = player(fields(0, 10, 2), &q);
    pl.keep_stage(i32::MAX);
    first_advance(&mut pl, &q);
    assert_eq!(pl.stage(), i32::MIN);
    assert_eq!(pl.remaining_passes(), Some(2));
    assert!(!pl.ended());
}

/// spec: S-0205 claim 1 (wrap); S-0611 claim 11 and S-0109 claim 16 (a
/// refused end steps the stage back by one)
#[test]
fn a_step_back_past_the_range_wraps() {
    let q = AnimQuirks::default();
    // One pass, so the limit is E = −2147483648 and every stage ends it.
    // The advance from 2147483647 wraps to −2147483648 and ends the
    // animation; the step back wraps to 2147483647.
    let mut pl = player(fields(0, i32::MIN, 1), &q);
    pl.keep_stage(i32::MAX);
    first_advance(&mut pl, &q);
    assert!(pl.ended());
    assert_eq!(pl.stage(), i32::MIN);
    pl.step_back();
    assert!(!pl.ended());
    assert_eq!(pl.stage(), i32::MAX);
}

/// spec: S-0205 claim 1 (wrap); S-0109 claim 1 (image frame a + k; outside
/// 0 … N − 1 it draws nothing)
#[test]
fn the_image_frame_past_the_range_wraps() {
    let q = AnimQuirks::default();
    let mut pl = player(fields(i32::MAX, 10, 1), &q);
    pl.keep_stage(1);
    assert_eq!(pl.frame(), i32::MIN);
    // A wrapped frame can land inside the sprite: −2147483648 twice is 0.
    let mut pl = player(fields(i32::MIN, 10, 1), &q);
    pl.keep_stage(i32::MIN);
    assert_eq!(pl.frame(), 0);
}
