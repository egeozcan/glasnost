//! A building grant after a fired one-time record that also got a
//! permanent grant is an ordinary building grant (#351; S-0512 claims 7, 12
//! and 13). On the synthetic superweapon rules of `sw_common`: invented
//! types and numbers, no game data.

mod sw_common;

use glasnost_sim::superweapons::charge;
use glasnost_sim::world::World;
use sw_common::*;

/// A one-time grant, then a permanent grant of the same type, then the
/// fire order that consumes the one-time record.
fn consumed(w: &mut World, permanent: bool) {
    charge::one_time_grant(w, ALPHA, CURTAIN);
    if permanent {
        charge::permanent_grant(w, ALPHA, CURTAIN);
    }
    fire_now(w, ALPHA, CURTAIN, at(5, 5));
    assert!(!rec(w, ALPHA, CURTAIN).granted, "set-up: consumed");
}

// spec: S-0512 claim 12 (the next grant check grants it again in the usual way) and claim 7 (an ordinary building grant is lost with its last giving building)
#[test]
fn issue_351_regrant_after_one_time_plus_permanent_is_lost_with_its_building() {
    for permanent in [false, true] {
        let r = ruleset();
        let mut w = game(&r);
        consumed(&mut w, permanent);
        let b = spawn(&mut w, "IRONB", ALPHA, 0, 0);
        until(&mut w, 20, |w| rec(w, ALPHA, CURTAIN).granted);
        remove(&mut w, b);
        run(&mut w, 5);
        let x = rec(&w, ALPHA, CURTAIN);
        assert!(
            !x.granted,
            "permanent grant before the fire order: {permanent}; building grant kept without a giving building (permanent = {})",
            x.permanent
        );
    }
}

// spec: S-0512 claim 12 (granted again in the usual way), claim 4 (a building grant under low power starts suspended and is resumed by the rules of claim 6) and claim 6 (resumed when the power is full)
#[test]
fn issue_351_regrant_under_low_power_after_one_time_plus_permanent_resumes_with_power() {
    for permanent in [false, true] {
        let r = ruleset();
        let mut w = game(&r);
        consumed(&mut w, permanent);
        let hog = spawn(&mut w, "HOG", ALPHA, -4, 0);
        run(&mut w, 3);
        spawn(&mut w, "IRONB", ALPHA, 0, 0);
        until(&mut w, 20, |w| rec(w, ALPHA, CURTAIN).granted);
        assert!(
            rec(&w, ALPHA, CURTAIN).suspended,
            "set-up: granted suspended (claim 4)"
        );
        remove(&mut w, hog);
        run(&mut w, 10);
        let x = rec(&w, ALPHA, CURTAIN);
        assert!(
            !x.suspended,
            "permanent grant before the fire order: {permanent}; still suspended after power returned (permanent = {})",
            x.permanent
        );
    }
}
