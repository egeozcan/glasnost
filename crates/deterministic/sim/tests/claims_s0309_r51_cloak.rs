//! Claim tests of S-0309 revision 51, claim 28 (the cloak drops of S-0516
//! claim 14 reset the holder's targeting timer as in claim 27), on the
//! synthetic cloak rules of `cloak_common` (every name and number invented,
//! no game data). Going underground is a seam: no underground locomotor is
//! built.

mod cloak_common;

use cloak_common::{ALPHA, BETA, game, ruleset, run, spawn, state, until};
use glasnost_sim::cloak::CloakState;
use glasnost_sim::world::{ObjectId, Target, World};

/// Runs one tick with `foe` and `own` holding `s` as their target and
/// `left` ticks on their targeting timers: the draws taken in that tick,
/// and the timers left after it.
fn one_tick(w: &World, s: ObjectId, foe: ObjectId, own: ObjectId, left: i32) -> (u64, i32, i32) {
    let mut c = w.clone();
    let tick = c.tick;
    for x in [foe, own] {
        let o = c.obj_mut(x).unwrap();
        o.target = Some(Target::Object(s));
        o.targeting_timer = tick + left;
    }
    let before = c.rng.taken();
    run(&mut c, 1);
    assert_eq!(c.obj(foe).unwrap().target, None, "the foe drops it");
    assert_eq!(
        c.obj(own).unwrap().target,
        Some(Target::Object(s)),
        "the same owner keeps it"
    );
    let t = |x: ObjectId| c.obj(x).unwrap().targeting_timer - tick;
    (c.rng.taken() - before, t(foe), t(own))
}

// spec: S-0309 claim 28 (each drop at either cloak moment resets the holder's targeting timer as in claim 27: one ranged draw over 4 to 8 ticks with more than 10 ticks left, none with at most 10; a holder that keeps the target under an exception takes no draw)
#[test]
fn claim_28_cloak_drops_reset_the_timer() {
    let r = ruleset();
    let mut w = game(&r);
    let s = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    let foe = spawn(&mut w, "PLAIN", BETA, 6, 0);
    let own = spawn(&mut w, "PLAIN", ALPHA, -6, 0);
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut e = w.rng.clone();
    e.ranged(4, 8, half);
    let raws = e.taken() - w.rng.taken();
    // The start of cloaking: the first tick.
    let (base, foe_left, own_left) = one_tick(&w, s, foe, own, 5);
    assert_eq!((foe_left, own_left), (5, 5), "5 ticks left: no restart");
    let (n, foe_left, own_left) = one_tick(&w, s, foe, own, 50);
    assert_eq!(n, base + raws, "one draw, the foe's");
    assert!((4..=9).contains(&foe_left), "restarted: {foe_left}");
    assert_eq!(own_left, 50, "kept under the exception: no draw");
    // The full cloak: the tick in which it becomes cloaked.
    let mut c = w.clone();
    run(&mut c, 1);
    assert_eq!(state(&c, s), CloakState::Cloaking);
    let mut probe = c.clone();
    let k = until(&mut probe, 40, |w| state(w, s) == CloakState::Cloaked);
    run(&mut c, k - 1);
    assert_eq!(state(&c, s), CloakState::Cloaking);
    let (base, foe_left, _) = one_tick(&c, s, foe, own, 5);
    assert_eq!(foe_left, 5);
    let (n, foe_left, own_left) = one_tick(&c, s, foe, own, 50);
    assert_eq!(n, base + raws, "one draw at the full cloak");
    assert!((4..=9).contains(&foe_left), "restarted: {foe_left}");
    assert_eq!(own_left, 50);
}
