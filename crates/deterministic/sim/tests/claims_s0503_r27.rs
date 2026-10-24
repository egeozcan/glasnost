//! Claim tests of S-0503 revision 27 (claim 19, Q-1773: the deploy order of
//! a moving hover) and its two switches, on a synthetic flat map with
//! invented types (every name and number here is made up; no game data).
//!
//! The variants follow scenario H16: HOVA deploys into a building (A), HOVB
//! deploys in place (B), HOVC is HOVB with the landing-first flag (C), HOVD
//! is HOVA with the balloon-hover field (D), and HOVE is HOVB with the
//! balloon-hover field (for `quirks.hover_deployed_keeps_moving`, which only
//! a retained destination tells apart).
//!
//! The height a deploy step reads is the hover height left by the previous
//! turn (claim 19: behaviour processing comes before movement in a turn), so
//! a test that needs a first deploy step at height 0 sets the vehicle's
//! height to its surface right before the tick, in place of H16 part j's
//! climb.

mod common;

use common::{Scene, cell};
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType,
};
use glasnost_sim::econ::inplace::{self, Form};
use glasnost_sim::hover::{self, HoverState};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};
use glasnost_sim::{save, testkit, tick};

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn hover_type() -> UnitType {
    UnitType {
        strength: Some(100),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Hover),
        locomotor: Some(Locomotor::Hover),
        movement_zone: Some(MovementZone::Amphibious),
        ..Default::default()
    }
}

/// The ruleset: the testkit's combat ruleset, buildable clear ground, the
/// hover fields of [General] (invented values), the building PAD (1 × 1)
/// and the five hover variants.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.land_types.insert(
        LandType::Clear,
        LandRow {
            foot: dec("1"),
            track: dec("1"),
            wheel: dec("1"),
            hover: dec("1"),
            float: dec("0"),
            amphibious: dec("1"),
            float_beach: dec("0"),
            buildable: Some(true),
        },
    );
    s.general.hover_height = Some(100);
    s.general.hover_bob = dec("0.02");
    s.general.hover_boost = dec("1.5");
    s.general.hover_acceleration = dec("0.02");
    s.general.hover_brake = dec("0.01");
    s.general.hover_dampen = dec("0.75");
    let mut pad = BuildingType {
        strength: Some(500),
        cost: Some(500),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    pad.econ.deploy_facing = Some(64);
    r.sim.buildings.insert("PAD".into(), pad);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("PAD".into());
    let a = UnitType {
        deploys_into: Some("PAD".into()),
        ..hover_type()
    };
    let b = UnitType {
        is_simple_deployer: Some(true),
        ..hover_type()
    };
    let c = UnitType {
        deploy_to_land: Some(true),
        ..b.clone()
    };
    let d = UnitType {
        balloon_hover: Some(true),
        ..a.clone()
    };
    let e = UnitType {
        balloon_hover: Some(true),
        ..b.clone()
    };
    for (n, u) in [
        ("HOVER", hover_type()),
        ("HOVA", a),
        ("HOVB", b),
        ("HOVC", c),
        ("HOVD", d),
        ("HOVE", e),
    ] {
        r.sim.vehicles.insert(n.into(), u);
        r.registries
            .entry("vehicles".into())
            .or_default()
            .push(n.into());
    }
    r
}

fn world_with(r: Ruleset) -> World {
    let mut s = Scene::new();
    s.ruleset = r;
    s.build()
}

fn world() -> World {
    world_with(rules())
}

fn quirk(w: &mut World, f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) {
    let mut rules = (*w.rules_arc()).clone();
    f(&mut rules.quirks);
    w.rules = Some(std::sync::Arc::new(rules));
}

fn at(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, name, 0, cell(dx, dy), 16384)
}

fn hs(w: &World, id: ObjectId) -> HoverState {
    w.obj(id)
        .unwrap()
        .hover
        .clone()
        .expect("a hovering vehicle")
}

/// Lets a new hover rise to its hover height and the order window open.
fn settle(w: &mut World) {
    common::run(w, 300);
}

/// Issues a move order and runs ticks until the vehicle has a heading point.
fn go(w: &mut World, id: ObjectId, c: CellXY) {
    common::order_move(w, id, c);
    common::run_until(w, 20, |w| hs(w, id).point.is_some());
    assert!(hs(w, id).point.is_some(), "the move is taken");
}

/// Runs ticks until the vehicle has travelled `cells` cells east of `from`
/// and is midway between two cell centres.
fn midway(w: &mut World, id: ObjectId, from: CellXY, cells: i32) {
    let n = common::run_until(w, 2000, |w| {
        let p = w.obj(id).unwrap().pos;
        p.cell().x >= from.x + cells && (p.x % 256 - 128).abs() > 64
    });
    assert!(n < 2000, "it travels");
}

/// A deploy order taking effect now: issued and applied as at the end of
/// the tick just run (orders take effect after the turn pass, S-0203 claim
/// 7), so the state before and after it can be compared.
fn deploy(w: &mut World, id: ObjectId) {
    let owner = w.obj(id).unwrap().owner.unwrap();
    assert!(orders::issue(w, owner, Order::Deploy { actor: id }));
    orders::apply_pending(w);
}

/// Runs until the hover is no longer moving (S-0503 Terms); returns ticks.
fn until_stopped(w: &mut World, id: ObjectId, max: i32) -> i32 {
    common::run_until(w, max, |w| w.obj(id).is_none() || !hover::moving(w, id))
}

/// The movement quantities claim 19 says the order keeps.
fn quantities(h: &HoverState) -> (F64, F64, F64) {
    (h.throttle, h.phi, h.rho)
}

fn same(a: (F64, F64, F64), b: (F64, F64, F64)) -> bool {
    a.0.eq_num(b.0) && a.1.eq_num(b.1) && a.2.eq_num(b.2)
}

// spec: S-0503 claim 19 (table, D ≠ P: D and the rest of the path are dropped; the vehicle goes on to P and stops on its arrival there, plus at most the final step; heading point, throttle, hover speed and ρ kept, no immediate halt; H16 part a) and claim 16 (the stop order's point-equality rule)
#[test]
fn claim_19_midway_drops_d_and_the_path() {
    let mut w = world();
    let from = cell(-8, 0);
    let id = at(&mut w, "HOVER", -8, 0);
    settle(&mut w);
    go(&mut w, id, cell(0, 0));
    midway(&mut w, id, from, 2);
    let before = hs(&w, id);
    let p = before.point.expect("a heading point");
    assert!(!w.obj(id).unwrap().path.steps.is_empty(), "a path remains");
    deploy(&mut w, id);
    let o = w.obj(id).unwrap();
    assert_eq!(o.dest, None, "the order-level destination is cleared");
    assert!(o.path.steps.is_empty(), "the stored path is dropped");
    let after = hs(&w, id);
    assert_eq!(after.retained, None, "D is dropped");
    assert_eq!(after.point, Some(p), "the heading point is kept");
    assert!(
        same(quantities(&before), quantities(&after)),
        "no reset of throttle, hover speed or ρ at the order"
    );
    assert!(hover::moving(&w, id), "still moving towards P");
    let x0 = w.obj(id).unwrap().pos.x;
    tick::step(&mut w);
    assert_ne!(w.obj(id).unwrap().pos.x, x0, "no immediate halt");
    assert!(until_stopped(&mut w, id, 2000) < 2000);
    assert_eq!(w.obj(id).unwrap().pos.cell(), p.cell(), "it stops at P");
    common::run(&mut w, 300);
    assert_eq!(
        w.obj(id).unwrap().pos.cell(),
        p.cell(),
        "the old path is not resumed"
    );
}

/// The ticks from now until the hover next chooses a new heading point.
fn ticks_to_next_point(w: &World, id: ObjectId) -> i32 {
    let mut c = w.clone();
    let p = hs(&c, id).point;
    let n = common::run_until(&mut c, 500, |c| hs(c, id).point != p);
    assert!(n < 500);
    n
}

// spec: S-0503 claim 19 (an order in the tick in which the vehicle arrives at a cell centre and chooses its next point sees that new point: it goes one cell further; an earlier order stops it on its arrival at the point it is reaching; H16 parts b and c)
#[test]
fn claim_19_arrival_tick_and_the_tick_before() {
    let setup = || {
        let mut w = world();
        let id = at(&mut w, "HOVER", -8, 0);
        settle(&mut w);
        go(&mut w, id, cell(0, 0));
        midway(&mut w, id, cell(-8, 0), 2);
        (w, id)
    };
    // (b) The order after the tick of arrival and choice.
    let (mut w, id) = setup();
    let n = ticks_to_next_point(&w, id);
    let old = hs(&w, id).point.unwrap();
    common::run(&mut w, n);
    let new = hs(&w, id).point.unwrap();
    assert_eq!(new.cell().x, old.cell().x + 1);
    deploy(&mut w, id);
    until_stopped(&mut w, id, 2000);
    assert_eq!(
        w.obj(id).unwrap().pos.cell(),
        new.cell(),
        "the new point is the last"
    );
    // (c) The order one tick before that arrival.
    let (mut w, id) = setup();
    let n = ticks_to_next_point(&w, id);
    let old = hs(&w, id).point.unwrap();
    common::run(&mut w, n - 1);
    assert_eq!(hs(&w, id).point, Some(old));
    deploy(&mut w, id);
    until_stopped(&mut w, id, 2000);
    assert_eq!(
        w.obj(id).unwrap().pos.cell(),
        old.cell(),
        "it stops at the point it was reaching"
    );
}

// spec: S-0503 claim 19 (table, D = P on the final leg with equal heights: D is kept and the move completes at the destination as an ordinary arrival; H16 parts d and e) and claim 16 (the retained D)
#[test]
fn claim_19_final_leg_keeps_d() {
    for (start, goal) in [(-4, 0), (-1, 0)] {
        let mut w = world();
        let id = at(&mut w, "HOVER", start, 0);
        settle(&mut w);
        let dest = cell(goal, 0);
        go(&mut w, id, dest);
        common::run_until(&mut w, 2000, |w| {
            hs(w, id).point.is_some_and(|p| p.cell() == dest)
                && (w.obj(id).unwrap().pos.x % 256 - 128).abs() > 64
        });
        let p = hs(&w, id).point.unwrap();
        assert_eq!(p.cell(), dest, "the final leg");
        let before = hs(&w, id);
        deploy(&mut w, id);
        assert_eq!(w.obj(id).unwrap().dest, None);
        assert_eq!(hs(&w, id).retained, Some(p), "D is kept (D = P)");
        assert!(same(quantities(&before), quantities(&hs(&w, id))));
        assert!(until_stopped(&mut w, id, 2000) < 2000);
        assert_eq!(w.obj(id).unwrap().pos.cell(), dest, "the move completes");
        assert_eq!(hs(&w, id).retained, None, "the completion removes D");
    }
}

// spec: S-0503 claim 19 (the suppressed assignment: with the order-level destination already empty and D and P unchanged since the clear that emptied it, D, the path and P are untouched; H16 parts f and h)
#[test]
fn claim_19_repeated_order_on_unchanged_points_changes_nothing() {
    let run = |twice: bool| {
        let mut w = world();
        let id = at(&mut w, "HOVER", -8, 0);
        settle(&mut w);
        go(&mut w, id, cell(0, 0));
        midway(&mut w, id, cell(-8, 0), 2);
        deploy(&mut w, id);
        tick::step(&mut w);
        let o = w.obj(id).unwrap().clone();
        if twice {
            deploy(&mut w, id);
            let n = w.obj(id).unwrap();
            assert_eq!(n.path, o.path, "no change to the path or retry timer");
            assert_eq!(n.dest, None);
            assert_eq!(
                n.hover.as_ref().unwrap().point,
                o.hover.as_ref().unwrap().point
            );
            assert_eq!(
                n.hover.as_ref().unwrap().retained,
                o.hover.as_ref().unwrap().retained
            );
        }
        let mut track = Vec::new();
        for _ in 0..300 {
            tick::step(&mut w);
            track.push(w.obj(id).map(|o| o.pos));
        }
        track
    };
    assert_eq!(run(true), run(false), "(f) shows no difference from (a)");
    // (h) A stop on the final leg, then the deploy order: the move completes.
    let mut w = world();
    let id = at(&mut w, "HOVER", -1, 0);
    settle(&mut w);
    go(&mut w, id, cell(0, 0));
    let owner = w.obj(id).unwrap().owner.unwrap();
    assert!(orders::issue(&mut w, owner, Order::Stop { actor: id }));
    tick::step(&mut w);
    let d = hs(&w, id).retained;
    assert!(d.is_some(), "the stop kept D = P");
    let before = w.obj(id).unwrap().clone();
    deploy(&mut w, id);
    let after = w.obj(id).unwrap();
    assert_eq!(after.path, before.path);
    assert_eq!(hs(&w, id).retained, d);
    assert_eq!(hs(&w, id).point, before.hover.as_ref().unwrap().point);
    until_stopped(&mut w, id, 2000);
    assert_eq!(w.obj(id).unwrap().pos.cell(), cell(0, 0));
}

// spec: S-0503 claim 19 (table, no heading point: D is unequal to the absent point and is cleared; the vehicle is no longer moving from the moment the order takes effect, in its present cell; H16 part g, a synthetic waiting state)
#[test]
fn claim_19_no_heading_point_stops_in_its_cell() {
    let mut w = world();
    let id = at(&mut w, "HOVER", -8, 0);
    settle(&mut w);
    go(&mut w, id, cell(0, 0));
    midway(&mut w, id, cell(-8, 0), 2);
    // A vehicle waiting with D and no heading point (as after a refusal, claim 12).
    w.obj_mut(id).unwrap().hover.as_mut().unwrap().point = None;
    assert!(hover::moving(&w, id));
    let here = w.obj(id).unwrap().pos.cell();
    deploy(&mut w, id);
    assert!(!hover::moving(&w, id), "no longer moving at once");
    assert_eq!(hs(&w, id).retained, None);
    common::run(&mut w, 200);
    assert_eq!(
        w.obj(id).unwrap().pos.cell(),
        here,
        "it does not leave its cell"
    );
}

// spec: S-0503 claim 19 (deploying into a building: no deploy attempt while still moving, S-0415 claim 2's moving rule; the attempt is made once D and P are both absent, at the cell where it stopped; H16 parts a and c for variant A) and S-0215 claim 11 (unload stays queued while the hover is moving now under move)
#[test]
fn claim_19_deploy_into_a_building_waits_for_the_stop() {
    let mut w = world();
    let id = at(&mut w, "HOVA", -8, 0);
    settle(&mut w);
    go(&mut w, id, cell(0, 0));
    midway(&mut w, id, cell(-8, 0), 2);
    let p = hs(&w, id).point.unwrap();
    deploy(&mut w, id);
    let mut last = w.obj(id).unwrap().pos.cell();
    for _ in 0..600 {
        let Some(o) = w.obj(id) else { break };
        last = o.pos.cell();
        let moving = hover::moving(&w, id);
        if moving && hover::moving_now(&w, id) {
            assert_eq!(
                o.beh.queued,
                Some(Behaviour::Unload),
                "unload stays queued while moving now under move"
            );
        }
        assert!(
            !(moving && o.econ.deploy.attempted),
            "no attempt while a point remains"
        );
        tick::step(&mut w);
    }
    assert_eq!(last, p.cell(), "the attempt is made where it stopped");
    let pads = common::objects_of(&w, Kind::Building);
    assert_eq!(pads.len(), 1, "it deployed into the building");
    assert_eq!(w.obj(pads[0]).unwrap().pos.cell(), p.cell());
}

// spec: S-0503 claim 19 (S-0215 claim 11's targetless-attack exception makes the hover ready while it still moves: unload becomes current at the next first start, and still no attempt is made until both points are absent; H16 variant A's attack repeat)
#[test]
fn claim_19_targetless_attack_makes_unload_current_while_moving() {
    let mut w = world();
    let id = at(&mut w, "HOVA", -8, 0);
    settle(&mut w);
    go(&mut w, id, cell(0, 0));
    midway(&mut w, id, cell(-8, 0), 2);
    deploy(&mut w, id);
    // The order cleared the target of a hover sent to attack: attack is current without one.
    {
        let o = w.obj_mut(id).unwrap();
        o.beh.current = Some(Behaviour::Attack);
        o.target = None;
    }
    tick::step(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Unload),
        "current while moving"
    );
    assert!(hover::moving(&w, id));
    assert!(!o.econ.deploy.attempted, "no attempt while it moves");
}

// spec: S-0215 claim 11 (the vehicle's current locomotor's moving-now report) and S-0503 claim 19 and Terms (moving now: moving and a throttle that is not 0; a tick with throttle 0 and the heading point present leaves the hover ready; H16 part j run 3)
#[test]
fn claim_19_readiness_asks_moving_now() {
    let mut w = world();
    let id = at(&mut w, "HOVB", -8, 0);
    settle(&mut w);
    go(&mut w, id, cell(0, 0));
    midway(&mut w, id, cell(-8, 0), 2);
    {
        let o = w.obj_mut(id).unwrap();
        o.beh.current = Some(Behaviour::Move);
        o.beh.queued = Some(Behaviour::Unload);
    }
    assert!(hover::moving_now(&w, id));
    assert!(
        !glasnost_sim::process::ready(&w, id),
        "held back while moving now"
    );
    w.obj_mut(id).unwrap().hover.as_mut().unwrap().throttle = F64::ZERO;
    assert!(hover::moving(&w, id), "still moving: it holds its points");
    assert!(glasnost_sim::process::ready(&w, id), "throttle 0: ready");
}

/// A HOVB-like hover at rest given a deploy order, run tick by tick with its
/// height set to its surface before each tick when `ground`.
fn in_place(w: &mut World, id: ObjectId, ground: bool, ticks: i32) {
    deploy(w, id);
    for _ in 0..ticks {
        if ground {
            to_ground(w, id);
        }
        tick::step(w);
    }
}

fn to_ground(w: &mut World, id: ObjectId) {
    let p = w.obj(id).unwrap().pos;
    let s = w.surface_at(p.x, p.y);
    let o = w.obj_mut(id).unwrap();
    o.pos.z = s;
    o.hover.as_mut().unwrap().v = F64::ZERO;
}

// spec: S-0503 claim 19 (the in-place outcomes by the height the first deploy step reads: at 0 it deploys as S-0517 claim 4 gives; above 0, a type that is not landing-first does nothing and guard replaces the order, S-0517 claims 3 and 5, quirks.airborne_deploy_order_dropped; with that switch off the order is kept until a step reads 0; H16 part j, variant B)
#[test]
fn claim_19_in_place_by_hover_height() {
    // Above 0: guard replaces the order; no deploy.
    let mut w = world();
    let id = at(&mut w, "HOVB", 0, 0);
    settle(&mut w);
    assert!(
        hover::hover_height(&w, id) > 0,
        "a powered hover at rest hovers"
    );
    in_place(&mut w, id, false, 40);
    assert_eq!(inplace::form(&w, id), Form::Mobile);
    assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Guard));
    // A later height of 0 does not deploy it: the order was dropped.
    for _ in 0..40 {
        to_ground(&mut w, id);
        tick::step(&mut w);
    }
    assert_eq!(inplace::form(&w, id), Form::Mobile);
    // Height 0 at the first step: it deploys in that step.
    let mut w = world();
    let id = at(&mut w, "HOVB", 0, 0);
    settle(&mut w);
    in_place(&mut w, id, true, 40);
    assert_eq!(inplace::form(&w, id), Form::Deployed);
    // quirks.airborne_deploy_order_dropped off: kept until a step reads 0.
    let mut w = world();
    quirk(&mut w, |q| q.airborne_deploy_order_dropped = false);
    let id = at(&mut w, "HOVB", 0, 0);
    settle(&mut w);
    in_place(&mut w, id, false, 40);
    assert_eq!(inplace::form(&w, id), Form::Mobile);
    assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Unload));
    to_ground(&mut w, id);
    tick::step(&mut w);
    assert_eq!(inplace::form(&w, id), Form::Deployed);
}

// spec: S-0503 claim 19 and Edge cases (quirks.hover_landing_request_stands: a landing-first in-place hover whose first deploy step finds it above the ground makes the landing request; while it stands no deploy step deploys it, even at height 0, and unload stays current with a step each tick; off: once no longer moving, the request counts as met and the next step deploys it whatever its height; H16 part j, variant C)
#[test]
fn quirk_hover_landing_request_stands() {
    let run = |stands: bool| {
        let mut w = world();
        quirk(&mut w, |q| q.hover_landing_request_stands = stands);
        let id = at(&mut w, "HOVC", 0, 0);
        settle(&mut w);
        assert!(hover::hover_height(&w, id) > 0);
        deploy(&mut w, id);
        common::run_until(&mut w, 40, |w| hover::landing_requested(w, id));
        assert!(hover::landing_requested(&w, id), "the landing request");
        let mut steps = 0;
        for _ in 0..40 {
            let before = w.obj(id).unwrap().beh.last;
            to_ground(&mut w, id);
            tick::step(&mut w);
            if w.obj(id).unwrap().beh.last != before {
                steps += 1;
            }
        }
        (w, id, steps)
    };
    let (w, id, steps) = run(true);
    assert_eq!(inplace::form(&w, id), Form::Mobile, "never deploys");
    assert_eq!(w.obj(id).unwrap().beh.current, Some(Behaviour::Unload));
    assert_eq!(steps, 40, "a deploy step each tick");
    assert_eq!(w.obj(id).unwrap().beh.last.map(|l| l.1), Some(1));
    assert!(hover::landing_requested(&w, id), "the request stands");
    // The request is part of the saved state.
    let bytes = save::save(&w);
    let l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    assert!(hover::landing_requested(&l, id));
    // Off: the next step deploys it, whatever its height.
    let mut w = world();
    quirk(&mut w, |q| q.hover_landing_request_stands = false);
    let id = at(&mut w, "HOVC", 0, 0);
    settle(&mut w);
    deploy(&mut w, id);
    common::run_until(&mut w, 40, |w| hover::landing_requested(w, id));
    assert!(hover::landing_requested(&w, id));
    common::run_until(&mut w, 40, |w| inplace::form(w, id) != Form::Mobile);
    assert!(hover::hover_height(&w, id) > 0, "still above the ground");
    assert_eq!(inplace::form(&w, id), Form::Deployed);
    assert!(!hover::landing_requested(&w, id), "the request is met");
}

/// A balloon-hover in-place hover (HOVE) moving east to a far cell, with an
/// object ahead as the retention's target (S-0502 claim 28's direction row),
/// given the deploy order midway; its height is set to its surface before
/// each tick so that its first deploy step reads 0 while it still moves.
fn deployed_while_moving(keeps: bool) -> (World, ObjectId, CellXY, CellXY) {
    let mut w = world();
    quirk(&mut w, |q| q.hover_deployed_keeps_moving = keeps);
    let id = at(&mut w, "HOVE", -8, 0);
    let ahead = at(&mut w, "HOVER", 6, 2);
    settle(&mut w);
    let dest = cell(2, 0);
    go(&mut w, id, dest);
    midway(&mut w, id, cell(-8, 0), 2);
    w.obj_mut(id).unwrap().target = Some(Target::Object(ahead));
    let path = w.obj(id).unwrap().path.clone();
    deploy(&mut w, id);
    let o = w.obj(id).unwrap();
    assert_eq!(
        o.dest,
        Some(Target::Cell(dest)),
        "the retention keeps the destination"
    );
    assert_eq!(o.path, path, "and the stored path");
    assert_eq!(o.target, None, "the target is cleared");
    let p = hs(&w, id).point.unwrap();
    for _ in 0..200 {
        if inplace::form(&w, id) != Form::Mobile {
            break;
        }
        to_ground(&mut w, id);
        tick::step(&mut w);
    }
    assert_eq!(inplace::form(&w, id), Form::Deployed, "deployed in place");
    assert!(hover::moving(&w, id), "while it still moves");
    until_stopped(&mut w, id, 3000);
    (w, id, dest, p.cell())
}

// spec: S-0503 claim 19 and Edge cases (quirks.hover_deployed_keeps_moving: a hover that becomes deployed in place while it still holds movement finishes that movement deployed and then stands, an exception to S-0517 claim 9; off: becoming deployed drops D and the stored path as the unequal row, and it finishes only the leg to its heading point; H16 part j run 2) and S-0213 claim 10 with S-0502 claim 28 (the balloon-hover retention, extended to the hovering locomotor by claim 19)
#[test]
fn quirk_hover_deployed_keeps_moving() {
    let (w, id, dest, _) = deployed_while_moving(true);
    assert_eq!(
        w.obj(id).unwrap().pos.cell(),
        dest,
        "it finishes the move deployed"
    );
    assert_eq!(inplace::form(&w, id), Form::Deployed);
    assert_eq!(
        w.obj(id).unwrap().dest,
        None,
        "no destination while deployed"
    );
    let (w, id, dest, _) = deployed_while_moving(false);
    let here = w.obj(id).unwrap().pos.cell();
    assert!(here.x < dest.x, "it stops short of the old destination");
    assert_eq!(inplace::form(&w, id), Form::Deployed);
    assert!(w.obj(id).unwrap().path.steps.is_empty());
}

// spec: S-0503 claim 19 (balloon-hover retention, variant D: the order-level destination, D and path are unchanged at the order; the vehicle travels on until that move ends, and its deploy attempt is made only there; a plain move order behaves as part a; H16 part i)
#[test]
fn claim_19_balloon_hover_retention_into_a_building() {
    let mut w = world();
    let id = at(&mut w, "HOVD", -8, 0);
    let ahead = at(&mut w, "HOVER", 6, 2);
    settle(&mut w);
    let dest = cell(0, 0);
    go(&mut w, id, dest);
    midway(&mut w, id, cell(-8, 0), 2);
    w.obj_mut(id).unwrap().target = Some(Target::Object(ahead));
    let path = w.obj(id).unwrap().path.clone();
    deploy(&mut w, id);
    let o = w.obj(id).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(dest)));
    assert_eq!(o.path, path);
    let mut last = o.pos.cell();
    for _ in 0..800 {
        let Some(o) = w.obj(id) else { break };
        last = o.pos.cell();
        assert!(
            !(hover::moving(&w, id) && o.econ.deploy.attempted),
            "no attempt before the move ends"
        );
        tick::step(&mut w);
    }
    assert_eq!(last, dest, "the move ends at the destination");
    let pads = common::objects_of(&w, Kind::Building);
    assert_eq!(pads.len(), 1);
    assert_eq!(w.obj(pads[0]).unwrap().pos.cell(), dest);
    // A plain move order with no target: as part a.
    let mut w = world();
    let id = at(&mut w, "HOVD", -8, 0);
    settle(&mut w);
    go(&mut w, id, dest);
    midway(&mut w, id, cell(-8, 0), 2);
    deploy(&mut w, id);
    assert_eq!(w.obj(id).unwrap().dest, None);
    assert!(w.obj(id).unwrap().path.steps.is_empty());
}
