//! Claim tests of S-0515 (tech buildings) on synthetic rules: invented
//! building types and numbers, no game data. Houses Alpha (0), Beta (1),
//! and two passive civilian-side houses Civ (2) and Civ2 (3).

mod structures_common;

use std::sync::Arc;

use glasnost_map::triggers::{Action, Event, Triggers};
use glasnost_rules::quirks::Quirks;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::combat::check::Answer;
use glasnost_sim::econ::tech::Answer as TechAnswer;
use glasnost_sim::econ::{power, tech};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::structures::tech as cash;
use glasnost_sim::superweapons::charge;
use glasnost_sim::superweapons::warp::{Member, Stage};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, ObjectId, World};
use structures_common::*;

const CIV2: HouseId = HouseId(3);

/// Test cash types: (name, B, a, D, needs-engineer).
const TYPES: [(&str, i32, i32, i32, bool); 14] = [
    ("D11", 0, 7, 11, true),
    ("D2", 0, 7, 2, true),
    ("D3", 0, 7, 3, true),
    ("D51", 100, 10, 51, true),
    ("D1", 0, 7, 1, true),
    ("D0", 0, 7, 0, true),
    ("DM3", 0, 7, -3, true),
    ("PLC", 0, 10, 51, false),
    ("NEGB", -100, 100, 51, true),
    ("NEGA", 0, -5, 51, true),
    ("ZEROA", -100, 0, 51, true),
    ("MINA", 0, i32::MIN, 51, true),
    ("BIGA", 0, i32::MAX, 51, true),
    ("BIGB", i32::MAX, 0, 0, true),
];

fn rules_set() -> Ruleset {
    let mut r = ruleset();
    for (n, b, a, d, eng) in TYPES {
        r.registries
            .get_mut("buildings")
            .unwrap()
            .push(n.to_string());
        let x = BuildingType {
            strength: Some(400),
            foundation: Some([1, 1]),
            capturable: Some(true),
            needs_engineer: Some(eng),
            produce_cash_startup: Some(b),
            produce_cash_amount: Some(a),
            produce_cash_delay: Some(d),
            ..Default::default()
        };
        r.sim.buildings.insert(n.into(), x);
    }
    // A tech building of the outpost kind, one of the airport kind and a secret lab.
    for n in ["OUTP", "AIRP", "SLAB"] {
        r.registries
            .get_mut("buildings")
            .unwrap()
            .push(n.to_string());
        let mut x = BuildingType {
            strength: Some(400),
            foundation: Some([1, 1]),
            capturable: Some(true),
            needs_engineer: Some(true),
            ..Default::default()
        };
        if n == "AIRP" {
            x.econ.super_weapon = Some("CURT".into());
        }
        if n == "SLAB" {
            x.econ.secret_lab = Some(true);
        }
        r.sim.buildings.insert(n.into(), x);
    }
    r.sim.general.economy.secret_infantry =
        Some(vec!["WALKER".into(), "GARRI".into(), "ENGI".into()]);
    r
}

/// A game of Alpha, Beta, Civ and Civ2 (both passive) on an empty map.
fn world_with(r: &Ruleset, t: Triggers) -> World {
    let mut map = testkit::empty_map(SIZE);
    map.triggers = t;
    let mut s = testkit::setup(7);
    for n in ["Civ", "Civ2"] {
        s.houses.push(HouseSetup {
            name: n.into(),
            country: "Civland".into(),
            human: false,
            start: None,
            role: HouseRole::Neutral,
            ai: false,
        });
    }
    let rules = Arc::new(SimRules::new(r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(rules, &map, testkit::catalogue(), &s)
        .expect("synthetic test game loads");
    testkit::run(&mut w, 4);
    for h in &mut w.houses {
        h.credits = 0;
    }
    w
}

fn world() -> World {
    world_with(&rules_set(), Triggers::default())
}

fn quirk(w: &mut World, f: impl FnOnce(&mut Quirks)) {
    let mut rules = (*w.rules_arc()).clone();
    f(&mut rules.quirks);
    w.rules = Some(Arc::new(rules));
}

fn credits(w: &World, h: HouseId) -> i32 {
    w.houses[usize::from(h.0)].credits
}

/// The building put on the map for `house` at a cell of its own.
fn derrick(w: &mut World, name: &str, house: u8, slot: i32) -> ObjectId {
    put(w, Kind::Building, name, house, slot * 2 - 8, 0)
}

fn change(w: &mut World, b: ObjectId, h: HouseId) {
    ownership::change(w, b, Some(h), Cause::Trigger);
}

/// Runs `n` ticks one by one; the ticks (the number of the tick run) in
/// which `h`'s balance changed, with the change.
fn payments(w: &mut World, h: HouseId, n: i32) -> Vec<(i32, i32)> {
    let mut v = Vec::new();
    for _ in 0..n {
        let before = credits(w, h);
        let t = w.tick;
        testkit::run(w, 1);
        let d = credits(w, h).wrapping_sub(before);
        if d != 0 {
            v.push((t, d));
        }
    }
    v
}

fn ticks(v: &[(i32, i32)]) -> Vec<i32> {
    v.iter().map(|p| p.0).collect()
}

// spec: S-0515 claim 5 and TB1 (the period is D − 1 ticks: 100 payments in 1000 ticks for D = 11, 1000 for D = 2, 500 for D = 3; the first payment D − 1 ticks after the restart)
#[test]
fn claim_5_period_is_d_minus_one() {
    for (name, per) in [("D11", 10), ("D2", 1), ("D3", 2)] {
        let mut w = world();
        let b = derrick(&mut w, name, 2, 0);
        let t0 = w.tick;
        change(&mut w, b, A);
        let p = payments(&mut w, A, 1001);
        assert_eq!(p.len() as i32, 1000 / per, "{name}");
        assert!(p.iter().all(|x| x.1 == 7), "{name}");
        assert_eq!(
            p[0].0,
            t0 + per,
            "{name}: first cycle D - 1 ticks after the restart"
        );
        assert!(p.windows(2).all(|x| x[1].0 - x[0].0 == per), "{name}");
    }
}

// spec: S-0515 claim 5, F1 and quirks.cash_cycle_one_short (off: a cycle when the full D ticks have passed: 90 payments for D = 11, and D = 1 pays every tick)
#[test]
fn claim_5_cycle_one_short_off() {
    let mut w = world();
    quirk(&mut w, |q| q.cash_cycle_one_short = false);
    let b = derrick(&mut w, "D11", 2, 0);
    let t0 = w.tick;
    change(&mut w, b, A);
    let p = payments(&mut w, A, 1001);
    assert_eq!(p.len(), 90);
    assert_eq!(p[0].0, t0 + 11);
    let mut w = world();
    quirk(&mut w, |q| q.cash_cycle_one_short = false);
    let b = derrick(&mut w, "D1", 2, 0);
    change(&mut w, b, A);
    let p = payments(&mut w, A, 10);
    assert_eq!(p.len(), 9, "D = 1 pays every tick after the restart tick");
}

// spec: S-0515 claim 5 and TB9 (D = 1: at most one cycle per restart, in the restart's own tick when the building's turn comes after the restart, otherwise none; D = 0 and D = −3: no cycle at all)
#[test]
fn claim_5_delay_boundaries() {
    let mut w = world();
    let b = derrick(&mut w, "D1", 2, 0);
    let t0 = w.tick;
    change(&mut w, b, A);
    let p = payments(&mut w, A, 100);
    assert_eq!(
        p,
        vec![(t0, 7)],
        "exactly one payment, in the transfer tick"
    );
    // A restart that came after the building's turn of its tick: none.
    let mut w = world();
    let b = derrick(&mut w, "D1", 2, 0);
    change(&mut w, b, A);
    let tick = w.tick;
    w.obj_mut(b).unwrap().econ.building.income_restart = Some(tick - 1);
    assert!(payments(&mut w, A, 100).is_empty());
    for name in ["D0", "DM3"] {
        let mut w = world();
        let b = derrick(&mut w, name, 2, 0);
        change(&mut w, b, A);
        assert!(payments(&mut w, A, 100).is_empty(), "{name}");
        assert_eq!(cash::remaining(&w, b), 0);
    }
}

// spec: S-0515 claim 4 (a) and TB9 (a building put on the map restarts its countdown: the first cycle D − 1 ticks after placement, then every D − 1; an unmanned type without the needs-engineer flag pays at once from there)
#[test]
fn claim_4_placement_restarts_the_countdown() {
    let mut w = world();
    let p = w.tick;
    derrick(&mut w, "PLC", 0, 0);
    let v = payments(&mut w, A, 200);
    assert_eq!(ticks(&v), vec![p + 50, p + 100, p + 150]);
    assert!(v.iter().all(|x| x.1 == 10));
}

// spec: S-0515 claim 4 (a) (a countdown that was not restarted has run out and has no cycle; the restart is made once, at the first entry to the map)
#[test]
fn claim_4_only_the_first_placement_restarts() {
    let mut w = world();
    let b = derrick(&mut w, "PLC", 0, 0);
    let first = w.obj(b).unwrap().econ.building.income_restart;
    assert_eq!(first, Some(w.tick));
    testkit::run(&mut w, 3);
    glasnost_sim::structures::tech::entered_map(&mut w, b);
    assert_eq!(w.obj(b).unwrap().econ.building.income_restart, first);
    // An unrestarted countdown reads 0 and pays nothing.
    w.obj_mut(b).unwrap().econ.building.income_restart = None;
    assert_eq!(cash::remaining(&w, b), 0);
    assert!(payments(&mut w, A, 120).is_empty());
}

// spec: S-0515 claims 4 (c) and 5 (a cycle without payment still restarts the countdown, so payments missed for want of operation are not made up and later cycles do not shift) and TB3 (a, b)
#[test]
fn claim_5_missed_payments_are_not_made_up() {
    let mut w = world();
    let b = derrick(&mut w, "PLC", 0, 0);
    let p = w.tick;
    // Switched off from p + 60 to p + 110: the cycles at p + 100 is missed.
    testkit::run(&mut w, 60);
    w.obj_mut(b).unwrap().econ.building.switched_off = true;
    let mid = payments(&mut w, A, 50);
    w.obj_mut(b).unwrap().econ.building.switched_off = false;
    let rest = payments(&mut w, A, 60);
    let _ = mid;
    assert_eq!(credits(&w, A), 10 + 10, "the cycles at p+50 and p+150 only");
    assert_eq!(ticks(&rest), vec![p + 150]);
}

// spec: S-0515 claim 5 (the payment needs the owner to be a non-passive house; a derrick held by a passive house keeps cycling and pays nobody) and claim 7
#[test]
fn claim_7_passive_owner_keeps_cycling_and_pays_nobody() {
    let mut w = world();
    let b = derrick(&mut w, "D51", 2, 0);
    let before: Vec<i32> = (0..4).map(|i| credits(&w, HouseId(i))).collect();
    testkit::run(&mut w, 200);
    let after: Vec<i32> = (0..4).map(|i| credits(&w, HouseId(i))).collect();
    assert_eq!(before, after);
    assert!(
        w.obj(b).unwrap().econ.building.income_restart.unwrap() > 100,
        "the countdown restarts at its cycles under a passive owner"
    );
}

// spec: S-0515 claims 3, 4 and 7 and TB2 (the bonus once, no payment to the old owner after the change, no bonus when it returns from a house that is not passive, the countdown restarted at the return)
#[test]
fn claim_7_bonus_phase_and_recipients() {
    let mut w = world();
    let b = derrick(&mut w, "D51", 2, 0);
    let t0 = w.tick;
    change(&mut w, b, A);
    assert_eq!(
        credits(&w, A),
        100,
        "the bonus at once, in the tick of the change"
    );
    // Alpha's payments at t0 + 50k; m = 2 payments, then a change to Beta 20 ticks after the second.
    let v = payments(&mut w, A, 50 * 2 + 20);
    assert_eq!(ticks(&v), vec![t0 + 50, t0 + 100]);
    let t1 = w.tick;
    assert_eq!(t1, t0 + 120);
    change(&mut w, b, B);
    assert_eq!(
        credits(&w, B),
        0,
        "from a house that is not passive: no bonus"
    );
    let vb = payments(&mut w, B, 49);
    assert!(
        vb.is_empty(),
        "Beta's first payment is due D - 1 ticks after the change"
    );
    // Back to Alpha 200 ticks after the first change to Beta.
    let _ = payments(&mut w, B, 200 - 49);
    let t2 = w.tick;
    let a_before = credits(&w, A);
    change(&mut w, b, A);
    assert_eq!(credits(&w, A), a_before, "no bonus on the return from Beta");
    let v = payments(&mut w, A, 45);
    assert!(
        v.is_empty(),
        "no payment in the 45 ticks after the return: the phase restarted"
    );
    let v = payments(&mut w, A, 10);
    assert_eq!(ticks(&v), vec![t2 + 50]);
    assert_eq!(credits(&w, A), a_before + 10);
}

// spec: S-0515 claim 3 and TB2 (b) to (f) (the bonus again from a passive house; from a passive house to a passive one, to the new owner; whatever the cause; none for the same owner and no restart; no factors; B = 0 pays nothing)
#[test]
fn claim_3_when_the_bonus_is_paid() {
    // (b) The neutral house takes it back and a second capture pays again.
    let mut w = world();
    let b = derrick(&mut w, "D51", 2, 0);
    change(&mut w, b, A);
    change(&mut w, b, HouseId(2));
    assert_eq!(credits(&w, A), 100);
    change(&mut w, b, A);
    assert_eq!(
        credits(&w, A),
        200,
        "a building that returns to a passive house pays again"
    );
    // (d) Passive to passive: the new owner gains B.
    let mut w = world();
    let b = derrick(&mut w, "D51", 2, 0);
    change(&mut w, b, CIV2);
    assert_eq!((credits(&w, CIV2), credits(&w, HouseId(2))), (100, 0));
    // (f) The same owner: not an ownership change, no bonus, no restart.
    let mut w = world();
    let b = derrick(&mut w, "D51", 2, 0);
    change(&mut w, b, A);
    testkit::run(&mut w, 20);
    let mark = w.obj(b).unwrap().econ.building.income_restart;
    change(&mut w, b, A);
    assert_eq!(credits(&w, A), 100);
    assert_eq!(w.obj(b).unwrap().econ.building.income_restart, mark);
    // A type with B = 0 pays nothing; an unowned old owner (none) pays nothing.
    let mut w = world();
    let b = derrick(&mut w, "D11", 2, 0);
    change(&mut w, b, A);
    assert_eq!(credits(&w, A), 0);
    let mut w = world();
    let b = derrick(&mut w, "D51", 0, 0);
    ownership::change(&mut w, b, None, Cause::Trigger);
    change(&mut w, b, A);
    assert_eq!(credits(&w, A), 0, "the old owner was no house");
}

// spec: S-0515 claim 3 (the bonus does not depend on the cause: an engineer capture pays it as a trigger does) and S-0509 claim 8
#[test]
fn claim_3_an_engineer_capture_pays_the_bonus() {
    let mut w = world();
    let b = derrick(&mut w, "D51", 2, 0);
    let e = put(&mut w, Kind::Infantry, "ENGI", 0, 6, 6);
    send(&mut w, e, b);
    let t = until(&mut w, 800, |w| w.obj(e).is_none()).expect("captured");
    assert!(t > 0);
    assert_eq!(owner(&w, b), Some(A));
    assert_eq!(credits(&w, A), 100);
}

// spec: S-0515 claim 3 and S-0011 claim 24 (the change-owner trigger action pays it too)
#[test]
fn claim_3_the_change_owner_action_pays_the_bonus() {
    let ev = Event {
        code: 1,
        kind: 0,
        value: -1,
        type_name: None,
    };
    let t = trigger(
        "OWN",
        ev,
        vec![Action {
            code: 14,
            number: 0,
            ..Default::default()
        }],
    );
    let tr = Triggers {
        tags: vec![tag_def(0, &t, 2)],
        triggers: vec![t],
        ..Default::default()
    };
    let mut w = world_with(&rules_set(), tr);
    let b = derrick(&mut w, "D51", 2, 0);
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, 6, 6);
    tag(&mut w, b, "OWN");
    glasnost_sim::mp_triggers::entry_raise(&mut w, b, u);
    assert_eq!(owner(&w, b), Some(A));
    assert_eq!(credits(&w, A), 100);
}

// spec: S-0515 claim 3 and F3 (signed 32-bit wrapping, no floor: a negative bonus takes the balance below 0; a bonus that crosses the largest balance wraps)
#[test]
fn f3_bonus_signs_and_wrapping() {
    let mut w = world();
    w.houses[0].credits = 50;
    let b = derrick(&mut w, "NEGB", 2, 0);
    change(&mut w, b, A);
    assert_eq!(credits(&w, A), -50, "no floor");
    let mut w = world();
    w.houses[0].credits = 5;
    let b = derrick(&mut w, "BIGB", 2, 0);
    change(&mut w, b, A);
    assert_eq!(credits(&w, A), i32::MAX.wrapping_add(5));
}

// spec: S-0515 F2 and TB5 (payments: a negative balance plus a positive amount; a negative amount cannot take the balance below 0; a = 0 with a negative balance sets it to 0; wrapping above the largest balance; the smallest integer as the amount)
#[test]
fn f2_payment_forms() {
    // (a) -50 + 100.
    let mut w = world();
    w.houses[0].credits = 50;
    let b = derrick(&mut w, "NEGB", 2, 0);
    change(&mut w, b, A);
    let _ = payments(&mut w, A, 51);
    assert_eq!(credits(&w, A), 50, "-50 + 100 at the first cycle");
    // (b) a = -5 with a balance of 3.
    let mut w = world();
    w.houses[0].credits = 3;
    let b = derrick(&mut w, "NEGA", 2, 0);
    change(&mut w, b, A);
    let _ = payments(&mut w, A, 51);
    assert_eq!(credits(&w, A), 0);
    // (c) a = 0 with a negative balance.
    let mut w = world();
    w.houses[0].credits = 0;
    let b = derrick(&mut w, "ZEROA", 2, 0);
    change(&mut w, b, A);
    assert_eq!(credits(&w, A), -100);
    let _ = payments(&mut w, A, 51);
    assert_eq!(credits(&w, A), 0);
    // (d) wrapping.
    let mut w = world();
    w.houses[0].credits = 10;
    let b = derrick(&mut w, "BIGA", 2, 0);
    change(&mut w, b, A);
    let _ = payments(&mut w, A, 51);
    assert_eq!(credits(&w, A), 10i32.wrapping_add(i32::MAX));
    // (e) the smallest integer: Y' = wrap(0 - wrap(-MIN)) = MIN.
    let mut w = world();
    let b = derrick(&mut w, "MINA", 2, 0);
    change(&mut w, b, A);
    let _ = payments(&mut w, A, 51);
    assert_eq!(credits(&w, A), i32::MIN);
}

// spec: S-0515 claim 2 and TB3 (c, d, e) (a building put on the map is never manned, so a needs-engineer derrick that a map or a trigger gives to a house pays nothing until its first ownership change; two changes and the payments flow)
#[test]
fn claim_2_unmanned_pays_nothing_until_it_changes_owner() {
    let mut w = world();
    let b = derrick(&mut w, "D51", 0, 0);
    assert!(!power::manned(&w, b));
    assert!(payments(&mut w, A, 1000).is_empty(), "no rise at all");
    let t0 = w.tick;
    change(&mut w, b, HouseId(2));
    change(&mut w, b, A);
    assert!(power::manned(&w, b));
    let v = payments(&mut w, A, 1001);
    assert_eq!(v.len(), 1000 / 50, "a payment every D - 1 ticks");
    assert_eq!(v[0].0, t0 + 50);
}

// spec: S-0515 claim 2 (manned for good from the first ownership change: damage, a power switch and later changes do not end it; a change between two passive houses mans it; the same owner does not)
#[test]
fn claim_2_manned_for_good() {
    let mut w = world();
    let b = derrick(&mut w, "D51", 2, 0);
    change(&mut w, b, HouseId(2));
    assert!(!power::manned(&w, b), "no change");
    change(&mut w, b, CIV2);
    assert!(power::manned(&w, b), "passive to passive mans it");
    change(&mut w, b, A);
    w.obj_mut(b).unwrap().econ.building.switched_off = true;
    testkit::run(&mut w, 3);
    w.obj_mut(b).unwrap().econ.building.switched_off = false;
    w.obj_mut(b).unwrap().health = 10;
    testkit::run(&mut w, 3);
    change(&mut w, b, HouseId(2));
    assert!(power::manned(&w, b));
    // TB3 (f): after the interruptions the payments resume on the original phase.
    let mut w = world();
    let b = derrick(&mut w, "D51", 2, 0);
    let t0 = w.tick;
    change(&mut w, b, A);
    let _ = payments(&mut w, A, 50 + 10);
    w.obj_mut(b).unwrap().econ.building.switched_off = true;
    let _ = payments(&mut w, A, 20);
    w.obj_mut(b).unwrap().econ.building.switched_off = false;
    let v = payments(&mut w, A, 100);
    assert_eq!(ticks(&v), vec![t0 + 100, t0 + 150]);
}

// spec: S-0515 claim 2 and TB8 (an outpost given to a house at the start cannot fire; after two ownership changes it can) and claim 1 (the firing gate is S-0405 claim 9 table T1)
#[test]
fn claim_2_unmanned_outpost_cannot_fire() {
    let mut w = world();
    let b = put(&mut w, Kind::Building, "OUTP", 0, 0, 0);
    assert_eq!(power::firing_answer(&w, b), Some(Answer::Impossible));
    change(&mut w, b, HouseId(2));
    change(&mut w, b, A);
    assert_ne!(power::firing_answer(&w, b), Some(Answer::Impossible));
}

// spec: S-0515 claim 1 and TB6 (an airport gives its superweapon to its owner whether or not it is manned; the old owner loses it; the new owner gets it; S-0512 claims 2, 3 and 7)
#[test]
fn claim_1_airport_grants_while_unmanned() {
    let mut w = world();
    let b = put(&mut w, Kind::Building, "AIRP", 0, 0, 0);
    let other = put(&mut w, Kind::Building, "AIRP", 2, 4, 0);
    testkit::run(&mut w, 10);
    assert!(!power::manned(&w, b));
    assert!(
        charge::get(&w, A, 0).unwrap_or_default().granted,
        "granted although unmanned"
    );
    change(&mut w, b, B);
    testkit::run(&mut w, 10);
    assert!(
        !charge::get(&w, A, 0).unwrap_or_default().granted,
        "lost in the next house update"
    );
    assert!(charge::get(&w, B, 0).unwrap_or_default().granted);
    change(&mut w, other, A);
    testkit::run(&mut w, 10);
    assert!(
        charge::get(&w, A, 0).unwrap_or_default().granted,
        "the captured airport gives it back"
    );
}

/// [`payments`] while the building is being erased: a warp member in its
/// leaving stage (S-0513 claim 39), renewed before each tick so that the
/// warp itself never completes.
fn payments_erased(w: &mut World, b: ObjectId, h: HouseId, n: i32) -> Vec<(i32, i32)> {
    let mut v = Vec::new();
    for _ in 0..n {
        let (t, pos) = (w.tick, w.obj(b).unwrap().pos);
        w.obj_mut(b).unwrap().sw.warp = Some(Member {
            stage: Stage::Leaving(t),
            house: B,
            landing: pos,
            redirected: false,
            fell_back: false,
            refused: false,
            entry_ok: false,
        });
        v.extend(payments(w, h, 1));
    }
    w.obj_mut(b).unwrap().sw.warp = None;
    v
}

// spec: S-0515 claim 6, TB4 and quirks.cash_cycle_lapse (a cycle that falls while the building is being erased is lost with all later cycles until the next ownership change; a shorter erasure changes nothing)
#[test]
fn claim_6_erasure_lapses_the_cycles() {
    // A lock that begins 10 ticks after a payment and ends 100 ticks later.
    let mut w = world();
    let b = derrick(&mut w, "D51", 2, 0);
    let t0 = w.tick;
    change(&mut w, b, A);
    let v = payments(&mut w, A, 50 + 1 + 10);
    assert_eq!(ticks(&v), vec![t0 + 50]);
    let v = payments_erased(&mut w, b, A, 100);
    assert!(v.is_empty());
    let v = payments(&mut w, A, 2000);
    assert!(v.is_empty(), "payments do not resume after the lock ends");
    // Then a change to the neutral house and back: payments again from 50 ticks after the return.
    change(&mut w, b, HouseId(2));
    change(&mut w, b, A);
    let t2 = w.tick;
    let v = payments(&mut w, A, 60);
    assert_eq!(ticks(&v), vec![t2 + 50]);
    // The control: a lock that begins 5 ticks after a payment and ends 30 ticks later.
    let mut w = world();
    let b = derrick(&mut w, "D51", 2, 0);
    let t0 = w.tick;
    change(&mut w, b, A);
    let _ = payments(&mut w, A, 50 + 1 + 5);
    let _ = payments_erased(&mut w, b, A, 30);
    let v = payments(&mut w, A, 100);
    assert_eq!(ticks(&v), vec![t0 + 100, t0 + 150]);
    // Switch off: the first turn at or after the due tick has the cycle, cycles continue from there.
    let mut w = world();
    quirk(&mut w, |q| q.cash_cycle_lapse = false);
    let b = derrick(&mut w, "D51", 2, 0);
    let t0 = w.tick;
    change(&mut w, b, A);
    let _ = payments(&mut w, A, 50 + 1 + 10);
    let _ = payments_erased(&mut w, b, A, 100);
    let v = payments(&mut w, A, 100);
    assert_eq!(
        ticks(&v),
        vec![t0 + 161, t0 + 211],
        "the first turn after the lock has the cycle, then every 50 ticks"
    );
}

// spec: S-0515 claim 8 and TB7 (an ownership change does not change a secret lab's secret type and takes no random draw: the type stays through five round trips, and the owner follows by S-0407 claims 4 and 15)
#[test]
fn claim_8_secret_lab_keeps_its_type_and_takes_no_draw() {
    let mut w = world();
    let lab = derrick(&mut w, "SLAB", 2, 0);
    tech::assign_secret_labs(&mut w);
    let secret = w.obj(lab).unwrap().econ.secret;
    assert!(secret.is_some());
    let taken = w.rng.taken();
    for _ in 0..5 {
        change(&mut w, lab, A);
        assert_eq!(w.obj(lab).unwrap().econ.secret, secret);
        assert_eq!(tech::answer(&w, A, secret.unwrap()), TechAnswer::Allowed);
        assert_ne!(tech::answer(&w, B, secret.unwrap()), TechAnswer::Allowed);
        change(&mut w, lab, HouseId(2));
        assert_eq!(w.obj(lab).unwrap().econ.secret, secret);
    }
    assert_eq!(w.rng.taken(), taken, "no draw for the secret type");
}

// spec: S-0515 claims 2 to 7 (the new state is saved: the countdown and the manned state survive a round trip with an identical state hash, and the run goes on alike)
#[test]
fn income_state_is_saved_and_hashed() {
    let mut w = world();
    let b = derrick(&mut w, "D51", 2, 0);
    change(&mut w, b, A);
    testkit::run(&mut w, 30);
    let h = glasnost_sim::save::state_hash(&w);
    let bytes = glasnost_sim::save::save(&w);
    let mut l =
        glasnost_sim::save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(glasnost_sim::save::state_hash(&l), h);
    assert_eq!(
        l.obj(b).unwrap().econ.building.income_restart,
        w.obj(b).unwrap().econ.building.income_restart
    );
    testkit::run(&mut w, 60);
    testkit::run(&mut l, 60);
    assert_eq!(credits(&l, A), credits(&w, A));
    assert_eq!(
        glasnost_sim::save::state_hash(&l),
        glasnost_sim::save::state_hash(&w)
    );
    // The restart tick is part of the hash.
    let mut m = w.clone();
    m.obj_mut(b).unwrap().econ.building.income_restart = Some(1);
    assert_ne!(
        glasnost_sim::save::state_hash(&m),
        glasnost_sim::save::state_hash(&w)
    );
}

// spec: S-0515 Terms (a cash-producing type has a nonzero acquisition bonus or a positive income delay)
#[test]
fn terms_cash_producing_type() {
    let mut w = world();
    let yes = derrick(&mut w, "D51", 2, 0);
    let bonus_only = derrick(&mut w, "BIGB", 2, 1);
    let neither = derrick(&mut w, "D0", 2, 2);
    let plain = put(&mut w, Kind::Building, "OUTP", 2, 4, 4);
    assert!(cash::cash_producing(&w, yes));
    assert!(cash::cash_producing(&w, bonus_only));
    assert!(!cash::cash_producing(&w, neither));
    assert!(!cash::cash_producing(&w, plain));
}
