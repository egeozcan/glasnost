//! S-0518 claim 2: a building whose own delayed shot is pending is not
//! eligible as a supporter, even once it has left *attack* (#341).
//! Synthetic data: the `linked_common` fixture (prism type PRISM, weapon
//! PrismGun, a target vehicle VICT).

#[path = "linked_common/mod.rs"]
mod linked_common;

use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use linked_common::*;

fn quiet(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let b = spawn(w, Kind::Building, name, house, at(dx, dy));
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(b) {
        o.beh.force(Behaviour::Sleep, tick, stale);
    }
    b
}

// spec: S-0518 claim 2 (B's delayed-fire countdown is 0: this keeps out a building whose beam or shot is pending)
#[test]
fn issue_341_a_building_with_its_own_shot_pending_is_not_recruited() {
    let mut w = world(|r| {
        r.sim.buildings.get_mut("PRISM").unwrap().delayed_fire_delay = Some(10);
        r.sim.weapons.get_mut("PrismGun").unwrap().range = Some(d("10"));
    });
    let a = quiet(&mut w, "PRISM", 0, 5, 0);
    let b = quiet(&mut w, "PRISM", 0, -2, 0);
    let v1 = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(9, 0));
    let v2 = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(-9, 0));
    run_to(&mut w, 5);
    // B is rearming while A starts, so A recruits nobody and starts its own delayed shot.
    w.obj_mut(b).unwrap().combat.rearm_until = w.tick + 500;
    attack(&mut w, a, v1);
    testkit::run(&mut w, 1);
    let due = w
        .obj(a)
        .unwrap()
        .combat
        .delayed_shot
        .expect("A's own delayed shot is pending");
    assert_eq!(support(&w, a), 0);
    // A's target leaves play during the delay.
    glasnost_sim::objects::remove(&mut w, v1);
    let mut left = false;
    for _ in 0..4 {
        testkit::run(&mut w, 1);
        if w.obj(a).unwrap().beh.current != Some(Behaviour::Attack) {
            left = true;
            break;
        }
    }
    assert!(left, "A left attack");
    assert!(w.tick < due, "still inside the delay");
    assert_eq!(
        w.obj(a).unwrap().combat.delayed_shot,
        Some(due),
        "A's shot is still pending"
    );
    // B's rearm has run out; B gets can fire on its own target.
    w.obj_mut(b).unwrap().combat.rearm_until = 0;
    attack(&mut w, b, v2);
    testkit::run(&mut w, 1);
    assert!(w.tick <= due, "B was processed before A's delay ended");
    assert_eq!(
        w.obj(a).unwrap().combat.support_beam,
        None,
        "A, with its own shot pending, was recruited"
    );
    assert_eq!(support(&w, b), 0, "B counted A");
}
