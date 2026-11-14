//! S-0309 claim 2 on synthetic data: a harvesting slave does not search
//! passively (sim issues S12, issue 244).

use glasnost_sim::combat::acquire;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{ObjectId, World};

const SIZE: i32 = 40;

fn force(w: &mut World, id: ObjectId, b: Behaviour) {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(id).unwrap().beh.force(b, t, stale);
}

// spec: S-0309 claim 2 (it is not a harvesting slave, S-0215); scenario A1 part l with its control
#[test]
fn claim_2_a_harvesting_slave_does_not_search_passively() {
    let mut got = Vec::new();
    for slave in [false, true] {
        let mut r = testkit::combat_ruleset();
        r.sim.infantry.get_mut("WALKER").unwrap().primary = Some("Gun".into());
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        cart.econ.resource_gatherer = Some(true);
        cart.econ.resource_destination = Some(true);
        cart.econ.enslaves = Some("WALKER".into());
        cart.econ.slaves_number = Some(1);
        let mut w = testkit::game_with(&r, SIZE, 5);
        let m = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 6, 6), 0);
        let v = if slave {
            let s = glasnost_sim::econ::slaves::slaves_of(&w, m)[0];
            let p = glasnost_sim::world::Pos::centre(mid(SIZE, 0, 0), 0);
            glasnost_sim::objects::place(&mut w, s, p, 0);
            s
        } else {
            spawn(&mut w, Kind::Infantry, "WALKER", 1, mid(SIZE, 0, 0), 0)
        };
        let e = spawn(&mut w, Kind::Vehicle, "BUGGY", 0, mid(SIZE, -2, 0), 64);
        force(&mut w, e, Behaviour::Sleep);
        force(&mut w, v, Behaviour::Guard);
        assert_eq!(glasnost_sim::seams::is_harvesting_slave(&w, v), slave);
        let gate = acquire::may_search_passively(&w, v);
        acquire::per_tick(&mut w, v);
        let target = w.obj(v).unwrap().target;
        got.push((slave, gate, target.is_some()));
    }
    // The control (no master) searches and picks the enemy.
    assert_eq!(got[0], (false, true, true), "control");
    // The harvesting slave may not search and picks nothing.
    assert_eq!(got[1], (true, false, false), "harvesting slave");
}
