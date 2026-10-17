//! Claim tests for S-0601 revision 9 claim 16 (*Where the overload update
//! falls in the controller's turn* and *Updates in which the overload
//! update runs*), which answers Q-1201: after the berserk countdown, before
//! self-healing, behaviour, movement and firing; not while being erased; a
//! lethal pulse ends the update. Synthetic data only: the chrono-weapon
//! fixture `tests/temporal_common` with an invented unlimited controller.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{WarheadType, WeaponType};
use glasnost_sim::mind;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::transport;
use glasnost_sim::world::{ObjectId, Target, World};
use temporal_common::*;

const ALPHA: u8 = 0;
const BETA: u8 = 1;
/// The pulse damage of every level in these tests.
const PULSE: i32 = 7;
/// MASTER's strength.
const STRENGTH: i32 = 10_000;

/// The temporal fixture plus MIND (a takeover warhead), BOOM (the
/// demolition warhead), RayU (MIND, damage 3, unlimited), MASTER (an
/// unlimited controller of strength 10000 that self-heals every tick),
/// MASTERI (an unlimited controller on foot) and
/// the overload lists count (0, 5), damage (7, 7), frames (3, 3).
fn rules() -> Ruleset {
    let mut r = ruleset();
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            mind_control: Some(true),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "BOOM".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "RayU".into(),
        WeaponType {
            damage: Some(3),
            rof: Some(30),
            range: Some(d("10")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            infinite_mind_control: Some(true),
            ..Default::default()
        },
    );
    vehicle(&mut r, "MASTER", |t| {
        t.primary = Some("RayU".into());
        t.strength = Some(STRENGTH);
        t.combat.self_healing = Some(true);
    });
    infantry(&mut r, "MASTERI", |t| t.primary = Some("RayU".into()));
    let g = &mut r.sim.general;
    g.c4_warhead = Some("BOOM".into());
    // spec: S-0314 F1 and Edge cases (a rate whose period is 0 ticks is treated as 1: a heal at every tick)
    g.repair_rate = Some(d("0.001"));
    g.overload_count = Some(vec![0, 5]);
    g.overload_damage = Some(vec![PULSE, PULSE]);
    g.overload_frames = Some(vec![3, 3]);
    r
}

/// A MASTER of BETA that does nothing by itself.
fn master(w: &mut World, dx: i32) -> ObjectId {
    let m = spawn(w, Kind::Vehicle, "MASTER", BETA, at(dx, 0));
    sleep(w, m);
    m
}

fn sleep(w: &mut World, id: ObjectId) {
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(id)
        .unwrap()
        .beh
        .force(Behaviour::Sleep, tick, stale);
}

fn countdown(w: &World, m: ObjectId) -> i32 {
    mind::controller(w, m).unwrap().countdown
}

fn set_countdown(w: &mut World, m: ObjectId, c: i32) {
    w.mind.controllers.get_mut(&m).unwrap().countdown = c;
}

// spec: S-0601 claim 16 (revision 9: the overload update runs in exactly the berserk countdown's updates; being erased by a temporal weapon, an excluded update of S-0215 claim 2, takes no overload step and none is made up)
#[test]
fn claim_16_no_overload_update_while_erased() {
    let mut w = game(&rules());
    let m = master(&mut w, 0);
    let e = attacker(&mut w, "ERASER", ALPHA, at(2, 0));
    set_countdown(&mut w, m, 25);
    erase(&mut w, e, m);
    assert!(erased(&w, m));
    let c = countdown(&w, m);
    assert!(c > 0);
    testkit::run(&mut w, 5);
    assert!(erased(&w, m), "still being erased");
    assert_eq!(countdown(&w, m), c, "no step while being erased");
}

// spec: S-0601 claim 16 (revision 9: a heal of the pulse's update is applied to the health the pulse leaves; a heal never softens a pulse) and S-0314 claim 9
#[test]
fn claim_16_heal_follows_the_pulse() {
    let mut w = game(&rules());
    let m = master(&mut w, 0);
    testkit::run(&mut w, 2);
    assert_eq!(health(&w, m), STRENGTH);
    set_countdown(&mut w, m, 0);
    testkit::run(&mut w, 1);
    assert_eq!(
        health(&w, m),
        STRENGTH - PULSE + 1,
        "pulsed from full health, then healed"
    );
    assert_eq!(countdown(&w, m), 3);
}

/// The world one tick before MASTER's shot at a BOX of ALPHA leaves.
fn about_to_fire() -> (World, ObjectId, ObjectId) {
    let mut w = game(&rules());
    let m = master(&mut w, 0);
    let b = spawn(&mut w, Kind::Vehicle, "VICT", ALPHA, at(3, 0));
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    {
        let o = w.obj_mut(m).unwrap();
        o.target = Some(Target::Object(b));
        o.beh.force(Behaviour::Attack, tick, stale);
    }
    set_countdown(&mut w, m, 1000);
    for _ in 0..100 {
        let mut next = w.clone();
        testkit::run(&mut next, 1);
        if fired(&next, m, b) {
            return (w, m, b);
        }
        w = next;
    }
    panic!("MASTER never fired");
}

/// Whether MASTER's takeover shot left: a bullet in flight or BOX taken.
fn fired(w: &World, m: ObjectId, b: ObjectId) -> bool {
    !w.projectiles.is_empty() || mind::controller(w, m).is_some_and(|r| r.victims.contains(&b))
}

// spec: S-0601 claim 16 (revision 9: a shot the controller fires in an update leaves after that update's pulse; a pulse that kills a vehicle controller ends the update's remaining work: no self-healing, behaviour, movement or firing, so the shot does not leave)
#[test]
fn claim_16_lethal_pulse_ends_the_update() {
    let (w, m, b) = about_to_fire();
    // A surviving pulse: the shot still leaves, after the pulse.
    let mut a = w.clone();
    let h = health(&a, m);
    set_countdown(&mut a, m, 0);
    testkit::run(&mut a, 1);
    assert!(fired(&a, m, b), "the shot leaves in the pulse's update");
    assert_eq!(health(&a, m), (h - PULSE + 1).min(STRENGTH));
    // A lethal pulse: no heal and no shot.
    let mut k = w.clone();
    k.obj_mut(m).unwrap().health = PULSE;
    set_countdown(&mut k, m, 0);
    testkit::run(&mut k, 1);
    assert!(
        k.obj(m).is_none_or(|o| o.health <= 0),
        "the pulse killed it and no heal followed"
    );
    assert!(!fired(&k, m, b), "no shot after a lethal pulse");
}

// spec: S-0601 claim 16 (revision 9: the overload update runs in the berserk countdown's updates, after it) and S-0610 claim 5 (one countdown step in each update)
#[test]
fn claim_16_steps_with_the_berserk_countdown() {
    let mut w = game(&rules());
    let m = master(&mut w, 0);
    w.obj_mut(m).unwrap().combat.berserk = Some(50);
    set_countdown(&mut w, m, 20);
    testkit::run(&mut w, 3);
    assert_eq!(w.obj(m).unwrap().combat.berserk, Some(47));
    assert_eq!(countdown(&w, m), 17);
}

// spec: S-0601 claim 16 (revision 9: a controller inside a transport that is not open-topped takes no update, so its overload countdown stays as it is; one in an open-topped transport is in play and steps)
#[test]
fn claim_16_transports() {
    let mut w = game(&rules());
    let closed = spawn(&mut w, Kind::Vehicle, "GUNNER", BETA, at(0, 0));
    let p = spawn(&mut w, Kind::Infantry, "MASTERI", BETA, at(0, 1));
    transport::board(&mut w, p, closed);
    let open = spawn(&mut w, Kind::Vehicle, "OPEN", BETA, at(6, 0));
    let q = spawn(&mut w, Kind::Infantry, "MASTERI", BETA, at(6, 1));
    transport::board(&mut w, q, open);
    assert!(transport::open_topped_passenger(&w, q));
    set_countdown(&mut w, p, 20);
    set_countdown(&mut w, q, 20);
    testkit::run(&mut w, 5);
    assert_eq!(countdown(&w, p), 20, "kept inside");
    assert_eq!(countdown(&w, q), 15, "in play: steps");
}
