//! Issue #458: an ordinary firer attacking a target that another firer's
//! temporal beam is erasing. S-0301 claim 9 leaves the firing check's answer
//! for such a target open (Q-3314), so S-0309 claim 29's check never drops
//! it. Guard test of today's behaviour: the firer keeps the target and
//! fires, and its shots do no damage (S-0302 claim 8, stage 1). Synthetic
//! rules (`temporal_common`, plus SHOOTER, an ordinary vehicle with the
//! Gun); every name and number is invented.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::temporal;
use glasnost_sim::world::{HouseId, Target};
use temporal_common::*;

// spec: S-0301 claim 9 (open: "what that one answers for a target that a temporal beam holds outside a warp is not settled here"; Q-3314) and S-0309 claim 29 (the 16-tick check drops a target only on *illegal* or *impossible*); S-0302 claim 8, stage 1 (row "The target is being erased": no damage); S-0514 claims 4 and 7 (E1: the lock removes the target after 34 updates, its health unchanged)
#[test]
fn issue_458_another_firer_keeps_an_erased_target_and_its_shots_do_nothing() {
    let mut w = world(|r| {
        vehicle(r, "SHOOTER", |t| t.primary = Some("Gun".into()));
    });
    while orders::blocked(&w) {
        glasnost_sim::tick::step(&mut w);
    }
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    let s = spawn(&mut w, Kind::Vehicle, "SHOOTER", 0, at(-3, 0));
    let n = erase(&mut w, a, v);
    assert!(erased(&w, v), "void unless the beam erases it");
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
    let mut shots = 0;
    let mut ticks = 0;
    while w.obj(v).is_some() {
        assert!(temporal::erased(&w, v), "being erased until it is removed");
        assert_eq!(
            w.obj(s).unwrap().target,
            Some(Target::Object(v)),
            "the other firer keeps the target (tick {})",
            w.tick
        );
        assert_eq!(health(&w, v), 100, "no damage (tick {})", w.tick);
        let before: Vec<_> = w.projectiles.keys().copied().collect();
        glasnost_sim::testkit::run(&mut w, 1);
        shots += w
            .projectiles
            .iter()
            .filter(|(id, p)| p.firer == Some(s) && !before.contains(id))
            .count();
        ticks += 1;
        assert!(ticks < 60, "the lock removes it");
    }
    assert!(w.tick >= n + 34, "removed after the stated updates");
    assert!(ticks >= 32, "void unless two of claim 29's 16-tick checks passed");
    assert!(shots >= 1, "void unless the other firer fired");
}
