//! Claim tests for S-0405 (power) on synthetic data: invented building and
//! vehicle types, no stock values. Houses Alpha (0) and Beta (1) of the
//! testkit's sandbox game, not allied.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::combat::check::Answer;
use glasnost_sim::econ::power;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);

fn building(strength: i32, power: i32) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(strength),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    b.econ.power = Some(power);
    b
}

/// PLANT (output 120, strength 500), CONS (drain 40), DEF (needs power,
/// drain 10), NODRAIN (needs power, no drain), SPECIAL (special-power flag),
/// ENG (needs-engineer flag: works only when manned), ABS (absorbing, extra output 30), XTRA (output
/// 100, extra output 30), UPx upgrades, CENTRE (control centre for DRONE),
/// GAPPER (super-gap radius, extra drain 20), GAPGEN (gap generator, radius 3,
/// drain 10); vehicle DRONE (powered unit).
fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    let names = [
        "PLANT", "CONS", "DEF", "NODRAIN", "SPECIAL", "ENG", "ABS", "XTRA", "UP10", "UP20", "UPD5",
        "UPT20", "CENTRE", "CENTRE2", "GAPPER", "SMALL", "WALLY", "GAPGEN",
    ];
    r.registries.insert(
        "buildings".into(),
        names.iter().map(|s| s.to_string()).collect(),
    );
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .extend(["DRONE".to_string(), "DRONE2".to_string()]);
    let s = &mut r.sim;
    s.buildings.insert("PLANT".into(), building(500, 120));
    s.buildings.insert("SMALL".into(), building(500, 30));
    s.buildings.insert("CONS".into(), building(500, -40));
    let mut def = building(500, -10);
    def.powered = Some(true);
    s.buildings.insert("DEF".into(), def);
    let mut nd = building(500, 0);
    nd.powered = Some(true);
    s.buildings.insert("NODRAIN".into(), nd);
    let mut sp = building(500, 0);
    sp.econ.powered_special = Some(true);
    s.buildings.insert("SPECIAL".into(), sp);
    let mut eng = building(500, 0);
    eng.needs_engineer = Some(true);
    s.buildings.insert("ENG".into(), eng);
    let mut abs = building(500, 0);
    abs.infantry_absorb = Some(true);
    abs.econ.extra_power = Some(30);
    s.buildings.insert("ABS".into(), abs);
    let mut x = building(500, 100);
    x.econ.extra_power = Some(30);
    s.buildings.insert("XTRA".into(), x);
    for (n, p) in [("UP10", 10), ("UP20", 20), ("UPD5", -5)] {
        let mut u = building(100, p);
        u.econ.powers_up_building = Some("PLANT".into());
        s.buildings.insert(n.into(), u);
    }
    let mut upt = building(100, 20);
    upt.econ.powers_up_building = Some("PLANT".into());
    upt.turret = Some(true);
    s.buildings.insert("UPT20".into(), upt);
    let mut c = building(500, -10);
    c.powered = Some(true);
    c.econ.powers_unit = Some("DRONE".into());
    s.buildings.insert("CENTRE".into(), c);
    let mut c2 = building(500, -10);
    c2.powered = Some(true);
    c2.econ.powers_unit = Some("DRONE2".into());
    s.buildings.insert("CENTRE2".into(), c2);
    let mut g = building(500, -10);
    g.econ.super_gap_radius_in_cells = Some(5);
    g.econ.extra_power = Some(-20);
    s.buildings.insert("GAPPER".into(), g);
    let mut gg = building(500, -10);
    gg.gap_generator = Some(true);
    gg.econ.gap_radius_in_cells = Some(3);
    s.buildings.insert("GAPGEN".into(), gg);
    let mut wall = building(500, 0);
    wall.firestorm_wall = Some(true);
    wall.econ.power = Some(-5);
    s.buildings.insert("WALLY".into(), wall);
    for n in ["DRONE", "DRONE2"] {
        let mut v = UnitType {
            strength: Some(100),
            speed: Some(6),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            ..Default::default()
        };
        v.econ.powered_unit = Some(true);
        s.vehicles.insert(n.into(), v);
    }
    r
}

/// A sandbox game, run past the ticks in which orders are blocked (S-0201
/// claim 14).
fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn put(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(
        w,
        Kind::Building,
        name,
        house,
        testkit::mid(SIZE, dx, dy),
        0,
    )
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn order(w: &mut World, h: HouseId, o: Order) {
    assert!(glasnost_sim::orders::issue(w, h, o), "order accepted");
}

// spec: S-0405 claim 1 and F1 (output = trunc(RZ₅₃(RZ₅₃(H / S) × P)); the invented example 120, 333/500 gives 79)
#[test]
fn claim_1_damaged_plant_output() {
    let mut w = game(&rules());
    let p = put(&mut w, "PLANT", 0, 0, 0);
    assert_eq!(power::building_output(&w, p), 120);
    w.obj_mut(p).unwrap().health = 333;
    assert_eq!(power::building_output(&w, p), 79);
    w.obj_mut(p).unwrap().health = 1;
    assert_eq!(power::building_output(&w, p), 0);
    // A pulse does not change the output, and a switched-off building makes none.
    w.obj_mut(p).unwrap().health = 500;
    w.obj_mut(p).unwrap().econ.building.switched_off = true;
    assert_eq!(power::building_output(&w, p), 0);
}

// spec: S-0405 claim 1 (the extra output of a building that is not absorbing adds nothing; absorbed passengers are a seam, n_a = 0)
#[test]
fn claim_1_extra_output_only_for_absorbing() {
    let mut w = game(&rules());
    let x = put(&mut w, "XTRA", 0, 0, 0);
    assert_eq!(power::building_output(&w, x), 100);
    let a = put(&mut w, "ABS", 0, 2, 0);
    assert_eq!(power::building_output(&w, a), 0);
}

// spec: S-0405 claims 1 and 2, Terms (upgrades in the three positions count; W12)
// spec: S-0405 claims 1 and 2, F1 and F2 (no recorded upgrade counts at count 0)
#[test]
fn claims_1_2_upgrades() {
    let r = rules();
    let mut w = game(&r);
    let p = put(&mut w, "PLANT", 0, 0, 0);
    let idx = |n: &str| w.rules().index_of(Kind::Building, n).unwrap();
    let (u10, u20, ud5) = (idx("UP10"), idx("UP20"), idx("UPD5"));
    let set = |w: &mut World, ups: [Option<u32>; 3], count: i32| {
        let b = &mut w.obj_mut(p).unwrap().econ.building;
        b.upgrades = ups;
        b.upgrade_count = count;
    };
    set(&mut w, [Some(u10), None, None], 1);
    assert_eq!(power::building_output(&w, p), 130);
    set(&mut w, [Some(u10), Some(u20), Some(ud5)], 3);
    assert_eq!(power::building_output(&w, p), 150);
    assert_eq!(power::building_drain(&w, p), 5);
    // A gap in the positions: the recorded ones count.
    set(&mut w, [Some(u10), None, Some(ud5)], 2);
    assert_eq!(power::building_output(&w, p), 130);
    assert_eq!(power::building_drain(&w, p), 5);
    // Count 0 with recorded positions: none counts.
    set(&mut w, [Some(u10), Some(u20), None], 0);
    assert_eq!(power::building_output(&w, p), 120);
}

// spec: S-0405 claim 2 (health does not change the drain; the super-gap adds the extra drain, claim 16)
#[test]
fn claim_2_drain() {
    let mut w = game(&rules());
    let c = put(&mut w, "CONS", 0, 0, 0);
    w.obj_mut(c).unwrap().health = 100;
    assert_eq!(power::building_drain(&w, c), 40);
    let g = put(&mut w, "GAPPER", 0, 2, 0);
    assert_eq!(power::building_drain(&w, g), 10);
    w.obj_mut(g).unwrap().econ.building.super_gap = true;
    assert_eq!(power::building_drain(&w, g), 30);
}

// spec: S-0405 claim 3 (counted from placement, during construction and selling, until removed)
#[test]
fn claim_3_counted_buildings() {
    let mut w = game(&rules());
    let p = put(&mut w, "PLANT", 0, 0, 0);
    let t = w.tick;
    w.obj_mut(p)
        .unwrap()
        .beh
        .force(Behaviour::Construction, t, true);
    step(&mut w);
    assert_eq!(power::totals(&w, A), (120, 0));
    w.obj_mut(p).unwrap().beh.force(Behaviour::Selling, t, true);
    step(&mut w);
    assert_eq!(power::totals(&w, A), (120, 0));
    glasnost_sim::objects::remove(&mut w, p);
    step(&mut w);
    assert_eq!(power::totals(&w, A), (0, 0));
}

// spec: S-0405 claim 4 and F3 (low power when D > 0 and O < D; p = O / D, 0 at O = 0, 1 without drain)
#[test]
fn claim_4_low_power_and_fraction() {
    let mut w = game(&rules());
    let mode = w.rules().mode;
    let one = glasnost_num::F64::from_i64(1, mode);
    // No drain at all: full power, including O = 0.
    step(&mut w);
    assert!(!power::low_power(&w, A));
    assert!(power::fraction(&w, A).eq_num(one));
    let _c = put(&mut w, "CONS", 0, 0, 0);
    step(&mut w);
    assert_eq!(power::totals(&w, A), (0, 40));
    assert!(power::low_power(&w, A));
    assert!(power::fraction(&w, A).eq_num(glasnost_num::F64::ZERO));
    let _s = put(&mut w, "SMALL", 0, 2, 0);
    step(&mut w);
    assert!(power::low_power(&w, A));
    let want =
        glasnost_num::F64::from_i64(30, mode).div(glasnost_num::F64::from_i64(40, mode), mode);
    assert!(power::fraction(&w, A).eq_num(want));
    let _p = put(&mut w, "PLANT", 0, 4, 0);
    step(&mut w);
    assert!(!power::low_power(&w, A));
    assert!(power::fraction(&w, A).eq_num(one));
}

/// Ticks in which house A's recorded output is 0, over `n` ticks.
fn zero_ticks(w: &mut World, n: i32) -> Vec<i32> {
    let mut out = Vec::new();
    for _ in 0..n {
        let t = w.tick;
        step(w);
        if power::totals(w, A).0 == 0 {
            out.push(t);
        }
    }
    out
}

// spec: S-0405 claims 5 and 6 (a timed blackout of length d counts as 0 output in the house updates t₀ … t₀ + d − 2; full power at t₀ + d − 1)
#[test]
fn claims_5_6_timed_blackout() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0, 0, 0);
    put(&mut w, "CONS", 0, 2, 0);
    step(&mut w);
    let t0 = w.tick;
    // Started before the house's update of t₀ (between ticks counts as before).
    power::start_blackout(&mut w, A, 30);
    assert!(power::blackout_running(&w, A));
    let z = zero_ticks(&mut w, 40);
    assert_eq!(z, (t0..=t0 + 28).collect::<Vec<_>>());
    assert!(!power::blackout_running(&w, A));
}

// spec: S-0405 claim 6 (length 1 or less gives no house update with a blackout; a new start replaces the running one)
#[test]
fn claim_6_short_and_replaced_blackouts() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0, 0, 0);
    put(&mut w, "CONS", 0, 2, 0);
    step(&mut w);
    power::start_blackout(&mut w, A, 1);
    assert!(zero_ticks(&mut w, 5).is_empty());
    power::start_blackout(&mut w, A, 0);
    assert!(zero_ticks(&mut w, 5).is_empty());
    power::start_blackout(&mut w, A, -4);
    assert!(!power::blackout_running(&w, A));
    // Replacement: the end is computed from the new start; lengths do not add.
    let t0 = w.tick;
    power::start_blackout(&mut w, A, 20);
    for _ in 0..10 {
        step(&mut w);
    }
    let t1 = w.tick;
    power::start_blackout(&mut w, A, 20);
    let z = zero_ticks(&mut w, 30);
    assert_eq!(z.last().copied(), Some(t1 + 18));
    assert!(t1 + 18 < t0 + 40);
    // A start of length 0 ends a running one at once.
    power::start_blackout(&mut w, A, 50);
    assert!(power::blackout_running(&w, A));
    power::start_blackout(&mut w, A, 0);
    assert!(!power::blackout_running(&w, A));
}

// spec: S-0405 Edge cases (quirks.blackout_one_short off: exactly d house updates with output 0)
#[test]
fn claim_6_blackout_one_short_off() {
    let mut r = rules();
    r.globals.quirks.blackout_one_short = false;
    let mut w = game(&r);
    put(&mut w, "PLANT", 0, 0, 0);
    put(&mut w, "CONS", 0, 2, 0);
    step(&mut w);
    let t0 = w.tick;
    power::start_blackout(&mut w, A, 30);
    assert_eq!(zero_ticks(&mut w, 40), (t0..t0 + 30).collect::<Vec<_>>());
    power::start_blackout(&mut w, A, 1);
    assert_eq!(zero_ticks(&mut w, 5).len(), 1);
}

// spec: S-0405 claim 7 (a power order in tick n changes the totals in the house update of tick n + 1)
#[test]
fn claim_7_power_order_lag() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0, 0, 0);
    let c = put(&mut w, "CONS", 0, 2, 0);
    for _ in 0..5 {
        step(&mut w);
    }
    assert_eq!(power::totals(&w, A), (120, 40));
    let n = w.tick;
    order(&mut w, A, Order::PowerOff { actor: c });
    step(&mut w); // tick n: orders applied after the house update
    assert_eq!(w.tick, n + 1);
    assert_eq!(power::totals(&w, A), (120, 40));
    step(&mut w); // tick n + 1
    assert_eq!(power::totals(&w, A), (120, 0));
}

// spec: S-0405 claim 7 (a health change is noticed in the building's own turn and the house works out its power in the same tick, with the live health)
#[test]
fn claim_7_health_change_noticed() {
    let mut w = game(&rules());
    let p = put(&mut w, "PLANT", 0, 0, 0);
    put(&mut w, "CONS", 0, 2, 0);
    for _ in 0..3 {
        step(&mut w);
    }
    // Damage between ticks: the plant notices it in its turn of the next tick.
    w.obj_mut(p).unwrap().health = 250;
    step(&mut w);
    assert_eq!(power::totals(&w, A), (60, 40));
    // Without an event the totals are kept: a change made after the house update is not seen.
    assert!(!w.houses[0].econ.power.due);
}

// spec: S-0405 claim 8 (the in-operation test: switch, low power with drain, no drain, special power and blackout, manned when required, construction and selling)
#[test]
fn claim_8_in_operation_rows() {
    let mut w = game(&rules());
    let def = put(&mut w, "DEF", 0, 0, 0);
    let nd = put(&mut w, "NODRAIN", 0, 2, 0);
    let sp = put(&mut w, "SPECIAL", 0, 4, 0);
    let eng = put(&mut w, "ENG", 0, 6, 0);
    step(&mut w);
    // Low power (drain 10, output 0): the drain-needing defence stops, the no-drain one does not.
    assert!(power::low_power(&w, A));
    assert!(!power::in_operation(&w, def));
    assert!(power::in_operation(&w, nd));
    // The needs-engineer type works only when manned; placement does not man it (S-0515 claim 2).
    assert!(!power::in_operation(&w, eng));
    // Special power: stops during a running timed blackout.
    assert!(power::in_operation(&w, sp));
    power::start_blackout(&mut w, A, 10);
    assert!(!power::in_operation(&w, sp));
    assert!(power::in_operation(&w, nd));
    // Full power again: the defence works; switched off it does not.
    put(&mut w, "PLANT", 0, 8, 0);
    for _ in 0..12 {
        step(&mut w);
    }
    assert!(power::in_operation(&w, def));
    assert!(power::in_operation(&w, sp));
    w.obj_mut(def).unwrap().econ.building.switched_off = true;
    assert!(!power::in_operation(&w, def));
    w.obj_mut(def).unwrap().econ.building.switched_off = false;
    // Construction or selling as the behaviour read: while a behaviour is
    // current, the queued one is not read (r57).
    let t = w.tick;
    w.obj_mut(def).unwrap().beh.queued = Some(Behaviour::Selling);
    assert!(power::in_operation(&w, def));
    w.obj_mut(def)
        .unwrap()
        .beh
        .force(Behaviour::Selling, t, true);
    assert!(!power::in_operation(&w, def));
    w.obj_mut(def)
        .unwrap()
        .beh
        .force(Behaviour::Construction, t, true);
    assert!(!power::in_operation(&w, def));
    // Health 0 is not alive.
    w.obj_mut(nd).unwrap().health = 0;
    assert!(!power::in_operation(&w, nd));
}

// spec: S-0405 claim 9 and T1 (a building's own answers: illegal while building or selling unless a pulse cannon; impossible when not in operation or a pulse cannon)
#[test]
fn claim_9_firing_answers() {
    let mut r = rules();
    let mut pulse = building(500, 0);
    pulse.emp_cannon = Some(true);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("PULSE".into());
    r.sim.buildings.insert("PULSE".into(), pulse);
    let mut occ = building(500, 0);
    occ.can_be_occupied = Some(true);
    occ.can_occupy_fire = Some(true);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("OCC".into());
    r.sim.buildings.insert("OCC".into(), occ);
    let mut w = game(&r);
    let def = put(&mut w, "DEF", 0, 0, 0);
    let pc = put(&mut w, "PULSE", 0, 2, 0);
    let oc = put(&mut w, "OCC", 0, 4, 0);
    step(&mut w);
    assert_eq!(power::firing_answer(&w, def), Some(Answer::Impossible));
    assert_eq!(power::firing_answer(&w, pc), Some(Answer::Impossible));
    // An occupiable building without occupants (a seam: no occupants yet) is illegal.
    assert_eq!(power::firing_answer(&w, oc), Some(Answer::Illegal));
    let t = w.tick;
    w.obj_mut(def)
        .unwrap()
        .beh
        .force(Behaviour::Construction, t, true);
    assert_eq!(power::firing_answer(&w, def), Some(Answer::Illegal));
    w.obj_mut(pc)
        .unwrap()
        .beh
        .force(Behaviour::Selling, t, true);
    assert_eq!(power::firing_answer(&w, pc), Some(Answer::Impossible));
    put(&mut w, "PLANT", 0, 6, 0);
    w.obj_mut(def).unwrap().beh.force(Behaviour::Guard, t, true);
    step(&mut w);
    assert_eq!(power::firing_answer(&w, def), None);
}

// spec: S-0405 claim 9 (the one-off effects happen in the building's own turn: a building that loses power by the house update of tick n changes in its turn of tick n + 1)
#[test]
fn claim_9_one_off_effects_in_own_turn() {
    let mut w = game(&rules());
    let p = put(&mut w, "PLANT", 0, 0, 0);
    let c = put(&mut w, "CENTRE", 0, 2, 0);
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(w.obj(c).unwrap().econ.building.centre);
    // The plant goes between ticks: the house update of tick n works out low power.
    glasnost_sim::objects::remove(&mut w, p);
    step(&mut w);
    assert!(power::low_power(&w, A));
    assert!(!power::in_operation(&w, c));
    assert!(
        w.obj(c).unwrap().econ.building.centre,
        "still active until its next turn"
    );
    step(&mut w);
    assert!(!w.obj(c).unwrap().econ.building.centre);
}

// spec: S-0405 claim 10 (power off only with a base drain above 0 or the needs-power flag; power on; the firestorm wall ignores both, S-0213 claim 13)
#[test]
fn claim_10_power_switch() {
    let mut w = game(&rules());
    let p = put(&mut w, "PLANT", 0, 0, 0);
    let c = put(&mut w, "CONS", 0, 2, 0);
    let nd = put(&mut w, "NODRAIN", 0, 4, 0);
    let fw = put(&mut w, "WALLY", 0, 6, 0);
    power::switch(&mut w, p, false);
    assert!(!w.obj(p).unwrap().econ.building.switched_off);
    power::switch(&mut w, c, false);
    assert!(w.obj(c).unwrap().econ.building.switched_off);
    power::switch(&mut w, nd, false);
    assert!(w.obj(nd).unwrap().econ.building.switched_off);
    power::switch(&mut w, fw, false);
    assert!(!w.obj(fw).unwrap().econ.building.switched_off);
    power::switch(&mut w, c, true);
    assert!(!w.obj(c).unwrap().econ.building.switched_off);
}

// spec: S-0405 claim 10 (an order repeating the state already set changes nothing; a dead or off-map building takes no order)
#[test]
fn claim_10_repeat_dead_and_off_map() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0, 0, 0);
    let c = put(&mut w, "CONS", 0, 2, 0);
    power::switch(&mut w, c, true);
    assert!(!w.obj(c).unwrap().econ.building.switched_off);
    power::switch(&mut w, c, false);
    power::switch(&mut w, c, false);
    assert!(w.obj(c).unwrap().econ.building.switched_off);
    let d = put(&mut w, "CONS", 0, 4, 0);
    w.obj_mut(d).unwrap().health = 0;
    power::switch(&mut w, d, false);
    assert!(!w.obj(d).unwrap().econ.building.switched_off, "dead");
    let e = put(&mut w, "CONS", 0, 6, 0);
    w.obj_mut(e).unwrap().on_map = false;
    power::switch(&mut w, e, false);
    assert!(!w.obj(e).unwrap().econ.building.switched_off, "off the map");
}

// spec: S-0405 claims 9 and 10 (an accepted switch-off ends the gap field at once; switched on, it starts again in the next building turn in operation)
#[test]
fn claim_10_switch_ends_gap_at_once() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0, 0, 0);
    let g = put(&mut w, "GAPGEN", 0, 2, 0);
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(w.obj(g).unwrap().econ.sight.gap.is_some());
    power::switch(&mut w, g, false);
    assert!(
        w.obj(g).unwrap().econ.sight.gap.is_none(),
        "ended without waiting for the building's turn"
    );
    power::switch(&mut w, g, true);
    assert!(w.obj(g).unwrap().econ.sight.gap.is_none());
    step(&mut w);
    assert!(w.obj(g).unwrap().econ.sight.gap.is_some());
}

// spec: S-0405 claim 13 (the count of active control centres: from 1 to 0 disables the powered unit's vehicles; from 0 to 1 enables them; one count for all types)
#[test]
fn claim_13_control_centres() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0, 0, 0);
    let k = put(&mut w, "CENTRE", 0, 2, 0);
    let k2 = put(&mut w, "CENTRE2", 0, 4, 0);
    let d1 = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "DRONE",
        0,
        testkit::mid(SIZE, -4, 4),
        0,
    );
    let d2 = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "DRONE2",
        0,
        testkit::mid(SIZE, -4, 6),
        0,
    );
    for _ in 0..3 {
        step(&mut w);
    }
    assert_eq!(w.houses[0].econ.power.centres, 2);
    // Switching K off: one count for all types, so nothing is disabled (quirks.shared_control_centre_count).
    order(&mut w, A, Order::PowerOff { actor: k });
    for _ in 0..3 {
        step(&mut w);
    }
    assert_eq!(w.houses[0].econ.power.centres, 1);
    assert!(!power::unit_disabled(&w, d1));
    // K2 off too: the count reaches 0 and only K2's powered unit is disabled.
    order(&mut w, A, Order::PowerOff { actor: k2 });
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(!power::unit_disabled(&w, d1));
    assert!(power::unit_disabled(&w, d2));
    // K on again: count 0 to 1 enables K's powered unit only.
    order(&mut w, A, Order::PowerOn { actor: k });
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(!power::unit_disabled(&w, d1));
    assert!(power::unit_disabled(&w, d2));
}

// spec: S-0215 claim 9 (the guard reset applies to an actor that is deactivated or reactivated: B45 f, a control-centre switch gives the dependent vehicle guard current at once, an empty queue, no target, destination or guard spot, the timer expired now and the start tick reset, owner unchanged and the suspended slot kept) and S-0405 claim 13
#[test]
fn claim_13_control_centre_switch_gives_the_dependent_vehicle_a_guard_reset() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0, 0, 0);
    let k = put(&mut w, "CENTRE", 0, 2, 0);
    let d = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "DRONE",
        0,
        testkit::mid(SIZE, -4, 4),
        0,
    );
    for _ in 0..3 {
        step(&mut w);
    }
    let foe = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "DRONE",
        1,
        testkit::mid(SIZE, -4, 8),
        0,
    );
    for round in 0..2 {
        {
            let o = w.obj_mut(d).unwrap();
            o.beh.current = Some(Behaviour::Attack);
            o.beh.queued = Some(Behaviour::Move);
            o.beh.suspended = Some(Behaviour::AreaGuard);
            o.target = Some(glasnost_sim::world::Target::Object(foe));
            o.dest = Some(glasnost_sim::world::Target::Cell(testkit::mid(SIZE, 0, 8)));
            o.guard_spot = Some(glasnost_sim::world::Target::Cell(testkit::mid(SIZE, 0, 9)));
        }
        let order_on = round == 1;
        order(
            &mut w,
            A,
            if order_on {
                Order::PowerOn { actor: k }
            } else {
                Order::PowerOff { actor: k }
            },
        );
        for _ in 0..3 {
            step(&mut w);
        }
        assert_eq!(power::unit_disabled(&w, d), !order_on, "round {round}");
        let o = w.obj(d).unwrap();
        assert_eq!(o.owner, Some(A));
        assert_eq!(o.beh.current, Some(Behaviour::Guard), "round {round}");
        assert_eq!(o.beh.queued, None);
        assert_eq!(o.target, None);
        assert_eq!(o.guard_spot, None);
        assert_eq!(o.beh.suspended, Some(Behaviour::AreaGuard), "kept");
    }
}

// spec: S-0405 Edge cases (quirks.shared_control_centre_count off: the count is kept per powered-unit type)
#[test]
fn claim_13_per_type_count() {
    let mut r = rules();
    r.globals.quirks.shared_control_centre_count = false;
    let mut w = game(&r);
    put(&mut w, "PLANT", 0, 0, 0);
    let k = put(&mut w, "CENTRE", 0, 2, 0);
    let _k2 = put(&mut w, "CENTRE2", 0, 4, 0);
    let d1 = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "DRONE",
        0,
        testkit::mid(SIZE, -4, 4),
        0,
    );
    for _ in 0..3 {
        step(&mut w);
    }
    order(&mut w, A, Order::PowerOff { actor: k });
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(power::unit_disabled(&w, d1));
}

// spec: S-0405 claim 13 (a powered-unit vehicle that enters a cell without a building while its house has no active control centre is disabled)
#[test]
fn claim_13_cell_entry_rule() {
    let mut w = game(&rules());
    let d1 = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "DRONE",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    step(&mut w);
    assert!(!power::unit_disabled(&w, d1));
    order(
        &mut w,
        A,
        Order::Behaviour {
            actor: d1,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Cell(testkit::mid(SIZE, 3, 0))),
        },
    );
    for _ in 0..60 {
        step(&mut w);
    }
    assert!(power::unit_disabled(&w, d1));
}

// spec: S-0405 claim 16 (the super-gap ends, and its extra drain goes, when the building leaves operation)
#[test]
fn claim_16_super_gap_ends_with_operation() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0, 0, 0);
    let g = put(&mut w, "GAPPER", 0, 2, 0);
    for _ in 0..3 {
        step(&mut w);
    }
    w.obj_mut(g).unwrap().econ.building.super_gap = true;
    power::make_due(&mut w, A);
    step(&mut w);
    assert_eq!(power::totals(&w, A), (120, 30));
    let t = w.tick;
    w.obj_mut(g).unwrap().beh.force(Behaviour::Selling, t, true);
    step(&mut w);
    assert!(!w.obj(g).unwrap().econ.building.super_gap);
    step(&mut w);
    assert_eq!(power::totals(&w, A), (120, 10));
}

// spec: S-0405 claim 17 (no rule depends on whether the house is human- or computer-controlled)
#[test]
fn claim_17_all_houses_alike() {
    let mut w = game(&rules());
    w.houses[1].human = false;
    put(&mut w, "CONS", 1, 0, 0);
    let d = put(&mut w, "DEF", 1, 2, 0);
    step(&mut w);
    assert!(power::low_power(&w, HouseId(1)));
    assert!(!power::in_operation(&w, d));
}

// spec: S-0405 claim 1 (the map can place a building switched off, S-0009 claim 40; switched off it makes no output)
#[test]
fn switched_off_building_makes_nothing() {
    let mut w = game(&rules());
    let p = put(&mut w, "PLANT", 0, 0, 0);
    w.obj_mut(p).unwrap().econ.building.switched_off = true;
    power::make_due(&mut w, A);
    step(&mut w);
    assert_eq!(power::totals(&w, A), (0, 0));
}

// spec: S-0215 claim 32 and Terms (the SAM power row: needs power, drains power, owner short of power, S-0405 claim 4)
#[test]
fn sam_power_row_reads_low_power() {
    let mut w = game(&rules());
    let d = put(&mut w, "DEF", 0, 0, 0);
    let nd = put(&mut w, "NODRAIN", 0, 2, 0);
    step(&mut w);
    assert!(glasnost_sim::seams::short_of_power(&w, A));
    assert!(glasnost_sim::seams::sam_held_by_low_power(&w, d));
    assert!(!glasnost_sim::seams::sam_held_by_low_power(&w, nd));
    put(&mut w, "PLANT", 0, 4, 0);
    step(&mut w);
    assert!(!glasnost_sim::seams::sam_held_by_low_power(&w, d));
}

// spec: S-0405 Terms, claims 1 and 2 (W12 f: selling a turret upgrade resets the count to 0 and clears only its position; the lower recorded upgrade stays but adds no output or drain; the next sale sells the host, S-0408 claims 5 and 6)
#[test]
fn w12_f_turret_upgrade_sale() {
    for (lower, before) in [("UP10", (150, 0)), ("UPD5", (140, 5))] {
        let mut r = rules();
        // A build-up sprite, so that a sell order sells (S-0408 claim 4).
        r.sim.buildings.get_mut("PLANT").unwrap().econ.buildup = Some(true);
        let mut w = game(&r);
        let p = put(&mut w, "PLANT", 0, 0, 0);
        let lo = w.rules().index_of(Kind::Building, lower).unwrap();
        let upt = w.rules().index_of(Kind::Building, "UPT20").unwrap();
        {
            let b = &mut w.obj_mut(p).unwrap().econ.building;
            b.upgrades = [Some(lo), Some(upt), None];
            b.upgrade_count = 2;
        }
        assert_eq!(
            (power::building_output(&w, p), power::building_drain(&w, p)),
            before
        );
        order(&mut w, A, Order::Sell { actor: p });
        for _ in 0..3 {
            step(&mut w);
        }
        let b = &w.obj(p).unwrap().econ.building;
        assert_eq!(b.upgrade_count, 0);
        assert_eq!(b.upgrades, [Some(lo), None, None]);
        assert_eq!(
            (power::building_output(&w, p), power::building_drain(&w, p)),
            (120, 0)
        );
        order(&mut w, A, Order::Sell { actor: p });
        for _ in 0..3 {
            step(&mut w);
        }
        let o = w.obj(p).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Selling));
        assert_eq!(o.econ.building.upgrades, [Some(lo), None, None]);
    }
}

// spec: S-0405 claims 1 and 2 (W12 g, Q-0224: with a positive upgrade count every recorded position contributes, including positions numbered above the count; the count is a gate, not a limit)
#[test]
fn w12_g_positions_above_the_count_contribute() {
    // PLANT makes 120 here (W12's P makes 100).
    for (middle, after) in [("UP20", (170, 0)), ("UPD5", (150, 5))] {
        let mut r = rules();
        r.sim.buildings.get_mut("PLANT").unwrap().econ.buildup = Some(true);
        r.sim.buildings.get_mut("PLANT").unwrap().econ.upgrades = Some(3);
        r.registries
            .get_mut("buildings")
            .unwrap()
            .extend(["UPT0".to_string(), "UP30".to_string()]);
        let mut upt0 = building(100, 0);
        upt0.econ.powers_up_building = Some("PLANT".into());
        upt0.turret = Some(true);
        r.sim.buildings.insert("UPT0".into(), upt0);
        let mut up30 = building(100, 30);
        up30.econ.powers_up_building = Some("PLANT".into());
        r.sim.buildings.insert("UP30".into(), up30);
        let mut w = game(&r);
        let p = put(&mut w, "PLANT", 0, 0, 0);
        let ix = |w: &World, n: &str| w.rules().index_of(Kind::Building, n).unwrap();
        let (lo, mid, top, rep) = (
            ix(&w, "UP10"),
            ix(&w, middle),
            ix(&w, "UPT0"),
            ix(&w, "UP30"),
        );
        {
            let b = &mut w.obj_mut(p).unwrap().econ.building;
            b.upgrades = [Some(lo), Some(mid), Some(top)];
            b.upgrade_count = 3;
        }
        order(&mut w, A, Order::Sell { actor: p });
        for _ in 0..3 {
            step(&mut w);
        }
        let b = &w.obj(p).unwrap().econ.building;
        assert_eq!(b.upgrade_count, 0);
        assert_eq!(b.upgrades, [Some(lo), Some(mid), None]);
        assert_eq!(
            (power::building_output(&w, p), power::building_drain(&w, p)),
            (120, 0)
        );
        glasnost_sim::econ::placement::add_upgrade(&mut w, p, rep);
        let b = &w.obj(p).unwrap().econ.building;
        assert_eq!(b.upgrade_count, 1);
        assert_eq!(b.upgrades, [Some(rep), Some(mid), None]);
        assert_eq!(
            (power::building_output(&w, p), power::building_drain(&w, p)),
            after
        );
    }
}

// spec: S-0405 claim 19 (the meter's scale B is base output plus base drain, without health scaling or switching)
#[test]
fn claim_19_meter_base() {
    let mut w = game(&rules());
    let base = power::meter_base(&w, A);
    let p = put(&mut w, "PLANT", 0, 0, 0);
    let c = put(&mut w, "CONS", 0, 2, 0);
    w.obj_mut(p).unwrap().health = 100;
    w.obj_mut(c).unwrap().econ.building.switched_off = true;
    assert_eq!(power::meter_base(&w, A), base + 160);
}

// spec: S-0405 claim 19 and F4 (A = N − min(N − 1, max(0, trunc(RZ₅₃(RZ₅₃(400 / RZ₅₃(B + 400)) × N)))); at least one band at B = 0; the RZ₅₃ product below 1 at B = 3600)
#[test]
fn claim_19_meter_capacity() {
    let rz = glasnost_num::Rounding::TowardZero;
    let rn = glasnost_num::Rounding::Nearest;
    assert_eq!(power::meter(0, 0, 0, 10, rz).0, 1);
    assert_eq!(power::meter(400, 0, 0, 10, rz).0, 5);
    assert_eq!(power::meter(3600, 0, 0, 10, rz).0, 10);
    assert_eq!(power::meter(3600, 0, 0, 10, rn).0, 9);
    assert_eq!(power::meter(1_000_000, 0, 0, 10, rz).0, 10);
}

// spec: S-0405 claim 19 and F4 (supplied A × O/T, near shortage A × V/T for the first 100 units of unmet drain, further shortage A × W/T; full power only the supplied part; T = 0 the supplied extent A)
#[test]
fn claim_19_meter_parts() {
    let rz = glasnost_num::Rounding::TowardZero;
    let i = |e: glasnost_num::F64| e.to_i32_common();
    // N = 20, B = 400: A = 10.
    let (a, e) = power::meter(400, 150, 300, 20, rz);
    assert_eq!(a, 10);
    assert_eq!(e.map(i), [5, 3, 1]);
    let (_, e) = power::meter(400, 50, 100, 20, rz);
    assert_eq!(
        e.map(i),
        [5, 5, 0],
        "below 100 units of shortage: no further part"
    );
    let (_, e) = power::meter(400, 200, 100, 20, rz);
    assert_eq!(e.map(i), [10, 0, 0], "unused output is no part");
    let (_, e) = power::meter(400, 0, 0, 20, rz);
    assert_eq!(e.map(i), [10, 0, 0]);
    let (_, e) = power::meter(400, 0, 200, 20, rz);
    assert_eq!(e.map(i), [0, 5, 5]);
}

fn change_owner(w: &mut World, id: ObjectId, to: HouseId) {
    let old = w.obj(id).unwrap().owner;
    w.obj_mut(id).unwrap().owner = Some(to);
    glasnost_sim::econ::events::owner_changed(w, id, old);
}

// spec: S-0405 claim 8 (manned when required) and S-0515 claim 2 (a building put on the map is not manned)
#[test]
fn claim_8_placed_building_not_manned() {
    let mut w = game(&rules());
    let eng = put(&mut w, "ENG", 0, 0, 0);
    let plain = put(&mut w, "NODRAIN", 0, 2, 0);
    step(&mut w);
    assert!(!power::manned(&w, eng));
    assert!(!power::manned(&w, plain));
    assert!(
        !power::in_operation(&w, eng),
        "never in operation before its first ownership change"
    );
    // Only the needs-engineer flag makes the row read the state.
    assert!(power::in_operation(&w, plain));
    for _ in 0..5 {
        step(&mut w);
    }
    assert!(!power::in_operation(&w, eng));
}

// spec: S-0405 claim 8 (manned when required) and S-0515 claim 2 (manned from the first ownership change of any cause, for good)
#[test]
fn claim_8_manned_by_ownership_change_for_good() {
    let mut w = game(&rules());
    let eng = put(&mut w, "ENG", 0, 0, 0);
    step(&mut w);
    assert!(!power::in_operation(&w, eng));
    // A call that leaves the owner unchanged is no ownership change (glossary).
    change_owner(&mut w, eng, A);
    assert!(!power::manned(&w, eng));
    change_owner(&mut w, eng, HouseId(1));
    assert!(power::manned(&w, eng));
    assert!(power::in_operation(&w, eng));
    step(&mut w);
    assert!(power::in_operation(&w, eng));
    // A second change does not end it, whatever the houses.
    change_owner(&mut w, eng, A);
    assert!(power::manned(&w, eng));
    assert!(power::in_operation(&w, eng));
    // Nor does damage.
    w.obj_mut(eng).unwrap().health = 100;
    step(&mut w);
    assert!(power::manned(&w, eng));
    assert!(power::in_operation(&w, eng));
}

// spec: S-0405 claim 8 and S-0515 claim 2 (the manned state is saved game state: it survives a save/load round trip and enters the state hash)
#[test]
fn claim_8_manned_state_saved_and_hashed() {
    let mut w = game(&rules());
    let eng = put(&mut w, "ENG", 0, 0, 0);
    step(&mut w);
    let unmanned_hash = glasnost_sim::save::state_hash(&w);
    let mut m = w.clone();
    m.obj_mut(eng).unwrap().econ.building.manned = true;
    assert_ne!(
        unmanned_hash,
        glasnost_sim::save::state_hash(&m),
        "the flag is hashed"
    );
    change_owner(&mut w, eng, HouseId(1));
    let bytes = glasnost_sim::save::save(&w);
    let l = glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert!(power::manned(&l, eng));
    assert!(power::in_operation(&l, eng));
    assert_eq!(
        glasnost_sim::save::state_hash(&w),
        glasnost_sim::save::state_hash(&l)
    );
}

/// [`rules`] with the combat ruleset's weapons, projectiles and warheads,
/// and DRONE armed with Gun.
fn armed_rules() -> Ruleset {
    let mut r = rules();
    let c = testkit::combat_ruleset();
    r.sim.weapons = c.sim.weapons.clone();
    r.sim.projectiles = c.sim.projectiles.clone();
    r.sim.warheads = c.sim.warheads.clone();
    for (k, v) in &c.registries {
        if k != "buildings" && k != "vehicles" && k != "infantry" {
            r.registries.insert(k.clone(), v.clone());
        }
    }
    r.sim.vehicles.get_mut("DRONE").unwrap().primary = Some("Gun".into());
    r
}

fn drone(w: &mut World, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(
        w,
        Kind::Vehicle,
        "DRONE",
        house,
        testkit::mid(SIZE, dx, dy),
        0,
    )
}

fn effects_of(w: &World, id: ObjectId) -> (bool, Vec<glasnost_sim::combat::effects::Sound>) {
    use glasnost_sim::combat::effects::Effect;
    let deselected = w
        .effects
        .iter()
        .any(|e| matches!(e, Effect::Deactivated { unit } if *unit == id));
    let pos = w.obj(id).map(|o| o.pos);
    let sounds = w
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::Sound { sound, pos: p, .. } if *p == pos => Some(*sound),
            _ => None,
        })
        .collect();
    (deselected, sounds)
}

// spec: S-0405 claim 20 (revision 20, Q-1503 answered: at disabling a guard reset, deselection, the locomotor switched off and the deactivation sound; the firing check answers illegal for every target (S-0301 claim 9), so it acquires nothing and a force-fire is refused; orders that need a movable or able-to-fire actor are refused when given; it is still a target like any other vehicle)
#[test]
fn claim_20_what_a_disabled_vehicle_cannot_do() {
    use glasnost_sim::combat::effects::Sound;
    use glasnost_sim::world::Target;
    let mut w = game(&armed_rules());
    put(&mut w, "PLANT", 0, 0, 0);
    let k = put(&mut w, "CENTRE", 0, 2, 0);
    let d = drone(&mut w, 0, -4, 4);
    let foe = drone(&mut w, 1, -2, 4);
    for _ in 0..3 {
        step(&mut w);
    }
    let shot =
        |w: &World| glasnost_sim::combat::check::check(w, d, Some(Target::Object(foe)), 0, false);
    assert_ne!(shot(&w), Answer::Illegal, "enabled: a legal target");
    w.effects.clear();
    order(&mut w, A, Order::PowerOff { actor: k });
    let mut seen = (false, Vec::new());
    for _ in 0..3 {
        step(&mut w);
        let (des, s) = effects_of(&w, d);
        seen.0 |= des;
        seen.1.extend(s);
        w.effects.clear();
    }
    assert!(power::unit_disabled(&w, d));
    assert!(seen.0, "deselected");
    assert_eq!(seen.1, [Sound::Deactivate]);
    assert_eq!(shot(&w), Answer::Illegal, "illegal for every target");
    let o = w.obj(d).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert_eq!(o.target, None);
    assert!(
        !glasnost_sim::orders::issue(
            &mut w,
            A,
            Order::Behaviour {
                actor: d,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(foe)),
                dest: None,
            }
        ),
        "a force-fire order is refused"
    );
    assert!(
        !glasnost_sim::orders::issue(&mut w, A, Order::Stop { actor: d }),
        "a stop order is refused"
    );
    assert!(!glasnost_sim::targeting::movable(&w, A, d));
    assert!(!glasnost_sim::targeting::able_to_fire(&w, A, d));
    assert!(!glasnost_sim::targeting::box_eligible(&w, A, &[], d));
    for _ in 0..60 {
        step(&mut w);
    }
    assert_eq!(w.obj(d).unwrap().target, None, "it acquires no target");
    assert!(
        glasnost_sim::combat::check::check(&w, foe, Some(Target::Object(d)), 0, false)
            != Answer::Illegal,
        "it is a target like any other vehicle"
    );
    // Enabling: the activation sound and a guard reset.
    order(&mut w, A, Order::PowerOn { actor: k });
    let mut sounds = Vec::new();
    for _ in 0..3 {
        step(&mut w);
        sounds.extend(effects_of(&w, d).1);
        w.effects.clear();
    }
    assert!(!power::unit_disabled(&w, d));
    assert_eq!(sounds, [Sound::Activate]);
}

// spec: S-0301 claim 9 (Deactivated firers within rank 4: the deactivated refusal outranks the absent-weapon impossible) and S-0405 claim 20
#[test]
fn claim_20_deactivated_illegal_outranks_absent_weapon() {
    use glasnost_sim::world::Target;
    let mut w = game(&armed_rules());
    put(&mut w, "PLANT", 0, 0, 0);
    let k = put(&mut w, "CENTRE", 0, 2, 0);
    let d = drone(&mut w, 0, -4, 4);
    let foe = drone(&mut w, 1, -2, 4);
    for _ in 0..3 {
        step(&mut w);
    }
    // Slot 1 holds no weapon.
    let empty =
        |w: &World| glasnost_sim::combat::check::check(w, d, Some(Target::Object(foe)), 1, false);
    assert_eq!(
        empty(&w),
        Answer::Impossible,
        "enabled: no weapon in the slot"
    );
    order(&mut w, A, Order::PowerOff { actor: k });
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(power::unit_disabled(&w, d));
    assert_eq!(empty(&w), Answer::Illegal, "deactivated: illegal first");
}

// spec: S-0301 claim 9 rank 4 (a deactivated firer that is materialising answers rearming, not illegal) and S-0207 claim 20 (f) (phasing in)
#[test]
fn claim_20_deactivated_firer_phasing_in_answers_rearming() {
    use glasnost_sim::teleport::TeleportState;
    use glasnost_sim::world::Target;
    let mut w = game(&armed_rules());
    put(&mut w, "PLANT", 0, 0, 0);
    let k = put(&mut w, "CENTRE", 0, 2, 0);
    let d = drone(&mut w, 0, -4, 4);
    let foe = drone(&mut w, 1, -2, 4);
    for _ in 0..3 {
        step(&mut w);
    }
    order(&mut w, A, Order::PowerOff { actor: k });
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(power::unit_disabled(&w, d));
    let ask =
        |w: &World| glasnost_sim::combat::check::check(w, d, Some(Target::Object(foe)), 0, false);
    assert_eq!(ask(&w), Answer::Illegal, "deactivated, not phasing in");
    let t = w.tick;
    w.obj_mut(d).unwrap().teleport = Some(TeleportState {
        phasing: Some((t, 10)),
        ..Default::default()
    });
    assert!(glasnost_sim::teleport::phasing_in(&w, d));
    assert_eq!(ask(&w), Answer::Rearming, "deactivated and phasing in");
}

// spec: S-0405 claim 20 (at enabling every vehicle claim 13 enables gets the guard reset, also one that was not disabled; it plays no activation sound) and claim 13 (a vehicle off the map at either transition is passed over and keeps its state)
#[test]
fn claim_20_enabling_resets_an_enabled_vehicle_and_skips_off_map() {
    let mut r = rules();
    r.globals.quirks.shared_control_centre_count = false;
    let mut w = game(&r);
    put(&mut w, "PLANT", 0, 0, 0);
    let k = put(&mut w, "CENTRE", 0, 2, 0);
    for _ in 0..3 {
        step(&mut w);
    }
    order(&mut w, A, Order::PowerOff { actor: k });
    for _ in 0..3 {
        step(&mut w);
    }
    // Two vehicles that arrive while the centre is off: placement is not an arrival, so both stay enabled.
    let on_map = drone(&mut w, 0, -4, 4);
    let off_map = drone(&mut w, 0, -4, 6);
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(
        !power::unit_disabled(&w, on_map),
        "placement is not an arrival"
    );
    w.obj_mut(off_map).unwrap().econ.building.unit_disabled = true;
    glasnost_sim::objects::take_inside(&mut w, off_map);
    let spot = glasnost_sim::world::Target::Cell(testkit::mid(SIZE, 0, 9));
    w.obj_mut(on_map).unwrap().guard_spot = Some(spot);
    order(&mut w, A, Order::PowerOn { actor: k });
    w.effects.clear();
    for _ in 0..3 {
        step(&mut w);
    }
    let o = w.obj(on_map).unwrap();
    assert_eq!(o.guard_spot, None, "the guard reset at enabling");
    assert!(effects_of(&w, on_map).1.is_empty(), "no activation sound");
    assert!(
        power::unit_disabled(&w, off_map),
        "off the map at enabling: passed over"
    );
}

// spec: S-0405 claim 13 (revision 20, Q-1503 answered: at the 1-to-0 transition a vehicle tethered to the building standing in its own cell is passed over and stays enabled)
#[test]
fn claim_13_tethered_vehicle_is_passed_over() {
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0, 0, 0);
    let k = put(&mut w, "CENTRE", 0, 2, 0);
    let f = put(&mut w, "CONS", 0, -4, 4);
    let d = drone(&mut w, 0, -4, 4);
    for _ in 0..3 {
        step(&mut w);
    }
    {
        let o = w.obj_mut(d).unwrap();
        o.contact = Some(f);
        o.econ.exit = Some(glasnost_sim::econ::release::Exit {
            factory: f,
            tethered: true,
            place: 0,
        });
    }
    assert!(power::tethered_in_own_cell(&w, d));
    order(&mut w, A, Order::PowerOff { actor: k });
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(
        !power::unit_disabled(&w, d),
        "tethered in its factory's cell"
    );
}

// spec: S-0405 claim 10 (revision 36: an ownership change leaves the power switch as it is: a building switched off stays off for its new owner, and one switched on stays on; a switched-off building makes no output and needs no drain)
#[test]
fn claim_10_ownership_change_keeps_the_switch() {
    let mut w = game(&rules());
    let b = HouseId(1);
    put(&mut w, "PLANT", 0, 0, 0);
    put(&mut w, "PLANT", 1, 0, 6);
    let off = put(&mut w, "CONS", 0, 2, 0);
    let on = put(&mut w, "CONS", 0, 4, 0);
    power::switch(&mut w, off, false);
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(w.obj(off).unwrap().econ.building.switched_off);
    assert_eq!(power::totals(&w, b), (120, 0));
    change_owner(&mut w, off, b);
    change_owner(&mut w, on, b);
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(
        w.obj(off).unwrap().econ.building.switched_off,
        "off stays off for its new owner"
    );
    assert!(
        !w.obj(on).unwrap().econ.building.switched_off,
        "on stays on"
    );
    assert_eq!(
        power::totals(&w, b),
        (120, 40),
        "only the switched-on building drains for its new owner"
    );
}

/// [`rules`] with DRONE3, a vehicle type without the powered-unit flag,
/// and CENTRE3, a control centre whose powered unit is DRONE3.
fn unflagged_rules() -> Ruleset {
    let mut r = rules();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("CENTRE3".into());
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("DRONE3".into());
    let mut c = building(500, -10);
    c.powered = Some(true);
    c.econ.powers_unit = Some("DRONE3".into());
    r.sim.buildings.insert("CENTRE3".into(), c);
    let mut v = r.sim.vehicles["DRONE"].clone();
    v.econ.powered_unit = Some(false);
    r.sim.vehicles.insert("DRONE3".into(), v);
    r
}

/// Gives a disabled vehicle a destination as a request accepted after
/// the disabling would (claim 20: "When an accepted request supplies a
/// destination, it moves there ... still disabled"), and runs until it
/// has none or 200 ticks pass. Returns whether it was deselected again,
/// the sounds at it, and the ticks run.
fn moved_while_disabled(
    w: &mut World,
    id: ObjectId,
    to: glasnost_sim::world::Target,
) -> (bool, Vec<glasnost_sim::combat::effects::Sound>, i32) {
    {
        let o = w.obj_mut(id).unwrap();
        o.dest = Some(to);
        o.path.steps.clear();
        o.beh.queue(Some(Behaviour::Move));
    }
    w.effects.clear();
    let mut seen = (false, Vec::new());
    for n in 1..=200 {
        step(w);
        let (des, s) = effects_of(w, id);
        seen.0 |= des;
        seen.1.extend(s);
        w.effects.clear();
        if w.obj(id).unwrap().dest.is_none() {
            return (seen.0, seen.1, n);
        }
    }
    (seen.0, seen.1, 200)
}

// spec: S-0405 claim 20 (revision 36: a disabled vehicle that is given a destination moves there still disabled; its arrival is an arrival in claim 13's sense, so with the powered-unit flag, no building in the cell and no active control centre, it gets the disabling's guard reset and locomotor switch-off again, with no sound; one whose type lacks the flag gets no such second switch-off) and claim 13 (the cell-entry rule)
#[test]
fn claim_20_arrival_of_a_disabled_vehicle_disables_it_again() {
    use glasnost_sim::world::Target;
    let mut r = unflagged_rules();
    // A count per powered-unit type, so that each centre's own transition disables its vehicles.
    r.globals.quirks.shared_control_centre_count = false;
    let mut w = game(&r);
    put(&mut w, "PLANT", 0, 0, 0);
    let k = put(&mut w, "CENTRE", 0, 2, 0);
    let k3 = put(&mut w, "CENTRE3", 0, 4, 0);
    let d = drone(&mut w, 0, -6, 4);
    let d3 = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "DRONE3",
        0,
        testkit::mid(SIZE, -6, 8),
        0,
    );
    for _ in 0..3 {
        step(&mut w);
    }
    order(&mut w, A, Order::PowerOff { actor: k });
    order(&mut w, A, Order::PowerOff { actor: k3 });
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(power::unit_disabled(&w, d));
    assert!(
        power::unit_disabled(&w, d3),
        "the 1-to-0 transition disables it whatever its flag"
    );
    // With the flag: the first arrival in a cell without a building resets it again.
    let start = w.obj(d).unwrap().pos.cell();
    let far = Target::Cell(testkit::mid(SIZE, 6, 4));
    let (deselected, sounds, _) = moved_while_disabled(&mut w, d, far);
    let o = w.obj(d).unwrap();
    assert_eq!(o.dest, None, "the guard reset cleared the destination");
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert_ne!(o.pos.cell(), start, "after an arrival of its own movement");
    assert_ne!(
        o.pos.cell(),
        testkit::mid(SIZE, 6, 4),
        "well before the destination"
    );
    assert!(deselected, "the disabling's removal from the selection");
    assert!(sounds.is_empty(), "no sound: it was already disabled");
    assert!(power::unit_disabled(&w, d), "still disabled");
    // Without the flag: no second switch-off, it reaches its destination.
    let to = testkit::mid(SIZE, 6, 8);
    let (deselected, sounds, n) = moved_while_disabled(&mut w, d3, Target::Cell(to));
    assert!(n < 200, "it arrives");
    assert_eq!(
        w.obj(d3).unwrap().pos.cell(),
        to,
        "it reaches its destination"
    );
    assert!(!deselected, "no second switch-off");
    assert!(sounds.is_empty());
    assert!(power::unit_disabled(&w, d3));
}

// spec: S-0405 claim 20 (at disabling the locomotor is switched off, and at enabling switched on again; S-0207 claim 16 refuses a scatter request in case (g) while the vehicle's locomotor is switched off) and S-0207 claim 16 (g): a disabled drive vehicle refuses a scatter request, which it accepts enabled
#[test]
fn claim_20_disabled_drive_vehicle_refuses_a_scatter_request() {
    use glasnost_sim::scatter::{ScatterRequest, vehicle_request};
    let mut w = game(&rules());
    put(&mut w, "PLANT", 0, 0, 0);
    let k = put(&mut w, "CENTRE", 0, 2, 0);
    let d = drone(&mut w, 0, -4, 4);
    for _ in 0..3 {
        step(&mut w);
    }
    let req = ScatterRequest {
        threat: None,
        forced: true,
        anywhere: false,
    };
    assert!(vehicle_request(&mut w.clone(), d, req), "enabled: accepted");
    order(&mut w, A, Order::PowerOff { actor: k });
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(power::unit_disabled(&w, d));
    assert!(
        !vehicle_request(&mut w.clone(), d, req),
        "disabled: refused by case (g)"
    );
    order(&mut w, A, Order::PowerOn { actor: k });
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(vehicle_request(&mut w.clone(), d, req), "enabled again");
}

// spec: S-0405 claim 8 (r57, Q-3060: the last row reads the current behaviour, or the queued one while none is current; while a behaviour is current the queued one is not read)
#[test]
fn claim_8_r57_queued_behaviour_read_while_none_is_current() {
    let mut w = game(&rules());
    let nd = put(&mut w, "NODRAIN", 0, 2, 0);
    step(&mut w);
    assert!(power::in_operation(&w, nd));
    {
        let o = w.obj_mut(nd).unwrap();
        o.beh.current = None;
        o.beh.queued = Some(Behaviour::Construction);
    }
    assert!(!power::in_operation(&w, nd), "queued construction, none current");
    w.obj_mut(nd).unwrap().beh.queued = Some(Behaviour::Selling);
    assert!(!power::in_operation(&w, nd), "queued selling, none current");
    w.obj_mut(nd).unwrap().beh.current = Some(Behaviour::Guard);
    assert!(power::in_operation(&w, nd), "a current guard hides the queue");
}

// spec: S-0405 claim 8 and Transitions (r57, Q-3060: a sell order makes selling the building's current behaviour at once, whatever was current, so it is not in operation from the order on; S-0408 claim 4)
#[test]
fn claim_8_r57_sell_order_ends_operation_at_once() {
    for before in [Behaviour::Guard, Behaviour::Construction] {
        let mut r = rules();
        // A build-up sprite, so that a sell order sells (S-0408 claim 4).
        r.sim.buildings.get_mut("NODRAIN").unwrap().econ.buildup = Some(true);
        let mut w = game(&r);
        let nd = put(&mut w, "NODRAIN", 0, 2, 0);
        step(&mut w);
        let t = w.tick;
        w.obj_mut(nd).unwrap().beh.force(before, t, true);
        glasnost_sim::econ::sell::sell(&mut w, A, nd);
        let o = w.obj(nd).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Selling), "{before:?}");
        assert_eq!(o.beh.queued, None, "never only queued");
        assert!(!power::in_operation(&w, nd), "{before:?}");
    }
}
