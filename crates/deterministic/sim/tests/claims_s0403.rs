//! Claim tests for S-0403 (build time) on synthetic data: invented costs,
//! factors and game-wide values, no stock numbers.

mod econ_common;

use econ_common::*;
use glasnost_num::{F64, Rounding};
use glasnost_sim::econ::production::{self, TimeInputs, build_time_of, period_of};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::HouseId;

const RZ: Rounding = Rounding::TowardZero;

fn one() -> F64 {
    F64::from_i64(1, RZ)
}

/// A double from decimal text (the value of a double field).
fn d(s: &str) -> F64 {
    glasnost_num::decimal::parse(s, glasnost_num::decimal::Field::Decimal, RZ).unwrap()
}

/// The single nearest to decimal text, widened (a single field).
fn s(s: &str) -> F64 {
    glasnost_num::decimal::parse(s, glasnost_num::decimal::Field::Single, RZ).unwrap()
}

fn rules_default() -> production::Rules {
    production::Rules::new(&ruleset(), RZ)
}

fn inputs(cost: i32) -> TimeInputs {
    TimeInputs {
        cost,
        country: one(),
        multiplier: one(),
        power: one(),
        factories: 1,
        wall: false,
    }
}

fn time(r: &production::Rules, x: TimeInputs) -> i32 {
    build_time_of(r, x, RZ)
}

// spec: S-0403 claim 1 (R = min(255, max(1, T ÷ 54)); the B1 cases)
#[test]
fn claim_1_step_period() {
    assert_eq!(period_of(5400, true), 100);
    assert_eq!(period_of(5401, true), 100);
    assert_eq!(period_of(5453, true), 100);
    assert_eq!(period_of(100, true), 1);
    assert_eq!(period_of(54, true), 1);
    assert_eq!(period_of(20000, true), 255);
    assert_eq!(period_of(-7, true), 1);
    // spec: S-0403 Edge cases (quirks.build_step_period_cap off: no upper limit)
    assert_eq!(period_of(20000, false), 370);
}

// spec: S-0403 claim 1 (a step every R ticks, the first R ticks after the order's tick; done in tick n + 54R)
#[test]
fn claim_1_done_tick() {
    let mut w = game(&ruleset());
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let tank = t(&w, Kind::Vehicle, "TANK");
    let n = w.tick;
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let i = item(&w, 0, tank).expect("started");
    // C = 1000, S = 1.0: T = 900, R = 16 (S-0403 Formulas, example).
    assert_eq!(i.period, 16);
    let mut steps = Vec::new();
    let mut last = 0;
    let mut done = None;
    for _ in 0..(54 * 16) {
        step(&mut w);
        if let Some(i) = item(&w, 0, tank)
            && i.progress != last
        {
            last = i.progress;
            steps.push(w.tick - 1);
            done = done.or(i.done_tick);
        }
    }
    assert_eq!(steps.len(), 54);
    assert_eq!(steps[0], n + 16);
    assert!(steps.windows(2).all(|p| p[1] - p[0] == 16));
    assert_eq!(done, Some(n + 54 * 16));
}

// spec: S-0403 claim 2 (T₀ = trunc(RZ₅₃(RZ₅₃(C × S) × k₀)); B2)
#[test]
fn claim_2_base_time() {
    let mut r = rules_default();
    assert_eq!(time(&r, inputs(1000)), 900);
    assert_eq!(time(&r, inputs(6000)), 5400);
    for (speed, cost, want) in [("0.7", 1000, 630), ("1.3", 1000, 1170), ("1.0", 999, 899)] {
        r.build_speed = s(speed);
        let x = time(&r, inputs(cost));
        // The single nearest the decimal, widened, then F1.
        let f = |v: i32| F64::from_i64(i64::from(v), RZ);
        let expect = f(cost).mul(s(speed), RZ).mul(r.k0, RZ).to_i32_common();
        assert_eq!(x, expect);
        assert!((x - want).abs() <= 1, "{speed} {cost}: {x}");
    }
}

// spec: S-0403 claim 2 (the price a house pays has no effect on build time)
#[test]
fn claim_2_price_independent() {
    let mut r0 = ruleset();
    let mut w = game(&r0);
    building(&mut w, "WARFACT", 0, 0, 0);
    let tank = t(&w, Kind::Vehicle, "TANK");
    let plain = production::build_time(&w, HouseId(0), tank);
    r0.sim
        .countries
        .get_mut("Alphaland")
        .unwrap()
        .econ
        .cost_units_mult = dec("0.5");
    let mut w2 = game(&r0);
    building(&mut w2, "WARFACT", 0, 0, 0);
    assert_eq!(production::price(&w2, HouseId(0), tank), 500);
    assert_eq!(production::build_time(&w2, HouseId(0), tank), plain);
}

// spec: S-0403 claim 3 (the country's factor per kind; ground and naval vehicles share one; B3)
#[test]
fn claim_3_country_factors() {
    let mut r = ruleset();
    let c = &mut r.sim.countries.get_mut("Alphaland").unwrap().econ;
    c.build_time_infantry_mult = dec("1.5");
    c.build_time_units_mult = dec("1.25");
    c.build_time_aircraft_mult = dec("0.625");
    c.build_time_buildings_mult = dec("1.125");
    c.build_time_defenses_mult = dec("0.75");
    let w = game(&r);
    let h = HouseId(0);
    let bt = |k, n| production::build_time(&w, h, t(&w, k, n));
    // C = 1000 everywhere: T₀ = 900.
    assert_eq!(bt(Kind::Infantry, "GRUNT"), 1350);
    assert_eq!(bt(Kind::Vehicle, "TANK"), 1125);
    assert_eq!(bt(Kind::Vehicle, "BOAT"), 1125);
    assert_eq!(bt(Kind::Aircraft, "PLANE"), 562);
    assert_eq!(bt(Kind::Building, "HUT"), 1012);
    assert_eq!(bt(Kind::Building, "TOWER"), 675);
    // The other house's country keeps factors of 1.
    assert_eq!(
        production::build_time(&w, HouseId(1), t(&w, Kind::Building, "TOWER")),
        900
    );
}

// spec: S-0403 claim 4 (T₂ = trunc(RZ₅₃(T₁ × m)), m a single)
#[test]
fn claim_4_type_multiplier() {
    let mut r = ruleset();
    r.sim
        .vehicles
        .get_mut("TANK")
        .unwrap()
        .econ
        .build_time_multiplier = dec("1.3");
    r.sim
        .countries
        .get_mut("Alphaland")
        .unwrap()
        .econ
        .build_time_units_mult = dec("1.25");
    let w = game(&r);
    // T₁ = 1125; 1125 × (the single just below 1.3) = 1462.49…
    assert_eq!(
        production::build_time(&w, HouseId(0), t(&w, Kind::Vehicle, "TANK")),
        1462
    );
}

// spec: S-0403 claim 5 (F4 with the absent-key defaults; B4)
#[test]
fn claim_5_low_power() {
    let r = rules_default();
    let at = |p: F64, cost: i32| {
        time(
            &r,
            TimeInputs {
                power: p,
                ..inputs(cost)
            },
        )
    };
    // Full power: f = 1.
    assert_eq!(at(one(), 6000), 5400);
    // Half power: f = 0.5 (the Formulas example: T₃ = 1800, R = 33).
    assert_eq!(at(d("0.5"), 1000), 1800);
    assert_eq!(period_of(at(d("0.5"), 1000), true), 33);
    // 0.99: f held at the single nearest 0.9, about 11 % slower.
    assert_eq!(at(d("0.99"), 6000), 6000);
    // 0.75 inside the limits.
    assert_eq!(at(d("0.75"), 6000), 7200);
    // 0.3 and 0: f held at 0.5.
    assert_eq!(at(d("0.3"), 6000), 10800);
    assert_eq!(at(F64::ZERO, 6000), 10800);
}

// spec: S-0403 claim 5 (pₛ narrowed to a single with RZ₂₄: B4 part c, R = 205 not 204)
#[test]
fn claim_5_narrowing() {
    let r = rules_default();
    let p = F64::from_i64(2814, RZ).div(F64::from_i64(4217, RZ), RZ);
    let x = TimeInputs {
        power: p,
        ..inputs(8208)
    };
    let t2 = time(&r, inputs(8208));
    assert_eq!(t2, 7387);
    let t3 = time(&r, x);
    assert_eq!(t3, 11070);
    assert_eq!(period_of(t3, true), 205);
}

// spec: S-0403 claim 5 (the order of the limits: the lower always, the upper only under a deficit; the 0.01 replacement)
#[test]
fn claim_5_limits() {
    let mut r = rules_default();
    // q = 0.5, fmin = 0.2, fmax = 1.0 (B4 second set).
    r.penalty = s("0.5");
    r.min_speed = s("0.2");
    r.max_speed = one();
    let at = |r: &production::Rules, p: F64| {
        time(
            r,
            TimeInputs {
                power: p,
                ..inputs(6000)
            },
        )
    };
    // f₁ = 1 − 0.5 × 0.5 = 0.75.
    assert_eq!(at(&r, d("0.5")), 7200);
    // fmin above fmax: a deficit gives fmax.
    r.min_speed = s("0.8");
    r.max_speed = s("0.625");
    assert_eq!(at(&r, d("0.5")), 8640);
    // At full power only the lower limit applies: fmin above 1 slows full power too.
    r.min_speed = s("1.25");
    assert_eq!(at(&r, one()), 4320);
    // f₂ = 0 under a deficit: the single nearest 0.01.
    r.min_speed = F64::ZERO;
    r.max_speed = F64::ZERO;
    let z = at(&r, d("0.5"));
    let expect = F64::from_i64(5400, RZ).div(s("0.01"), RZ).to_i32_common();
    assert_eq!(z, expect);
}

// spec: S-0403 claim 6 (× g once per factory beyond the first; B5)
#[test]
fn claim_6_formula() {
    let mut r = rules_default();
    r.multiple_factory = s("0.8");
    let at = |r: &production::Rules, n: i32| {
        time(
            r,
            TimeInputs {
                factories: n,
                ..inputs(6000)
            },
        )
    };
    assert_eq!(at(&r, 1), 5400);
    assert_eq!(at(&r, 2), 4320);
    assert_eq!(at(&r, 3), 3456);
    assert_eq!(at(&r, 4), 2764);
    r.multiple_factory = F64::ZERO;
    assert_eq!(at(&r, 4), 5400);
}

// spec: S-0403 claim 6 (the per-kind count: naval and ground factories apart, unpowered factories and other owners' types counted)
#[test]
fn claim_6_counting() {
    let mut r = ruleset();
    r.sim.general.economy.multiple_factory = dec("0.8");
    // A vehicle factory type that only Betaland may use.
    let mut foreign = factory("vehicle");
    foreign.owner = Some(vec!["Betaland".into()]);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("FOREIGN".into());
    r.sim.buildings.insert("FOREIGN".into(), foreign);
    let mut tank = r.sim.vehicles["TANK"].clone();
    tank.owner = Some(vec!["Alphaland".into()]);
    r.sim.vehicles.insert("TANK".into(), tank);
    let mut w = game(&r);
    let h = HouseId(0);
    let tank = t(&w, Kind::Vehicle, "TANK");
    let boat = t(&w, Kind::Vehicle, "BOAT");
    building(&mut w, "WARFACT", 0, -6, 0);
    assert_eq!(production::factory_count(&w, h, tank), 1);
    let second = building(&mut w, "WARFACT", 0, -3, 0);
    building(&mut w, "NAVYARD", 0, 3, 0);
    assert_eq!(production::factory_count(&w, h, tank), 2);
    assert_eq!(production::factory_count(&w, h, boat), 1);
    switch(&mut w, second, false);
    assert_eq!(production::factory_count(&w, h, tank), 2);
    building(&mut w, "FOREIGN", 0, 0, 4);
    // spec: S-0403 Edge cases (quirks.multiple_factory_counts_all: the foreign factory counts)
    assert_eq!(production::factory_count(&w, h, tank), 3);
    assert_eq!(production::build_time(&w, h, tank), 576);
}

// spec: S-0403 Edge cases (quirks.multiple_factory_counts_all off: only factories that could make the item)
#[test]
fn claim_6_quirk_off() {
    let mut r = ruleset();
    r.globals.quirks.multiple_factory_counts_all = false;
    let mut foreign = factory("vehicle");
    foreign.owner = Some(vec!["Betaland".into()]);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("FOREIGN".into());
    r.sim.buildings.insert("FOREIGN".into(), foreign);
    let mut tank = r.sim.vehicles["TANK"].clone();
    tank.owner = Some(vec!["Alphaland".into()]);
    r.sim.vehicles.insert("TANK".into(), tank);
    let mut w = game(&r);
    let tank = t(&w, Kind::Vehicle, "TANK");
    building(&mut w, "WARFACT", 0, -6, 0);
    building(&mut w, "FOREIGN", 0, 0, 4);
    assert_eq!(production::factory_count(&w, HouseId(0), tank), 1);
}

// spec: S-0403 claim 7 (a wall: × w after every other factor; B6)
#[test]
fn claim_7_walls() {
    let mut r = rules_default();
    let wall = |r: &production::Rules, on: bool| {
        time(
            r,
            TimeInputs {
                wall: on,
                ..inputs(6000)
            },
        )
    };
    assert_eq!(wall(&r, false), 5400);
    assert_eq!(wall(&r, true), 2700);
    r.wall = d("0.3");
    let expect = F64::from_i64(5400, RZ).mul(d("0.3"), RZ).to_i32_common();
    assert_eq!(wall(&r, true), expect);
    assert_eq!(wall(&r, false), 5400);
    // Through the rules: a wall building type.
    let mut rs = ruleset();
    rs.sim.buildings.get_mut("PLANT").unwrap().econ.wall = Some(true);
    rs.sim.buildings.get_mut("PLANT").unwrap().cost = Some(1000);
    let w = game(&rs);
    assert_eq!(
        production::build_time(&w, HouseId(0), t(&w, Kind::Building, "PLANT")),
        450
    );
}

// spec: S-0403 claim 8 (R worked out at the start and at a resume; the next step a full R after the resume; B7 (c))
#[test]
fn claim_8_resume() {
    let mut w = game(&ruleset());
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let start = item(&w, 0, tank).unwrap().due - 16;
    // Hold 10 ticks into the second step, resume 50 ticks later.
    while w.tick < start + 16 + 10 {
        step(&mut w);
    }
    assert!(issue(&mut w, 0, Order::Hold { what: tank }));
    step(&mut w);
    for _ in 0..50 {
        step(&mut w);
    }
    let progress = item(&w, 0, tank).unwrap().progress;
    let resume_tick = w.tick;
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    assert_eq!(item(&w, 0, tank).unwrap().due, resume_tick + 16);
    assert_eq!(item(&w, 0, tank).unwrap().progress, progress);
}

// spec: S-0403 Edge cases (quirks.resume_restarts_step off: the resumed line keeps the ticks that remained)
#[test]
fn claim_8_resume_quirk_off() {
    let mut r = ruleset();
    r.globals.quirks.resume_restarts_step = false;
    let mut w = game(&r);
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let due = item(&w, 0, tank).unwrap().due;
    // Hold at the end of tick due − 6: 6 ticks remain.
    while w.tick < due - 6 {
        step(&mut w);
    }
    assert!(issue(&mut w, 0, Order::Hold { what: tank }));
    step(&mut w);
    for _ in 0..30 {
        step(&mut w);
    }
    let resume_tick = w.tick;
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    assert_eq!(item(&w, 0, tank).unwrap().due, resume_tick + 6);
}

// spec: S-0403 claim 8 (a new factory changes R only at the house's power re-evaluation; the step under way keeps its period; B7 (b))
// spec: S-0405 claims 7 and 11 (placing a building makes the house due; working out its power re-evaluates the lines' periods)
#[test]
fn claim_8_refresh() {
    let mut r = ruleset();
    r.sim.general.economy.multiple_factory = dec("0.5");
    let mut w = game(&r);
    building(&mut w, "WARFACT", 0, -4, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let before = item(&w, 0, tank).unwrap();
    assert_eq!(before.period, 16);
    // A second factory, counted from now on, but R is not worked out until
    // the power re-evaluation: a direct call to the counting gives 8 while
    // the line still holds 16.
    building(&mut w, "WARFACT", 0, 2, 0);
    assert_eq!(production::period(&w, HouseId(0), tank), 8);
    assert_eq!(item(&w, 0, tank).unwrap().period, 16);
    let due = item(&w, 0, tank).unwrap().due;
    // The placement made the house due (S-0405 claim 7): its next house
    // update works out power and then every line's period (S-0405 claim 11).
    step(&mut w);
    let after = item(&w, 0, tank).unwrap();
    // T = 450: R = 8; the step under way keeps its due tick.
    assert_eq!(after.period, 8);
    assert_eq!(after.due, due);
}

// spec: S-0403 claim 9 (game speed, the difficulty and general factors change no build time)
#[test]
fn claim_9_no_other_factors() {
    let mut w = game(&ruleset());
    let tank = t(&w, Kind::Vehicle, "TANK");
    let a = production::period(&w, HouseId(0), tank);
    w.game_speed = 1;
    let b = production::period(&w, HouseId(0), tank);
    w.game_speed = 6;
    let c = production::period(&w, HouseId(0), tank);
    assert_eq!((a, b, c), (16, 16, 16));
}

// spec: S-0403 claim 10 (identical rules for every house: human or AI)
#[test]
fn claim_10_every_house() {
    let mut w = game(&ruleset());
    building(&mut w, "WARFACT", 0, -4, 0);
    building(&mut w, "WARFACT", 1, 4, 0);
    w.houses[1].human = false;
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert_eq!(
        production::build_time(&w, HouseId(0), tank),
        production::build_time(&w, HouseId(1), tank)
    );
}
