//! A drain-mode superweapon of the missile kind launches through its silo
//! as an ordinary launch does (#350; S-0512 claim 19, S-0513 claim 7). On
//! the synthetic superweapon rules of `sw_common`: invented types and
//! numbers, no game data.

mod sw_common;

use glasnost_sim::superweapons::{DrainState, SwEvent, nuke};
use glasnost_sim::world::World;
use sw_common::*;

fn lifts(w: &World) -> usize {
    let wp = *w.rules().combat.weapon_index.get("Lift").unwrap();
    w.projectiles.values().filter(|p| p.weapon == wp).count()
}

// spec: S-0513 claim 7 (a launch of an ordinary nuclear strike record looks for the launching building)
#[test]
fn issue_350_control_ordinary_missile_type_launches_through_its_silo() {
    let r = ruleset();
    let mut w = game(&r);
    let silo = spawn(&mut w, "SILO", ALPHA, -6, -6);
    ready_tick(&mut w, ALPHA, STRIKE, 200);
    assert_eq!(nuke::silo_choice(&w, ALPHA, STRIKE), Some(silo));
    w.effects.clear();
    fire_now(&mut w, ALPHA, STRIKE, at(3, 3));
    assert!(events(&w).iter().any(|e| e.0 == SwEvent::Siren));
    assert_eq!(w.sw.houses[0].nuke_target, Some(at(3, 3)));
    step(&mut w);
    assert_eq!(lifts(&w), 1);
}

// spec: S-0512 claim 19 (a fire order in ready starts the kind's launch effect and moves to active) and S-0513 claim 7 (a launch of an ordinary nuclear strike record looks for the launching building)
#[test]
fn issue_350_drain_mode_missile_type_launches_through_its_silo() {
    let mut r = ruleset();
    r.sim
        .superweapons
        .get_mut("STRIKE")
        .unwrap()
        .use_charge_drain = Some(true);
    let mut w = game(&r);
    let silo = spawn(&mut w, "SILO", ALPHA, -6, -6);
    until(&mut w, 200, |w| {
        rec(w, ALPHA, STRIKE).drain == DrainState::Ready
    });
    assert!(!rec(&w, ALPHA, STRIKE).one_time);
    assert_eq!(nuke::silo_choice(&w, ALPHA, STRIKE), Some(silo));
    assert!(nuke::accepts(&w, silo));
    w.effects.clear();
    fire_now(&mut w, ALPHA, STRIKE, at(3, 3));
    assert_eq!(rec(&w, ALPHA, STRIKE).drain, DrainState::Active);
    let siren = events(&w).iter().any(|e| e.0 == SwEvent::Siren);
    let target = w.sw.houses[0].nuke_target;
    step(&mut w);
    let n = lifts(&w);
    assert!(
        siren && target == Some(at(3, 3)) && n == 1,
        "drain-mode launch: siren {siren}, recorded target {target:?}, upward missiles {n}"
    );
}
