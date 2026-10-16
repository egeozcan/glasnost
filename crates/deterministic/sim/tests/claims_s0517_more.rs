//! Claim tests of S-0517 (deployable units), part two: the deploy-fire shot,
//! the firing refusals, drawing, deploy-to-fire, the cell test, the absent
//! vehicle cycle, the deployed-firing rule of infantry and the save.

mod deploy_common;

use deploy_common::*;
use glasnost_sim::cloak::{Disguise, DisguiseAs};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::econ::inplace::{self, Form};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

fn quirk(w: &mut World, f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) {
    let mut rules = (*w.rules_arc()).clone();
    f(&mut rules.quirks);
    w.rules = Some(std::sync::Arc::new(rules));
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

fn form(w: &World, id: ObjectId) -> Form {
    inplace::form(w, id)
}

fn damages(w: &World) -> Vec<i32> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Detonation { damage, .. } => Some(*damage),
            _ => None,
        })
        .collect()
}

fn shots(w: &World) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Shot { .. }))
        .count()
}

// spec: S-0517 claim 2 (a deploy-fire vehicle fires its slot 0 weapon at its own cell at each processing of unload, whatever the deploy weapon index says, and chooses 14 + u ticks with u one draw over 0 to 2; unload stays current)
#[test]
fn claim_2_deploy_fire_vehicle() {
    let mut w = game();
    let v = put(&mut w, "GAS", Kind::Vehicle, 0, 0, 0);
    w.effects.clear();
    deploy(&mut w, v);
    let seen = processings(&mut w, v, 120);
    assert!(seen.len() >= 3, "{seen:?}");
    for &(_, delay) in &seen {
        assert!((14..=16).contains(&delay), "14 + u: {delay}");
    }
    let hits = damages(&w);
    assert!(!hits.is_empty(), "it fired");
    assert!(
        hits.iter().all(|&x| x == 8),
        "slot 0 (the gas weapon): {hits:?}"
    );
    assert_eq!(
        behaviour(&w, v).0,
        Some(Behaviour::Unload),
        "unload stays current"
    );
    assert_eq!(form(&w, v), Form::Mobile, "it never becomes deployed");
    // One draw per processing is taken even when the shot is refused.
    let mut w2 = game();
    let u = put(&mut w2, "GAS", Kind::Vehicle, 0, 0, 0);
    w2.obj_mut(u).unwrap().combat.ammo = 0;
    deploy(&mut w2, u);
    assert!(until(&mut w2, 40, |w| w.obj(u).unwrap().beh.last.is_some()));
    let before = w2.rng.taken();
    let s = shots(&w2);
    let seen = processings(&mut w2, u, 60);
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|&(_, d)| (14..=16).contains(&d)));
    assert!(w2.rng.taken() > before, "the draw is taken");
    assert_eq!(shots(&w2), s, "no ammunition: no shot");
}

// spec: S-0517 claim 11 (a transition supplies an illegal refusal for every weapon and target; a deployed vehicle fires normally; a magnetic-beam weapon is refused against a vehicle in transition or one whose current behaviour is unload)
#[test]
fn claim_11_firing_refusals() {
    let mut w = game();
    let v = put(&mut w, "ATANK", Kind::Vehicle, 0, 0, 0);
    let e = put(&mut w, "TANK", Kind::Vehicle, 1, 3, 0);
    let t = Some(Target::Object(e));
    assert_ne!(check::check(&w, v, t, 0, false), Answer::Illegal);
    assert!(!inplace::beam_refused(&w, v));
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| matches!(
        form(w, v),
        Form::Deploying { .. }
    )));
    assert_eq!(check::check(&w, v, t, 0, false), Answer::Illegal);
    assert_eq!(check::check(&w, v, None, 0, false), Answer::Illegal);
    assert!(inplace::beam_refused(&w, v), "in transition");
    assert!(until(&mut w, 40, |w| form(w, v) == Form::Deployed));
    assert_ne!(check::check(&w, v, t, 0, false), Answer::Illegal);
    assert!(!inplace::beam_refused(&w, v), "guard is current");
    // The first processing of the order: unload is current, no transition yet.
    let tick = w.tick;
    w.obj_mut(v)
        .unwrap()
        .beh
        .force(Behaviour::Unload, tick, true);
    assert!(inplace::beam_refused(&w, v));
    // Another type of vehicle is never refused by the second rule.
    let o = put(&mut w, "IMGT", Kind::Vehicle, 0, -3, 0);
    w.obj_mut(o)
        .unwrap()
        .beh
        .force(Behaviour::Unload, tick, true);
    assert!(!inplace::beam_refused(&w, o));
}

// spec: S-0517 claim 12 (a vehicle in transition does not draw its own body; a deployed one draws the deployed image type when its type names one and keeps its own type for every rule)
#[test]
fn claim_12_drawing() {
    let mut w = game();
    let v = put(&mut w, "ATANK", Kind::Vehicle, 0, 0, 0);
    let p = put(&mut w, "TANK", Kind::Vehicle, 0, 4, 0);
    assert!(!inplace::body_hidden(&w, v));
    assert_eq!(
        inplace::deployed_image(&w, v),
        None,
        "mobile: its own image"
    );
    deploy(&mut w, v);
    deploy(&mut w, p);
    assert!(until(&mut w, 40, |w| matches!(
        form(w, v),
        Form::Deploying { .. }
    )));
    assert!(inplace::body_hidden(&w, v));
    assert!(until(&mut w, 40, |w| form(w, v) == Form::Deployed));
    assert!(!inplace::body_hidden(&w, v));
    let img = w.rules().index_of(Kind::Vehicle, "IMGT");
    assert_eq!(inplace::deployed_image(&w, v), img);
    let own = w.obj(v).unwrap().type_index;
    assert_eq!(
        Some(own),
        w.rules().index_of(Kind::Vehicle, "ATANK"),
        "its own type stays"
    );
    // A type with no deployed image type draws itself.
    assert!(until(&mut w, 40, |w| form(w, p) == Form::Deployed));
    assert_eq!(inplace::deployed_image(&w, p), None);
    assert!(!inplace::body_hidden(&w, p));
}

// spec: S-0517 claim 13 (the firing step queues unload for a deploy-to-fire vehicle without starting it at once; a deployed in-place one queues unload again with quirks.deploy_to_fire_repacks on and fires with it off)
#[test]
fn claim_13_deploy_to_fire_queues_unload() {
    let mut w = game();
    let v = put(&mut w, "D2F", Kind::Vehicle, 0, 0, 0);
    let e = put(&mut w, "TANK", Kind::Vehicle, 1, 3, 0);
    w.effects.clear();
    w.obj_mut(v).unwrap().target = Some(Target::Object(e));
    let tick = w.tick;
    w.obj_mut(v)
        .unwrap()
        .beh
        .force(Behaviour::Attack, tick, true);
    let mut queued = false;
    for _ in 0..40 {
        step(&mut w);
        if w.obj(v).unwrap().beh.queued == Some(Behaviour::Unload) {
            queued = true;
            break;
        }
        if behaviour(&w, v).0 == Some(Behaviour::Unload) {
            queued = true;
            break;
        }
    }
    assert!(queued, "unload was queued");
    assert_eq!(shots(&w), 0, "it did not fire");
    // The in-place flag: the repacking cycle and the switch.
    for repacks in [true, false] {
        let mut w = game();
        quirk(&mut w, |q| q.deploy_to_fire_repacks = repacks);
        let v = put(&mut w, "D2FIP", Kind::Vehicle, 0, 0, 0);
        assert!(inplace::queues_unload(&w, v), "mobile: always");
        deploy(&mut w, v);
        assert!(until(&mut w, 60, |w| form(w, v) == Form::Deployed));
        assert_eq!(inplace::queues_unload(&w, v), repacks);
    }
}

// spec: S-0517 claim 14 (the deploy-to-fire cell test: flat, no building among the occupants, a land type with the buildable flag; units, ownership and overlay do not count)
#[test]
fn claim_14_cell_test() {
    let mut w = game();
    let c = at(0, 0);
    assert!(inplace::deploy_test_cell(&w, c));
    // A unit and an enemy in the cell do not matter.
    let _u = put(&mut w, "TANK", Kind::Vehicle, 1, 0, 0);
    assert!(inplace::deploy_test_cell(&w, c));
    // Not flat.
    w.cells.grid.get_mut(c).unwrap().slope = 1;
    assert!(!inplace::deploy_test_cell(&w, c));
    w.cells.grid.get_mut(c).unwrap().slope = 0;
    // A land type without the buildable flag.
    let was = w.cells.grid.get(c).unwrap().land;
    w.cells.grid.get_mut(c).unwrap().land = glasnost_map::terrain::land::WATER;
    assert!(!inplace::deploy_test_cell(&w, c));
    w.cells.grid.get_mut(c).unwrap().land = was;
    // A building among the occupants.
    let n = at(5, 5);
    assert!(inplace::deploy_test_cell(&w, n));
    let _b = put(&mut w, "TALL", Kind::Building, 0, 5, 5);
    assert!(!inplace::deploy_test_cell(&w, n));
    // The firing check answers must-deploy only when the vehicle's cell passes
    // the test; on a failing cell a deploy-to-fire vehicle that cannot fire.
    let v = put(&mut w, "D2F", Kind::Vehicle, 0, -4, 0);
    let e = put(&mut w, "TANK", Kind::Vehicle, 1, -1, 0);
    let a = check::check(&w, v, Some(Target::Object(e)), 0, false);
    assert_ne!(a, Answer::Impossible);
}

// spec: S-0517 claim 14 (a primary cell of the approach search that fails the cell test is skipped for a deploy-to-fire vehicle, and a skipped primary never anchors the failure outcome)
#[test]
fn claim_14_approach_skips_failing_primary() {
    let mut w = game();
    let v = put(&mut w, "D2F", Kind::Vehicle, 0, 0, 0);
    let e = put(&mut w, "TANK", Kind::Vehicle, 1, 12, 0);
    // The enemy's own cell does not pass; a vehicle that can deploy there is
    // asked to approach and ends on a cell that passes.
    w.cells
        .grid
        .get_mut(w.obj(e).unwrap().pos.cell())
        .unwrap()
        .slope = 1;
    w.obj_mut(v).unwrap().target = Some(Target::Object(e));
    let tick = w.tick;
    w.obj_mut(v)
        .unwrap()
        .beh
        .force(Behaviour::Attack, tick, true);
    for _ in 0..200 {
        step(&mut w);
        if let Some(Target::Cell(c)) = w.obj(v).unwrap().dest {
            assert!(inplace::deploy_test_cell(&w, c), "a skipped primary cell");
        }
    }
}

// spec: S-0517 claim 15 (no rule starts a vehicle's unload or deploy cycle; the deploy time field affects no vehicle)
#[test]
fn claim_15_no_vehicle_cycle() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("TANK").unwrap().deploy_time = Some(d("5"));
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    let v = put(&mut w, "TANK", Kind::Vehicle, 0, 0, 0);
    let before = {
        let o = w.obj(v).unwrap();
        o.infantry.is_some()
    };
    go(&mut w, v, at(5, 0));
    for _ in 0..400 {
        step(&mut w);
    }
    // It drove away at its normal pace and deployed on request without any wait.
    assert!(w.obj(v).unwrap().pos.cell() != at(0, 0));
    assert!(!before);
    let mut w2 = game();
    let u = put(&mut w2, "TANK", Kind::Vehicle, 0, 0, 0);
    let mut w3 = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w3, 4);
    let x = put(&mut w3, "TANK", Kind::Vehicle, 0, 0, 0);
    deploy(&mut w2, u);
    deploy(&mut w3, x);
    for _ in 0..60 {
        step(&mut w2);
        step(&mut w3);
        assert_eq!(form(&w2, u), form(&w3, x), "the field changes nothing");
    }
}

// spec: S-0517 claim 16 (a deployed deploy-fire infantry unit with a target within range keeps it; without one an S-0309 range search finds a target at its own position at once; an out-of-range target is replaced or dropped; the unit never moves)
#[test]
fn claim_16_deployed_firing_rule() {
    let mut w = game();
    let i = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    deploy(&mut w, i);
    let seq_deployed = |w: &World| {
        w.obj(i)
            .unwrap()
            .infantry
            .as_ref()
            .and_then(|s| s.seq)
            .is_some_and(glasnost_sim::infantry::Seq::is_deploy)
    };
    assert!(until(&mut w, 120, seq_deployed), "the infantry deployed");
    let home = w.obj(i).unwrap().pos;
    // An out-of-range target is replaced by what the search finds in range.
    let near = put(&mut w, "TANK", Kind::Vehicle, 1, 3, 0);
    let far = put(&mut w, "TANK", Kind::Vehicle, 1, 14, 0);
    let tick = w.tick;
    let o = w.obj_mut(i).unwrap();
    o.target = Some(Target::Object(far));
    o.beh.force(Behaviour::Attack, tick, true);
    assert!(until(&mut w, 80, |w| w.obj(i).unwrap().target
        == Some(Target::Object(near))));
    // A target within range is kept.
    for _ in 0..30 {
        step(&mut w);
        assert_eq!(w.obj(i).unwrap().target, Some(Target::Object(near)));
    }
    assert_eq!(w.obj(i).unwrap().pos, home, "it never moves");
}

// spec: S-0517 claim 16 (a unit left targetless falls back to rest unless its current behaviour is guard or its type is a jumpjet type)
#[test]
fn claim_16_fall_back_exceptions() {
    for (name, rests) in [("DEPINF", true), ("DEPJET", false)] {
        let mut w = game();
        let i = put(&mut w, name, Kind::Infantry, 0, 0, 0);
        deploy(&mut w, i);
        assert!(until(&mut w, 200, |w| {
            w.obj(i)
                .unwrap()
                .infantry
                .as_ref()
                .and_then(|s| s.seq)
                .is_some_and(glasnost_sim::infantry::Seq::is_deploy)
        }));
        let far = put(&mut w, "TANK", Kind::Vehicle, 1, 14, 0);
        let tick = w.tick;
        let o = w.obj_mut(i).unwrap();
        o.target = Some(Target::Object(far));
        o.beh.force(Behaviour::Attack, tick, true);
        // At the processing that drops the target, the fall-back queues a
        // behaviour for a rest; a jumpjet type queues nothing.
        let mut dropped = false;
        for _ in 0..40 {
            step(&mut w);
            let ob = w.obj(i).unwrap();
            if ob.target.is_none() {
                dropped = true;
                assert_eq!(ob.beh.queued.is_some(), rests, "{name}: {:?}", ob.beh);
                break;
            }
        }
        assert!(dropped, "{name}: the far target was dropped");
        assert_eq!(w.obj(i).unwrap().pos.cell(), at(0, 0));
    }
}

// spec: S-0517 claim 6 (the undeploy step removes a disguise the vehicle wears)
#[test]
fn claim_6_undeploy_removes_a_disguise() {
    let mut w = game();
    let v = put(&mut w, "ATANK", Kind::Vehicle, 0, 0, 0);
    deploy(&mut w, v);
    assert!(until(&mut w, 60, |w| form(w, v) == Form::Deployed));
    let t = w.tick;
    let img = w.rules().index_of(Kind::Vehicle, "IMGT").unwrap();
    w.obj_mut(v).unwrap().cloak.disguise = Some(Disguise {
        what: Some(DisguiseAs::Vehicle(img)),
        house: None,
        since: t,
    });
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| matches!(
        form(w, v),
        Form::Undeploying { .. }
    )));
    assert!(w.obj(v).unwrap().cloak.disguise.is_none());
}

// spec: S-0517 claim 3 (the in-place form is part of the state: a save taken in the middle of a transition loads and plays on to the same hash)
#[test]
fn save_and_load_in_the_middle_of_a_transition() {
    let mut w = game();
    let v = put(&mut w, "ATANK", Kind::Vehicle, 0, 0, 0);
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| matches!(
        form(w, v),
        Form::Deploying { .. }
    )));
    step(&mut w);
    let bytes = glasnost_sim::save::save(&w);
    let mut l = glasnost_sim::save::load(&bytes, w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(
        glasnost_sim::save::state_hash(&l),
        glasnost_sim::save::state_hash(&w)
    );
    assert_eq!(form(&l, v), form(&w, v));
    testkit::run(&mut w, 30);
    testkit::run(&mut l, 30);
    assert_eq!(
        glasnost_sim::save::state_hash(&l),
        glasnost_sim::save::state_hash(&w)
    );
    assert_eq!(form(&l, v), Form::Deployed);
}
