//! S-0206 claims 8 and 9: a vehicle's turret without a target.

use std::sync::Arc;

use glasnost_num::geom::direction;
use glasnost_sim::load::new_game;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, Pos, Target};

// spec: S-0206 claim 8 (the turret facing is absolute: the body turns, the turret keeps or follows its own target)
// spec: S-0206 claim 9 (no target: the turret turns to the direction of the movement destination, else to the body)
#[test]
fn turret_rests_on_the_destination_then_the_body() {
    let mut r = testkit::ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().turret = Some(true);
    let rules = Arc::new(SimRules::new(&r, "turret"));
    let mut w = new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(2),
    )
    .unwrap();
    // The rest rows need G + 5 ticks since the last shot (never fired: −100).
    while w.tick < 4 {
        tick::step(&mut w);
    }
    let id = testkit::house_objects(&w, 0)
        .into_iter()
        .find(|&i| {
            w.obj(i)
                .is_some_and(|o| o.kind == Kind::Vehicle && o.type_index == 0)
        })
        .unwrap();
    let dest = testkit::cell(-6, 12);
    orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(dest)),
        },
    );
    for _ in 0..3 {
        tick::step(&mut w);
    }
    let o = w.obj(id).unwrap();
    let p = Pos::centre(dest, 0);
    let want = direction(i64::from(p.x - o.pos.x), i64::from(p.y - o.pos.y), true);
    assert_eq!(o.turret.target, want);
    for _ in 0..800 {
        tick::step(&mut w);
    }
    let o = w.obj(id).unwrap();
    assert_eq!(o.dest, None, "the cart arrived");
    assert_eq!(o.turret.value(w.tick), o.body.value(w.tick));
}
