//! The turret rest rows and a deployed simple-deployer vehicle (S-0206
//! claim 9, scenario R9 runs c and d; sim issue #184). Synthetic types of
//! the S-0517 fixtures, with the turret flag added.

mod deploy_common;

use deploy_common::*;
use glasnost_sim::econ::inplace::{self, Form};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::turning::Turning;

// spec: S-0206 claim 9 (the rest rows: not a deployed simple-deployer vehicle; otherwise no change), scenario R9 runs c and d
#[test]
fn a_deployed_simple_deployer_turret_keeps_its_facing() {
    let mut r = ruleset();
    let mut t = r.sim.vehicles["TANK"].clone();
    t.turret = Some(true);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("TTANK".into());
    r.sim.vehicles.insert("TTANK".into(), t);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    let dep = put(&mut w, "TTANK", Kind::Vehicle, 0, -8, 0);
    let ctl = put(&mut w, "TTANK", Kind::Vehicle, 0, -4, 0);
    deploy(&mut w, dep);
    for _ in 0..200 {
        if inplace::form(&w, dep) == Form::Deployed {
            break;
        }
        step(&mut w);
    }
    assert_eq!(inplace::form(&w, dep), Form::Deployed);
    // The state right after a last shot at a facing a quarter turn off the
    // body.
    let now = w.tick;
    for id in [dep, ctl] {
        let o = w.obj_mut(id).unwrap();
        let f = o.body.value(now).wrapping_add(16384);
        o.turret = Turning::at(f);
        o.last_fired = now;
        o.target = None;
    }
    let fired_at = w.obj(dep).unwrap().turret.value(now);
    let g = w.rules_arc().general.guard_area_targeting_delay;
    testkit::run(&mut w, g + 50);
    let t = w.tick;
    for id in [dep, ctl] {
        let o = w.obj(id).unwrap();
        assert_eq!(o.target, None);
        assert_eq!(o.dest, None);
    }
    assert_eq!(inplace::form(&w, dep), Form::Deployed);
    let c = w.obj(ctl).unwrap();
    assert_eq!(
        c.turret.target,
        c.body.value(t),
        "run d: the mobile vehicle's turret rests on the body"
    );
    let o = w.obj(dep).unwrap();
    assert_eq!(
        o.turret.target, fired_at,
        "run c: the deployed vehicle's turret keeps the facing at which it last fired"
    );
}
