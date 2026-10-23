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
/// occupier), DELAYINF (DEPINF with an undeploy delay of 10) and ZEROINF
/// (DEPINF with an undeploy delay of 0).
fn world() -> World {
    world_with(|_| {})
}

fn world_with(f: impl FnOnce(&mut glasnost_rules::schema::Ruleset)) -> World {
    let mut r = ruleset();
    f(&mut r);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("HUT".to_string());
    r.registries
        .get_mut("infantry")
        .unwrap()
        .extend(["GARRI", "DELAYINF", "ZEROINF"].map(String::from));
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
    s.infantry.insert("DELAYINF".into(), delayed.clone());
    delayed.undeploy_delay = Some(0);
    s.infantry.insert("ZEROINF".into(), delayed);
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

// spec: S-0213 *Deployed, for the deploy rule* (Q-1753: a vehicle that is deploying counts as deployed; an infantry unit undeploying counts as not deployed, so beside a deployed one the selection is mixed)
#[test]
fn members_in_transition_count_by_the_side_they_head_to() {
    let mut w = world();
    let d = deployed_tank(&mut w, 0, 0);
    let c = put(&mut w, "ATANK", Kind::Vehicle, 0, 4, 0);
    deploy(&mut w, c);
    assert!(until(&mut w, 40, |w| matches!(
        inplace::form(w, c),
        Form::Deploying { .. }
    )));
    // Deployed beside one deploying: both count as deployed, not mixed.
    assert_eq!(key(&w, &[d, c]), orders_for(&[d, c]));
    // Deploying beside an undeployed one: mixed, the deploying one is skipped.
    let u = put(&mut w, "TANK", Kind::Vehicle, 0, 0, 4);
    assert_eq!(key(&w, &[c, u]), orders_for(&[u]));
    // An undeploying infantry unit beside a deployed one: mixed, only the
    // undeploying one (not deployed) gets the order.
    let i = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 6);
    let j = put(&mut w, "DEPINF", Kind::Infantry, 0, 2, 6);
    set_seq(&mut w, i, Seq::Deployed);
    set_seq(&mut w, j, Seq::Undeploy);
    assert_eq!(key(&w, &[i, j]), orders_for(&[j]));
    // A deploying infantry unit counts as deployed: beside an undeployed one
    // it is skipped.
    let k = put(&mut w, "DEPINF", Kind::Infantry, 0, 4, 6);
    set_seq(&mut w, j, Seq::Deploy);
    assert_eq!(key(&w, &[j, k]), orders_for(&[k]));
    // A member that fails another condition is still sorted: a deployed
    // DELAYINF (no readiness) makes the selection mixed, so the deployed
    // DEPINF beside the undeployed one gets no order either.
    let q = put(&mut w, "DELAYINF", Kind::Infantry, 0, 6, 6);
    set_seq(&mut w, q, Seq::Deployed);
    set_seq(&mut w, i, Seq::Deployed);
    assert!(!targeting::deploy_readiness(&w, q));
    assert_eq!(key(&w, &[q, i, k]), orders_for(&[k]));
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

// spec: S-0213 the deploy rule with S-0214 Terms (deploy check: a disabled vehicle fails it; S-0405 claim 20) and deploy readiness (a vehicle needs a deploy kind; an infantry unit with an undeploy delay has none while deploying, deployed or undeploying)
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
    // spec: S-0214 Terms (Q-1754: deploy itself is a deploy kind, so a unit still deploying lacks it)
    set_seq(&mut w, i, Seq::Deploy);
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

// ---------------------------------------------------------------------------
// Members part-way through a transition (S-0213, *Deployed, for the deploy
// rule*, Q-1753; scenario O6 part B, cases a to e) and infantry deploy
// readiness with the rearm countdown (S-0214 Terms and Edge cases, Q-1754;
// scenario T24).

/// Where a member stands in its deploy cycle, as the tests read it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Undeployed,
    Deploying,
    Deployed,
    Undeploying,
}

fn phase_of(w: &World, id: ObjectId) -> Phase {
    let o = w.obj(id).unwrap();
    if o.kind == Kind::Vehicle {
        return match inplace::form(w, id) {
            Form::Mobile => Phase::Undeployed,
            Form::Deploying { .. } => Phase::Deploying,
            Form::Deployed => Phase::Deployed,
            Form::Undeploying { .. } => Phase::Undeploying,
        };
    }
    match o.infantry.as_ref().and_then(|s| s.seq) {
        Some(Seq::Deploy) => Phase::Deploying,
        Some(Seq::Deployed | Seq::DeployedFire | Seq::DeployedIdle) => Phase::Deployed,
        Some(Seq::Undeploy) => Phase::Undeploying,
        _ => Phase::Undeployed,
    }
}

/// Plays `n` ticks and returns each member's phases, consecutive repeats
/// merged, starting with the phase it is in now.
fn course(w: &mut World, ids: &[ObjectId], n: i32) -> Vec<Vec<Phase>> {
    let mut out: Vec<Vec<Phase>> = ids.iter().map(|&id| vec![phase_of(w, id)]).collect();
    for _ in 0..n {
        step(w);
        for (k, &id) in ids.iter().enumerate() {
            let p = phase_of(w, id);
            if out[k].last() != Some(&p) {
                out[k].push(p);
            }
        }
    }
    out
}

/// Starts one member's transition with a press with it selected alone and
/// plays until it is in `phase`.
fn press_alone_until(w: &mut World, id: ObjectId, phase: Phase) {
    for o in key(w, &[id]) {
        orders::issue(w, A, o);
    }
    assert!(
        until(w, 200, |w| phase_of(w, id) == phase),
        "{:?}",
        phase_of(w, id)
    );
}

/// A member made deployed by a press alone.
fn deployed_member(w: &mut World, id: ObjectId) {
    press_alone_until(w, id, Phase::Deploying);
    assert!(until(w, 200, |w| phase_of(w, id) == Phase::Deployed));
}

/// Makes the measured press with `sel` selected and issues its orders.
fn press(w: &mut World, sel: &[ObjectId]) -> Vec<Order> {
    let o = key(w, sel);
    for x in &o {
        orders::issue(w, A, x.clone());
    }
    o
}

use Phase::{Deployed as Dd, Deploying as Dg, Undeployed as Ud, Undeploying as Ug};
use glasnost_sim::orders;

// spec: S-0213 *Deployed, for the deploy rule* (Q-1753: a vehicle counts as deployed while deploying or deployed and not undeploying; its deploy readiness does not depend on its transition state, S-0214 Terms) and S-0517 claims 3 and 8 (a transition under way carries on to its end and completion gives guard)
#[test]
fn o6b_vehicle_rows() {
    // (a) one deployed, one deploying: not mixed, both get the order; the
    // deployed one undeploys, the other finishes deploying and stays.
    let mut w = world();
    let a = put(&mut w, "ATANK", Kind::Vehicle, 0, 0, 0);
    let b = put(&mut w, "ATANK", Kind::Vehicle, 0, 4, 0);
    deployed_member(&mut w, a);
    press_alone_until(&mut w, b, Dg);
    assert_eq!(press(&mut w, &[a, b]), orders_for(&[a, b]));
    let c = course(&mut w, &[a, b], 120);
    assert_eq!(c[0], vec![Dd, Ug, Ud], "{c:?}");
    assert_eq!(c[1], vec![Dg, Dd], "{c:?}");
    // (b) one undeploying, one undeployed: both get the order; the
    // undeployed one deploys, the other finishes undeploying and stays.
    let mut w = world();
    let a = put(&mut w, "ATANK", Kind::Vehicle, 0, 0, 0);
    let b = put(&mut w, "ATANK", Kind::Vehicle, 0, 4, 0);
    deployed_member(&mut w, a);
    press_alone_until(&mut w, a, Ug);
    assert_eq!(press(&mut w, &[a, b]), orders_for(&[a, b]));
    let c = course(&mut w, &[a, b], 120);
    assert_eq!(c[0], vec![Ug, Ud], "{c:?}");
    assert_eq!(c[1], vec![Ud, Dg, Dd], "{c:?}");
    // (c) one deploying, one undeploying: mixed; the deploying one gets no
    // order; each finishes its own transition.
    let mut w = world();
    let a = put(&mut w, "ATANK", Kind::Vehicle, 0, 0, 0);
    let b = put(&mut w, "ATANK", Kind::Vehicle, 0, 4, 0);
    deployed_member(&mut w, a);
    press_alone_until(&mut w, a, Ug);
    press_alone_until(&mut w, b, Dg);
    assert_eq!(phase_of(&w, a), Ug);
    assert_eq!(press(&mut w, &[a, b]), orders_for(&[a]));
    let c = course(&mut w, &[a, b], 120);
    assert_eq!(c[0], vec![Ug, Ud], "{c:?}");
    assert_eq!(c[1], vec![Dg, Dd], "{c:?}");
    // (d) both deploying: both get the order; each finishes deploying and
    // stays deployed.
    let mut w = world();
    let a = put(&mut w, "ATANK", Kind::Vehicle, 0, 0, 0);
    let b = put(&mut w, "ATANK", Kind::Vehicle, 0, 4, 0);
    press(&mut w, &[a, b]);
    assert!(until(&mut w, 200, |w| phase_of(w, a) == Dg
        && phase_of(w, b) == Dg));
    assert_eq!(press(&mut w, &[a, b]), orders_for(&[a, b]));
    let c = course(&mut w, &[a, b], 120);
    assert_eq!(c, vec![vec![Dg, Dd], vec![Dg, Dd]]);
    // (e) both undeploying: both get the order; each finishes undeploying
    // and stays undeployed.
    press(&mut w, &[a, b]);
    assert!(until(&mut w, 200, |w| phase_of(w, a) == Ug
        && phase_of(w, b) == Ug));
    assert_eq!(press(&mut w, &[a, b]), orders_for(&[a, b]));
    let c = course(&mut w, &[a, b], 120);
    assert_eq!(c, vec![vec![Ug, Ud], vec![Ug, Ud]]);
}

// spec: S-0213 *Deployed, for the deploy rule* (Q-1753: with some other behaviour queued at the press, the order replaces it with unload, and completion discards that too under S-0517 claim 8's quirks.deploy_transition_discards_orders; with the switch off the vehicle goes the other way after completing)
#[test]
fn o6b_vehicle_queued_behaviour_and_the_switch() {
    for discard in [true, false] {
        let mut w = world_with(|r| r.globals.quirks.deploy_transition_discards_orders = discard);
        let a = put(&mut w, "ATANK", Kind::Vehicle, 0, 0, 0);
        press_alone_until(&mut w, a, Dg);
        // Nothing queued: the press changes nothing, whatever the switch.
        press(&mut w, &[a]);
        let c = course(&mut w, &[a], 120);
        assert_eq!(c, vec![vec![Dg, Dd]], "discard {discard}");
        // A move order mid-transition queues move; the press replaces it
        // with unload.
        press_alone_until(&mut w, a, Ug);
        orders::issue(
            &mut w,
            A,
            Order::Behaviour {
                actor: a,
                behaviour: glasnost_sim::rules::Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(glasnost_sim::world::Target::Cell(at(6, 6))),
            },
        );
        step(&mut w);
        assert_eq!(phase_of(&w, a), Ug);
        press(&mut w, &[a]);
        step(&mut w);
        assert_eq!(
            behaviour(&w, a).1,
            Some(glasnost_sim::rules::Behaviour::Unload),
            "discard {discard}"
        );
        // Completion chooses 450 ticks in either setting (S-0517 claim 8).
        let c = course(&mut w, &[a], 600);
        let expected = if discard {
            vec![Ug, Ud]
        } else {
            vec![Ug, Ud, Dg, Dd]
        };
        assert_eq!(c, vec![expected], "discard {discard}");
    }
}

// spec: S-0213 *Deployed, for the deploy rule* (Q-1753: infantry counts as deployed while its sequence is a deploy kind, deploy included; a negative undeploy delay leaves deploy readiness in every sequence, and the queued unload starts the opposite transition once the uninterruptible one ends, S-0107 table T1, S-0215 claims 11 and 30)
#[test]
fn o6b_infantry_negative_delay_rows() {
    // (a) one deployed, one deploying: both get the order; the deployed one
    // undeploys; the other finishes deploying, then undeploys.
    let mut w = world();
    let a = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    let b = put(&mut w, "DEPINF", Kind::Infantry, 0, 2, 0);
    deployed_member(&mut w, a);
    press_alone_until(&mut w, b, Dg);
    assert_eq!(press(&mut w, &[a, b]), orders_for(&[a, b]));
    let c = course(&mut w, &[a, b], 200);
    assert_eq!(c[0][..3], [Dd, Ug, Ud], "{c:?}");
    assert_eq!(c[1][..4], [Dg, Dd, Ug, Ud], "{c:?}");
    // (b) one undeploying, one undeployed: both get the order; the
    // undeployed one deploys; the other finishes undeploying, then deploys.
    let mut w = world();
    let a = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    let b = put(&mut w, "DEPINF", Kind::Infantry, 0, 2, 0);
    deployed_member(&mut w, a);
    press_alone_until(&mut w, a, Ug);
    assert_eq!(press(&mut w, &[a, b]), orders_for(&[a, b]));
    let c = course(&mut w, &[a, b], 200);
    assert_eq!(c[0][..3], [Ug, Ud, Dg], "{c:?}");
    assert_eq!(c[1][..2], [Ud, Dg], "{c:?}");
    // (c) one deploying, one undeploying: mixed; the deploying one gets no
    // order and stays deployed; the other finishes undeploying, then deploys.
    let mut w = world();
    let a = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    let b = put(&mut w, "DEPINF", Kind::Infantry, 0, 2, 0);
    deployed_member(&mut w, a);
    press_alone_until(&mut w, a, Ug);
    press_alone_until(&mut w, b, Dg);
    assert_eq!(phase_of(&w, a), Ug, "a undeploy outlasts b's start");
    assert_eq!(press(&mut w, &[a, b]), orders_for(&[a]));
    let c = course(&mut w, &[a, b], 200);
    assert_eq!(c[0][..3], [Ug, Ud, Dg], "{c:?}");
    assert_eq!(c[1], vec![Dg, Dd], "{c:?}");
    // (d) both deploying: each finishes deploying, then undeploys.
    let mut w = world();
    let a = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    let b = put(&mut w, "DEPINF", Kind::Infantry, 0, 2, 0);
    press(&mut w, &[a, b]);
    assert!(until(&mut w, 200, |w| phase_of(w, a) == Dg
        && phase_of(w, b) == Dg));
    assert_eq!(press(&mut w, &[a, b]), orders_for(&[a, b]));
    let c = course(&mut w, &[a, b], 200);
    for k in 0..2 {
        assert_eq!(c[k][..4], [Dg, Dd, Ug, Ud], "{c:?}");
    }
    // (e) both undeploying: each finishes undeploying, then deploys.
    let mut w = world();
    let a = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    let b = put(&mut w, "DEPINF", Kind::Infantry, 0, 2, 0);
    press(&mut w, &[a, b]);
    assert!(until(&mut w, 200, |w| phase_of(w, a) == Dd
        && phase_of(w, b) == Dd));
    press(&mut w, &[a, b]);
    assert!(until(&mut w, 200, |w| phase_of(w, a) == Ug
        && phase_of(w, b) == Ug));
    assert_eq!(press(&mut w, &[a, b]), orders_for(&[a, b]));
    let c = course(&mut w, &[a, b], 200);
    for k in 0..2 {
        assert_eq!(c[k][..3], [Ug, Ud, Dg], "{c:?}");
    }
}

// spec: S-0214 Terms, deploy readiness (Q-1754: an infantry type with an undeploy delay of 0 or more lacks it while its sequence is a deploy kind, deploy itself included, or undeploy) and S-0213 *Deployed, for the deploy rule* (such a member in transition gets no order; the undeployed one of row b gets one)
#[test]
fn o6b_infantry_delay_rows() {
    let mut w = world();
    let a = put(&mut w, "DELAYINF", Kind::Infantry, 0, 0, 0);
    let b = put(&mut w, "DELAYINF", Kind::Infantry, 0, 2, 0);
    let c = put(&mut w, "DELAYINF", Kind::Infantry, 0, 4, 0);
    press(&mut w, &[a]);
    assert!(until(&mut w, 200, |w| phase_of(w, a) == Dg));
    // Deploying: no readiness, no order, alone or with a deployed or
    // undeployed partner.
    assert!(!targeting::deploy_readiness(&w, a));
    assert_eq!(key(&w, &[a]), Vec::new());
    assert!(until(&mut w, 200, |w| phase_of(w, a) == Dd));
    assert!(!targeting::deploy_readiness(&w, a));
    press(&mut w, &[b]);
    assert!(until(&mut w, 200, |w| phase_of(w, b) == Dg));
    // (a) one deployed, one deploying: neither gets an order.
    assert_eq!(phase_of(&w, a), Dd);
    assert_eq!(key(&w, &[a, b]), Vec::new());
    // (b) one undeploying (by itself, after its delay), one undeployed: only
    // the undeployed one gets an order.
    assert!(until(&mut w, 400, |w| phase_of(w, a) == Ug));
    assert!(!targeting::deploy_readiness(&w, a));
    assert_eq!(key(&w, &[a, c]), orders_for(&[c]));
    // (c) one deploying, one undeploying: neither.
    let d = put(&mut w, "DELAYINF", Kind::Infantry, 0, 6, 0);
    press(&mut w, &[d]);
    assert!(until(&mut w, 200, |w| phase_of(w, d) == Dg));
    if phase_of(&w, a) == Ug {
        assert_eq!(key(&w, &[a, d]), Vec::new());
    }
    assert_eq!(key(&w, &[b, d]), Vec::new());
}

// spec: S-0214 Terms and Edge cases (Q-1754: the timer of deploy readiness is the rearm countdown, which shots start and deploying does not; an undeployed unit gets no order while it runs and one from the first press after it ends; a move or stop order does not end it) and Scenario T24 (c), (e)
#[test]
fn t24_rearm_countdown_refuses_until_it_ends() {
    let mut w = world();
    let a = put(&mut w, "DELAYINF", Kind::Infantry, 0, 0, 0);
    assert!(targeting::deploy_readiness(&w, a));
    // A shot sets the countdown (S-0301 claim 1): here set directly to end
    // 50 ticks from now.
    let end = w.tick + 50;
    w.obj_mut(a).unwrap().combat.rearm_until = end;
    assert!(!targeting::deploy_readiness(&w, a));
    assert_eq!(key(&w, &[a]), Vec::new());
    // (e) a move order and then a stop order change nothing.
    orders::issue(
        &mut w,
        A,
        Order::Behaviour {
            actor: a,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Cell(at(1, 0))),
        },
    );
    for _ in 0..20 {
        step(&mut w);
    }
    orders::issue(&mut w, A, Order::Stop { actor: a });
    step(&mut w);
    assert_eq!(w.obj(a).unwrap().combat.rearm_until, end);
    while w.tick < end {
        assert!(!targeting::deploy_readiness(&w, a), "tick {}", w.tick);
        step(&mut w);
    }
    // The first press after the countdown has ended gives the order.
    assert!(targeting::deploy_readiness(&w, a));
    assert_eq!(key(&w, &[a]), orders_for(&[a]));
    // Deploying starts no countdown (T24 a): through a whole cycle the
    // countdown stays where it ended.
    press(&mut w, &[a]);
    assert!(until(&mut w, 200, |w| phase_of(w, a) == Dd));
    assert_eq!(w.obj(a).unwrap().combat.rearm_until, end);
}

// spec: S-0214 Edge cases (Q-1754: for a type with a negative undeploy delay neither the sequence nor the countdown affects deploy readiness) and Scenario T24 (g)
#[test]
fn t24_negative_delay_ignores_sequence_and_countdown() {
    let mut w = world();
    let a = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    let end = w.tick + 500;
    w.obj_mut(a).unwrap().combat.rearm_until = end;
    for s in [
        Seq::Guard,
        Seq::Deploy,
        Seq::Deployed,
        Seq::DeployedFire,
        Seq::DeployedIdle,
        Seq::Undeploy,
    ] {
        set_seq(&mut w, a, s);
        assert!(targeting::deploy_readiness(&w, a), "{s:?}");
    }
    // With an undeploy delay of 0 the same sequences refuse (the delay is
    // "0 or more").
    let z = put(&mut w, "ZEROINF", Kind::Infantry, 0, 2, 0);
    for s in [
        Seq::Deploy,
        Seq::Deployed,
        Seq::DeployedFire,
        Seq::DeployedIdle,
        Seq::Undeploy,
    ] {
        set_seq(&mut w, z, s);
        assert!(!targeting::deploy_readiness(&w, z), "{s:?}");
    }
    set_seq(&mut w, z, Seq::Guard);
    assert!(targeting::deploy_readiness(&w, z));
}
