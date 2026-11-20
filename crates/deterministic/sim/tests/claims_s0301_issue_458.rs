//! Issue #458: an ordinary firer attacking a target that another firer's
//! temporal beam is erasing. S-0301 r80 claim 9 (Q-3314 answered) gives
//! such a target *illegal* for every weapon that is not temporal, so from
//! the lock on the firer makes no shot at it, drops it at S-0309 claim 29's
//! first check tick after the lock and does not take it again. Synthetic
//! rules (`temporal_common`, plus SHOOTER, an ordinary vehicle with the
//! Gun); every name and number is invented.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::temporal;
use glasnost_sim::world::{HouseId, Target};
use temporal_common::*;

// spec: S-0301 claim 9 (r80, Targets being erased within rank 4, Q-3314: illegal for every weapon that is not temporal, from the lock on; no shot from the lock on; a vehicle keeps the target until its checking turn in the first check tick of S-0309 claim 29 after the lock and drops it then; a search rejects it, S-0309 table C firing check row); S-0309 claim 29 (Phase: a multiple of 16); S-0302 claim 8, stage 1 (a projectile already in flight does no damage); S-0514 claims 4 and 7 (E1: the lock removes the target after 34 updates)
#[test]
fn issue_458_another_firer_drops_an_erased_target_at_the_first_check_tick() {
    let mut w = world(|r| {
        vehicle(r, "SHOOTER", |t| t.primary = Some("Gun".into()));
    });
    while orders::blocked(&w) {
        glasnost_sim::tick::step(&mut w);
    }
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-6, -6));
    let s = spawn(&mut w, Kind::Vehicle, "SHOOTER", 0, at(-3, 0));
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: s,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(v)),
            dest: None,
        },
    ));
    glasnost_sim::testkit::run(&mut w, 1);
    assert_eq!(w.obj(s).unwrap().target, Some(Target::Object(v)));
    // The lock: L is the tick at whose end the beam is first attached.
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        a,
        0,
        Target::Object(v)
    ));
    let mut l = None;
    for _ in 0..60 {
        let k = w.tick;
        glasnost_sim::tick::step(&mut w);
        if temporal::erased(&w, v) {
            l = Some(k);
            break;
        }
    }
    let l = l.expect("void unless the beam locks");
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    let o = w.obj_mut(a).unwrap();
    o.target = Some(Target::Object(v));
    o.beh.force(Behaviour::Attack, tick, stale);
    let c = (l / 16 + 1) * 16;
    let strength = health(&w, v);
    let mark = w.obj(s).unwrap().combat.rearm_until;
    let mut ticks = 0;
    while w.obj(v).is_some() {
        let k = w.tick;
        glasnost_sim::testkit::run(&mut w, 1);
        if w.obj(v).is_none() {
            break;
        }
        assert!(temporal::erased(&w, v), "being erased until it is removed");
        let held = w.obj(s).unwrap().target;
        if k < c && l % 16 != 0 {
            assert_eq!(held, Some(Target::Object(v)), "held until c (tick {k})");
        } else if k >= c {
            assert_eq!(held, None, "dropped at c, not taken again (tick {k})");
        }
        assert_eq!(health(&w, v), strength, "no damage (tick {k})");
        assert_eq!(
            w.obj(s).unwrap().combat.rearm_until,
            mark,
            "no shot from the lock on (tick {k})"
        );
        ticks += 1;
        assert!(ticks < 60, "the lock removes it");
    }
    assert!(w.tick >= l + 34, "removed after the stated updates");
    assert!(ticks >= 16, "void unless claim 29's check tick passed");
}
