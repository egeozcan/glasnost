//! Claim tests for the charging of overpowerable buildings (S-0518 claims
//! 11 to 15, scenarios TC1 to TC4) on synthetic data.

#[path = "linked_common/mod.rs"]
mod linked_common;

use glasnost_sim::combat::weapon;
use glasnost_sim::econ::power;
use glasnost_sim::linked;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};
use linked_common::*;

fn sleeper(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let id = spawn(w, kind, name, house, at(dx, dy));
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(id) {
        o.beh.force(Behaviour::Sleep, tick, stale);
    }
    id
}

/// A coil of house 0 at the origin and `n` chargers of house 0 around it
/// that have registered on it and aim at it.
fn charged(
    n: i32,
    tweak: impl FnOnce(&mut glasnost_rules::schema::Ruleset),
) -> (World, ObjectId, Vec<ObjectId>) {
    let mut w = world(tweak);
    let coil = sleeper(&mut w, Kind::Building, "COIL", 0, 0, 0);
    let us: Vec<ObjectId> = (0..n)
        .map(|i| sleeper(&mut w, Kind::Infantry, "CHARGER", 0, 1 + i, 2))
        .collect();
    run_to(&mut w, 3);
    for &u in &us {
        w.obj_mut(u).unwrap().target = Some(Target::Object(coil));
        zap(&mut w, u, coil);
    }
    (w, coil, us)
}

fn zap(w: &mut World, unit: ObjectId, building: ObjectId) {
    assert!(glasnost_sim::combat::fire::shot(
        w,
        unit,
        1,
        Target::Object(building)
    ));
    while !w.projectiles.is_empty() {
        testkit::run(w, 1);
    }
}

// spec: S-0518 claim 11 (TC4: an electric-charge detonation registers an infantry firer on any building, whatever its type or owner, once; no ordinary area damage; a vehicle registers nothing)
#[test]
fn a_charge_shot_registers_an_infantry_firer_once() {
    let mut w = world(|_| {});
    let enemy = spawn(&mut w, Kind::Building, "TOWER2", 1, at(0, 0));
    let c = sleeper(&mut w, Kind::Infantry, "CHARGER", 0, 3, 0);
    let car = sleeper(&mut w, Kind::Vehicle, "ZAPCAR", 0, 3, 2);
    let before = health(&w, enemy);
    zap(&mut w, c, enemy);
    zap(&mut w, c, enemy);
    zap(&mut w, car, enemy);
    assert_eq!(w.obj(enemy).unwrap().combat.chargers, vec![c]);
    assert_eq!(
        health(&w, enemy),
        before,
        "the charge shot does no area damage"
    );
    assert_eq!(linked::chargers(&w, enemy), 1);
}

// spec: S-0518 claim 13 and F3 (TC1: with 0, 1, 2 and 3 chargers: overpowered with 3 whatever the state, with 1 only while switched on at full power)
#[test]
fn the_overpowered_state_follows_the_charger_count() {
    for (n, off, want) in [
        (0, false, false),
        (1, false, true),
        (1, true, false),
        (2, false, true),
        (2, true, false),
        (3, true, true),
    ] {
        let (mut w, coil, _) = charged(n, |_| {});
        w.obj_mut(coil).unwrap().econ.building.switched_off = off;
        testkit::run(&mut w, 2);
        assert_eq!(
            w.obj(coil).unwrap().combat.overpowered,
            want,
            "{n} chargers, off {off}"
        );
    }
}

// spec: S-0518 claim 13 (TC1: with one charger on a low-power house the building is not overpowered; three overpower it whatever the power)
#[test]
fn low_power_needs_three_chargers() {
    for (n, want) in [(1, false), (3, true)] {
        let (mut w, coil, _) = charged(n, |_| {});
        w.houses[0].econ.power.drain = 100;
        w.houses[0].econ.power.output = 10;
        testkit::run(&mut w, 2);
        assert_eq!(
            w.obj(coil).unwrap().combat.overpowered,
            want,
            "{n} chargers"
        );
    }
}

// spec: S-0518 claim 12 and S-0405 claim 8 (TC1: a charger that stops aiming is pruned at the building's next turn; one that leaves play at once; n is S-0405's n_c)
#[test]
fn chargers_leave_the_list() {
    let (mut w, coil, us) = charged(3, |_| {});
    testkit::run(&mut w, 2);
    assert_eq!(power::chargers(&w, coil), 3);
    glasnost_sim::spawn::clear_own_target(w.obj_mut(us[0]).unwrap());
    assert_eq!(power::chargers(&w, coil), 3, "not yet: the building's turn");
    testkit::run(&mut w, 2);
    assert_eq!(power::chargers(&w, coil), 2);
    glasnost_sim::objects::remove(&mut w, us[1]);
    assert_eq!(power::chargers(&w, coil), 1, "at once");
}

// spec: S-0518 claim 12 and Edge cases (TC4: a non-overpowerable building keeps chargers that stopped aiming, and is never overpowered; quirks.stale_chargers_kept off prunes it like the other type)
#[test]
fn another_type_keeps_stale_chargers_unless_the_quirk_is_off() {
    for (kept, want) in [(true, 2), (false, 0)] {
        let mut w = world(|r| r.globals.quirks.stale_chargers_kept = kept);
        let b = sleeper(&mut w, Kind::Building, "TOWER2", 0, 0, 0);
        let us: Vec<ObjectId> = (0..2)
            .map(|i| sleeper(&mut w, Kind::Infantry, "CHARGER", 0, 1 + i, 2))
            .collect();
        for &u in &us {
            zap(&mut w, u, b);
        }
        testkit::run(&mut w, 3);
        assert_eq!(power::chargers(&w, b), want, "kept: {kept}");
        assert!(!w.obj(b).unwrap().combat.overpowered);
    }
}

// spec: S-0308 table P rows 10 and 11 and S-0518 claim 13 (an overpowered building uses slot 1; a unit with an electric-charge secondary uses slot 1 against an allied overpowerable building)
#[test]
fn weapon_choice_reads_the_overpowered_state() {
    let (mut w, coil, us) = charged(3, |_| {});
    let enemy = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(5, 0));
    testkit::run(&mut w, 2);
    assert!(w.obj(coil).unwrap().combat.overpowered);
    assert_eq!(
        weapon::would_use(&w, coil, Some(Target::Object(enemy))).0,
        1
    );
    w.obj_mut(coil).unwrap().combat.overpowered = false;
    assert_eq!(
        weapon::would_use(&w, coil, Some(Target::Object(enemy))).0,
        0
    );
    // Row 11: the charger against the allied (own-house) coil.
    assert_eq!(
        weapon::would_use(&w, us[0], Some(Target::Object(coil))).0,
        1
    );
    // Against a coil of the other house the charger takes its primary.
    let foe = spawn(&mut w, Kind::Building, "COIL", 1, at(8, 8));
    assert_eq!(weapon::would_use(&w, us[0], Some(Target::Object(foe))).0, 0);
}

// spec: S-0518 claim 14 (TC2: a charger with no target next to own-house overpowerable buildings takes the first in step-direction order; an enemy's building is never chosen; it gets attack and the charging mark)
#[test]
fn a_charger_picks_the_first_own_building_by_step_direction() {
    let mut w = world(|_| {});
    let u = spawn(&mut w, Kind::Infantry, "CHARGER", 0, at(0, 0));
    let _foe = spawn(&mut w, Kind::Building, "COIL", 1, at(0, -1)); // direction 0, an enemy's
    let east = spawn(&mut w, Kind::Building, "COIL", 0, at(1, 0)); // direction 2
    let _south = spawn(&mut w, Kind::Building, "COIL", 0, at(0, 1)); // direction 4
    assert!(linked::start_charging(&mut w, u));
    let o = w.obj(u).unwrap();
    assert_eq!(o.target, Some(Target::Object(east)));
    assert!(o.combat.charging);
    // spec: S-0518 claim 14 (revision 3: attack is queued without starting immediately)
    assert_eq!(o.beh.queued, Some(Behaviour::Attack));
    assert_ne!(o.beh.current, Some(Behaviour::Attack));
    // Nothing next to a unit that already has a target, or no building.
    assert!(!linked::start_charging(&mut w, u));
    let lone = spawn(&mut w, Kind::Infantry, "CHARGER", 0, at(10, 10));
    assert!(!linked::start_charging(&mut w, lone));
}

// spec: S-0518 claim 14 (TC2: in the guard behaviour the unit starts charging by itself, in the ordinary processing)
#[test]
fn a_guarding_charger_charges_by_itself() {
    let mut w = world(|_| {});
    let coil = spawn(&mut w, Kind::Building, "COIL", 0, at(1, 0));
    let u = spawn(&mut w, Kind::Infantry, "CHARGER", 0, at(0, 0));
    testkit::run(&mut w, 40);
    assert_eq!(w.obj(u).unwrap().target, Some(Target::Object(coil)));
    assert!(w.obj(coil).unwrap().combat.chargers.contains(&u) || w.obj(u).unwrap().combat.charging);
}

// spec: S-0518 claim 15 (TC3: an infantry unit charging by itself breaks off for an enemy in range and its mark clears; a vehicle with the mark does not; a target set by any means clears the mark)
#[test]
fn a_charging_infantry_unit_breaks_off_for_an_enemy() {
    let mut w = world(|_| {});
    let coil = sleeper(&mut w, Kind::Building, "COIL", 0, 1, 0);
    let u = spawn(&mut w, Kind::Infantry, "CHARGER", 0, at(0, 0));
    assert!(linked::start_charging(&mut w, u));
    let enemy = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(4, 0));
    testkit::run(&mut w, 40);
    let o = w.obj(u).unwrap();
    assert_eq!(o.target, Some(Target::Object(enemy)));
    assert!(!o.combat.charging);
    let _ = coil;
    // A vehicle does not break off.
    let mut w = world(|_| {});
    sleeper(&mut w, Kind::Building, "COIL", 0, 1, 0);
    let car = spawn(&mut w, Kind::Vehicle, "ZAPCAR", 0, at(0, 0));
    assert!(linked::start_charging(&mut w, car));
    let coil = w.obj(car).unwrap().target;
    spawn(&mut w, Kind::Vehicle, "VICT", 1, at(4, 0));
    testkit::run(&mut w, 40);
    assert_eq!(w.obj(car).unwrap().target, coil, "it keeps charging");
    assert!(w.obj(car).unwrap().combat.charging);
    // With quirks.vehicle_charge_no_break_off off it breaks off like infantry.
    let mut w = world(|r| r.globals.quirks.vehicle_charge_no_break_off = false);
    sleeper(&mut w, Kind::Building, "COIL", 0, 1, 0);
    let car = spawn(&mut w, Kind::Vehicle, "ZAPCAR", 0, at(0, 0));
    assert!(linked::start_charging(&mut w, car));
    let enemy = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(4, 0));
    testkit::run(&mut w, 40);
    assert_eq!(w.obj(car).unwrap().target, Some(Target::Object(enemy)));
    // An ordinary target change clears the mark.
    let mut w = world(|_| {});
    sleeper(&mut w, Kind::Building, "COIL", 0, 1, 0);
    let car = spawn(&mut w, Kind::Vehicle, "ZAPCAR", 0, at(0, 0));
    assert!(linked::start_charging(&mut w, car));
    glasnost_sim::spawn::set_own_target(w.obj_mut(car).unwrap(), None);
    assert!(!w.obj(car).unwrap().combat.charging);
}

// spec: S-0518 Determinism: the state hash survives a save and load with chargers, an overpowered building and a pending beam
#[test]
fn linked_state_survives_a_save_and_load() {
    let (mut w, coil, _) = charged(3, |_| {});
    let sup = spawn(&mut w, Kind::Building, "PRISM", 0, at(9, 9));
    w.obj_mut(sup).unwrap().combat.support_beam =
        Some((w.tick + 30, glasnost_sim::world::Pos::default()));
    testkit::run(&mut w, 3);
    assert!(w.obj(coil).unwrap().combat.overpowered);
    let bytes = glasnost_sim::save::save(&w);
    let h = glasnost_sim::save::state_hash(&w);
    let w2 = glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue.clone().unwrap()).unwrap();
    assert_eq!(glasnost_sim::save::state_hash(&w2), h);
    assert_eq!(
        w2.obj(coil).unwrap().combat.chargers,
        w.obj(coil).unwrap().combat.chargers
    );
}

// Sim issues S11, #231: the charging scan of area guard (S-0215 claim 26).

/// A charger in *area guard* next to an own overpowerable building, its
/// targeting timer set to `timer`. Returns the world, the unit, the
/// building and whether the unit took the building within 600 ticks.
fn area_guarding_charger(
    timer: i32,
    tweak: impl FnOnce(&mut glasnost_rules::schema::Ruleset),
) -> (World, ObjectId, ObjectId, bool) {
    let mut w = world(tweak);
    let coil = spawn(&mut w, Kind::Building, "COIL", 0, at(1, 0));
    let u = spawn(&mut w, Kind::Infantry, "CHARGER", 0, at(0, 0));
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(u) {
        o.beh.force(Behaviour::AreaGuard, tick, stale);
        o.targeting_timer = timer;
    }
    let mut took = false;
    for _ in 0..600 {
        testkit::run(&mut w, 1);
        if w.obj(u).unwrap().target == Some(Target::Object(coil)) {
            took = true;
            break;
        }
    }
    (w, u, coil, took)
}

// spec: S-0215 claim 26 (no target once the leash and the search have taken effect: the automatic charging scan runs; a search that finds nothing is followed by the idle action) and S-0518 claim 14 (attack queued, the charging mark set)
#[test]
fn issue_231_area_guard_scan_after_a_search_that_finds_nothing() {
    let (w, u, _, took) = area_guarding_charger(0, |_| {});
    assert!(took, "the scan took the building");
    let o = w.obj(u).unwrap();
    assert!(o.combat.charging);
}

// spec: S-0215 claim 26 (the scan runs whether or not the actor may search: its targeting timer has not run out)
#[test]
fn issue_231_area_guard_scan_when_the_timer_has_not_run_out() {
    let (w, u, _, took) = area_guarding_charger(1_000_000, |_| {});
    let o = w.obj(u).unwrap();
    assert!(o.guard_spot.is_some(), "area guard was processed");
    assert!(took, "the scan took the building; target {:?}", o.target);
    assert!(o.combat.charging);
}

// spec: S-0215 claim 26 (the scan runs whether or not the actor may search: a type with no passive acquisition)
#[test]
fn issue_231_area_guard_scan_without_passive_acquisition() {
    let (w, u, _, took) = area_guarding_charger(0, |r| {
        r.sim
            .infantry
            .get_mut("CHARGER")
            .unwrap()
            .combat
            .can_passive_aquire = Some(false);
    });
    let o = w.obj(u).unwrap();
    assert!(took, "the scan took the building; target {:?}", o.target);
    assert!(o.combat.charging);
}
