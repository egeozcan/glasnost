//! S-0213's deploy rule for the deploy key, with S-0214's deploy check and
//! deploy readiness, on synthetic data: the deploy fixtures of the S-0517
//! tests plus an invented garrisonable building, an occupier and an
//! infantry type with an undeploy delay. Every name and number is invented.

mod deploy_common;

use deploy_common::*;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::econ::inplace::{self, Form};
use glasnost_sim::infantry::Seq;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::structures::garrison;
use glasnost_sim::targeting::{self, KeyCommand};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};

/// The deploy fixtures plus HUT (garrisonable, capacity 3), GARRI (an
/// occupier) and DELAYINF (DEPINF with an undeploy delay of 10).
fn world() -> World {
    let mut r = ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("HUT".to_string());
    r.registries
        .get_mut("infantry")
        .unwrap()
        .extend(["GARRI", "DELAYINF"].map(String::from));
    let s = &mut r.sim;
    s.buildings.insert(
        "HUT".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([1, 1]),
            can_be_occupied: Some(true),
            max_number_occupants: Some(3),
            ..Default::default()
        },
    );
    s.infantry.insert(
        "GARRI".into(),
        UnitType {
            strength: Some(100),
            speed: Some(4),
            locomotion_class: Some(LocomotionClass::Foot),
            locomotor: Some(Locomotor::Walk),
            occupier: Some(true),
            ..Default::default()
        },
    );
    let mut delayed = s.infantry["DEPINF"].clone();
    delayed.undeploy_delay = Some(10);
    s.infantry.insert("DELAYINF".into(), delayed);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn key(w: &World, sel: &[ObjectId]) -> Vec<Order> {
    targeting::key_command(w, A, sel, KeyCommand::Deploy)
}

fn orders_for(sel: &[ObjectId]) -> Vec<Order> {
    sel.iter().map(|&actor| Order::Deploy { actor }).collect()
}

fn until(w: &mut World, max: i32, f: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if f(w) {
            return true;
        }
        step(w);
    }
    f(w)
}

fn set_seq(w: &mut World, id: ObjectId, s: Seq) {
    w.obj_mut(id).unwrap().infantry.as_mut().unwrap().seq = Some(s);
}

/// A vehicle of TANK deployed in place through an ordinary deploy order.
fn deployed_tank(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    let v = put(w, "TANK", Kind::Vehicle, 0, dx, dy);
    deploy(w, v);
    assert!(until(w, 60, |w| inplace::form(w, v) == Form::Deployed));
    v
}

// spec: S-0213 the deploy rule (the first object in selection order a building of the player's holding garrison occupants: only that building gets a deploy order)
#[test]
fn garrisoned_building_first_gets_the_only_order() {
    let mut w = world();
    let hut = put(&mut w, "HUT", Kind::Building, 0, 0, 0);
    let g = put(&mut w, "GARRI", Kind::Infantry, 0, 1, 0);
    assert!(garrison::attempt(&mut w, g, hut));
    assert!(garrison::count(&w, hut) > 0);
    let t = put(&mut w, "TANK", Kind::Vehicle, 0, 4, 0);
    assert_eq!(key(&w, &[hut, t]), orders_for(&[hut]));
    // Not first: the rule for every member applies, and a building never has
    // deploy readiness (S-0214 Terms), so only the vehicle gets an order.
    assert_eq!(key(&w, &[t, hut]), orders_for(&[t]));
    // An empty garrisonable building first gets nothing by the first rule.
    let empty = put(&mut w, "HUT", Kind::Building, 0, -4, 0);
    assert_eq!(key(&w, &[empty, t]), orders_for(&[t]));
}

// spec: S-0213 the deploy rule (a selection holding both deployed in-place deployers and ones that are not: the deployed ones get no order)
#[test]
fn mixed_vehicles_deployed_ones_skipped() {
    let mut w = world();
    let d = deployed_tank(&mut w, 0, 0);
    let u = put(&mut w, "TANK", Kind::Vehicle, 0, 4, 0);
    assert_eq!(key(&w, &[d, u]), orders_for(&[u]));
    assert_eq!(key(&w, &[u, d]), orders_for(&[u]));
    // All deployed: every one gets an order (to undeploy).
    let d2 = deployed_tank(&mut w, -4, 0);
    assert_eq!(key(&w, &[d, d2]), orders_for(&[d, d2]));
    // All undeployed: every one gets an order.
    let u2 = put(&mut w, "TANK", Kind::Vehicle, 0, 0, 4);
    assert_eq!(key(&w, &[u, u2]), orders_for(&[u, u2]));
}

// spec: S-0213 the deploy rule (the deployed ones skipped in a mixed selection, infantry: S-0214 Terms, deployed by the deployed sequences)
#[test]
fn mixed_infantry_deployed_ones_skipped() {
    let mut w = world();
    let d = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    let f = put(&mut w, "DEPINF", Kind::Infantry, 0, 2, 0);
    let u = put(&mut w, "DEPINF", Kind::Infantry, 0, 4, 0);
    set_seq(&mut w, d, Seq::Deployed);
    set_seq(&mut w, f, Seq::DeployedFire);
    assert_eq!(key(&w, &[d, f, u]), orders_for(&[u]));
    assert_eq!(key(&w, &[d, f]), orders_for(&[d, f]));
    // Infantry and vehicles together: a deployed infantry unit is skipped
    // beside an undeployed vehicle.
    let t = put(&mut w, "TANK", Kind::Vehicle, 0, 0, 4);
    assert_eq!(key(&w, &[d, t]), orders_for(&[t]));
}

// spec: S-0213 the deploy rule; Q-1753 interim (a member still deploying or undeploying counts as neither deployed nor undeployed; its own deploy check and deploy readiness decide)
#[test]
fn member_in_transition_counts_as_neither() {
    let mut w = world();
    let d = deployed_tank(&mut w, 0, 0);
    let c = put(&mut w, "ATANK", Kind::Vehicle, 0, 4, 0);
    deploy(&mut w, c);
    assert!(until(&mut w, 40, |w| matches!(
        inplace::form(w, c),
        Form::Deploying { .. }
    )));
    // Deployed beside one in transition: not mixed, both get orders.
    assert_eq!(key(&w, &[d, c]), orders_for(&[d, c]));
    // An undeploying infantry unit beside a deployed one: not mixed.
    let i = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 4);
    let j = put(&mut w, "DEPINF", Kind::Infantry, 0, 2, 4);
    set_seq(&mut w, i, Seq::Deployed);
    set_seq(&mut w, j, Seq::Undeploy);
    assert_eq!(key(&w, &[i, j]), orders_for(&[i, j]));
}

// spec: S-0213 the deploy rule (a berserk member gets no order)
#[test]
fn berserk_member_gets_no_order() {
    let mut w = world();
    let a = put(&mut w, "TANK", Kind::Vehicle, 0, 0, 0);
    let b = put(&mut w, "TANK", Kind::Vehicle, 0, 4, 0);
    w.obj_mut(a).unwrap().combat.berserk = Some(100);
    assert!(glasnost_sim::psychic::berserk(&w, a));
    assert_eq!(key(&w, &[a, b]), orders_for(&[b]));
}

// spec: S-0213 the deploy rule with S-0214 Terms (deploy check: a disabled vehicle fails it; S-0405 claim 20) and deploy readiness (a vehicle needs a deploy kind; an infantry unit with an undeploy delay has none while deployed)
#[test]
fn deploy_check_and_readiness() {
    let mut w = world();
    let ok = put(&mut w, "TANK", Kind::Vehicle, 0, 0, 0);
    let off = put(&mut w, "TANK", Kind::Vehicle, 0, 4, 0);
    w.obj_mut(off).unwrap().econ.building.unit_disabled = true;
    assert!(!targeting::deploy_check(&w, off));
    assert!(targeting::deploy_check(&w, ok));
    assert_eq!(key(&w, &[ok, off]), orders_for(&[ok]));
    // A plain vehicle with no way to deploy has no deploy readiness.
    let plain = put(&mut w, "IMGT", Kind::Vehicle, 0, 0, 4);
    assert!(!targeting::deploy_readiness(&w, plain));
    assert_eq!(key(&w, &[plain]), Vec::new());
    // A deploy-fire vehicle has it (S-0214 Terms: a deploy weapon).
    let gas = put(&mut w, "GAS", Kind::Vehicle, 0, -4, 0);
    assert!(targeting::deploy_readiness(&w, gas));
    // Infantry with an undeploy delay: ready while undeployed, not while
    // deployed or undeploying.
    let i = put(&mut w, "DELAYINF", Kind::Infantry, 0, 2, 2);
    assert!(targeting::deploy_readiness(&w, i));
    set_seq(&mut w, i, Seq::Deployed);
    assert!(!targeting::deploy_readiness(&w, i));
    set_seq(&mut w, i, Seq::Undeploy);
    assert!(!targeting::deploy_readiness(&w, i));
    // Without an undeploy delay (DEPINF's is -1): ready when deployed.
    let n = put(&mut w, "DEPINF", Kind::Infantry, 0, -2, 2);
    set_seq(&mut w, n, Seq::Deployed);
    assert!(targeting::deploy_readiness(&w, n));
    // An infantry type that does not deploy has none.
    let walker = put(&mut w, "WALKER", Kind::Infantry, 0, -2, -2);
    assert!(!targeting::deploy_readiness(&w, walker));
    // Another house's unit gets no order (neither movable nor able to fire for A).
    let theirs = put(&mut w, "TANK", Kind::Vehicle, 1, 6, 6);
    assert_eq!(key(&w, &[theirs]), Vec::new());
}
