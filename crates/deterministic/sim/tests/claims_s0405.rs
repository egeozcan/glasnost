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
/// ENG (needs an engineer), ABS (absorbing, extra output 30), XTRA (output
/// 100, extra output 30), UPx upgrades, CENTRE (control centre for DRONE),
/// GAPPER (super-gap radius, extra drain 20); vehicle DRONE (powered unit).
fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    let names = [
        "PLANT", "CONS", "DEF", "NODRAIN", "SPECIAL", "ENG", "ABS", "XTRA", "UP10", "UP20", "UPD5",
        "CENTRE", "CENTRE2", "GAPPER", "SMALL", "WALLY",
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

// spec: S-0405 claims 1 and 2, Terms (upgrades in the three positions count; a fourth adds nothing; W12)
#[test]
fn claims_1_2_upgrades() {
    let r = rules();
    let mut w = game(&r);
    let p = put(&mut w, "PLANT", 0, 0, 0);
    let idx = |n: &str| w.rules().index_of(Kind::Building, n).unwrap();
    let (u10, u20, ud5) = (idx("UP10"), idx("UP20"), idx("UPD5"));
    w.obj_mut(p).unwrap().econ.building.upgrades = vec![u10];
    assert_eq!(power::building_output(&w, p), 130);
    w.obj_mut(p).unwrap().econ.building.upgrades = vec![u10, u20, ud5];
    assert_eq!(power::building_output(&w, p), 150);
    assert_eq!(power::building_drain(&w, p), 5);
    w.obj_mut(p).unwrap().econ.building.upgrades = vec![u10, u20, ud5, u20];
    assert_eq!(power::building_output(&w, p), 150);
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

// spec: S-0405 claim 8 (the in-operation test: switch, low power with drain, no drain, special power and blackout, engineer, construction and selling)
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
    // The needs-engineer type works only with an engineer inside (none: a seam).
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
    // The current behaviour construction or selling, not the queued one.
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
