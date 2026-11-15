//! A charge reset or set-charge after a first-stage source pick ends the
//! pick's silent re-completion: the next completion is announced and sets
//! the ready tick (#353; S-0512 claims 5, 10, 14 and 15). On the synthetic
//! superweapon rules of `sw_common`: invented types and numbers, no game
//! data.

mod sw_common;

use glasnost_sim::superweapons::SwEvent;
use glasnost_sim::superweapons::charge;
use glasnost_sim::world::World;
use sw_common::*;

/// Runs until the record is ready again; returns whether a ready
/// announcement was requested in the completing tick, and that tick.
fn recharge(w: &mut World) -> (bool, i32) {
    for _ in 0..(T + 5) {
        w.effects.clear();
        let t = w.tick;
        step(w);
        if rec(w, ALPHA, SOURCE).ready {
            let said = events(w)
                .iter()
                .any(|e| e.0 == SwEvent::Ready && e.1 == Some(ALPHA) && e.2 == Some(SOURCE));
            return (said, t);
        }
    }
    panic!("not ready again");
}

fn picked() -> World {
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "CHRONOB", ALPHA, 0, 0);
    ready_tick(&mut w, ALPHA, SOURCE, 200);
    // The pick is applied in this tick's order step, after the houses.
    fire_now(&mut w, ALPHA, SOURCE, at(3, 3));
    assert!(!rec(&w, ALPHA, SOURCE).ready, "set-up: picked");
    w
}

// spec: S-0512 claims 5 and 15 (a reset record becomes ready again with its announcement and a new ready tick)
#[test]
fn issue_353_control_reset_without_a_pick_announces_and_sets_the_ready_tick() {
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "CHRONOB", ALPHA, 0, 0);
    ready_tick(&mut w, ALPHA, SOURCE, 200);
    let l = w.tick;
    charge::reset(&mut w, ALPHA, SOURCE);
    let (said, done) = recharge(&mut w);
    assert_eq!(done, l + T);
    assert!(said, "control: announcement");
    assert_eq!(rec(&w, ALPHA, SOURCE).ready_tick, done, "control: ready tick");
}

// spec: S-0512 claim 10 (only the pick's own re-completion is silent), claim 15 (the reset) and claim 5 (becoming ready requests the announcement; the tick is the ready tick)
#[test]
fn issue_353_reset_after_a_source_pick_announces_the_full_recharge() {
    let mut w = picked();
    let first = rec(&w, ALPHA, SOURCE).ready_tick;
    // The reset of claim 15, before the next house update.
    let l = w.tick;
    charge::reset(&mut w, ALPHA, SOURCE);
    let x = rec(&w, ALPHA, SOURCE);
    assert!(!x.ready && x.running && x.remain == T && x.start == l, "set-up");
    let (said, done) = recharge(&mut w);
    assert_eq!(done, l + T, "a full charge of T ticks");
    assert!(said, "S-0512 claim 5: the full recharge requests its announcement");
    assert_eq!(
        rec(&w, ALPHA, SOURCE).ready_tick,
        done,
        "S-0512 claim 5: the ready tick is the completion tick (was {first})"
    );
}

// spec: S-0512 claim 10 (only the pick's own re-completion is silent) and claim 14 (after setting the charge the record becomes ready through claim 5, with its announcement)
#[test]
fn issue_353_set_charge_after_a_source_pick_announces_the_completion() {
    let mut w = picked();
    let first = rec(&w, ALPHA, SOURCE).ready_tick;
    charge::set_charge(&mut w, ALPHA, SOURCE, 50);
    let x = rec(&w, ALPHA, SOURCE);
    assert!(!x.ready && x.running && x.remain > 0, "set-up: charging again");
    let (said, done) = recharge(&mut w);
    assert!(said, "S-0512 claim 14: the completion requests its announcement");
    assert_eq!(
        rec(&w, ALPHA, SOURCE).ready_tick,
        done,
        "S-0512 claim 5: the ready tick is the completion tick (was {first})"
    );
}
