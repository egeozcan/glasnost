//! Claim tests for S-0314 (healing and repair), S-0315 (suppression and
//! panic) and S-0107 claim 16 (how damage raises fear) on synthetic data:
//! invented types and game-wide values, no stock numbers. Objects of one
//! house only, or unarmed, so that nothing fights unless a test wants it.

use std::sync::Arc;

use glasnost_num::{F32, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LocomotionClass, Locomotor, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::data::{repair_count, repair_period};
use glasnost_sim::combat::{fear, heal, weapon};
use glasnost_sim::infantry::{self, Seq};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};

const SIZE: i32 = 24;
const VETERAN: u32 = 0x3F80_0000; // 1.0

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn ones(v: &str) -> Option<Vec<Decimal>> {
    Some((0..11).map(|_| Decimal(v.into())).collect())
}

fn foot(strength: i32) -> UnitType {
    UnitType {
        strength: Some(strength),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        ..Default::default()
    }
}

fn wheel(strength: i32) -> UnitType {
    UnitType {
        strength: Some(strength),
        speed: Some(5),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    }
}

/// The combat ruleset with unarmed types: infantry GRUNT (strength 100),
/// vehicles TRUCK (strength 200) and OXCART (organic), aircraft KITE and
/// building SHED.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut push = |reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push("infantry", "GRUNT");
    push("vehicles", "TRUCK");
    push("vehicles", "OXCART");
    push("aircraft", "KITE");
    push("buildings", "SHED");
    push("buildings", "SHED2");
    let s = &mut r.sim;
    s.infantry.insert("GRUNT".into(), foot(100));
    s.vehicles.insert("TRUCK".into(), wheel(200));
    let mut ox = wheel(200);
    ox.combat.organic = Some(true);
    s.vehicles.insert("OXCART".into(), ox);
    s.aircraft.insert(
        "KITE".into(),
        UnitType {
            strength: Some(150),
            ..Default::default()
        },
    );
    let shed = BuildingType {
        strength: Some(400),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    s.buildings.insert("SHED".into(), shed.clone());
    s.buildings.insert("SHED2".into(), shed);
    r
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 5)
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).unwrap().health
}

fn set_health(w: &mut World, id: ObjectId, h: i32) {
    w.obj_mut(id).unwrap().health = h;
}

/// Runs `n` ticks and returns, per tick, the tick number and the health
/// gained by each object in it.
fn gains(w: &mut World, ids: &[ObjectId], n: i32) -> Vec<(i32, Vec<i32>)> {
    let mut out = Vec::new();
    for _ in 0..n {
        let t = w.tick;
        let before: Vec<i32> = ids.iter().map(|&i| health(w, i)).collect();
        glasnost_sim::tick::step(w);
        let g = ids
            .iter()
            .zip(before)
            .map(|(&i, b)| health(w, i) - b)
            .collect();
        out.push((t, g));
    }
    out
}

// ================================================================ S-0314

/// The rules of H3 part a: TRUCK of cost 900 and strength 600, repair step
/// 10, repair percentage 25 %.
fn repair_rules() -> Ruleset {
    let mut r = rules();
    let t = r.sim.vehicles.get_mut("TRUCK").unwrap();
    t.strength = Some(600);
    t.cost = Some(900);
    r.sim.general.repair_step = Some(10);
    r.sim.general.repair_percent = dec("25%");
    r
}

// spec: S-0314 F2 (q = S ÷ R, c = C ÷ q, step cost = max(1, trunc(RZ₅₃(c × p))): 900, 600, 10, 0.25 give 3)
// spec: S-0314 F2 and Edge cases (our engine: R′ = max(1, R), q = max(1, S ÷ R′): H3 part d, 4000, 500, −5, 0.25 give 2; a strength below the step gives q = 1)
#[test]
fn f2_step_cost_and_its_departure() {
    let rz = Rounding::TowardZero;
    let p = glasnost_sim::rules::nearest_double("0.25");
    assert_eq!(heal::f2_step_cost(900, 600, 10, p, rz), 3);
    assert_eq!(heal::f2_step_cost(4000, 500, -5, p, rz), 2);
    assert_eq!(heal::f2_step_cost(4000, 500, 0, p, rz), 2);
    assert_eq!(heal::f2_step_cost(100, 5, 10, p, rz), 25);
    // The minimum of 1 credit, also for a negative cost.
    assert_eq!(heal::f2_step_cost(10, 600, 10, p, rz), 1);
    assert_eq!(heal::f2_step_cost(-500, 600, 10, p, rz), 1);
}

// spec: S-0314 claim 4 (the step size: the infantry repair step for infantry, the repair step for others, at least 1)
// spec: S-0314 claim 5 (an infantry unit's step costs 0; a vehicle's is F2 on its type's cost field; a building's is F2 on S-0408's adjusted cost)
// spec: S-0408 claim 2 (the adjusted cost of a type without free unit or pad deduction is its cost field)
#[test]
fn claims_4_and_5_step_size_and_cost() {
    let mut r = repair_rules();
    r.sim.general.i_repair_step = Some(0);
    r.sim.buildings.get_mut("SHED").unwrap().cost = Some(900);
    let mut w = game(&r);
    let g = spawn(&mut w, Kind::Infantry, "GRUNT", 0, 0, 0);
    let t = spawn(&mut w, Kind::Vehicle, "TRUCK", 0, 2, 0);
    let b = spawn(&mut w, Kind::Building, "SHED", 0, 4, 0);
    assert_eq!(heal::step_size(&w, g), 1);
    assert_eq!(heal::step_size(&w, t), 10);
    assert_eq!(heal::step_cost(&w, g), Some(0));
    assert_eq!(heal::step_cost(&w, t), Some(3));
    let p = w.rules().combat.general.repair_percent;
    let sb = glasnost_sim::combat::damage::strength(&w, b);
    assert_eq!(
        heal::step_cost(&w, b),
        Some(heal::f2_step_cost(900, sb, 10, p, w.rules().mode))
    );
}

// spec: S-0314 claim 3 (the rows in order: full health is not needed, before the money test; cannot pay changes nothing; otherwise the paying house, the object's owner, pays and health rises by the step size, more needed, or becomes the strength, done)
// spec: S-0314 claim 3 (a step cost of 0 is never refused, even with no money)
#[test]
fn claim_3_repair_step_outcomes() {
    use heal::StepOutcome::*;
    let mut w = game(&repair_rules());
    let t = spawn(&mut w, Kind::Vehicle, "TRUCK", 0, 0, 0);
    // Full health and no money: not needed.
    w.houses[0].credits = 0;
    assert_eq!(heal::repair_step(&mut w, t), Some(NotNeeded));
    // Hurt, 2 credits for a step of 3: nothing changes.
    set_health(&mut w, t, 300);
    w.houses[0].credits = 2;
    assert_eq!(heal::repair_step(&mut w, t), Some(CannotPay));
    assert_eq!((health(&w, t), w.houses[0].credits), (300, 2));
    // 1000 credits: +10 and −3 per step (H3 part a).
    w.houses[0].credits = 1000;
    assert_eq!(heal::repair_step(&mut w, t), Some(MoreNeeded));
    assert_eq!((health(&w, t), w.houses[0].credits), (310, 997));
    // The last step brings the health to the strength exactly.
    set_health(&mut w, t, 595);
    assert_eq!(heal::repair_step(&mut w, t), Some(Done));
    assert_eq!((health(&w, t), w.houses[0].credits), (600, 994));
    // An infantry unit's free step with no money.
    let g = spawn(&mut w, Kind::Infantry, "GRUNT", 0, 2, 0);
    set_health(&mut w, g, 50);
    w.houses[0].credits = 0;
    assert_eq!(heal::repair_step(&mut w, g), Some(MoreNeeded));
    assert_eq!(health(&w, g), 51);
}

// spec: S-0314 F1 (P_b = trunc(RZ₅₃(r × 900)), K = the smallest positive integer with RZ₅₃(r × 900) ≤ K: the double 0.016 and the decimal 0.016 give 14 and 15, 0.1 gives 90 and 91, 0.125 gives 112 and 113)
#[test]
fn f1_periods_and_counts() {
    let rz = Rounding::TowardZero;
    let d = glasnost_sim::rules::nearest_double("0.016");
    assert_eq!((repair_period(d, rz), repair_count(d, rz)), (14, 15));
    for (text, p, k) in [("0.016", 14, 15), ("0.1", 90, 91), ("0.125", 112, 113)] {
        let v = Decimal(text.into()).value(rz);
        assert_eq!(
            (repair_period(v, rz), repair_count(v, rz)),
            (p, k),
            "{text}"
        );
    }
    // F1: a product of 1 or less gives K = 1.
    let small = Decimal("0.001".into()).value(rz);
    assert_eq!(repair_count(small, rz), 1);
}

// spec: S-0314 F1 and Import mapping (RepairRate absent: 0.016, P_b = 14; given: the rate's period)
#[test]
fn f1_building_repair_period_from_rules() {
    let r = rules();
    assert_eq!(game(&r).rules().combat.general.repair_period, 14);
    let mut r = rules();
    r.sim.general.repair_rate = dec("0.125");
    assert_eq!(game(&r).rules().combat.general.repair_period, 112);
}

/// The kinds that self-heal in [`claim_9_self_healing_every_kind`].
fn self_healing_rules() -> Ruleset {
    let mut r = rules();
    r.sim.infantry.get_mut("GRUNT").unwrap().combat.self_healing = Some(true);
    r.sim.vehicles.get_mut("TRUCK").unwrap().combat.self_healing = Some(true);
    r.sim.aircraft.get_mut("KITE").unwrap().combat.self_healing = Some(true);
    r.sim.buildings.get_mut("SHED").unwrap().combat.self_healing = Some(true);
    r
}

// spec: S-0314 claim 9 (a self-healing object gains 1 health at each tick whose number is a multiple of P_b, for every kind, free; F1 with the absent rate: P_b = 14)
#[test]
fn claim_9_self_healing_every_kind() {
    let r = self_healing_rules();
    let mut w = game(&r);
    let ids = [
        spawn(&mut w, Kind::Infantry, "GRUNT", 0, -4, 0),
        spawn(&mut w, Kind::Vehicle, "TRUCK", 0, -2, 0),
        spawn(&mut w, Kind::Aircraft, "KITE", 0, 0, 0),
        spawn(&mut w, Kind::Building, "SHED", 0, 2, 0),
    ];
    for &i in &ids {
        set_health(&mut w, i, 20);
    }
    let credits: Vec<_> = w.houses.iter().map(|h| h.combat.clone()).collect();
    for (t, g) in gains(&mut w, &ids, 60) {
        let e = i32::from(t % 14 == 0);
        assert_eq!(g, vec![e; 4], "tick {t}");
    }
    let after: Vec<_> = w.houses.iter().map(|h| h.combat.clone()).collect();
    assert_eq!(credits, after);
}

// spec: S-0314 claim 9 and F3 (nothing at full health or at 0 health)
#[test]
fn claim_9_not_at_full_or_zero_health() {
    let r = self_healing_rules();
    let mut w = game(&r);
    let a = spawn(&mut w, Kind::Vehicle, "TRUCK", 0, -2, 0);
    let b = spawn(&mut w, Kind::Vehicle, "TRUCK", 0, 2, 0);
    set_health(&mut w, b, 0);
    w.tick = 28;
    heal::turn(&mut w, a);
    heal::turn(&mut w, b);
    assert_eq!(health(&w, a), 200);
    assert_eq!(health(&w, b), 0);
}

// spec: S-0314 claim 9 (the self-heal veterancy ability heals as a self-healing type does, at the rank that has it: S-0311 claim 4)
#[test]
fn claim_9_self_heal_ability() {
    let mut r = rules();
    r.sim.vehicles.get_mut("TRUCK").unwrap().veteran_abilities = vec!["SELF_HEAL".into()];
    let mut w = game(&r);
    let rookie = spawn(&mut w, Kind::Vehicle, "TRUCK", 0, -2, 0);
    let vet = spawn(&mut w, Kind::Vehicle, "TRUCK", 0, 2, 0);
    w.obj_mut(vet).unwrap().combat.experience = F32(VETERAN);
    set_health(&mut w, rookie, 50);
    set_health(&mut w, vet, 50);
    testkit::run(&mut w, 30);
    assert_eq!(health(&w, rookie), 50);
    assert_eq!(health(&w, vet), 53); // ticks 0, 14 and 28
}

/// A ruleset whose SHED gives `inf` and `veh` self-heal counts, with
/// intervals 10 (infantry) and 15 (vehicles) and amounts 3 and 4.
fn house_heal_rules(inf: i32, veh: i32) -> Ruleset {
    let mut r = rules();
    for b in ["SHED", "SHED2"] {
        let s = r.sim.buildings.get_mut(b).unwrap();
        s.infantry_gain_self_heal = Some(inf);
        s.units_gain_self_heal = Some(veh);
    }
    let g = &mut r.sim.general;
    g.self_heal_infantry_frames = Some(10);
    g.self_heal_unit_frames = Some(15);
    g.self_heal_infantry_amount = Some(3);
    g.self_heal_unit_amount = Some(4);
    r
}

// spec: S-0314 claim 10 (house-wide healing: infantry and organic vehicles gain a_i × T_i at multiples of n_i, other vehicles a_v × T_v at multiples of n_v; aircraft and buildings are not healed this way)
#[test]
fn claim_10_house_wide_healing_by_kind() {
    let r = house_heal_rules(2, 1);
    let mut w = game(&r);
    let shed = spawn(&mut w, Kind::Building, "SHED", 0, 6, 6);
    assert_eq!(w.houses[0].combat.self_heal, [2, 1]);
    let ids = [
        spawn(&mut w, Kind::Infantry, "GRUNT", 0, -4, 0),
        spawn(&mut w, Kind::Vehicle, "TRUCK", 0, -2, 0),
        spawn(&mut w, Kind::Vehicle, "OXCART", 0, 0, 0),
        spawn(&mut w, Kind::Aircraft, "KITE", 0, 2, 0),
        shed,
    ];
    for &i in &ids {
        set_health(&mut w, i, 20);
    }
    for (t, g) in gains(&mut w, &ids, 46) {
        let inf = if t % 10 == 0 { 6 } else { 0 };
        let veh = if t % 15 == 0 { 4 } else { 0 };
        assert_eq!(g, vec![inf, veh, inf, 0, 0], "tick {t}");
    }
}

// spec: S-0314 claim 10 (several buildings add up: two double the amount, not the frequency; a house without a source heals nothing; the other house's buildings do not count)
#[test]
fn claim_10_two_buildings_double_the_amount() {
    let r = house_heal_rules(1, 0);
    let mut w = game(&r);
    spawn(&mut w, Kind::Building, "SHED", 0, 6, 6);
    spawn(&mut w, Kind::Building, "SHED2", 0, 8, 6);
    spawn(&mut w, Kind::Building, "SHED", 1, 6, -6);
    let a = spawn(&mut w, Kind::Infantry, "GRUNT", 0, -4, 0);
    let b = spawn(&mut w, Kind::Infantry, "GRUNT", 1, -4, 4);
    set_health(&mut w, a, 20);
    set_health(&mut w, b, 20);
    w.houses[1].combat.self_heal = [0, 0];
    for (t, g) in gains(&mut w, &[a, b], 21) {
        let e = if t % 10 == 0 { 6 } else { 0 };
        assert_eq!(g, vec![e, 0], "tick {t}");
    }
}

// spec: S-0314 claim 10 (an object gains no more than it lacks)
#[test]
fn claim_10_capped_at_strength() {
    let r = house_heal_rules(5, 0);
    let mut w = game(&r);
    spawn(&mut w, Kind::Building, "SHED", 0, 6, 6);
    let a = spawn(&mut w, Kind::Infantry, "GRUNT", 0, -4, 0);
    set_health(&mut w, a, 95);
    testkit::run(&mut w, 1);
    assert_eq!(health(&w, a), 100);
}

// spec: S-0314 F3 (claims 9 and 10 in one tick: min(H + 1 + a × T, S))
#[test]
fn f3_both_gains_add_and_cap() {
    let mut r = house_heal_rules(2, 0);
    r.sim.general.self_heal_infantry_frames = Some(7);
    r.sim.infantry.get_mut("GRUNT").unwrap().combat.self_healing = Some(true);
    let mut w = game(&r);
    spawn(&mut w, Kind::Building, "SHED", 0, 6, 6);
    let a = spawn(&mut w, Kind::Infantry, "GRUNT", 0, -4, 0);
    let b = spawn(&mut w, Kind::Infantry, "GRUNT", 0, -4, 2);
    set_health(&mut w, a, 50);
    set_health(&mut w, b, 95);
    for (t, g) in gains(&mut w, &[a], 15) {
        let e = i32::from(t % 14 == 0) + if t % 7 == 0 { 6 } else { 0 };
        assert_eq!(g, vec![e], "tick {t}");
    }
    // Tick 0 gave b 1 + 6, capped at 100.
    assert_eq!(health(&w, b), 100);
}

/// Totals of house 0's infantry self-heal after each step of the example
/// of S-0314 Edge cases.
fn negative_counts(on: bool) -> Vec<i32> {
    let mut r = rules();
    r.globals.quirks.negative_self_heal_counts = on;
    r.sim
        .buildings
        .get_mut("SHED")
        .unwrap()
        .infantry_gain_self_heal = Some(3);
    r.sim
        .buildings
        .get_mut("SHED2")
        .unwrap()
        .infantry_gain_self_heal = Some(-5);
    let mut w = game(&r);
    let mut out = Vec::new();
    let a = spawn(&mut w, Kind::Building, "SHED", 0, 6, 6);
    out.push(w.houses[0].combat.self_heal[0]);
    let b = spawn(&mut w, Kind::Building, "SHED2", 0, 8, 6);
    out.push(w.houses[0].combat.self_heal[0]);
    glasnost_sim::objects::remove(&mut w, a);
    out.push(w.houses[0].combat.self_heal[0]);
    glasnost_sim::objects::remove(&mut w, b);
    out.push(w.houses[0].combat.self_heal[0]);
    out
}

// spec: S-0314 Edge cases (quirks.negative_self_heal_counts on: a negative count is added without a floor and the total is floored only at removal: 3, −2, 0, 5)
#[test]
fn quirk_negative_self_heal_counts_on() {
    assert_eq!(negative_counts(true), vec![3, -2, 0, 5]);
}

// spec: S-0314 Edge cases (quirks.negative_self_heal_counts off: a negative count is read as 0: 3, 3, 0, 0)
#[test]
fn quirk_negative_self_heal_counts_off() {
    assert_eq!(negative_counts(false), vec![3, 3, 0, 0]);
}

// spec: S-0314 claim 10 (a total that is not above 0 heals nothing: the house of the example above, left with a total of 5 by the quirk, heals; one at −2 does not)
#[test]
fn claim_10_non_positive_total_heals_nothing() {
    let mut r = house_heal_rules(0, 0);
    r.sim
        .buildings
        .get_mut("SHED2")
        .unwrap()
        .infantry_gain_self_heal = Some(-2);
    let mut w = game(&r);
    spawn(&mut w, Kind::Building, "SHED2", 0, 8, 6);
    assert_eq!(w.houses[0].combat.self_heal[0], -2);
    let a = spawn(&mut w, Kind::Infantry, "GRUNT", 0, -4, 0);
    set_health(&mut w, a, 20);
    testkit::run(&mut w, 25);
    assert_eq!(health(&w, a), 20);
}

// spec: S-0314 Edge cases (periods below one tick: a building repair period of 0 is treated as 1 tick, so self-healing happens every tick)
#[test]
fn edge_zero_period_is_one_tick() {
    let mut r = self_healing_rules();
    r.sim.general.repair_rate = dec("0.001");
    let mut w = game(&r);
    assert_eq!(w.rules().combat.general.repair_period, 0);
    let a = spawn(&mut w, Kind::Vehicle, "TRUCK", 0, -2, 0);
    set_health(&mut w, a, 20);
    testkit::run(&mut w, 5);
    assert_eq!(health(&w, a), 25);
}

// spec: S-0314 Edge cases (a negative interval behaves as its absolute value; a negative amount is treated as 0)
#[test]
fn edge_negative_interval_and_amount() {
    let mut r = house_heal_rules(1, 1);
    r.sim.general.self_heal_infantry_frames = Some(-10);
    r.sim.general.self_heal_unit_amount = Some(-4);
    let mut w = game(&r);
    spawn(&mut w, Kind::Building, "SHED", 0, 6, 6);
    let a = spawn(&mut w, Kind::Infantry, "GRUNT", 0, -4, 0);
    let v = spawn(&mut w, Kind::Vehicle, "TRUCK", 0, -2, 0);
    set_health(&mut w, a, 20);
    set_health(&mut w, v, 20);
    for (t, g) in gains(&mut w, &[a, v], 31) {
        let e = if t % 10 == 0 { 3 } else { 0 };
        assert_eq!(g, vec![e, 0], "tick {t}");
    }
}

/// A healing warhead MEND with versus value 0.5 for every class.
fn mend(r: &mut Ruleset) -> u32 {
    r.sim.warheads.insert(
        "MEND".into(),
        WarheadType {
            verses: ones("0.5"),
            ..Default::default()
        },
    );
    r.sim.warheads.keys().position(|k| k == "MEND").unwrap() as u32
}

fn heal_hit(h: u32, amount: i32, distance: i32) -> Hit {
    Hit {
        amount,
        warhead: Some(h),
        distance,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    }
}

// spec: S-0314 claim 1 (a healing hit's negative amount is delivered unchanged, no versus value: −20 with versus 0.5 heals 20, only within 8 leptons of the point, never above strength; H1 part b)
#[test]
fn claim_1_healing_hit() {
    let mut r = rules();
    let h = mend(&mut r);
    let mut w = game(&r);
    let a = spawn(&mut w, Kind::Infantry, "GRUNT", 0, -4, 0);
    set_health(&mut w, a, 50);
    damage::apply(&mut w, a, heal_hit(h, -20, 0));
    assert_eq!(health(&w, a), 70);
    damage::apply(&mut w, a, heal_hit(h, -20, 7));
    assert_eq!(health(&w, a), 90);
    damage::apply(&mut w, a, heal_hit(h, -20, 8));
    assert_eq!(health(&w, a), 90);
    damage::apply(&mut w, a, heal_hit(h, -20, 0));
    assert_eq!(health(&w, a), 100);
}

/// An infantry type HEALER with primary damage `p` and secondary `s`.
fn healer(p: i32, s: i32) -> bool {
    let mut r = rules();
    for (n, d) in [("Minus", p), ("Plus", s)] {
        r.sim.weapons.insert(
            n.into(),
            WeaponType {
                damage: Some(d),
                rof: Some(20),
                range: dec("3"),
                speed: Some(100),
                projectile: Some("Bullet".into()),
                warhead: Some("SA".into()),
                ..Default::default()
            },
        );
    }
    let mut t = foot(100);
    t.primary = Some("Minus".into());
    t.secondary = Some("Plus".into());
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("HEALER".into());
    r.sim.infantry.insert("HEALER".into(), t);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Infantry, "HEALER", 0, 0, 0);
    weapon::healer_for_acquisition(&w, id)
}

// spec: S-0314 claim 2 (the automatic searches use the healer for acquisition: a primary of −3 and a secondary of +3 have a mean of 0 and do not make one; −5 and +3, a mean of −1, do)
#[test]
fn claim_2_healer_for_acquisition() {
    assert!(!healer(-3, 3));
    assert!(healer(-5, 3));
}

// spec: S-0314 claim 11 (healing on resources has no effect: the resource-heal ability heals nothing)
#[test]
fn claim_11_resource_heal_ability_heals_nothing() {
    let mut r = rules();
    let t = r.sim.vehicles.get_mut("TRUCK").unwrap();
    t.veteran_abilities = vec!["TIBERIUM_HEAL".into()];
    t.elite_abilities = vec!["TIBERIUM_HEAL".into()];
    let mut w = game(&r);
    let a = spawn(&mut w, Kind::Vehicle, "TRUCK", 0, 0, 0);
    w.obj_mut(a).unwrap().combat.experience = F32(0x4000_0000);
    set_health(&mut w, a, 30);
    testkit::run(&mut w, 200);
    assert_eq!(health(&w, a), 30);
}

// ================================================================ S-0107 claim 16

/// Infantry types for the fear table: FRAID (fraidycat), BRAVE
/// (fearless), FB (both), VB (fearless only through its veteran list).
fn fear_rules() -> Ruleset {
    let mut r = rules();
    let mut add = |n: &str, f: &dyn Fn(&mut UnitType)| {
        let mut t = foot(100);
        f(&mut t);
        r.registries.get_mut("infantry").unwrap().push(n.into());
        r.sim.infantry.insert(n.into(), t);
    };
    add("FRAID", &|t| t.fraidycat = Some(true));
    add("BRAVE", &|t| t.fearless = Some(true));
    add("FB", &|t| {
        t.fraidycat = Some(true);
        t.fearless = Some(true);
    });
    add("VB", &|t| t.veteran_abilities = vec!["FEARLESS".into()]);
    add("ZERO", &|t| t.strength = Some(0));
    r
}

/// The fear after `fear::damaged` for a unit of `ty` at `health` with fear
/// `before`.
fn fear_after(w: &mut World, ty: &str, health_now: i32, before: i32, known: bool) -> i32 {
    let id = spawn(w, Kind::Infantry, ty, 0, 0, 0);
    if ty == "VB" {
        w.obj_mut(id).unwrap().combat.experience = F32(VETERAN);
    }
    set_health(w, id, health_now);
    w.obj_mut(id).unwrap().infantry.as_mut().unwrap().fear = before;
    fear::damaged(w, id, known);
    let f = w.obj(id).unwrap().infantry.as_ref().unwrap().fear;
    glasnost_sim::objects::remove(w, id);
    w.objects.remove(&id);
    f
}

// spec: S-0107 claim 16 (the table: known attacker below 100: fraidycat 300, not fearless 100, fearless unchanged; 100 or more: min(fear + i, 300) or unchanged when fearless; no known attacker: min(fear + i, 300) or unchanged; fearless by type or ability)
#[test]
fn s0107_claim_16_table() {
    let r = fear_rules();
    let mut w = game(&r);
    let mut f = |ty: &str, before: i32, known: bool| fear_after(&mut w, ty, 90, before, known);
    // Row 1, both fearless settings of a fraidycat.
    assert_eq!(f("FRAID", 40, true), 300);
    assert_eq!(f("FB", 40, true), 300);
    // Row 2.
    assert_eq!(f("GRUNT", 40, true), 100);
    // Row 3, by type and by ability.
    assert_eq!(f("BRAVE", 40, true), 40);
    assert_eq!(f("VB", 40, true), 40);
    // Row 4 (health 90 of 100 lies above both default ratios: i = 12).
    assert_eq!(f("GRUNT", 150, true), 162);
    assert_eq!(f("FRAID", 150, true), 162);
    assert_eq!(f("GRUNT", 295, true), 300);
    // Row 5.
    assert_eq!(f("BRAVE", 150, true), 150);
    assert_eq!(f("FB", 150, true), 150);
    // Row 6.
    assert_eq!(f("GRUNT", 0, false), 12);
    assert_eq!(f("FRAID", 40, false), 52);
    // Row 7.
    assert_eq!(f("BRAVE", 40, false), 40);
    assert_eq!(f("VB", 40, false), 40);
}

// spec: S-0107 claim 16 and Edge cases (quirks.fraidycat_fear_ignores_fearless off: a fearless fraidycat keeps its fear level unchanged)
#[test]
fn quirk_fraidycat_fear_ignores_fearless_off() {
    let mut r = fear_rules();
    r.globals.quirks.fraidycat_fear_ignores_fearless = false;
    let mut w = game(&r);
    assert_eq!(fear_after(&mut w, "FB", 90, 40, true), 40);
    assert_eq!(fear_after(&mut w, "FRAID", 90, 40, true), 300);
}

// spec: S-0107 F5 (i₀ = 50 at or below red, 25 above; halved above yellow: with red 0.25 and yellow 0.5, i is 12 above both, 25 between them and 50 at or below both; with the ratios swapped, 25 between them too; the health ratio is taken after the damage)
#[test]
fn s0107_f5_increments() {
    let mut r = fear_rules();
    r.sim.general.condition_red = dec("0.25");
    r.sim.general.condition_yellow = dec("0.5");
    let mut w = game(&r);
    assert_eq!(fear_after(&mut w, "GRUNT", 75, 0, false), 12);
    assert_eq!(fear_after(&mut w, "GRUNT", 50, 0, false), 25);
    assert_eq!(fear_after(&mut w, "GRUNT", 26, 0, false), 25);
    assert_eq!(fear_after(&mut w, "GRUNT", 25, 0, false), 50);
    let mut r = fear_rules();
    r.sim.general.condition_red = dec("0.5");
    r.sim.general.condition_yellow = dec("0.25");
    let mut w = game(&r);
    assert_eq!(fear_after(&mut w, "GRUNT", 40, 0, false), 25);
    assert_eq!(fear_after(&mut w, "GRUNT", 75, 0, false), 12);
    assert_eq!(fear_after(&mut w, "GRUNT", 20, 0, false), 50);
    // Both at the absent-key default 0.5: never 25.
    let r = fear_rules();
    let mut w = game(&r);
    assert_eq!(fear_after(&mut w, "GRUNT", 50, 0, false), 50);
    assert_eq!(fear_after(&mut w, "GRUNT", 51, 0, false), 12);
}

// spec: S-0107 F5 (strength 0: taken to lie above both ratios, i = 12)
#[test]
fn s0107_f5_strength_zero() {
    let r = fear_rules();
    let mut w = game(&r);
    assert_eq!(fear_after(&mut w, "ZERO", 0, 0, false), 12);
}

// spec: S-0107 claim 16 (through damage resolution: a surviving unit hit with a known attacker below 100 gets 100; without an attacker it rises by i; a healing hit raises nothing)
#[test]
fn s0107_claim_16_through_hits() {
    let mut r = fear_rules();
    let h = mend(&mut r);
    let mut w = game(&r);
    let attacker = spawn(&mut w, Kind::Vehicle, "TRUCK", 1, 6, 0);
    let a = spawn(&mut w, Kind::Infantry, "GRUNT", 0, -4, 0);
    let fear_of = |w: &World| w.obj(a).unwrap().infantry.as_ref().unwrap().fear;
    let sa = r.sim.warheads.keys().position(|k| k == "SA").unwrap() as u32;
    let hit = |attacker| Hit {
        amount: 5,
        warhead: Some(sa),
        distance: 0,
        attacker,
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    damage::apply(&mut w, a, hit(None));
    assert_eq!(fear_of(&w), 12);
    damage::apply(&mut w, a, hit(Some(attacker)));
    assert_eq!(fear_of(&w), 100);
    damage::apply(&mut w, a, heal_hit(h, -5, 0));
    assert_eq!(fear_of(&w), 100);
}

// ================================================================ S-0315

// spec: S-0315 claim 1 (fear is 0 at creation and changes only by its triggers: an explosion that does no damage to the unit leaves it at 0; vehicles have no fear)
#[test]
fn s0315_claim_1_no_fear_without_damage() {
    let r = fear_rules();
    let mut w = game(&r);
    let a = spawn(&mut w, Kind::Infantry, "GRUNT", 0, -4, 0);
    let v = spawn(&mut w, Kind::Vehicle, "TRUCK", 0, 4, 0);
    assert_eq!(w.obj(a).unwrap().infantry.as_ref().unwrap().fear, 0);
    assert!(w.obj(v).unwrap().infantry.is_none());
    let sa = r.sim.warheads.keys().position(|k| k == "SA").unwrap() as u32;
    let at = glasnost_sim::world::Pos::centre(testkit::mid(SIZE, -1, 0), 0);
    glasnost_sim::combat::projectile::detonate_area(&mut w, sa, 50, at, None, None, false);
    testkit::run(&mut w, 5);
    assert_eq!(health(&w, a), 100);
    assert_eq!(w.obj(a).unwrap().infantry.as_ref().unwrap().fear, 0);
}

// spec: S-0315 claim 1 (every tick a fear level above 0 falls by 1, except for a fearless type; the fearless ability does not stop the decay)
#[test]
fn s0315_claim_1_decay() {
    let r = fear_rules();
    let mut w = game(&r);
    let ids: Vec<ObjectId> = ["GRUNT", "BRAVE", "VB"]
        .iter()
        .enumerate()
        .map(|(i, ty)| spawn(&mut w, Kind::Infantry, ty, 0, 3 * i as i32 - 3, 0))
        .collect();
    w.obj_mut(ids[2]).unwrap().combat.experience = F32(VETERAN);
    for &id in &ids {
        w.obj_mut(id).unwrap().infantry.as_mut().unwrap().fear = 200;
    }
    testkit::run(&mut w, 3);
    let fears: Vec<i32> = ids
        .iter()
        .map(|&id| w.obj(id).unwrap().infantry.as_ref().unwrap().fear)
        .collect();
    assert_eq!(fears, vec![197, 200, 197]);
}

/// A unit of `ty` that has just fired its last round (ammunition 0) in
/// behaviour `b`, with fear 40, after `fear::last_round`.
fn last_round(r: &Ruleset, ty: &str, b: Behaviour, veteran: bool) -> (i32, Option<Behaviour>) {
    let mut w = game(r);
    let id = spawn(&mut w, Kind::Infantry, ty, 0, 0, 0);
    let t = w.tick;
    let o = w.obj_mut(id).unwrap();
    o.beh.force(b, t, true);
    o.beh.queued = None;
    o.combat.ammo = 0;
    o.infantry.as_mut().unwrap().fear = 40;
    if veteran {
        o.combat.experience = F32(VETERAN);
    }
    fear::last_round(&mut w, id);
    let o = w.obj(id).unwrap();
    (o.infantry.as_ref().unwrap().fear, o.beh.queued)
}

fn armed_fear_rules() -> Ruleset {
    let mut r = fear_rules();
    for n in ["FRAID", "FB", "VB", "GRUNT"] {
        let t = r.sim.infantry.get_mut(n).unwrap();
        t.primary = Some("Rifle".into());
        t.combat.ammo = Some(2);
        t.combat.initial_ammo = Some(2);
    }
    r.sim.infantry.get_mut("VB").unwrap().fraidycat = Some(true);
    r
}

// spec: S-0315 claim 2 (a fraidycat left with no ammunition: fear 300; from attack or hunt, guard is queued; from any other behaviour, guard included, nothing is queued; not a fraidycat: nothing)
#[test]
fn s0315_claim_2_panic_and_queue() {
    let r = armed_fear_rules();
    assert_eq!(
        last_round(&r, "FRAID", Behaviour::Attack, false),
        (300, Some(Behaviour::Guard))
    );
    assert_eq!(
        last_round(&r, "FRAID", Behaviour::Hunt, false),
        (300, Some(Behaviour::Guard))
    );
    assert_eq!(
        last_round(&r, "FRAID", Behaviour::Guard, false),
        (300, None)
    );
    assert_eq!(last_round(&r, "FRAID", Behaviour::Move, false), (300, None));
    assert_eq!(
        last_round(&r, "GRUNT", Behaviour::Attack, false),
        (40, None)
    );
}

// spec: S-0315 claim 2 and Edge cases (quirks.fraidycat_ammo_panic_ignores_fearless on: the fearless flag and ability do not prevent the panic)
#[test]
fn quirk_fraidycat_ammo_panic_ignores_fearless_on() {
    let r = armed_fear_rules();
    assert_eq!(
        last_round(&r, "FB", Behaviour::Attack, false),
        (300, Some(Behaviour::Guard))
    );
    assert_eq!(
        last_round(&r, "VB", Behaviour::Attack, true),
        (300, Some(Behaviour::Guard))
    );
}

// spec: S-0315 Edge cases (quirks.fraidycat_ammo_panic_ignores_fearless off: nothing for a unit fearless by type or ability; every other unit as before)
#[test]
fn quirk_fraidycat_ammo_panic_ignores_fearless_off() {
    let mut r = armed_fear_rules();
    r.globals.quirks.fraidycat_ammo_panic_ignores_fearless = false;
    assert_eq!(last_round(&r, "FB", Behaviour::Attack, false), (40, None));
    assert_eq!(last_round(&r, "VB", Behaviour::Attack, true), (40, None));
    assert_eq!(
        last_round(&r, "FRAID", Behaviour::Attack, false),
        (300, Some(Behaviour::Guard))
    );
}

// spec: S-0315 claim 2 (with real shots: the shot that empties the ammunition sets fear 300 at once; earlier shots do not; a fearless fraidycat then keeps 300 since its fear never decays)
#[test]
fn s0315_claim_2_with_real_shots() {
    for ty in ["FRAID", "FB"] {
        let mut r = armed_fear_rules();
        r.sim.vehicles.get_mut("TRUCK").unwrap().strength = Some(5000);
        let mut w = game(&r);
        let a = spawn(&mut w, Kind::Infantry, ty, 0, 0, 0);
        spawn(&mut w, Kind::Vehicle, "TRUCK", 1, 2, 0);
        let mut emptied = None;
        for _ in 0..600 {
            testkit::run(&mut w, 1);
            let o = w.obj(a).unwrap();
            let f = o.infantry.as_ref().unwrap().fear;
            if o.combat.ammo == 1 {
                assert!(f < 300, "{ty}: the first shot does not panic");
            }
            if o.combat.ammo == 0 {
                emptied = Some(f);
                break;
            }
        }
        assert_eq!(emptied, Some(300), "{ty}");
        testkit::run(&mut w, 5);
        let f = w.obj(a).unwrap().infantry.as_ref().unwrap().fear;
        assert_eq!(f, if ty == "FB" { 300 } else { 295 }, "{ty}");
    }
}

// spec: S-0315 claim 2 (a unit whose type has no ammunition limit never triggers the panic)
#[test]
fn s0315_claim_2_unlimited_ammunition() {
    let r = fear_rules();
    let mut w = game(&r);
    // FRAID without the ammunition fields: no limit (S-0301, ammunition −1).
    let id = spawn(&mut w, Kind::Infantry, "FRAID", 0, 0, 0);
    assert_eq!(w.obj(id).unwrap().combat.ammo, -1);
    let t = w.tick;
    w.obj_mut(id).unwrap().beh.force(Behaviour::Attack, t, true);
    fear::last_round(&mut w, id);
    assert_eq!(w.obj(id).unwrap().infantry.as_ref().unwrap().fear, 0);
}

// spec: S-0315 claim 3 (a prone unit's damage from a positive hit is changed by the warhead's prone factor, at least 1; S-0302 claim 5)
#[test]
fn s0315_claim_3_prone_factor() {
    for (factor, expected) in [("0.5", 20), ("0", 1)] {
        let mut r = fear_rules();
        r.sim.warheads.insert(
            "PR".into(),
            WarheadType {
                verses: ones("1"),
                prone_damage: dec(factor),
                ..Default::default()
            },
        );
        let pr = r.sim.warheads.keys().position(|k| k == "PR").unwrap() as u32;
        let mut w = game(&r);
        let a = spawn(&mut w, Kind::Infantry, "GRUNT", 0, 0, 0);
        let b = spawn(&mut w, Kind::Infantry, "GRUNT", 0, 2, 0);
        w.obj_mut(a).unwrap().infantry.as_mut().unwrap().prone = true;
        let hit = heal_hit(pr, 40, 0);
        damage::apply(&mut w, a, hit);
        damage::apply(&mut w, b, hit);
        assert_eq!(health(&w, a), 100 - expected, "{factor}");
        assert_eq!(health(&w, b), 60, "{factor}");
    }
}

// spec: S-0315 claim 4 (a panicking unit, fear above 199, walks with the panic sequence; at 199 it walks normally; S-0107 claim 7)
#[test]
fn s0315_claim_4_panic_sequence() {
    // The walk entry needs image frames for the request to go ahead (S-0107 claim 6 row 1).
    let mut r2 = fear_rules();
    r2.sim.infantry.get_mut("GRUNT").unwrap().sequences.insert(
        "walk".into(),
        glasnost_rules::sim::SequenceEntry {
            start: 0,
            count: 4,
            multiplier: 0,
            direction: None,
        },
    );
    let mut w = glasnost_sim::load::new_game(
        Arc::new(SimRules::new(&r2, "s0315")),
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &testkit::setup(5),
    )
    .unwrap();
    let a = spawn(&mut w, Kind::Infantry, "GRUNT", 0, 0, 0);
    let b = spawn(&mut w, Kind::Infantry, "GRUNT", 0, 2, 0);
    w.obj_mut(a).unwrap().infantry.as_mut().unwrap().fear = 200;
    w.obj_mut(b).unwrap().infantry.as_mut().unwrap().fear = 199;
    infantry::request(&mut w, a, Seq::Walk, true);
    infantry::request(&mut w, b, Seq::Walk, true);
    let seq = |w: &World, id: ObjectId| w.obj(id).unwrap().infantry.as_ref().unwrap().seq;
    assert_eq!(seq(&w, a), Some(Seq::Panic));
    assert_eq!(seq(&w, b), Some(Seq::Walk));
}

/// A frightened walking unit of a type that can crawl, after one sequence
/// update while it moves; `human`: its owner is human-controlled.
fn frightened_walker(human: bool) -> Option<Seq> {
    let mut r = fear_rules();
    let t = r.sim.infantry.get_mut("GRUNT").unwrap();
    for k in ["stand", "walk", "go_prone", "prone", "crawl"] {
        t.sequences.insert(
            k.into(),
            glasnost_rules::sim::SequenceEntry {
                start: 0,
                count: 4,
                multiplier: 0,
                direction: None,
            },
        );
    }
    let mut setup = testkit::setup(5);
    setup.houses[0].human = human;
    let mut w = glasnost_sim::load::new_game(
        Arc::new(SimRules::new(&r, "s0315")),
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &setup,
    )
    .unwrap();
    let id = spawn(&mut w, Kind::Infantry, "GRUNT", 0, 0, 0);
    let st = w.obj_mut(id).unwrap().infantry.as_mut().unwrap();
    st.seq = Some(Seq::Walk);
    st.fear = 60;
    infantry::update(&mut w, id, true, true, true);
    w.obj(id).unwrap().infantry.as_ref().unwrap().seq
}

// spec: S-0315 claim 5 (an AI house, counted as human-controlled, does not lie down while moving; the original's computer-controlled rule, kept for the neutral and special houses, lies down in the middle of a move)
#[test]
fn s0315_claim_5_no_lying_down_while_moving() {
    assert_eq!(frightened_walker(true), Some(Seq::Walk));
    assert_eq!(frightened_walker(false), Some(Seq::GoProne));
}

/// A prone, crawling WALKER of house 0 at cell (0, 0), tick 10.
fn prone_walker(
    fraidycat: bool,
    cyborg: bool,
    quirk: bool,
) -> (glasnost_sim::world::World, ObjectId) {
    let mut r = testkit::combat_ruleset();
    r.globals.quirks.prone_get_up_repeated_destination = quirk;
    let wk = r.sim.infantry.get_mut("WALKER").unwrap();
    wk.fraidycat = Some(fraidycat);
    wk.cyborg = Some(cyborg);
    wk.crawls = Some(true);
    // Image frames for the kinds involved (S-0107 claim 6 row 1).
    for k in ["stand", "walk", "go_prone", "get_up", "prone", "crawl"] {
        wk.sequences.insert(
            k.into(),
            glasnost_rules::sim::SequenceEntry {
                start: 0,
                count: 4,
                multiplier: 0,
                direction: None,
            },
        );
    }
    let mut w = testkit::game_with(&r, 32, 3);
    let id = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(32, 0, 0),
        0,
    );
    w.obj_mut(id).unwrap().infantry.as_mut().unwrap().prone = true;
    w.tick = 10;
    (w, id)
}

/// Whether the prone unit has asked for get-up (or already stood up).
fn asked_get_up(w: &glasnost_sim::world::World, id: ObjectId) -> bool {
    let s = w.obj(id).unwrap().infantry.clone().unwrap();
    s.seq == Some(glasnost_sim::infantry::Seq::GetUp) || !s.prone
}

/// Whether a prone WALKER asks for get-up when a plain move order to cell
/// (4, 0) is applied, with `before` its destination beforehand.
fn prone_move_asks(fraidycat: bool, cyborg: bool, before: Option<(i32, i32)>, quirk: bool) -> bool {
    use glasnost_sim::orders::{self, Order};
    use glasnost_sim::world::{HouseId, Target};
    let (mut w, id) = prone_walker(fraidycat, cyborg, quirk);
    w.obj_mut(id).unwrap().dest = before.map(|(x, y)| Target::Cell(testkit::mid(32, x, y)));
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(testkit::mid(32, 4, 0))),
        },
    ));
    testkit::run(&mut w, 1);
    asked_get_up(&w, id)
}

// spec: S-0107 claim 15 (a prone unit of a human-controlled owner asks, not forced, for get-up only when its current non-empty destination is assigned again, unless a fraidycat or a cyborg)
// spec: S-0213 claim 21 (a plain move order's destination assignment governs the get-up; a first or different destination makes no request)
// spec: S-0315 claim 3 (the get-up of a prone unit on a repeated destination)
#[test]
fn prone_unit_gets_up_on_a_repeated_destination() {
    for (fraidycat, cyborg, want_up) in [
        (false, false, true),
        (true, false, false),
        (false, true, false),
    ] {
        assert_eq!(
            prone_move_asks(fraidycat, cyborg, Some((4, 0)), true),
            want_up,
            "fraidycat {fraidycat} cyborg {cyborg}"
        );
    }
    assert!(
        !prone_move_asks(false, false, None, true),
        "first destination"
    );
    assert!(
        !prone_move_asks(false, false, Some((5, 0)), true),
        "different destination"
    );
}

// spec: S-0107 claim 15 (quirks.prone_get_up_repeated_destination off: any non-empty destination assignment asks)
#[test]
fn prone_get_up_repeated_destination_off() {
    assert!(prone_move_asks(false, false, None, false));
    assert!(prone_move_asks(false, false, Some((5, 0)), false));
    assert!(!prone_move_asks(true, false, None, false));
}

// spec: S-0107 claim 15 (every destination assignment counts, not only orders: a first queued destination taken by the move fall-back is a first destination, which asks only with quirks.prone_get_up_repeated_destination off)
// spec: S-0215 claim 14 row 3 (the first queued destination becomes the destination)
#[test]
fn prone_get_up_on_a_queued_destination() {
    use glasnost_sim::world::Target;
    for quirk in [true, false] {
        let (mut w, id) = prone_walker(false, false, quirk);
        let t = w.tick;
        let o = w.obj_mut(id).unwrap();
        o.queued_dests = vec![Target::Cell(testkit::mid(32, 4, 0))];
        o.beh.force(Behaviour::Move, t, true);
        testkit::run(&mut w, 1);
        assert!(
            w.obj(id).unwrap().dest.is_some(),
            "the queued destination was taken"
        );
        assert_eq!(asked_get_up(&w, id), !quirk, "quirk {quirk}");
    }
}
