//! Claim tests of S-0517 (deployable units) on synthetic rules: invented
//! types and numbers, no game data. The in-place cycle, its steps, delays and
//! interruptions.

mod deploy_common;

use deploy_common::*;
use glasnost_sim::combat::effects::Sound;
use glasnost_sim::econ::deploy::{self, UnloadRule};
use glasnost_sim::econ::inplace::{self, Form};
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

fn quirk(w: &mut World, f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) {
    let mut rules = (*w.rules_arc()).clone();
    f(&mut rules.quirks);
    w.rules = Some(std::sync::Arc::new(rules));
}

fn tank(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    put(w, name, Kind::Vehicle, 0, dx, dy)
}

/// Runs ticks one by one until `f` holds, at most `max`.
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

// spec: S-0517 claim 1 (the in-place cycle comes after every rule of S-0415 claim 1: passenger capacity, deploy fire, harvesting and deploying into a building take the vehicle's unload first; a deploy-fire vehicle with the in-place flag deploys in place)
#[test]
fn claim_1_the_in_place_cycle_is_the_last_rule() {
    let mut r = ruleset();
    let t = r.sim.vehicles.get_mut("TANK").unwrap().clone();
    // A transport, a harvester, a building deployer and a deploy-fire in-place type.
    let mut cargo = t.clone();
    cargo.passengers = Some(3);
    let mut harv = t.clone();
    harv.harvester = Some(true);
    let mut dep = t.clone();
    dep.deploys_into = Some("TALL".into());
    let mut both = t.clone();
    both.deploy_fire = Some(true);
    for (n, u) in [
        ("CARGO", cargo),
        ("HARV", harv),
        ("DEPB", dep),
        ("BOTH", both),
    ] {
        r.registries.get_mut("vehicles").unwrap().push(n.into());
        r.sim.vehicles.insert(n.into(), u);
    }
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    let rule = |w: &mut World, n: &str, x: i32| {
        let v = tank(w, n, x, 0);
        w.obj_mut(v).unwrap().beh.current = Some(Behaviour::Unload);
        let r = deploy::process_unload(w, v);
        (v, r)
    };
    assert!(matches!(
        rule(&mut w, "TANK", -8).1,
        Some(UnloadRule::InPlace(_))
    ));
    assert_eq!(
        rule(&mut w, "CARGO", -6).1,
        None,
        "passengers first (S-0507)"
    );
    assert_eq!(
        rule(&mut w, "HARV", -4).1,
        None,
        "harvesting first (S-0402)"
    );
    let (v, r) = rule(&mut w, "DEPB", -2);
    assert_eq!(
        r,
        Some(UnloadRule::Deploy),
        "deploying into a building first"
    );
    assert_eq!(form(&w, v), Form::Mobile, "it never deploys in place");
    // Deploy fire without the in-place flag: rule 2; with the flag, the in-place cycle.
    let (v2, _) = rule(&mut w, "BOTH", 2);
    assert!(matches!(
        deploy::process_unload(&mut w, v2),
        Some(UnloadRule::InPlace(_))
    ));
}

// spec: S-0517 claim 3 and 4 (a deploy order: at the first processing a vehicle on the ground with no presentation becomes deployed; guard replaces any queued behaviour and is started at once; the processing chooses 450 ticks, no draw; a second deploy order packs it up)
#[test]
fn claim_3_immediate_deploy_and_undeploy() {
    let mut w = game();
    let v = tank(&mut w, "TANK", 0, 0);
    deploy(&mut w, v);
    let before = w.rng.clone();
    // Run to the first processing of unload.
    assert!(until(&mut w, 40, |w| form(w, v) == Form::Deployed));
    let o = w.obj(v).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Guard),
        "guard started at once"
    );
    assert_eq!(
        o.beh.last.map(|l| l.1),
        Some(450),
        "the processing chose 450"
    );
    assert!(w.rng.taken() >= before.taken());
    // A deploy order to a deployed vehicle packs it up.
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| form(w, v) == Form::Mobile));
    assert_eq!(behaviour(&w, v).0, Some(Behaviour::Guard));
}

// spec: S-0517 claims 3, 4 and 7 and F1 (a deploying presentation: the vehicle becomes deploying, processing every tick with delay 1 and no draw, and deployed at the first processing at or after t₀ + (E − 1) d; the presentation starts at the step and plays forward)
#[test]
fn claim_4_deploying_presentation_and_f1() {
    let mut w = game();
    let v = tank(&mut w, "ATANK", 0, 0);
    w.effects.clear();
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| matches!(
        form(w, v),
        Form::Deploying { .. }
    )));
    let Form::Deploying { since } = form(&w, v) else {
        unreachable!()
    };
    assert_eq!(anims(&w), vec![("DEPA".to_string(), false)], "forward");
    let o = w.obj(v).unwrap();
    assert_eq!(o.beh.last.map(|l| l.1), Some(1), "delay 1 in transition");
    // d = 2 and E = 5: the vehicle is deployed at since + 8, not before.
    let mut done = None;
    for _ in 0..30 {
        step(&mut w);
        if form(&w, v) == Form::Deployed {
            done = Some(w.tick - 1);
            break;
        }
    }
    assert_eq!(done, Some(since + 8), "t₀ + (E − 1) d");
    assert_eq!(behaviour(&w, v).0, Some(Behaviour::Guard));
    // While deploying the body is hidden and the deployed image is not yet used.
    assert!(inplace::deployed_image(&w, v).is_some());
}

// spec: S-0517 claim 6 and F2 (an undeploy with a presentation: it plays in reverse and the vehicle becomes mobile at the first processing at or after t₀ + (E − 2) d; without one, in that step; the body is hidden in transition)
#[test]
fn claim_6_undeploy_presentation_and_f2() {
    let mut w = game();
    let v = tank(&mut w, "ATANK", 0, 0);
    deploy(&mut w, v);
    assert!(until(&mut w, 60, |w| form(w, v) == Form::Deployed));
    assert!(!inplace::body_hidden(&w, v));
    w.effects.clear();
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| matches!(
        form(w, v),
        Form::Undeploying { .. }
    )));
    let Form::Undeploying { since } = form(&w, v) else {
        unreachable!()
    };
    assert_eq!(anims(&w), vec![("DEPA".to_string(), true)], "in reverse");
    assert!(inplace::body_hidden(&w, v));
    let mut done = None;
    for _ in 0..30 {
        step(&mut w);
        if form(&w, v) == Form::Mobile {
            done = Some(w.tick - 1);
            break;
        }
    }
    assert_eq!(done, Some(since + 6), "t₀ + (E − 2) d");
    assert!(!inplace::body_hidden(&w, v));
}

// spec: S-0517 claim 7, F1, F2 and Edge cases (E ≤ 1: completion at t₀ + 1; a zero stage delay with E > 1 never completes with quirks.deploy_stall_zero_delay on, at t₀ + 1 with it off)
#[test]
fn claim_7_durations_and_the_zero_delay_quirk() {
    let mut w = game();
    let e = tank(&mut w, "ETANK", -4, 0);
    deploy(&mut w, e);
    assert!(until(&mut w, 40, |w| matches!(
        form(w, e),
        Form::Deploying { .. }
    )));
    let Form::Deploying { since } = form(&w, e) else {
        unreachable!()
    };
    step(&mut w);
    assert_eq!(
        form(&w, e),
        Form::Deployed,
        "E ≤ 1: t₀ + 1 (started at {since})"
    );
    // The zero-delay presentation: stalled with the switch on.
    let z = tank(&mut w, "ZTANK", 4, 0);
    deploy(&mut w, z);
    for _ in 0..200 {
        step(&mut w);
    }
    assert!(matches!(form(&w, z), Form::Deploying { .. }), "stalled");
    assert!(inplace::body_hidden(&w, z));
    // The switch off: completes at the next processing.
    let mut w2 = game();
    quirk(&mut w2, |q| q.deploy_stall_zero_delay = false);
    let z2 = tank(&mut w2, "ZTANK", 4, 0);
    deploy(&mut w2, z2);
    assert!(until(&mut w2, 40, |w| matches!(
        form(w, z2),
        Form::Deploying { .. }
    )));
    step(&mut w2);
    assert_eq!(form(&w2, z2), Form::Deployed);
}

// spec: S-0517 claim 8 and Edge cases (an order given during a transition: the vehicle is not ready to change, so a queued behaviour does not start; completion replaces it by guard with quirks.deploy_transition_discards_orders on, starts it with the switch off; the completion processing chooses 450 ticks in both)
#[test]
fn claim_8_orders_during_a_transition() {
    for discards in [true, false] {
        let mut w = game();
        quirk(&mut w, |q| q.deploy_transition_discards_orders = discards);
        let v = tank(&mut w, "ATANK", 0, 0);
        deploy(&mut w, v);
        assert!(until(&mut w, 40, |w| matches!(
            form(w, v),
            Form::Deploying { .. }
        )));
        // An accepted area-guard behaviour order queues but does not start.
        let t = w.tick;
        w.obj_mut(v)
            .unwrap()
            .beh
            .queue_and_start(Some(Behaviour::AreaGuard), false, t, true);
        for _ in 0..5 {
            step(&mut w);
            assert_eq!(behaviour(&w, v).1, Some(Behaviour::AreaGuard));
            assert_eq!(behaviour(&w, v).0, Some(Behaviour::Unload));
        }
        assert!(until(&mut w, 30, |w| form(w, v) == Form::Deployed));
        let (cur, _) = behaviour(&w, v);
        assert_eq!(
            cur,
            Some(if discards {
                Behaviour::Guard
            } else {
                Behaviour::AreaGuard
            }),
            "discards: {discards}"
        );
        assert_eq!(w.obj(v).unwrap().beh.last.map(|l| l.1), Some(450));
    }
}

// spec: S-0517 claim 9 (a deployed vehicle or one in transition cannot receive a destination: a destination given by an order, an approach step or any other rule is replaced by none)
#[test]
fn claim_9_no_destination_while_deployed() {
    let mut w = game();
    let v = tank(&mut w, "ATANK", 0, 0);
    // Mobile: a destination is accepted.
    go(&mut w, v, at(6, 0));
    assert!(w.obj(v).unwrap().dest.is_some());
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| matches!(
        form(w, v),
        Form::Deploying { .. }
    )));
    go(&mut w, v, at(6, 0));
    assert_eq!(w.obj(v).unwrap().dest, None, "in transition");
    // A rule that writes the field directly: the turn replaces it by none.
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(at(5, 5)));
    step(&mut w);
    assert_eq!(w.obj(v).unwrap().dest, None);
    assert!(until(&mut w, 30, |w| form(w, v) == Form::Deployed));
    go(&mut w, v, at(6, 0));
    assert_eq!(w.obj(v).unwrap().dest, None, "deployed");
    // Undeployed again, a move order works.
    deploy(&mut w, v);
    assert!(until(&mut w, 60, |w| form(w, v) == Form::Mobile));
    go(&mut w, v, at(6, 0));
    assert!(w.obj(v).unwrap().dest.is_some());
}

// spec: S-0517 claim 10 (move processed for a vehicle that is deployed: guard becomes its queued behaviour and the processing chooses 1 tick)
#[test]
fn claim_10_move_falls_back_to_guard() {
    let mut w = game();
    let v = tank(&mut w, "TANK", 0, 0);
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| form(w, v) == Form::Deployed));
    let t = w.tick;
    let o = w.obj_mut(v).unwrap();
    o.beh.force(Behaviour::Move, t, true);
    o.beh.queued = None;
    let seen = processings(&mut w, v, 3);
    assert!(
        seen.iter().any(|&(_, d)| d == 1),
        "a processing of move chose 1 tick: {seen:?}"
    );
    assert_eq!(behaviour(&w, v).0, Some(Behaviour::Guard));
}

// spec: S-0517 claims 5 and 3 (a deploy step that finds a type that is not landing-first above the ground does nothing and the order is dropped: guard is queued and started, 450 ticks; with quirks.airborne_deploy_order_dropped off the order is retained)
#[test]
fn claim_5_airborne_without_landing() {
    for dropped in [true, false] {
        let mut w = game();
        quirk(&mut w, |q| q.airborne_deploy_order_dropped = dropped);
        let v = tank(&mut w, "DEPN", 0, 0);
        jump::start_in_flight(&mut w, v, Phase::Hovering, 500);
        deploy(&mut w, v);
        step(&mut w);
        step(&mut w);
        assert_eq!(form(&w, v), Form::Mobile);
        let cur = behaviour(&w, v).0;
        assert_eq!(cur == Some(Behaviour::Guard), dropped, "dropped: {dropped}");
        if !dropped {
            assert_eq!(cur, Some(Behaviour::Unload));
        }
    }
}

// spec: S-0517 claim 5 and S-0502 claims 13 and 15 (a landing-first vehicle above the ground requests landing for deploy: each processing before touchdown is another deploy step and chooses 1 tick; at touchdown the request ends and the next processing deploys; without the request an idle deploy-to-land jumpjet keeps hovering)
#[test]
fn claim_5_landing_for_deploy() {
    let mut w = game();
    let v = tank(&mut w, "DLAND0", 0, 0);
    // It hovers with an order of its own (it is moving) and no landing request.
    go(&mut w, v, at(0, 0));
    jump::start_in_flight(&mut w, v, Phase::Hovering, 300);
    for _ in 0..30 {
        step(&mut w);
    }
    assert_eq!(
        phase(&w, v),
        Phase::Hovering,
        "no request: it keeps hovering"
    );
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| jump::landing_requested(w, v)));
    assert_eq!(w.obj(v).unwrap().beh.last.map(|l| l.1), Some(1));
    assert_eq!(form(&w, v), Form::Mobile);
    assert!(until(&mut w, 200, |w| phase(w, v) == Phase::Grounded));
    assert!(
        !jump::landing_requested(&w, v),
        "the request ends at touchdown"
    );
    assert!(until(&mut w, 40, |w| form(w, v) == Form::Deployed));
    assert!(height(&w, v) <= 0);
}

// spec: S-0517 claim 6 (a landing-first vehicle that completes an animated undeploy is given a destination from a nearby-cell search centred on its position; an immediate undeploy without a presentation assigns none)
#[test]
fn claim_6_landing_first_take_off_destination() {
    let mut w = game();
    let v = tank(&mut w, "DLANDA", 0, 0);
    deploy(&mut w, v);
    assert!(until(&mut w, 80, |w| form(w, v) == Form::Deployed
        || jump::landing_requested(w, v)));
    assert!(until(&mut w, 120, |w| form(w, v) == Form::Deployed));
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| matches!(
        form(w, v),
        Form::Undeploying { .. }
    )));
    assert_eq!(w.obj(v).unwrap().dest, None);
    assert!(until(&mut w, 40, |w| form(w, v) == Form::Mobile));
    assert!(
        w.obj(v).unwrap().dest.is_some(),
        "a nearby destination: it may take off again"
    );
    let n = tank(&mut w, "DLAND0", 6, 0);
    deploy(&mut w, n);
    assert!(until(&mut w, 40, |w| form(w, n) == Form::Deployed));
    deploy(&mut w, n);
    assert!(until(&mut w, 40, |w| form(w, n) == Form::Mobile));
    assert_eq!(w.obj(n).unwrap().dest, None, "none without a presentation");
}

// spec: S-0517 claim 12 and Edge cases (the deploy sound is requested at each deploy step with quirks.deploy_sound_repeats on and the undeploy sound at each undeploy step; off: once at the first step that starts a transition and once when an undeploy step starts)
#[test]
fn claim_12_sound_requests() {
    for repeats in [true, false] {
        let mut w = game();
        quirk(&mut w, |q| q.deploy_sound_repeats = repeats);
        let v = tank(&mut w, "ATANK", 0, 0);
        w.effects.clear();
        deploy(&mut w, v);
        assert!(until(&mut w, 60, |w| form(w, v) == Form::Deployed));
        let n = sounds(&w).iter().filter(|s| **s == Sound::Deploy).count();
        if repeats {
            assert!(n >= 4, "a request at every step: {n}");
        } else {
            assert_eq!(n, 1, "once");
        }
        w.effects.clear();
        deploy(&mut w, v);
        assert!(until(&mut w, 60, |w| form(w, v) == Form::Mobile));
        let u = sounds(&w).iter().filter(|s| **s == Sound::Undeploy).count();
        if repeats {
            assert!(u >= 3, "{u}");
        } else {
            assert_eq!(u, 1);
        }
    }
}
