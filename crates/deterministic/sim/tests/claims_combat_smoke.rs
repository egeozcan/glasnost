//! A combat smoke test on the synthetic combat ruleset: two enemy vehicles
//! in range of each other fight until one dies (S-0301, S-0302, S-0306,
//! S-0309, S-0312).

use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;

// spec: S-0309 claim 1 (the per-tick search in guard) and S-0312 claim 15 (a destroyed vehicle leaves play)
#[test]
fn two_vehicles_fight_until_one_dies() {
    let mut r = testkit::combat_ruleset();
    // Both carts carry the gun so that they can hurt each other.
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("Gun".into());
    let mut w = testkit::game_with(&r, 24, 3);
    let a = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        0,
        testkit::mid(24, -2, 0),
        16384,
    );
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        1,
        testkit::mid(24, 2, 0),
        49152,
    );
    let mut over = false;
    for _ in 0..3000 {
        glasnost_sim::tick::step(&mut w);
        if w.obj(a).is_none() || w.obj(b).is_none() {
            over = true;
            break;
        }
    }
    assert!(over, "one of the two carts is destroyed");
    let kills: i32 = w.houses.iter().map(|h| h.combat.kills).sum();
    assert_eq!(kills, 1);
}
