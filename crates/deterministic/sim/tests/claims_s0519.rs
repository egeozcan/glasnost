//! Claim tests for planted bombs and suicide weapons (S-0519 claims 2 to 11),
//! on synthetic data. The numbers are invented by the fixture.

#[path = "terror_common/mod.rs"]
mod terror_common;

use glasnost_sim::bomb;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::HouseId;
use terror_common::*;

/// A bomber, a victim vehicle and a bystander next to it; the bomb planted.
fn planted() -> (
    glasnost_sim::world::World,
    glasnost_sim::world::ObjectId,
    glasnost_sim::world::ObjectId,
    glasnost_sim::world::ObjectId,
    i32,
) {
    let mut w = world(|_| {});
    let v = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let near = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(1, 0));
    let p = quiet(&mut w, Kind::Infantry, "BOMBER", 0, at(-5, 0));
    assert!(shoot(&mut w, p, v));
    let t = w.obj(v).unwrap().combat.bomb.expect("planted").planted;
    (w, p, v, near, t)
}

// spec: S-0519 claim 2 (T2: the planting does no area damage; the bomb records its planter, the planter's house and the plant tick)
#[test]
fn planting_records_the_bomb_and_hurts_nobody() {
    let (w, p, v, near, t) = planted();
    let b = w.obj(v).unwrap().combat.bomb.unwrap();
    assert_eq!((b.planter, b.house, b.planted), (Some(p), Some(HouseId(0)), t));
    assert_eq!((health(&w, v), health(&w, near)), (100_000, 100_000));
}

// spec: S-0519 claim 4 and F1 (T4: a carrier that takes a turn every tick sees the bomb go off in tick t + D + 1, D = 50)
#[test]
fn the_bomb_goes_off_in_the_tick_after_the_delay() {
    let (mut w, _, v, _, t) = planted();
    // The counter reads n + 1 after the step of tick n.
    run_to(&mut w, t + 51);
    assert_eq!(health(&w, v), 100_000, "nothing in the step of tick t + D");
    testkit::run(&mut w, 1);
    assert_eq!(health(&w, v), 100_000 - 100, "bomb damage 100 in the step of tick t + D + 1");
    assert!(w.obj(v).unwrap().combat.bomb.is_none());
}

// spec: S-0519 claim 5 (T2: one detonation with the bomb warhead's spread hits the adjacent second vehicle too)
#[test]
fn the_detonation_has_an_area() {
    let (mut w, _, _, near, t) = planted();
    run_to(&mut w, t + 52);
    assert!(health(&w, near) < 100_000);
}

// spec: S-0519 claim 2 (T2: the target must be a unit, infantry, aircraft or building; the planter must exist and be infantry: a vehicle firer plants nothing)
#[test]
fn only_an_infantry_planter_plants() {
    let mut w = world(|r| {
        vehicle(r, "TANKP", |t| t.primary = Some("PlantW".into()));
    });
    let v = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let p = quiet(&mut w, Kind::Vehicle, "TANKP", 0, at(-5, 0));
    shoot(&mut w, p, v);
    assert!(w.obj(v).unwrap().combat.bomb.is_none());
}

// spec: S-0519 claim 2 (T2: a planter killed while its projectile is in flight plants nothing)
#[test]
fn a_dead_planter_plants_nothing() {
    let mut w = world(|_| {});
    let v = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let p = quiet(&mut w, Kind::Infantry, "BOMBER", 0, at(-5, 0));
    glasnost_sim::combat::fire::shot(&mut w, p, 0, glasnost_sim::world::Target::Object(v));
    w.obj_mut(p).unwrap().health = 0;
    glasnost_sim::objects::remove(&mut w, p);
    for _ in 0..30 {
        testkit::run(&mut w, 1);
    }
    assert!(w.obj(v).unwrap().combat.bomb.is_none());
}

// spec: S-0519 claim 3 (T3: a bomb weapon is illegal against a carrier; a disarm weapon against a clean target)
#[test]
fn bomb_weapons_are_illegal_against_the_wrong_targets() {
    let (mut w, _, v, _, _) = planted();
    let bp = quiet(&mut w, Kind::Infantry, "BOMBER", 0, at(-6, 0));
    let w_plant = glasnost_sim::combat::weapon::slot_weapon(&w, bp, 0).unwrap();
    let clean = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(3, 3));
    let en = quiet(&mut w, Kind::Infantry, "ENG", 0, at(-6, 1));
    let w_dis = glasnost_sim::combat::weapon::slot_weapon(&w, en, 0).unwrap();
    use glasnost_sim::world::Target::Object;
    assert!(bomb::illegal_target(&w, w_plant, Object(v)));
    assert!(!bomb::illegal_target(&w, w_plant, Object(clean)));
    assert!(!bomb::illegal_target(&w, w_dis, Object(v)));
    assert!(bomb::illegal_target(&w, w_dis, Object(clean)));
}

// spec: S-0519 claim 7 (T6: a disarm removes the bomb with no damage, even when the carrier has moved; no distance test)
#[test]
fn a_disarm_removes_the_bomb_without_damage() {
    let (mut w, _, v, _, t) = planted();
    let e = quiet(&mut w, Kind::Infantry, "ENG", 0, at(-12, 0));
    assert!(shoot(&mut w, e, v));
    assert!(w.obj(v).unwrap().combat.bomb.is_none());
    run_to(&mut w, t + 60);
    assert_eq!(health(&w, v), 100_000);
}

// spec: S-0519 claim 5 and S-0302 claim 1 (T7: the planter is the attacker while it exists, so its house is credited with the kill; with the planter removed there is no attacker and no credit, claim 8)
#[test]
fn credit_goes_to_the_surviving_planter() {
    for planter_lives in [true, false] {
        let mut w = world(|r| r.sim.general.ivan_damage = Some(200));
        let v = quiet(&mut w, Kind::Vehicle, "WEAK", 1, at(0, 0));
        let p = quiet(&mut w, Kind::Infantry, "BOMBER", 0, at(-5, 0));
        shoot(&mut w, p, v);
        let t = w.obj(v).unwrap().combat.bomb.unwrap().planted;
        if !planter_lives {
            glasnost_sim::objects::remove(&mut w, p);
        }
        run_to(&mut w, t + 52);
        assert!(w.obj(v).is_none(), "the bomb killed its carrier");
        assert_eq!(w.houses[0].combat.kills, i32::from(planter_lives));
    }
}

// spec: S-0519 claim 8 (T9 d: a planter removed from play leaves the bomb running; it forgets its planter and keeps the house)
#[test]
fn a_removed_planter_is_forgotten_but_the_bomb_stays() {
    let (mut w, p, v, _, t) = planted();
    glasnost_sim::objects::remove(&mut w, p);
    let b = w.obj(v).unwrap().combat.bomb.unwrap();
    assert_eq!((b.planter, b.house), (None, Some(HouseId(0))));
    run_to(&mut w, t + 52);
    assert_eq!(health(&w, v), 100_000 - 100);
}

// spec: S-0519 claim 7 (T9 f: a carrier removed other than by death takes its bomb silently)
#[test]
fn a_removed_carrier_does_not_explode() {
    let (mut w, _, v, near, t) = planted();
    glasnost_sim::objects::remove(&mut w, v);
    run_to(&mut w, t + 60);
    assert_eq!(health(&w, near), 100_000);
}

// spec: S-0519 claim 6 (T8: a bomb cannot go off while its carrier is off the map; overdue, it goes off at the first turn back on the map)
#[test]
fn an_off_map_carrier_waits_and_goes_off_on_return() {
    let (mut w, _, v, _, t) = planted();
    w.obj_mut(v).unwrap().on_map = false;
    run_to(&mut w, t + 80);
    assert_eq!(health(&w, v), 100_000);
    w.obj_mut(v).unwrap().on_map = true;
    testkit::run(&mut w, 1);
    assert_eq!(health(&w, v), 100_000 - 100);
}

// spec: S-0519 claim 6 and 9 (T8, T10: a detonate order on an off-map carrier removes the bomb with no effect)
#[test]
fn an_off_map_detonation_is_silent() {
    let (mut w, _, v, near, _) = planted();
    w.obj_mut(v).unwrap().on_map = false;
    bomb::detonate(&mut w, v);
    assert!(w.obj(v).unwrap().combat.bomb.is_none());
    assert_eq!((health(&w, v), health(&w, near)), (100_000, 100_000));
}

// spec: S-0519 claim 9 and S-0213 claim 11 (T10: the detonate order sets the bomb off at once)
#[test]
fn the_detonate_order_goes_off_at_once() {
    let (mut w, _, v, _, t) = planted();
    run_to(&mut w, 6);
    assert!(orders::issue(&mut w, HouseId(1), Order::Detonate { actor: v }));
    testkit::run(&mut w, 2);
    assert!(w.tick < t + 50, "long before the timer");
    assert_eq!(health(&w, v), 100_000 - 100);
}

// spec: S-0519 claim 9 (T10: only the timed-bomb permission allows the order, row G3)
#[test]
fn the_permission_flag_gates_the_order() {
    let (w, _, v, _, _) = planted();
    assert!(bomb::detonable(&w, v));
    let mut w2 = world(|r| r.sim.general.can_detonate_time_bomb = Some(false));
    let v2 = quiet(&mut w2, Kind::Vehicle, "VICT", 1, at(0, 0));
    let p = quiet(&mut w2, Kind::Infantry, "BOMBER", 0, at(-5, 0));
    shoot(&mut w2, p, v2);
    assert!(!bomb::detonable(&w2, v2));
}

// spec: S-0519 claim 10 and F2 (T11: the planter's house sees the bomb; other houses only with a detector of theirs closer than its bomb-sight range, strictly: exactly 3 cells is not inside range 3)
#[test]
fn visibility_follows_the_planter_and_detectors() {
    let mut w = world(|r| vehicle(r, "DET", |t| t.combat.bomb_sight = Some(3)));
    let v = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let p = quiet(&mut w, Kind::Infantry, "BOMBER", 0, at(-5, 0));
    shoot(&mut w, p, v);
    assert!(bomb::visible(&w, HouseId(0), v));
    assert!(!bomb::visible(&w, HouseId(1), v));
    let det = quiet(&mut w, Kind::Vehicle, "DET", 1, at(0, 3));
    assert!(!bomb::visible(&w, HouseId(1), v), "exactly 3 cells: not inside");
    w.obj_mut(det).unwrap().pos = w.obj(v).unwrap().pos;
    assert!(bomb::visible(&w, HouseId(1), v), "at the carrier: inside");
    w.obj_mut(det).unwrap().on_map = false;
    assert!(!bomb::visible(&w, HouseId(1), v), "a detector off the map does not count");
}

// spec: S-0519 claim 11 (T12: a suicide shot kills the firer with no projectile and no rearm; the death weapon hurts the target)
#[test]
fn a_suicide_shot_kills_its_firer() {
    let mut w = world(|_| {});
    let k = quiet(&mut w, Kind::Infantry, "KAMI", 0, at(0, 0));
    let near = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(1, 0));
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        k,
        0,
        glasnost_sim::world::Target::Object(near)
    ));
    assert!(w.projectiles.is_empty(), "no projectile");
    testkit::run(&mut w, 2);
    assert!(w.obj(k).is_none(), "the firer is dead");
    assert_eq!(health(&w, near), 100_000 - 80, "the death weapon's damage, not the suicide weapon's 5");
}
