//! Claim tests of S-0513 claims 7 to 13 (the nuclear strike) and S-0524
//! claim 3 (the waypoint strike), on synthetic rules (`tests/sw_common`).

mod sw_common;

use glasnost_sim::combat::state::Projectile;
use glasnost_sim::superweapons::{self, SwEvent, nuke};
use glasnost_sim::world::{ObjectId, World};
use sw_common::*;

fn weapon(w: &World, name: &str) -> u32 {
    *w.rules().combat.weapon_index.get(name).unwrap()
}

fn missiles(w: &World, name: &str) -> Vec<Projectile> {
    let wp = weapon(w, name);
    w.projectiles
        .values()
        .filter(|p| p.weapon == wp)
        .cloned()
        .collect()
}

fn surface(w: &World, dx: i32, dy: i32) -> i32 {
    superweapons::cell_point(w, at(dx, dy), false).z
}

/// A game with Alpha's silo placed and its STRIKE record ready.
fn ready_silo() -> (World, ObjectId) {
    let r = ruleset();
    let mut w = game(&r);
    let silo = spawn(&mut w, "SILO", ALPHA, -6, -6);
    ready_tick(&mut w, ALPHA, STRIKE, 200);
    (w, silo)
}

// --- Claim 7 -------------------------------------------------------------------

#[test]
fn claim7_the_silo_owned_longest_is_chosen() {
    // spec: S-0513 claim 7 (of the house's on-map buildings of the selected type, the one owned longest)
    let r = ruleset();
    let mut w = game(&r);
    let first = spawn(&mut w, "SILO", ALPHA, -6, -6);
    let _second = spawn(&mut w, "SILO", ALPHA, 6, 6);
    assert_eq!(nuke::silo_choice(&w, ALPHA, STRIKE), Some(first));
    assert_eq!(nuke::silo_choice(&w, BETA, STRIKE), None);
}

#[test]
fn claim7_no_silo_no_sounds_and_no_missile() {
    // spec: S-0513 claim 7 (with no matching building, neither missile nor launch sounds)
    let r = ruleset();
    let mut w = game(&r);
    w.effects.clear();
    nuke::silo_launch(&mut w, ALPHA, STRIKE, at(0, 0), None);
    assert!(events(&w).is_empty());
    assert!(w.projectiles.is_empty());
}

#[test]
fn claim7_busy_silo_records_target_and_sounds_but_no_second_missile() {
    // spec: S-0513 claim 7 (a busy silo: target recorded, sounds play, no additional missile)
    let (mut w, silo) = ready_silo();
    nuke::silo_launch(&mut w, ALPHA, STRIKE, at(0, 0), Some(silo));
    assert!(!nuke::accepts(&w, silo));
    w.effects.clear();
    nuke::silo_launch(&mut w, ALPHA, STRIKE, at(3, 3), Some(silo));
    assert!(events(&w).iter().any(|e| e.0 == SwEvent::Siren));
    assert_eq!(w.sw.houses[0].nuke_target, Some(at(3, 3)));
    step(&mut w);
    assert_eq!(missiles(&w, "Lift").len(), 1);
}

// --- Claims 8, 9 and 12 ------------------------------------------------------------

#[test]
fn claim8_9_silo_launch_flight_and_falling_missile() {
    // spec: S-0513 claims 5, 8 and 9 (announcement and siren at once; the upward missile on the silo's next update; the falling missile at its burst)
    let (mut w, silo) = ready_silo();
    let target = at(2, 0);
    let cart = spawn(&mut w, "CART", BETA, 2, 0);
    fire(&mut w, ALPHA, STRIKE, target);
    step(&mut w);
    let ev = events(&w);
    assert!(ev.contains(&(SwEvent::Launch, Some(ALPHA), Some(STRIKE))));
    assert!(ev.contains(&(SwEvent::Siren, Some(ALPHA), Some(STRIKE))));
    assert_eq!(w.sw.houses[0].nuke_target, Some(target));
    assert!(
        missiles(&w, "Lift").is_empty(),
        "the missile waits for the silo's next update"
    );
    step(&mut w);
    let up = missiles(&w, "Lift");
    assert_eq!(up.len(), 1);
    assert_eq!(up[0].attacker, Some(silo));
    assert_eq!(up[0].top_speed, nuke::UP_TOP_SPEED);
    assert!(up[0].vel[0].eq_num(glasnost_num::F64::ZERO));
    // The upward missile bursts above the altitude; the falling one appears over the target.
    until(&mut w, 200, |w| !missiles(w, "NukePayload").is_empty());
    assert!(missiles(&w, "Lift").is_empty());
    let fall = &missiles(&w, "NukePayload")[0];
    assert_eq!(fall.attacker, Some(silo));
    assert_eq!(fall.house, Some(ALPHA));
    let c = superweapons::cell_point(&w, target, false);
    assert_eq!((fall.pos.x, fall.pos.y), (c.x, c.y));
    // spec: S-0513 F8 (z₀ = h(c) + A_up; its first turn may already have moved it down)
    assert!(fall.pos.z <= surface(&w, 2, 0) + UP_ALT);
    assert!(fall.pos.z > surface(&w, 2, 0) + UP_ALT - 10);
    let h0 = w.obj(cart).unwrap().health;
    until(&mut w, 400, |w| missiles(w, "NukePayload").is_empty());
    // spec: S-0513 claim 12 (the falling missile detonates with its own damage and warhead)
    assert!(w.obj(cart).is_none_or(|o| o.health < h0));
}

#[test]
fn claim13_silo_sold_or_destroyed_before_its_update_launches_nothing() {
    // spec: S-0513 claim 13 (destroyed before its next update: no missile leaves)
    let (mut w, silo) = ready_silo();
    fire(&mut w, ALPHA, STRIKE, at(0, 0));
    step(&mut w);
    kill(&mut w, silo);
    run(&mut w, 3);
    assert!(missiles(&w, "Lift").is_empty());
}

#[test]
fn claim13_destroyed_after_launch_falling_missile_has_no_owner() {
    // spec: S-0513 claim 13 (destroyed after the upward missile left: the falling missile has no owner)
    let (mut w, silo) = ready_silo();
    fire(&mut w, ALPHA, STRIKE, at(0, 0));
    run(&mut w, 2);
    assert_eq!(missiles(&w, "Lift").len(), 1);
    kill(&mut w, silo);
    until(&mut w, 200, |w| !missiles(w, "NukePayload").is_empty());
    let fall = &missiles(&w, "NukePayload")[0];
    assert_eq!(fall.attacker, None);
}

// --- Claim 10 --------------------------------------------------------------------

#[test]
fn claim10_one_time_strike_falls_from_20000_without_owner() {
    // spec: S-0513 claim 10 and F2 (no owner, 20 000 leptons above the surface, straight down at 100)
    let r = ruleset();
    let mut w = game(&r);
    w.effects.clear();
    nuke::one_time(&mut w, Some(ALPHA), STRIKE, at(1, 1));
    let fall = &missiles(&w, "NukePayload")[0];
    assert_eq!(fall.attacker, None);
    assert_eq!(fall.house, None);
    assert_eq!(fall.pos.z, surface(&w, 1, 1) + 20_000);
    assert_eq!(fall.vel[2].to_i32_common(), -100);
    assert!(events(&w).iter().any(|e| e.0 == SwEvent::Siren));
}

// --- S-0524 claim 3 ------------------------------------------------------------------

#[test]
fn s0524_claim3_waypoint_strike_speed_50_and_table_vector() {
    // spec: S-0524 claim 3 (no firer and no owning house; speed 50; the downward table vector)
    let r = ruleset();
    let mut w = game(&r);
    nuke::waypoint(&mut w, at(0, 0));
    let fall = &missiles(&w, "NukePayload")[0];
    assert_eq!(fall.top_speed, 50);
    assert_eq!(fall.attacker, None);
    assert_eq!(fall.pos.z, surface(&w, 0, 0) + 20_000);
    assert_eq!(fall.vel[2].to_i32_common(), -100);
    // spec: S-0524 Formulas (the table cosine is not exactly zero: tiny horizontal components)
    assert!(!fall.vel[1].eq_num(glasnost_num::F64::ZERO));
}
