//! The animation clock at extreme art values (#153): its differences are
//! integer results, and S-0205 claim 1 says an integer result outside the
//! signed 32-bit range wraps. A debug build must give the wrapped result,
//! as a release build does, not panic. Synthetic fields only, every number
//! our own.

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

/// Fields with start `a`, loop end `le`, loop count 2 and stage delay 1.
fn fields(a: i32, le: i32) -> Playback {
    Playback {
        start: a,
        end: 10,
        loop_end: le,
        loop_count: 2,
        delay: 1,
        ..Playback::default()
    }
}

/// A player created in tick `t` with fields `p`.
fn player(p: Playback, t: i32, q: &AnimQuirks) -> Player {
    Player::new(b"T", p, t, None, &mut Plain, q)
}

/// The first update makes no advance (S-0109 claim 5); the second, one tick
/// later with delay 1, makes one.
fn first_advance(pl: &mut Player, t: i32, q: &AnimQuirks) {
    pl.update(t + 1, &mut Plain, q);
    pl.update(t + 2, &mut Plain, q);
}

/// spec: S-0205 claim 1 (wrap); S-0109 claim 6 and F3 (limit λₑ − a while
/// r ≥ 2; a new pass at k′ = λₛ − a)
#[test]
fn a_loop_limit_past_the_range_wraps() {
    let q = AnimQuirks::default();
    // λₑ − a = 2147483647 + 1 wraps to −2147483648, so the first advance,
    // to k = 1, ends the first pass; the new pass starts at λₛ − a = 1.
    let mut pl = player(fields(-1, i32::MAX), 0, &q);
    assert_eq!(pl.remaining_passes(), Some(2));
    first_advance(&mut pl, 0, &q);
    assert_eq!(pl.remaining_passes(), Some(1));
    assert_eq!(pl.stage(), 1);
}

/// spec: S-0205 claim 1 (wrap); S-0109 claim 8 and F3 (a reversed start
/// k₀ = λₑ − 1)
#[test]
fn a_reversed_start_past_the_range_wraps() {
    let q = AnimQuirks::default();
    let mut p = fields(0, i32::MIN);
    p.reverse = true;
    let pl = player(p, 0, &q);
    assert_eq!(pl.stage(), i32::MIN.wrapping_sub(1));
    assert_eq!(pl.stage(), i32::MAX);
}

/// spec: S-0205 claim 1 (wrap); S-0109 Edge cases (quirks.anim_reverse_raw_loop_end
/// off: a reversed start and a new reversed pass at λₑ − a − 1)
#[test]
fn a_reversed_start_and_new_pass_wrap_with_the_raw_loop_end_off() {
    let q = AnimQuirks {
        anim_reverse_raw_loop_end: false,
        ..AnimQuirks::default()
    };
    let le = i32::MIN + 1;
    let mut p = fields(1, le);
    p.reverse = true;
    let mut pl = player(p, 0, &q);
    // k₀ = λₑ − a − 1 = −2147483649, wrapped.
    let k0 = le.wrapping_sub(1).wrapping_sub(1);
    assert_eq!(k0, i32::MAX);
    assert_eq!(pl.stage(), k0);
    // The advance steps to k₀ − 1, at or above the limit λₑ − a: a new pass
    // at λₑ − a − 1, wrapped again.
    first_advance(&mut pl, 0, &q);
    assert_eq!(pl.remaining_passes(), Some(1));
    assert_eq!(pl.stage(), k0);
}

/// spec: S-0205 claim 1 (wrap); S-0109 claim 9 (quirks.anim_pingpong_lower_turn
/// off: the lower turn at k ≤ λₛ − a)
#[test]
fn a_ping_pong_lower_turn_past_the_range_wraps() {
    let q = AnimQuirks {
        anim_pingpong_lower_turn: false,
        ..AnimQuirks::default()
    };
    let mut p = fields(1, 100);
    p.ping_pong = true;
    p.loop_start = i32::MIN;
    let mut pl = player(p, 0, &q);
    // λₛ − a = −2147483649 wraps to 2147483647: every stage is at or below
    // it, so each advance turns.
    first_advance(&mut pl, 0, &q);
    assert_eq!(pl.stage(), 1);
    pl.update(3, &mut Plain, &q);
    assert_eq!(pl.stage(), 0);
    pl.update(4, &mut Plain, &q);
    assert_eq!(pl.stage(), 1);
}

/// spec: S-0205 claim 1 (wrap); S-0109 claim 5 (an advance when at least d
/// ticks have passed since the last advance)
#[test]
fn the_ticks_since_the_last_advance_wrap() {
    let q = AnimQuirks::default();
    let mut pl = player(fields(0, 100), i32::MIN, &q);
    pl.update(i32::MIN, &mut Plain, &q);
    // 1 − (−2147483648) wraps to −2147483647, below the delay: no advance.
    pl.update(1, &mut Plain, &q);
    assert_eq!(pl.stage(), 0);
    // From −1 the difference is 2147483647: an advance.
    let mut pl = player(fields(0, 100), i32::MIN, &q);
    pl.update(i32::MIN, &mut Plain, &q);
    pl.update(-1, &mut Plain, &q);
    assert_eq!(pl.stage(), 1);
}
