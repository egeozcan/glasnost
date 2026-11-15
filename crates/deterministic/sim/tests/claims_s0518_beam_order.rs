//! S-0518 claim 6: the support beam's tick depends on where the supporter
//! stands in the update order (#339). Synthetic data: the `linked_common`
//! fixture (prism type PRISM, support rearm 100, a target vehicle VICT).

#[path = "linked_common/mod.rs"]
mod linked_common;

use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use linked_common::*;

fn quiet(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    let b = spawn(w, Kind::Building, "PRISM", 0, at(dx, dy));
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(b) {
        o.beh.force(Behaviour::Sleep, tick, stale);
    }
    b
}

/// (ticks from the recruitment tick to the beam tick, rearm_until minus the
/// recruitment tick), for a delayed-fire field `d`.
fn gap(supporter_first: bool, d: i32) -> (i32, i32) {
    let mut w = world(|r| {
        r.sim.buildings.get_mut("PRISM").unwrap().delayed_fire_delay = Some(d);
    });
    let (m, s);
    if supporter_first {
        s = quiet(&mut w, -1, 0);
        m = quiet(&mut w, 0, 0);
    } else {
        m = quiet(&mut w, 0, 0);
        s = quiet(&mut w, -1, 0);
    }
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(4, 0));
    let pm = w.update.order.iter().position(|&x| x == m).unwrap();
    let ps = w.update.order.iter().position(|&x| x == s).unwrap();
    assert_eq!(ps < pm, supporter_first, "the update order of the scene");
    run_to(&mut w, 5);
    attack(&mut w, m, v);
    let mut recruited = None;
    let mut beam = None;
    for _ in 0..20 {
        w.effects.clear();
        let t = w.tick;
        testkit::run(&mut w, 1);
        if recruited.is_none() && support(&w, m) == 1 {
            recruited = Some(t);
        }
        if beam.is_none()
            && w.effects
                .iter()
                .any(|e| matches!(e, Effect::SupportBeam { .. }))
        {
            beam = Some(t);
        }
    }
    let (r, b) = (recruited.expect("recruited"), beam.expect("beam"));
    (b - r, w.obj(s).unwrap().combat.rearm_until - r)
}

// spec: S-0518 claim 6 (When: max(d − 1, 0) ticks after the tick of recruitment for a supporter after the master in the update order; one tick later for a supporter before it; the rearm countdown of R_s starts in the beam's turn)
#[test]
fn issue_339_supporter_before_the_master_beams_one_tick_later() {
    for d in [0, 1, 2, 5] {
        let base = (d - 1).max(0);
        // The fixture's support rearm is 100 ticks.
        assert_eq!(gap(false, d), (base, base + 100), "d = {d}: after the master");
        assert_eq!(
            gap(true, d),
            (base + 1, base + 101),
            "d = {d}: before the master"
        );
    }
}
