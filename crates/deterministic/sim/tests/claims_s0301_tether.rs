//! S-0301 claim 9 rank 12 (#176): a vehicle that is tethered with a
//! building as its contact gets *impossible*. Synthetic data: the testkit's
//! combat ruleset (CART fires the invented `Gun`, BUGGY is unarmed) and an
//! invented 3 × 2 vehicle factory, as in `claims_s0417.rs`.

mod econ_common;

use econ_common::*;
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Target};

// spec: S-0301 claim 9 rank 12 (a vehicle that is tethered, glossary, with a building as its contact gets impossible; no shot from the tick the tether begins until it ends)
#[test]
fn a_tethered_released_vehicle_gets_impossible_and_does_not_fire() {
    let mut r = testkit::combat_ruleset();
    let mut b = factory("vehicle");
    b.foundation = Some([3, 2]);
    b.weapons_factory = Some(true);
    b.deploy_time = dec("0.01");
    r.registries.get_mut("buildings").unwrap().push("VF".into());
    r.sim.buildings.insert("VF".into(), b);
    let mut w = game(&r);
    let k = building(&mut w, "VF", 0, 0, 0);
    let foe = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(SIZE, -2, -1),
        0,
    );
    let full = w.obj(foe).unwrap().health;
    let ti = w.rules().index_of(Kind::Vehicle, "CART").unwrap();
    let cart = objects::create(&mut w, Kind::Vehicle, ti, Some(HouseId(0)));
    assert_eq!(release::release(&mut w, k, cart), Outcome::Released);
    assert!(glasnost_sim::econ::tethered(&w, cart));
    assert_eq!(w.obj(cart).unwrap().contact, Some(k));

    let mut answers = Vec::new();
    let mut shot = None;
    let mut ticks = 0;
    for _ in 0..600 {
        if !glasnost_sim::econ::tethered(&w, cart) {
            break;
        }
        ticks += 1;
        let a = check::check(&w, cart, Some(Target::Object(foe)), 0, true);
        if answers.last().map(|&(_, x)| x) != Some(a) {
            answers.push((w.tick, a));
        }
        step(&mut w);
        let still = glasnost_sim::econ::tethered(&w, cart);
        let hurt = w.obj(foe).is_none_or(|o| o.health < full);
        let rearming = w.obj(cart).unwrap().combat.rearm_until > w.tick;
        if still && shot.is_none() && (hurt || rearming) {
            shot = Some(w.tick);
        }
    }
    assert!(ticks > 1, "the tether held for {ticks} ticks");
    assert!(
        answers.iter().all(|&(_, a)| a == Answer::Impossible),
        "firing check answers while tethered to the factory: {answers:?}"
    );
    assert_eq!(shot, None, "a shot was made while tethered");
}

// spec: S-0301 r76 claim 9 rank 12 (Q-3012 answered: within rank 12 the tether comes first, so a tethered vehicle healer whose target fails S-0310 claim 11's test gets impossible) and S-0214 r80 Edge cases (an illegal-only ask passes over the impossible, so the healer's illegal still answers there; the own AI's lasting test passes over the tether, our design)
#[test]
fn a_tethered_healer_gets_impossible_before_its_illegal() {
    let mut r = testkit::combat_ruleset();
    // CART's weapon heals: a negative damage (S-0310 claim 11).
    r.sim.weapons.get_mut("Gun").unwrap().damage = Some(-10);
    let mut b = factory("vehicle");
    b.foundation = Some([3, 2]);
    b.weapons_factory = Some(true);
    b.deploy_time = dec("0.01");
    r.registries.get_mut("buildings").unwrap().push("VF".into());
    r.sim.buildings.insert("VF".into(), b);
    let mut w = game(&r);
    let k = building(&mut w, "VF", 0, 0, 0);
    // A vehicle at full health: the healer's test fails.
    let mate = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        0,
        testkit::mid(SIZE, -2, -1),
        0,
    );
    let ti = w.rules().index_of(Kind::Vehicle, "CART").unwrap();
    let cart = objects::create(&mut w, Kind::Vehicle, ti, Some(HouseId(0)));
    assert_eq!(release::release(&mut w, k, cart), Outcome::Released);
    assert!(glasnost_sim::econ::tethered(&w, cart));
    let tg = Some(Target::Object(mate));
    assert_eq!(check::check(&w, cart, tg, 0, true), Answer::Impossible);
    assert!(check::illegal_at_any_rank(&w, cart, tg, 0));
    assert_eq!(check::check_lasting(&w, cart, tg, 0), Answer::Illegal);
}

// spec: S-0301 claim 9 rank 12 (impossible is not an illegal reason: a caller that asks for illegal reasons only gets none from the tether). The own AI's lasting test (our design, PLAN §4.7) passes over the tether as a reason of the moment: the release tether ends once the vehicle clears the apron.
#[test]
fn the_tether_is_no_illegal_reason() {
    let mut r = testkit::combat_ruleset();
    let mut b = factory("vehicle");
    b.foundation = Some([3, 2]);
    b.weapons_factory = Some(true);
    b.deploy_time = dec("0.01");
    r.registries.get_mut("buildings").unwrap().push("VF".into());
    r.sim.buildings.insert("VF".into(), b);
    let mut w = game(&r);
    let k = building(&mut w, "VF", 0, 0, 0);
    let foe = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(SIZE, -2, -1),
        0,
    );
    let ti = w.rules().index_of(Kind::Vehicle, "CART").unwrap();
    let cart = objects::create(&mut w, Kind::Vehicle, ti, Some(HouseId(0)));
    assert_eq!(release::release(&mut w, k, cart), Outcome::Released);
    assert!(glasnost_sim::econ::tethered(&w, cart));
    let tg = Some(Target::Object(foe));
    assert_eq!(
        check::check(&w, cart, tg, 0, true),
        Answer::Impossible,
        "a full ask"
    );
    assert!(
        !check::illegal_at_any_rank(&w, cart, tg, 0),
        "an illegal-only ask"
    );
    assert_ne!(
        check::check_lasting(&w, cart, tg, 0),
        Answer::Impossible,
        "the lasting test passes over the tether"
    );
}
