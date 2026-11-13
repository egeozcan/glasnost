//! Claim tests for chrono weapons (S-0514): delivery, locks, progress,
//! joins, releases and completion timing, on synthetic data. The numbers
//! are the spec's own scenarios (E1 to E12), invented there too.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use glasnost_sim::rules::Kind;
use glasnost_sim::temporal;
use glasnost_sim::testkit;
use temporal_common::*;

fn two() -> (
    glasnost_sim::world::World,
    glasnost_sim::world::ObjectId,
    glasnost_sim::world::ObjectId,
) {
    let mut w = world(|_| {});
    let victim = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    (w, a, victim)
}

// spec: S-0514 claims 4, 6 to 8 and F1 to F3 (E1: a lock at tick n sets R = 10 x 100 = 1000; one raw damage 30 per update; completion after ceil(1000 / 30) = 34 updates, removal in tick n + 34)
#[test]
fn an_erasing_lock_removes_the_target_after_the_stated_updates() {
    let (mut w, a, v) = two();
    let n = erase(&mut w, a, v);
    assert!(erased(&w, v));
    assert_eq!(amount(&w, v), Some(1000));
    run_to(&mut w, n + 34);
    assert!(w.obj(v).is_some(), "still there after tick n + 33");
    assert_eq!(amount(&w, v), Some(1000 - 33 * 30));
    testkit::run(&mut w, 1);
    assert!(w.obj(v).is_none(), "removed in tick n + 34");
    assert!(w.temporal.locks.is_empty() && w.temporal.beams.is_empty());
}

// spec: S-0514 claim 7 (E1: health does not change while the target is erased; the damage is raw and bypasses versus values, which are 0.1 here)
#[test]
fn erasure_does_not_damage_the_target() {
    let (mut w, a, v) = two();
    let b = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(1, 0));
    erase(&mut w, a, v);
    run_to(&mut w, 20);
    assert_eq!(health(&w, v), 100);
    assert_eq!(health(&w, b), 100, "an adjacent bystander is untouched");
}

// spec: S-0514 claim 6 (E2: the starting amount reads the type's strength, not the health: a target below half health takes as long)
#[test]
fn the_starting_amount_ignores_the_current_health() {
    let (mut w, a, v) = two();
    w.obj_mut(v).unwrap().health = 30;
    erase(&mut w, a, v);
    assert_eq!(amount(&w, v), Some(1000));
}

// spec: S-0514 claim 4 and claim 7 (E3, E1 b: a second attacker joins without resetting the amount; the step becomes the sum of the raw damages 30 + 20)
#[test]
fn a_joining_attacker_adds_its_raw_damage() {
    let (mut w, a, v) = two();
    let b = attacker(&mut w, "ERASER2", 0, at(-12, -11));
    let n = erase(&mut w, a, v);
    run_to(&mut w, n + 6);
    let before = amount(&w, v).unwrap();
    assert_eq!(before, 1000 - 5 * 30);
    erase(&mut w, b, v);
    assert_eq!(amount(&w, v), Some(before - 30), "no reset on joining");
    let l = &w.temporal.locks[&v];
    assert_eq!((l.leader, l.joiners.clone()), (a, vec![b]));
    let at_join = amount(&w, v).unwrap();
    testkit::run(&mut w, 2);
    assert_eq!(amount(&w, v), Some(at_join - 2 * 50));
}

// spec: S-0514 claim 9 (E3: when the leader lets go the most recent remaining joiner leads with the same amount; the target stays erased)
#[test]
fn the_newest_joiner_takes_over_the_lead() {
    let (mut w, a, v) = two();
    let b = attacker(&mut w, "ERASER2", 0, at(-12, -11));
    let c = attacker(&mut w, "ERASER2", 0, at(-12, -10));
    erase(&mut w, a, v);
    erase(&mut w, b, v);
    erase(&mut w, c, v);
    let r = amount(&w, v);
    temporal::let_go(&mut w, a);
    assert_eq!(w.temporal.locks[&v].leader, c);
    assert_eq!(amount(&w, v), r);
    assert!(erased(&w, v));
    // A departing joiner simply stops contributing.
    temporal::let_go(&mut w, b);
    assert_eq!(w.temporal.locks[&v].joiners, vec![]);
}

// spec: S-0514 claim 10 (E3: when the last attacker lets go the target is free and loses all progress; a later lock starts from the full amount)
#[test]
fn the_last_release_forgets_the_progress() {
    let (mut w, a, v) = two();
    erase(&mut w, a, v);
    run_to(&mut w, 15);
    assert!(amount(&w, v).unwrap() < 1000);
    temporal::let_go(&mut w, a);
    assert!(!erased(&w, v));
    testkit::run(&mut w, 45);
    erase(&mut w, a, v);
    assert_eq!(amount(&w, v), Some(1000));
}

// spec: S-0514 claim 7 (the contributors are the leader and the 51 most recent joiners: E12, the oldest joiner of 53 attackers does not contribute)
#[test]
fn only_the_leader_and_the_newest_51_joiners_contribute() {
    let mut w = world(|r| r.sim.vehicles.get_mut("VICT").unwrap().strength = Some(1_000_000));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    // 53 attackers: the leader and 52 joiners; joiner 1 is the oldest and is left out.
    let ids: Vec<_> = (0..53)
        .map(|i| attacker(&mut w, "ERASER", 0, at(-15 + i % 10, -15 + i / 10)))
        .collect();
    for &i in &ids {
        erase(&mut w, i, v);
    }
    let before = amount(&w, v).unwrap();
    testkit::run(&mut w, 1);
    let step = before - amount(&w, v).unwrap();
    assert_eq!(step, 52 * 30, "the leader and 51 joiners, 30 each");
}

// spec: S-0514 claims 1 and 14 (E4: a shot at a cell only lets the firer's old beam go; E10 part: another erasing hit, even at the same target, releases first and starts from a fresh amount)
#[test]
fn a_cell_shot_and_a_repeated_hit_release_the_old_beam() {
    let (mut w, a, v) = two();
    erase(&mut w, a, v);
    run_to(&mut w, 15);
    erase_cell(&mut w, a, at(5, 5));
    assert!(!erased(&w, v), "force-firing at a cell lets go");
    erase(&mut w, a, v);
    assert_eq!(amount(&w, v), Some(1000));
    run_to(&mut w, 25);
    let n = erase(&mut w, a, v);
    assert_eq!(
        amount(&w, v),
        Some(1000),
        "a repeat at tick {n}: a fresh amount"
    );
}

// spec: S-0514 claim 1 (E4: an object that is not in the ground render layer is not affected: no lock)
#[test]
fn a_target_in_the_air_is_not_locked() {
    let mut w = world(|r| {
        let plane = glasnost_rules::sim::UnitType {
            strength: Some(100),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(glasnost_rules::sim::Locomotor::Fly),
            ..Default::default()
        };
        r.sim.aircraft.insert("PLANE".into(), plane);
        r.registries
            .entry("aircraft".into())
            .or_default()
            .push("PLANE".into());
    });
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    let landed = spawn(&mut w, Kind::Aircraft, "PLANE", 1, at(0, 0));
    erase(&mut w, a, landed);
    assert!(
        erased(&w, landed),
        "a landed aircraft is in the ground layer"
    );
    temporal::let_go(&mut w, a);
    w.obj_mut(landed).unwrap().pos.z += 600;
    erase(&mut w, a, landed);
    assert!(
        !erased(&w, landed),
        "above the ground it is in another layer"
    );
}

// spec: S-0514 claim 3 (E4: a target type that is not erasable is not locked)
#[test]
fn a_type_that_is_not_warpable_is_refused() {
    let mut w = world(|r| r.sim.vehicles.get_mut("VICT").unwrap().warpable = Some(false));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    erase(&mut w, a, v);
    assert!(!erased(&w, v));
}

// spec: S-0514 claim 3 (E4: a target protected by an iron curtain or a force shield is refused; the hit side effects of claim 2 still apply)
#[test]
fn a_protected_target_is_refused() {
    use glasnost_sim::superweapons::{ProtectKind, Protection};
    for kind in [ProtectKind::Curtain, ProtectKind::ForceShield] {
        let (mut w, a, v) = two();
        w.obj_mut(v).unwrap().sw.protection = Some(Protection {
            start: 0,
            duration: 10_000,
            kind,
            tint: Default::default(),
        });
        erase(&mut w, a, v);
        assert!(!erased(&w, v), "{kind:?}");
    }
}

// spec: S-0514 claim 3 (E4: a vehicle still standing in the cell of the vehicle factory it is in contact with is refused; once it has left, it is locked)
#[test]
fn a_vehicle_in_its_factory_is_refused() {
    let mut w = world(|r| {
        let mut f = r.sim.buildings["TOWER"].clone();
        f.weapons_factory = Some(true);
        r.registries
            .entry("buildings".into())
            .or_default()
            .push("WFAC".into());
        r.sim.buildings.insert("WFAC".into(), f);
    });
    let factory = spawn(&mut w, Kind::Building, "WFAC", 1, at(0, 0));
    assert!(glasnost_sim::econ::release::vehicle_factory(&w, factory));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    w.obj_mut(v).unwrap().contact = Some(factory);
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    erase(&mut w, a, v);
    assert!(!erased(&w, v), "inside the factory's cell");
    w.obj_mut(v).unwrap().pos = glasnost_sim::world::Pos::centre(at(3, 3), 0);
    erase(&mut w, a, v);
    assert!(erased(&w, v), "outside it");
}

// spec: S-0514 claim 3 (the refusal names the vehicle factory the vehicle is in contact with; a vehicle in contact with another building, standing in its cell, has no refusal and is locked)
#[test]
fn a_vehicle_in_contact_with_a_building_that_is_no_factory_is_locked() {
    let mut w = world(|_| {});
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 0));
    assert!(!glasnost_sim::econ::release::vehicle_factory(&w, tower));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    w.obj_mut(v).unwrap().contact = Some(tower);
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    erase(&mut w, a, v);
    assert!(
        erased(&w, v),
        "a building that is no vehicle factory refuses nothing"
    );
}

// spec: S-0514 claim 3 (E4: a beam already attached to the firer refuses its new lock; a counter-lock releases the firer's own beam)
#[test]
fn a_locked_firer_cannot_lock_and_loses_its_beam() {
    let mut w = world(|_| {});
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    let b = attacker(&mut w, "ERASER2", 1, at(12, 12));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    erase(&mut w, a, v);
    assert!(temporal::holds(&w, a, v));
    // b locks a: a's own beam lets go.
    erase(&mut w, b, a);
    assert!(erased(&w, a));
    assert!(!erased(&w, v), "a's beam let go");
    // a, locked, cannot lock again.
    erase(&mut w, a, v);
    assert!(!erased(&w, v));
}

// spec: S-0514 claim 4 (a lock deselects the target, which stays unselectable) and claim 17 (E11 part a: a mobile target being erased drops its target and destination and skips movement)
#[test]
fn an_erased_vehicle_stops() {
    let (mut w, a, v) = two();
    run_to(&mut w, 10);
    glasnost_sim::orders::issue(
        &mut w,
        glasnost_sim::world::HouseId(1),
        glasnost_sim::orders::Order::Behaviour {
            actor: v,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Cell(at(8, 8))),
        },
    );
    testkit::run(&mut w, 15);
    let moved = w.obj(v).unwrap().pos;
    assert_ne!(moved, glasnost_sim::world::Pos::centre(at(0, 0), 0));
    erase(&mut w, a, v);
    let p = w.obj(v).unwrap().pos;
    testkit::run(&mut w, 10);
    assert_eq!(w.obj(v).unwrap().pos, p);
    assert!(w.obj(v).unwrap().dest.is_none());
    assert!(!glasnost_sim::targeting::selectable(&w, v));
}

// spec: S-0514 claim 19 (E10 part f: holding the same target the firing check answers rearming, so a stationary attacker does not fire repeated erasing shots)
#[test]
fn the_firing_check_answers_rearming_while_holding() {
    use glasnost_sim::combat::check::{Answer, check};
    let mut w = world(|r| r.sim.vehicles.get_mut("VICT").unwrap().strength = Some(100_000));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    testkit::run(&mut w, 60);
    let t = glasnost_sim::world::Target::Object(v);
    assert_eq!(check(&w, a, Some(t), 0, false), Answer::CanFire);
    erase(&mut w, a, v);
    testkit::run(&mut w, 60);
    assert_eq!(check(&w, a, Some(t), 0, false), Answer::Rearming);
}

// spec: S-0514 claim 14 (E10 part a: an attached attacker lets go on arriving in a new cell)
#[test]
fn a_new_cell_lets_the_beam_go() {
    let mut w = world(|r| r.sim.vehicles.get_mut("VICT").unwrap().strength = Some(100_000));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    erase(&mut w, a, v);
    run_to(&mut w, 10);
    glasnost_sim::orders::issue(
        &mut w,
        glasnost_sim::world::HouseId(0),
        glasnost_sim::orders::Order::Behaviour {
            actor: a,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Cell(at(-9, -12))),
        },
    );
    assert!(erased(&w, v));
    for _ in 0..60 {
        testkit::run(&mut w, 1);
    }
    assert!(
        w.obj(v).is_some() && !erased(&w, v),
        "released by the move, not by completion"
    );
}

// spec: S-0514 claim 14 (E10 part b: an attached attacker lets go on entering idle behaviour, falling back to rest)
#[test]
fn falling_back_to_rest_lets_the_beam_go() {
    let (mut w, a, v) = two();
    erase(&mut w, a, v);
    glasnost_sim::process::fall_back(&mut w, a, false);
    assert!(!erased(&w, v));
}

// spec: S-0514 claim 14 and claim 16 (E10 part h: clearing the attacker's ordinary target alone does not detach its beam)
#[test]
fn clearing_the_attackers_target_keeps_the_beam() {
    let (mut w, a, v) = two();
    erase(&mut w, a, v);
    glasnost_sim::spawn::clear_own_target(w.obj_mut(a).unwrap());
    assert!(erased(&w, v));
}

// spec: S-0514 claim 14 (E10: removing the target invalidates all attached beams and its attackers go idle; removing an attacker lets its beam go)
#[test]
fn removal_detaches_beams() {
    let (mut w, a, v) = two();
    let b = attacker(&mut w, "ERASER2", 0, at(-12, -11));
    erase(&mut w, a, v);
    erase(&mut w, b, v);
    glasnost_sim::objects::remove(&mut w, v);
    assert!(w.temporal.beams.is_empty() && w.temporal.locks.is_empty());
    // An attacker removed from play lets its beam go.
    let v2 = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    erase(&mut w, a, v2);
    glasnost_sim::objects::remove(&mut w, a);
    assert!(!erased(&w, v2));
}

// spec: S-0514 claim 15 (E10 part g: a destroy-all detaches the beams on each erased victim before the destructive hit; leadership is not handed on)
#[test]
fn a_destroy_all_detaches_the_victims_beams() {
    let (mut w, a, v) = two();
    let b = attacker(&mut w, "ERASER2", 0, at(-12, -11));
    erase(&mut w, a, v);
    erase(&mut w, b, v);
    temporal::detach_for_destruction(&mut w, v);
    assert!(!erased(&w, v));
    assert!(w.temporal.beams.is_empty());
}
