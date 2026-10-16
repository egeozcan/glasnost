//! Claim tests for S-0519: ownership and removal of bombs, the detonate
//! order's name, carriers that die or leave the map, demolition charges.

#[path = "terror_common/mod.rs"]
mod terror_common;

use glasnost_num::{F64, Rounding};
use glasnost_sim::bomb;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use terror_common::*;

fn bombed(w: &mut World, kind: Kind, name: &str, c: glasnost_map::CellXY) -> (ObjectId, i32) {
    let v = quiet(w, kind, name, 1, c);
    let p = quiet(w, Kind::Infantry, "BOMBER", 0, at(-8, -8));
    assert!(shoot(w, p, v));
    (v, w.obj(v).unwrap().combat.bomb.expect("planted").planted)
}

// spec: S-0519 claim 7 and S-0509 claim 13 (T9 c: a non-garrisonable building that changes owner loses its bomb with no explosion)
#[test]
fn a_captured_building_loses_its_bomb() {
    let mut w = world(|_| {});
    let (b, t) = bombed(&mut w, Kind::Building, "BLDG", at(0, 0));
    ownership::change(&mut w, b, Some(HouseId(0)), Cause::Capture);
    assert!(w.obj(b).unwrap().combat.bomb.is_none());
    run_to(&mut w, t + 60);
    assert_eq!(health(&w, b), 800);
}

// spec: S-0519 claim 8 (T9 e: a garrisonable building keeps its bomb and its expiry tick when its owner changes)
#[test]
fn a_garrisonable_building_keeps_its_bomb() {
    let mut w = world(|_| {});
    let (b, t) = bombed(&mut w, Kind::Building, "HUT", at(0, 0));
    ownership::change(&mut w, b, Some(HouseId(0)), Cause::Trigger);
    assert_eq!(w.obj(b).unwrap().combat.bomb.unwrap().planted, t);
    run_to(&mut w, t + 52);
    assert_eq!(health(&w, b), 800 - 100);
}

// spec: S-0519 claim 8 (T9 b: a unit that changes owner keeps its bomb, its timer and the planter's house)
#[test]
fn a_unit_changing_owner_keeps_its_bomb() {
    let mut w = world(|_| {});
    let (v, t) = bombed(&mut w, Kind::Vehicle, "VICT", at(0, 0));
    w.obj_mut(v).unwrap().owner = Some(HouseId(0));
    let b = w.obj(v).unwrap().combat.bomb.unwrap();
    assert_eq!((b.planted, b.house), (t, Some(HouseId(0))));
    run_to(&mut w, t + 52);
    assert_eq!(health(&w, v), 100_000 - 100);
}

// spec: S-0519 claim 5 and S-0312 claim 7 (a bomb whose detonation kills the carrier goes off once: the carrier's death does not detonate it a second time)
#[test]
fn a_killed_carrier_sets_its_bomb_off_once() {
    let mut w = world(|_| {});
    let (v, _) = bombed(&mut w, Kind::Vehicle, "VICT", at(0, 0));
    let near = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(1, 0));
    w.obj_mut(v).unwrap().health = 50;
    run_to(&mut w, 70);
    assert!(w.obj(v).is_none());
    assert_eq!(health(&w, near), 100_000 - 100, "one detonation, not two");
}

// spec: S-0519 claim 6 (T8 b: a carrier that dies off the map does not set its bomb off)
#[test]
fn a_carrier_that_dies_off_the_map_is_silent() {
    let mut w = world(|_| {});
    let (v, _) = bombed(&mut w, Kind::Vehicle, "VICT", at(0, 0));
    let near = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(1, 0));
    w.obj_mut(v).unwrap().on_map = false;
    bomb::carrier_killed(&mut w, v);
    assert!(w.obj(v).unwrap().combat.bomb.is_none());
    assert_eq!(health(&w, near), 100_000);
}

// spec: S-0213 claim 11 and quirks.detonate_in_planter_name (the order fills the planter's house's list; off: the issuing house's, and the bomb is resolved either way)
#[test]
fn the_detonate_order_names_the_planters_house() {
    for planter_name in [true, false] {
        let mut w = world(|r| r.globals.quirks.detonate_in_planter_name = planter_name);
        let (v, _) = bombed(&mut w, Kind::Vehicle, "VICT", at(0, 0));
        run_to(&mut w, 6);
        assert!(orders::issue(
            &mut w,
            HouseId(1),
            Order::Detonate { actor: v }
        ));
        let (planter_list, issuer_list) = (w.houses[0].pending.len(), w.houses[1].pending.len());
        assert_eq!(
            (planter_list, issuer_list),
            if planter_name { (1, 0) } else { (0, 1) }
        );
        testkit::run(&mut w, 2);
        assert_eq!(health(&w, v), 100_000 - 100);
    }
}

// spec: S-0519 claim 11 and S-0301 claim 9 (T12: the suicide shot uses no ammunition, plays no rearm: the firer is dead whatever its health)
#[test]
fn a_hurt_suicide_firer_still_dies() {
    let mut w = world(|_| {});
    let k = quiet(&mut w, Kind::Infantry, "KAMI", 0, at(0, 0));
    w.obj_mut(k).unwrap().health = 1;
    glasnost_sim::combat::fire::shot(&mut w, k, 0, Target::Cell(at(2, 0)));
    assert!(w.obj(k).is_none() || health(&w, k) <= 0);
}

// ---- charges ----------------------------------------------------------

fn sabotage(w: &mut World, name: &str, house: u8, target: ObjectId) -> ObjectId {
    let cell = w.obj(target).unwrap().pos.cell();
    let s = quiet(w, Kind::Infantry, name, house, cell);
    let (tick, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
    let o = w.obj_mut(s).unwrap();
    o.beh.force(Behaviour::Sabotage, tick, stale);
    o.dest = Some(Target::Object(target));
    s
}

fn target_building(w: &mut World) -> ObjectId {
    quiet(w, Kind::Building, "BLDG", 1, at(0, 0))
}

// spec: S-0519 claim 13, 14 and F4 (T13 a: an arrival plants a charge: a countdown of K = trunc(0.0111 x 900) = 9 ticks starts in the arrival tick and the walker is recorded as the planter)
#[test]
fn an_arrival_plants_a_charge() {
    let mut w = world(|_| {});
    let b = target_building(&mut w);
    let s = sabotage(&mut w, "SABO", 0, b);
    assert_eq!(bomb::charge_countdown(&w), 9);
    assert_eq!(bomb::charge_flash(&w), 4);
    let now = w.tick;
    assert!(bomb::charge_arrival(&mut w, s));
    let o = w.obj(b).unwrap();
    assert_eq!(o.combat.delay_kill_until, Some(now + 9));
    assert_eq!(o.combat.charge_planter, Some(s));
    assert_ne!(
        w.obj(s).unwrap().dest,
        Some(Target::Object(b)),
        "destination cleared (the scatter may set another)"
    );
}

// spec: S-0519 claim 14 and S-0302 claim 16 (T13 a: the building dies at expiry and the planter is credited)
#[test]
fn the_charge_kills_and_credits_the_planter() {
    let mut w = world(|_| {});
    let b = target_building(&mut w);
    let s = sabotage(&mut w, "SABO", 0, b);
    bomb::charge_arrival(&mut w, s);
    run_to(&mut w, 40);
    assert!(w.obj(b).is_none(), "destroyed");
    assert_eq!(w.houses[0].combat.kills, 1);
}

// spec: S-0519 claim 13 row 2 (T13 b: a second arrival with a countdown running plants nothing; the countdown and its planter stay)
#[test]
fn a_second_charge_is_refused() {
    let mut w = world(|_| {});
    let b = target_building(&mut w);
    let first = sabotage(&mut w, "SABO", 0, b);
    bomb::charge_arrival(&mut w, first);
    let until = w.obj(b).unwrap().combat.delay_kill_until;
    let second = sabotage(&mut w, "SABO", 0, b);
    assert!(bomb::charge_arrival(&mut w, second));
    let o = w.obj(b).unwrap();
    assert_eq!(
        (o.combat.delay_kill_until, o.combat.charge_planter),
        (until, Some(first))
    );
    assert_ne!(w.obj(second).unwrap().dest, Some(Target::Object(b)));
}

// spec: S-0519 claim 13 row 1 (T13 c: a building being sold refuses the charge)
#[test]
fn a_building_being_sold_refuses_the_charge() {
    let mut w = world(|_| {});
    let b = target_building(&mut w);
    let (tick, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
    w.obj_mut(b)
        .unwrap()
        .beh
        .force(Behaviour::Selling, tick, stale);
    let s = sabotage(&mut w, "SABO", 0, b);
    assert!(bomb::charge_arrival(&mut w, s));
    assert!(w.obj(b).unwrap().combat.delay_kill_until.is_none());
}

// spec: S-0519 claim 13 (rearm: the planter's rearm countdown is the delay of a shot of its second weapon, rate 30)
#[test]
fn the_planter_rearms_with_the_second_weapon_delay() {
    let mut w = world(|_| {});
    let b = target_building(&mut w);
    let s = sabotage(&mut w, "SABO", 0, b);
    bomb::charge_arrival(&mut w, s);
    let d = w.obj(s).unwrap().combat.rearm_delay;
    assert!((30..=32).contains(&d), "rate 30 plus the draw R(0, 2): {d}");
}

// spec: S-0519 claim 13 and Edge cases (a type without the explosive-charges flag plants nothing; only the veterancy ability gives it with quirks.charge_needs_type_flag off)
#[test]
fn the_veterancy_ability_plants_only_with_the_quirk_off() {
    for needs_flag in [true, false] {
        let mut w = world(|r| {
            r.globals.quirks.charge_needs_type_flag = needs_flag;
            let mut t = r.sim.infantry["PLAIN"].clone();
            t.veteran_abilities = vec!["C4".into()];
            r.sim.infantry.insert("PLAIN".into(), t);
        });
        let b = target_building(&mut w);
        let s = sabotage(&mut w, "PLAIN", 0, b);
        let rz = Rounding::TowardZero;
        w.obj_mut(s).unwrap().combat.experience = F64::from_i64(2, rz).to_f32(rz);
        bomb::charge_arrival(&mut w, s);
        assert_eq!(
            w.obj(b).unwrap().combat.delay_kill_until.is_some(),
            !needs_flag
        );
    }
}

// spec: S-0519 claim 14 (a change of owner does not cancel the countdown)
#[test]
fn an_owner_change_keeps_the_countdown() {
    let mut w = world(|_| {});
    let b = target_building(&mut w);
    let s = sabotage(&mut w, "SABO", 0, b);
    bomb::charge_arrival(&mut w, s);
    ownership::change(&mut w, b, Some(HouseId(0)), Cause::Capture);
    run_to(&mut w, 40);
    assert!(w.obj(b).is_none());
}
