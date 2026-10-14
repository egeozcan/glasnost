//! Claim tests for S-0404 (production queue and payment) on synthetic data:
//! invented costs, factors and types, no stock numbers.

mod econ_common;

use econ_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::production::{self, Hold, LineKind, payment};
use glasnost_sim::objects;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::HouseId;

fn add_building(r: &mut Ruleset, n: &str, b: glasnost_rules::sim::BuildingType) {
    r.registries.get_mut("buildings").unwrap().push(n.into());
    r.sim.buildings.insert(n.into(), b);
}

/// A building type that grants a vehicle cost bonus.
fn bonus_plant(units: &str) -> glasnost_rules::sim::BuildingType {
    let mut b = plain(100);
    b.econ.factory_plant = Some(true);
    b.econ.units_cost_bonus = dec(units);
    b
}

fn rich(w: &mut glasnost_sim::world::World) {
    set_credits(w, 0, 1_000_000);
}

// spec: S-0404 claim 1 (six independent lines; P1)
#[test]
fn claim_1_six_lines() {
    let mut w = game(&ruleset());
    for (n, dx) in [
        ("BARRACKS", -9),
        ("WARFACT", -6),
        ("NAVYARD", -3),
        ("AIRPAD", 0),
        ("CONYARD", 3),
    ] {
        building(&mut w, n, 0, dx, 0);
    }
    ready(&mut w);
    rich(&mut w);
    let types = [
        t(&w, Kind::Infantry, "GRUNT"),
        t(&w, Kind::Vehicle, "TANK"),
        t(&w, Kind::Vehicle, "BOAT"),
        t(&w, Kind::Aircraft, "PLANE"),
        t(&w, Kind::Building, "HUT"),
        t(&w, Kind::Building, "TOWER"),
    ];
    for ty in types {
        assert!(issue(&mut w, 0, Order::Build { what: ty }));
    }
    step(&mut w);
    assert_eq!(w.econ.lines.lines.len(), 6);
    let kinds: Vec<LineKind> = w.econ.lines.lines.iter().map(|l| l.kind).collect();
    assert_eq!(
        kinds,
        vec![
            LineKind::Infantry,
            LineKind::Ground,
            LineKind::Naval,
            LineKind::Aircraft,
            LineKind::Building,
            LineKind::Defence
        ]
    );
    for _ in 0..40 {
        step(&mut w);
    }
    for ty in types {
        assert!(item(&w, 0, ty).unwrap().progress >= 2);
    }
}

// spec: S-0404 claim 1 (lines updated in creation order: the earlier line is paid when only one step can be; P1 part B)
#[test]
fn claim_1_update_order() {
    for first_vehicle in [false, true] {
        let mut w = game(&ruleset());
        building(&mut w, "BARRACKS", 0, -4, 0);
        building(&mut w, "WARFACT", 0, 2, 0);
        ready(&mut w);
        let grunt = t(&w, Kind::Infantry, "GRUNT");
        let tank = t(&w, Kind::Vehicle, "TANK");
        let order = if first_vehicle {
            [tank, grunt]
        } else {
            [grunt, tank]
        };
        for ty in order {
            assert!(issue(&mut w, 0, Order::Build { what: ty }));
        }
        // Equal cost and factors: equal R; the first steps pay 18 each.
        set_credits(&mut w, 0, 18);
        step(&mut w);
        for _ in 0..16 {
            step(&mut w);
        }
        let paid_first = item(&w, 0, order[0]).unwrap().progress;
        let paid_second = item(&w, 0, order[1]).unwrap().progress;
        assert_eq!(
            (paid_first, paid_second),
            (1, 0),
            "vehicle first: {first_vehicle}"
        );
        assert!(item(&w, 0, order[1]).unwrap().waiting);
    }
}

// spec: S-0404 claim 1 (a line ceases with neither item nor queue; created again it counts as new, after the others)
#[test]
fn claim_1_recreated_line() {
    let mut w = game(&ruleset());
    building(&mut w, "BARRACKS", 0, -4, 0);
    building(&mut w, "WARFACT", 0, 2, 0);
    ready(&mut w);
    rich(&mut w);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    assert!(issue(&mut w, 0, Order::Cancel { what: grunt }));
    step(&mut w);
    assert!(line(&w, 0, grunt).is_none());
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    step(&mut w);
    let kinds: Vec<LineKind> = w.econ.lines.lines.iter().map(|l| l.kind).collect();
    assert_eq!(kinds, vec![LineKind::Ground, LineKind::Infantry]);
}

// spec: S-0404 claim 2 (P2 part A: the bonus product narrowed with RZ₂₄, total 319)
#[test]
fn claim_2_bonus_rounding() {
    let mut r = ruleset();
    add_building(&mut r, "FPLANT", bonus_plant("0.8"));
    let mut w = game(&r);
    building(&mut w, "FPLANT", 0, 0, 0);
    building(&mut w, "FPLANT", 0, 2, 0);
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    assert_eq!(production::price(&w, HouseId(0), jeep), 319);
    // The other house owns no bonus building.
    assert_eq!(production::price(&w, HouseId(1), jeep), 500);
}

// spec: S-0404 claim 2 (P2 part B: country factor and bonuses; F1)
#[test]
fn claim_2_factors() {
    let mut r = ruleset();
    add_building(&mut r, "FPLANT", bonus_plant("0.9"));
    r.sim.vehicles.get_mut("TANK").unwrap().cost = Some(1001);
    r.sim
        .countries
        .get_mut("Alphaland")
        .unwrap()
        .econ
        .cost_units_mult = dec("0.8");
    let mut w = game(&r);
    building(&mut w, "FPLANT", 0, 0, 0);
    building(&mut w, "FPLANT", 0, 2, 0);
    assert_eq!(
        production::price(&w, HouseId(0), t(&w, Kind::Vehicle, "TANK")),
        648
    );
}

fn pad_rules(dock_cost: i32, pad_costs: (i32, i32)) -> Ruleset {
    let mut r = ruleset();
    add_building(&mut r, "DOCK", plain(dock_cost));
    for (n, c) in [("PADA", pad_costs.0), ("PADB", pad_costs.1)] {
        r.registries.get_mut("aircraft").unwrap().push(n.into());
        let mut a = r.sim.aircraft["PLANE"].clone();
        a.cost = Some(c);
        a.econ.dock = Some(vec!["DOCK".into()]);
        r.sim.aircraft.insert(n.into(), a);
    }
    r.sim.general.economy.pad_aircraft = Some(vec!["PADA".into(), "PADB".into()]);
    r.sim
        .countries
        .get_mut("Alphaland")
        .unwrap()
        .econ
        .cost_aircraft_mult = dec("0.5");
    r
}

// spec: S-0404 claim 2 (the pad term: own part with C′, plus (P₁ + P₂) ÷ 2; P2 building part)
#[test]
fn claim_2_pad_term() {
    let w = game(&pad_rules(3000, (1000, 1002)));
    let dock = t(&w, Kind::Building, "DOCK");
    assert_eq!(
        glasnost_sim::econ::sell::adjusted_cost(&w, dock.index),
        1999
    );
    assert_eq!(production::price(&w, HouseId(0), dock), 2499);
}

// spec: S-0404 Edge cases (negative own part: the price is the sum of the parts, no lower limit; the first step pays B < 0)
#[test]
fn claim_2_negative_own_part() {
    let w = game(&pad_rules(100, (1000, 1000)));
    let dock = t(&w, Kind::Building, "DOCK");
    assert_eq!(production::price(&w, HouseId(0), dock), -400);
    assert_eq!(payment(1, -400), -400);
    assert_eq!(payment(2, 0), 0);
}

// spec: S-0404 claim 2 (a free unit: own part with C′ plus the free unit's price at its kind's factors; P2 free unit part)
#[test]
fn claim_2_free_unit() {
    let mut r = ruleset();
    let mut b = plain(3000);
    b.econ.free_unit = Some("FREEV".into());
    add_building(&mut r, "GIFTED", b);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("FREEV".into());
    r.sim.vehicles.insert("FREEV".into(), wheel(800));
    r.sim
        .countries
        .get_mut("Alphaland")
        .unwrap()
        .econ
        .cost_units_mult = dec("0.5");
    let w = game(&r);
    assert_eq!(
        production::price(&w, HouseId(0), t(&w, Kind::Building, "GIFTED")),
        2600
    );
}

// spec: S-0404 claim 3 (P3 (a): an item starts at 0 credits and waits; five queued, then refusals)
#[test]
fn claim_3_start_and_queue() {
    let mut w = game(&ruleset());
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 0);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    assert_eq!(item(&w, 0, tank).unwrap().owed, 1000);
    for _ in 0..17 {
        step(&mut w);
    }
    let i = item(&w, 0, tank).unwrap();
    assert_eq!(i.progress, 0);
    assert!(i.waiting);
    for _ in 0..7 {
        issue(&mut w, 0, Order::Build { what: tank });
    }
    step(&mut w);
    assert_eq!(line(&w, 0, tank).unwrap().queue.len(), 5);
}

// spec: S-0404 claim 3 (P3 (b): a build-limit-1 type ordered twice: the second refused by the queue-time limit)
#[test]
fn claim_3_build_limit() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("TANK").unwrap().econ.build_limit = Some(1);
    let mut w = game(&r);
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    issue(&mut w, 0, Order::Build { what: tank });
    step(&mut w);
    assert!(line(&w, 0, tank).unwrap().queue.is_empty());
}

// spec: S-0404 claim 3 (P3 (e): a queue limit of 0 refuses every queued type)
#[test]
fn claim_3_queue_limit_zero() {
    let mut r = ruleset();
    r.sim.general.economy.maximum_queued_objects = Some(0);
    let mut w = game(&r);
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    issue(&mut w, 0, Order::Build { what: tank });
    step(&mut w);
    assert!(item(&w, 0, tank).is_some());
    assert!(line(&w, 0, tank).unwrap().queue.is_empty());
}

// spec: S-0404 claim 3 (P3 (f), (g), (h): a switched-off only factory, owner countries and the naval match)
#[test]
fn claim_3_factory_conditions() {
    let mut r = ruleset();
    let mut barr = factory("infantry");
    barr.owner = Some(vec!["Alphaland".into()]);
    add_building(&mut r, "BARR_A", barr);
    let mut grunt_b = foot(100);
    grunt_b.owner = Some(vec!["Betaland".into()]);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("GRUNTB".into());
    r.sim.infantry.insert("GRUNTB".into(), grunt_b);
    let mut w = game(&r);
    let b = building(&mut w, "BARRACKS", 0, -6, 0);
    building(&mut w, "BARR_A", 1, -6, 6);
    building(&mut w, "WARFACT", 0, 0, 0);
    building(&mut w, "NAVYARD", 1, 6, 0);
    ready(&mut w);
    rich(&mut w);
    set_credits(&mut w, 1, 100_000);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    switch(&mut w, b, false);
    force(&mut w, 0, Order::Build { what: grunt });
    // (g): house 1's only infantry factory shares no owner country with GRUNTB.
    let ty = t(&w, Kind::Infantry, "GRUNTB");
    force(&mut w, 1, Order::Build { what: ty });
    // (h): a naval vehicle with only a ground factory, and the reverse.
    let ty = t(&w, Kind::Vehicle, "BOAT");
    force(&mut w, 0, Order::Build { what: ty });
    let ty = t(&w, Kind::Vehicle, "TANK");
    force(&mut w, 1, Order::Build { what: ty });
    step(&mut w);
    assert!(w.econ.lines.lines.is_empty());
}

// spec: S-0404 claim 3 (a done item of the ordered type: nothing added; a player-held item of the ordered type: resumed)
#[test]
fn claim_3_done_and_held() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("TANK").unwrap().cost = Some(50);
    let mut w = game(&r);
    let f = building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    assert!(issue(&mut w, 0, Order::Hold { what: tank }));
    step(&mut w);
    assert_eq!(item(&w, 0, tank).unwrap().hold, Hold::Player);
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let i = item(&w, 0, tank).unwrap();
    assert_eq!(i.hold, Hold::None);
    assert!(line(&w, 0, tank).unwrap().queue.is_empty());
    run_to_done(&mut w, 0, tank, 200).expect("done");
    switch(&mut w, f, false);
    force(&mut w, 0, Order::Build { what: tank });
    step(&mut w);
    step(&mut w);
    assert!(item(&w, 0, tank).unwrap().done());
    assert!(line(&w, 0, tank).unwrap().queue.is_empty());
}

// spec: S-0404 claim 4 (F2: P = 1000 pays 18 at steps 1 to 7, 19 at 8 to 53, 0 at 54; the worked payments)
#[test]
fn claim_4_payments() {
    let seq = |p: i32| {
        let mut b = p;
        (1..=54)
            .map(|k| {
                let x = payment(k, b);
                b -= x;
                x
            })
            .collect::<Vec<_>>()
    };
    let s = seq(1000);
    assert!(s[..7].iter().all(|&x| x == 18));
    assert!(s[7..53].iter().all(|&x| x == 19));
    assert_eq!(s[53], 0);
    assert_eq!(s.iter().sum::<i32>(), 1000);
    let s = seq(100);
    assert!(s[..6].iter().all(|&x| x == 1) && s[6..53].iter().all(|&x| x == 2));
    let s = seq(10);
    assert!(s[..43].iter().all(|&x| x == 0) && s[43..53].iter().all(|&x| x == 1));
}

// spec: S-0404 claim 4 (P4: payments in the game, refusal without partial payment, the retry every R at the same phase)
#[test]
fn claim_4_waiting_for_money() {
    let mut w = game(&ruleset());
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let first = item(&w, 0, tank).unwrap().due;
    while w.tick <= first + 16 * 5 {
        step(&mut w);
    }
    // Five steps of 18 paid, the sixth refused: 10 credits left.
    assert_eq!(item(&w, 0, tank).unwrap().progress, 5);
    assert_eq!(credits(&w, 0), 10);
    assert!(item(&w, 0, tank).unwrap().waiting);
    // A gift between retries: the next retry, at the original phase, is made.
    for _ in 0..5 {
        step(&mut w);
    }
    set_credits(&mut w, 0, 60);
    let retry = item(&w, 0, tank).unwrap().due;
    assert_eq!((retry - first) % 16, 0);
    while w.tick <= retry {
        step(&mut w);
    }
    assert_eq!(item(&w, 0, tank).unwrap().progress, 6);
    assert_eq!(credits(&w, 0), 42);
}

// spec: S-0404 claim 4 (P4 part C: zero-payment steps at 0 credits; exactly P credits completes with 0 left)
#[test]
fn claim_4_cheap_and_exact() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("JEEP").unwrap().cost = Some(10);
    let mut w = game(&r);
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 0);
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    for _ in 0..200 {
        step(&mut w);
    }
    assert_eq!(item(&w, 0, jeep).unwrap().progress, 43);
    assert!(item(&w, 0, jeep).unwrap().waiting);
    // Exactly P.
    let mut w = game(&ruleset());
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 1000);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    run_to_done(&mut w, 0, tank, 2000).expect("done");
    assert_eq!(credits(&w, 0), 0);
}

// spec: S-0404 claim 5 (P5: no steps or payments while held, nothing refunded; hold to a held line or a done item does nothing)
#[test]
fn claim_5_hold() {
    let mut w = game(&ruleset());
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    for _ in 0..40 {
        step(&mut w);
    }
    assert!(issue(&mut w, 0, Order::Hold { what: tank }));
    step(&mut w);
    let (p, c) = (item(&w, 0, tank).unwrap().progress, credits(&w, 0));
    assert!(issue(&mut w, 0, Order::Hold { what: tank }));
    for _ in 0..100 {
        step(&mut w);
    }
    let i = item(&w, 0, tank).unwrap();
    assert_eq!((i.progress, credits(&w, 0), i.hold), (p, c, Hold::Player));
}

// spec: S-0404 claim 5 (P5 part B: a step due in the tick of the hold order is made first)
#[test]
fn claim_5_hold_in_step_tick() {
    let mut w = game(&ruleset());
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let due = item(&w, 0, tank).unwrap().due;
    while w.tick < due {
        step(&mut w);
    }
    assert!(issue(&mut w, 0, Order::Hold { what: tank }));
    step(&mut w);
    let i = item(&w, 0, tank).unwrap();
    assert_eq!((i.progress, i.hold), (1, Hold::Player));
}

fn started(r: &Ruleset) -> (glasnost_sim::world::World, TypeRef) {
    let mut w = game(r);
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    (w, tank)
}

fn run_progress(w: &mut glasnost_sim::world::World, ty: TypeRef, p: i32) {
    while item(w, 0, ty).unwrap().progress < p {
        step(w);
    }
}

// spec: S-0404 claim 6 (P6 (a): a cancel with an unchanged price refunds what was paid)
#[test]
fn claim_6_refund() {
    let (mut w, tank) = started(&ruleset());
    let c0 = credits(&w, 0);
    run_progress(&mut w, tank, 20);
    assert!(credits(&w, 0) < c0);
    assert!(issue(&mut w, 0, Order::Cancel { what: tank }));
    step(&mut w);
    assert_eq!(credits(&w, 0), c0);
    assert!(line(&w, 0, tank).is_none());
}

// spec: S-0404 claim 6 (P6 (b), (c), (h): queued copies first; cancel-all removes all and the item; an unrelated cancel does nothing)
#[test]
fn claim_6_queue_and_all() {
    let (mut w, tank) = started(&ruleset());
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    issue(&mut w, 0, Order::Build { what: tank });
    issue(&mut w, 0, Order::Build { what: tank });
    issue(&mut w, 0, Order::Build { what: jeep });
    step(&mut w);
    assert_eq!(line(&w, 0, tank).unwrap().queue.len(), 3);
    issue(&mut w, 0, Order::Cancel { what: tank });
    step(&mut w);
    let l = line(&w, 0, tank).unwrap();
    assert_eq!(l.queue, vec![tank, jeep]);
    assert_eq!(l.item.unwrap().t, tank);
    // (h): a cancel for a type neither queued nor the item.
    let boat = t(&w, Kind::Vehicle, "BOAT");
    force(&mut w, 0, Order::Cancel { what: boat });
    step(&mut w);
    assert_eq!(line(&w, 0, tank).unwrap().queue, vec![tank, jeep]);
    // (c): cancel-all: the queued copy and the item go, the next other type starts.
    run_progress(&mut w, tank, 3);
    let c0 = credits(&w, 0);
    issue(&mut w, 0, Order::CancelAll { what: tank });
    step(&mut w);
    let l = line(&w, 0, tank).unwrap();
    assert!(l.queue.is_empty());
    assert_eq!(l.item.unwrap().t, jeep);
    assert!(credits(&w, 0) > c0);
}

// spec: S-0404 claim 6 and quirks.refund_at_current_price (P6 (d): refund P′ − B = 500; off: the 300 paid)
#[test]
fn claim_6_current_price() {
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.refund_at_current_price = quirk;
        add_building(&mut r, "FPLANT", bonus_plant("0.8"));
        let mut w = game(&r);
        building(&mut w, "WARFACT", 0, 0, 0);
        let plant = building(&mut w, "FPLANT", 0, 5, 5);
        ready(&mut w);
        rich(&mut w);
        let tank = t(&w, Kind::Vehicle, "TANK");
        assert!(issue(&mut w, 0, Order::Build { what: tank }));
        step(&mut w);
        assert_eq!(item(&w, 0, tank).unwrap().owed, 800);
        run_progress(&mut w, tank, 20);
        let i = item(&w, 0, tank).unwrap();
        let paid = 800 - i.owed;
        objects::remove(&mut w, plant);
        let c0 = credits(&w, 0);
        assert!(issue(&mut w, 0, Order::Cancel { what: tank }));
        step(&mut w);
        let refund = credits(&w, 0) - c0;
        if quirk {
            assert_eq!(refund, 1000 - i.owed);
        } else {
            assert_eq!(refund, paid);
        }
    }
}

// spec: S-0404 claim 6 (P6 (g): a negative refund takes credits below 0; every step, even one paying 0, is then refused)
#[test]
fn claim_6_negative_refund() {
    let mut r = ruleset();
    add_building(&mut r, "FPLANT", bonus_plant("0.5"));
    r.sim.infantry.get_mut("GRUNT").unwrap().cost = Some(10);
    // A slow item whose early steps pay 0 (T = 900, R = 16).
    r.sim
        .infantry
        .get_mut("GRUNT")
        .unwrap()
        .econ
        .build_time_multiplier = dec("100");
    let mut w = game(&r);
    building(&mut w, "WARFACT", 0, 0, 0);
    building(&mut w, "BARRACKS", 0, -6, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 473);
    let tank = t(&w, Kind::Vehicle, "TANK");
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    step(&mut w);
    run_progress(&mut w, tank, 20);
    building(&mut w, "FPLANT", 0, 5, 5);
    let owed = item(&w, 0, tank).unwrap().owed;
    let c0 = credits(&w, 0);
    assert!(issue(&mut w, 0, Order::Cancel { what: tank }));
    step(&mut w);
    assert_eq!(credits(&w, 0), c0 + 500 - owed);
    assert!(credits(&w, 0) < 0);
    let p = item(&w, 0, grunt).unwrap().progress;
    for _ in 0..40 {
        step(&mut w);
    }
    assert_eq!(item(&w, 0, grunt).unwrap().progress, p);
    set_credits(&mut w, 0, 0);
    for _ in 0..20 {
        step(&mut w);
    }
    assert!(item(&w, 0, grunt).unwrap().progress > p);
}

// spec: S-0404 claim 7 (P7: the later-acquired factory releases without a primary mark, the primary one with it)
#[test]
fn claim_7_releasing_building() {
    let mut w = game(&ruleset());
    let a = building(&mut w, "WARFACT", 0, -6, 0);
    let b = building(&mut w, "WARFACT", 0, 4, 0);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert_eq!(
        production::releasing_building(&w, HouseId(0), tank),
        Some(b)
    );
    w.obj_mut(a).unwrap().econ.building.primary = true;
    assert_eq!(
        production::releasing_building(&w, HouseId(0), tank),
        Some(a)
    );
    switch(&mut w, a, false);
    assert_eq!(
        production::releasing_building(&w, HouseId(0), tank),
        Some(b)
    );
}

// spec: S-0404 claim 7 (the unit leaves; the next queued type starts in the release tick)
#[test]
fn claim_7_release_and_next() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("JEEP").unwrap().cost = Some(50);
    let mut w = game(&r);
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let obj = item(&w, 0, jeep).unwrap().object;
    let done = run_to_done(&mut w, 0, jeep, 200).expect("done");
    while w.tick <= done + 1 {
        step(&mut w);
    }
    assert!(w.obj(obj).unwrap().on_map);
    let i = item(&w, 0, tank).unwrap();
    assert_eq!(i.progress, 0);
    assert_eq!(i.due, done + 1 + i.period);
}

// spec: S-0404 claim 7 (no releasing building: the place order does nothing and the item stays done)
#[test]
fn claim_7_no_release() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("JEEP").unwrap().cost = Some(50);
    let mut w = game(&r);
    let f = building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    let done = run_to_done(&mut w, 0, jeep, 200).expect("done");
    // The only factory switched off before the automatic place order of tick done + 1.
    assert_eq!(w.tick, done + 1);
    switch(&mut w, f, false);
    for _ in 0..5 {
        step(&mut w);
    }
    let i = item(&w, 0, jeep).unwrap();
    assert!(i.done());
    assert!(!w.obj(i.object).unwrap().on_map);
}

// spec: S-0404 claim 7 (a done building waits for a place order with a cell; S-0406 places it: a seam until S-0406)
#[test]
fn claim_7_building_waits() {
    let mut r = ruleset();
    r.sim.buildings.get_mut("HUT").unwrap().cost = Some(50);
    let mut w = game(&r);
    building(&mut w, "CONYARD", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let hut = t(&w, Kind::Building, "HUT");
    assert!(issue(&mut w, 0, Order::Build { what: hut }));
    run_to_done(&mut w, 0, hut, 200).expect("done");
    for _ in 0..5 {
        step(&mut w);
    }
    assert!(item(&w, 0, hut).unwrap().done());
    assert_eq!(
        production::item_object(&w, HouseId(0), hut),
        Some(item(&w, 0, hut).unwrap().object)
    );
}

fn barracks_game(
    r: &Ruleset,
) -> (
    glasnost_sim::world::World,
    glasnost_sim::world::ObjectId,
    TypeRef,
) {
    let mut w = game(r);
    let b = building(&mut w, "BARRACKS", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    issue(&mut w, 0, Order::Build { what: grunt });
    issue(&mut w, 0, Order::Build { what: grunt });
    step(&mut w);
    for _ in 0..30 {
        step(&mut w);
    }
    (w, b, grunt)
}

// spec: S-0404 claim 8 (P8 runs 1 and 2: the only factory leaves: refunded, queue cleared; with a second factory: unchanged)
#[test]
fn claim_8_factory_leaves() {
    let (mut w, b, grunt) = barracks_game(&ruleset());
    let c = credits(&w, 0);
    objects::remove(&mut w, b);
    assert!(line(&w, 0, grunt).is_none());
    assert_eq!(credits(&w, 0), 1_000_000);
    assert!(credits(&w, 0) > c);
    let (mut w, b, grunt) = barracks_game(&ruleset());
    building(&mut w, "BARRACKS", 0, 6, 6);
    objects::remove(&mut w, b);
    let l = line(&w, 0, grunt).unwrap();
    assert_eq!(l.queue.len(), 2);
    assert!(l.item.unwrap().running());
}

// spec: S-0404 claim 8 (P8 run 3: switched off, an automatic hold; switched on, lifted with a full step period)
#[test]
fn claim_8_automatic_hold() {
    let (mut w, b, grunt) = barracks_game(&ruleset());
    switch(&mut w, b, false);
    assert_eq!(item(&w, 0, grunt).unwrap().hold, Hold::Auto);
    let p = item(&w, 0, grunt).unwrap().progress;
    for _ in 0..50 {
        step(&mut w);
    }
    assert_eq!(item(&w, 0, grunt).unwrap().progress, p);
    switch(&mut w, b, true);
    let i = item(&w, 0, grunt).unwrap();
    assert_eq!(i.hold, Hold::None);
    assert_eq!(i.due, w.tick + i.period);
    // A build order is ignored while the line is on an automatic hold (no switched-on factory).
    switch(&mut w, b, false);
    force(&mut w, 0, Order::Build { what: grunt });
    step(&mut w);
    assert_eq!(item(&w, 0, grunt).unwrap().hold, Hold::Auto);
}

// spec: S-0404 claim 8 (P8 run 7: a player hold is never lifted by the check)
#[test]
fn claim_8_player_hold_kept() {
    let (mut w, b, grunt) = barracks_game(&ruleset());
    assert!(issue(&mut w, 0, Order::Hold { what: grunt }));
    step(&mut w);
    switch(&mut w, b, false);
    assert_eq!(item(&w, 0, grunt).unwrap().hold, Hold::Player);
    switch(&mut w, b, true);
    assert_eq!(item(&w, 0, grunt).unwrap().hold, Hold::Player);
}

// spec: S-0404 claim 8 (P8 run 8: a queued type only the leaving factory could make leaves the queue; the item continues)
#[test]
fn claim_8_queue_rule() {
    let mut r = ruleset();
    let mut ba = factory("infantry");
    ba.owner = Some(vec!["Alphaland".into()]);
    add_building(&mut r, "BARR_A", ba);
    r.sim.infantry.get_mut("MEDIC").unwrap().owner = Some(vec!["Betaland".into()]);
    let mut w = game(&r);
    let b = building(&mut w, "BARRACKS", 0, 0, 0);
    building(&mut w, "BARR_A", 0, 6, 6);
    ready(&mut w);
    rich(&mut w);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    let medic = t(&w, Kind::Infantry, "MEDIC");
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    assert!(issue(&mut w, 0, Order::Build { what: medic }));
    step(&mut w);
    assert_eq!(line(&w, 0, grunt).unwrap().queue, vec![medic]);
    objects::remove(&mut w, b);
    let l = line(&w, 0, grunt).unwrap();
    assert!(l.queue.is_empty());
    assert_eq!(l.item.unwrap().t, grunt);
}

fn building_lines(
    r: &Ruleset,
) -> (
    glasnost_sim::world::World,
    glasnost_sim::world::ObjectId,
    TypeRef,
    TypeRef,
) {
    let mut w = game(r);
    let c = building(&mut w, "CONYARD", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let hut = t(&w, Kind::Building, "HUT");
    let tower = t(&w, Kind::Building, "TOWER");
    assert!(issue(&mut w, 0, Order::Build { what: hut }));
    assert!(issue(&mut w, 0, Order::Build { what: tower }));
    step(&mut w);
    (w, c, hut, tower)
}

// spec: S-0404 claim 8 and quirks.defence_line_partial_factory_check (the check covers only the ordinary-building line)
#[test]
fn claim_8_defence_line() {
    let (mut w, c, hut, tower) = building_lines(&ruleset());
    switch(&mut w, c, false);
    assert_eq!(item(&w, 0, hut).unwrap().hold, Hold::Auto);
    assert_eq!(item(&w, 0, tower).unwrap().hold, Hold::None);
    let mut r = ruleset();
    r.globals.quirks.defence_line_partial_factory_check = false;
    let (mut w, c, _, tower) = building_lines(&r);
    switch(&mut w, c, false);
    assert_eq!(item(&w, 0, tower).unwrap().hold, Hold::Auto);
}

// spec: S-0404 claim 8 (P8 run 9: a factory of buildings removed for good cancels the defence line too)
#[test]
fn claim_8_removed_for_good() {
    let (mut w, c, hut, tower) = building_lines(&ruleset());
    objects::remove(&mut w, c);
    assert!(line(&w, 0, hut).is_none());
    assert!(line(&w, 0, tower).is_none());
    assert_eq!(credits(&w, 0), 1_000_000);
}

// spec: S-0404 claim 8 and Terms (P8 run 11: a factory building queued to be sold counts in no test; the start of the sale is no event, so the item keeps stepping and paying; a check run by another event judges without it and cancels with the refund)
#[test]
fn claim_8_factory_being_sold() {
    let (mut w, b, grunt) = barracks_game(&ruleset());
    w.obj_mut(b)
        .unwrap()
        .beh
        .queue(Some(glasnost_sim::rules::Behaviour::Selling));
    // No body range runs, so no ready mark starts the queued sale (S-0215 claim 11).
    w.obj_mut(b).unwrap().econ.idle_body = None;
    w.obj_mut(b).unwrap().beh.ready_mark = false;
    let p = item(&w, 0, grunt).unwrap().progress;
    let c = credits(&w, 0);
    for _ in 0..20 {
        step(&mut w);
    }
    assert!(w.obj(b).is_some_and(|o| o.on_map));
    assert!(
        item(&w, 0, grunt).unwrap().progress > p,
        "it keeps stepping"
    );
    assert!(credits(&w, 0) < c, "and paying");
    w.obj_mut(b)
        .unwrap()
        .beh
        .queue(Some(glasnost_sim::rules::Behaviour::Selling));
    production::check_line(&mut w, HouseId(0), LineKind::Infantry, None);
    assert!(line(&w, 0, grunt).is_none());
    assert_eq!(credits(&w, 0), 1_000_000);
}

// spec: S-0404 claim 9 (P9 (a): a prerequisite destroyed in the turn pass of tick n: both orders issued and applied in tick n; the item refunded, the queued copy removed)
#[test]
fn claim_9_unbuildable() {
    let mut r = ruleset();
    add_building(&mut r, "PREQ", plain(100));
    r.sim.infantry.get_mut("GRUNT").unwrap().econ.prerequisite = Some(vec!["PREQ".into()]);
    let mut w = game(&r);
    building(&mut w, "BARRACKS", 0, 0, 0);
    let p = building(&mut w, "PREQ", 0, 6, 6);
    ready(&mut w);
    rich(&mut w);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    issue(&mut w, 0, Order::Build { what: grunt });
    step(&mut w);
    for _ in 0..40 {
        step(&mut w);
    }
    objects::remove(&mut w, p);
    step(&mut w);
    assert!(line(&w, 0, grunt).is_none());
    assert_eq!(credits(&w, 0), 1_000_000);
}

// spec: S-0404 claim 9 (P9 (b): a loss through a building switched off by an order applied at the end of tick n: the recheck and both orders in tick n + 1, applied at its end)
// spec: S-0407 claim 8 (c) (a switched-off building no longer meets an upgrade prerequisite)
#[test]
fn claim_9_switch_off_next_tick() {
    let mut r = ruleset();
    let mut up = plain(100);
    up.econ.powers_up_building = Some("HUT".into());
    add_building(&mut r, "UPG", up);
    r.sim.infantry.get_mut("GRUNT").unwrap().econ.prerequisite = Some(vec!["UPG".into()]);
    let mut w = game(&r);
    building(&mut w, "BARRACKS", 0, 0, 0);
    let hut = building(&mut w, "HUT", 0, 6, 6);
    let upg = t(&w, Kind::Building, "UPG").index;
    w.obj_mut(hut).unwrap().econ.building.upgrades[0] = Some(upg);
    glasnost_sim::econ::tech::recheck_due(&mut w, HouseId(0));
    ready(&mut w);
    rich(&mut w);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    issue(&mut w, 0, Order::Build { what: grunt });
    for _ in 0..30 {
        step(&mut w);
    }
    let paid = 1_000_000 - credits(&w, 0);
    assert!(paid > 0);
    // Switched off at the order boundary of tick n (the end of the last step).
    switch(&mut w, hut, false);
    assert!(w.houses[0].pending.is_empty());
    // Tick n + 1: the recheck in its house update issues the orders, which
    // are applied at its end; the item is refunded, the copy removed.
    step(&mut w);
    assert!(line(&w, 0, grunt).is_none());
    assert_eq!(credits(&w, 0), 1_000_000);
}

// spec: S-0404 claim 9 (P9 (c): a condition that fails without a recheck event gives no orders; production goes on until a later recheck removes the type)
// spec: S-0407 claim 12 (o) and quirks.unit_changes_no_recheck (a unit that stops existing while not at limit is no event)
#[test]
fn claim_9_stale_until_recheck() {
    let mut r = ruleset();
    r.sim.general.economy.prerequisite_proc_alternate = Some("JEEP".into());
    r.sim.infantry.get_mut("GRUNT").unwrap().econ.prerequisite = Some(vec!["PROC".into()]);
    let mut w = game(&r);
    building(&mut w, "BARRACKS", 0, 0, 0);
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    let v = glasnost_sim::testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "JEEP",
        0,
        glasnost_sim::testkit::mid(SIZE, 4, 4),
        0,
    );
    let _ = jeep;
    ready(&mut w);
    rich(&mut w);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    step(&mut w);
    // The refinery-alternative vehicle stops existing: no event.
    objects::remove(&mut w, v);
    w.objects.remove(&v);
    for _ in 0..20 {
        step(&mut w);
        assert!(w.houses[0].pending.is_empty());
    }
    assert!(item(&w, 0, grunt).is_some_and(|i| i.progress > 0));
    // An unrelated building placed: its recheck removes the type, and the
    // orders refund the item.
    building(&mut w, "HUT", 0, -6, 6);
    step(&mut w);
    assert!(line(&w, 0, grunt).is_none());
    assert_eq!(credits(&w, 0), 1_000_000);
}

// spec: S-0404 claim 10 (a build order for a building or defence is not legal while that line has an item; a held one resumes)
#[test]
fn claim_10_legality() {
    let (mut w, _, hut, _) = building_lines(&ruleset());
    let plant = t(&w, Kind::Building, "PLANT");
    assert!(!issue(&mut w, 0, Order::Build { what: plant }));
    assert!(!issue(&mut w, 0, Order::Build { what: hut }));
    assert!(issue(&mut w, 0, Order::Hold { what: hut }));
    step(&mut w);
    assert!(!issue(&mut w, 0, Order::Build { what: plant }));
    assert!(issue(&mut w, 0, Order::Build { what: hut }));
    // A type not offered (no factory of its kind): not legal.
    let boat = t(&w, Kind::Vehicle, "BOAT");
    assert!(!issue(&mut w, 0, Order::Build { what: boat }));
}

// spec: S-0404 claim 10 and quirks.forced_start_player_hold (P10 part A: a forced start is held when A ≥ P ÷ 54, running otherwise)
#[test]
fn claim_10_forced_start() {
    for (quirk, money, held) in [(true, 18, true), (true, 17, false), (false, 1000, false)] {
        let mut r = ruleset();
        r.globals.quirks.forced_start_player_hold = quirk;
        let mut w = game(&r);
        let f = building(&mut w, "WARFACT", 0, 0, 0);
        ready(&mut w);
        rich(&mut w);
        let tank = t(&w, Kind::Vehicle, "TANK");
        let jeep = t(&w, Kind::Vehicle, "JEEP");
        assert!(issue(&mut w, 0, Order::Build { what: jeep }));
        assert!(issue(&mut w, 0, Order::Build { what: tank }));
        step(&mut w);
        switch(&mut w, f, false);
        // JEEP's price 500: P ÷ 54 = 9; TANK's price 1000: 18.
        let owed = item(&w, 0, jeep).unwrap().owed;
        set_credits(&mut w, 0, money - 500 + owed);
        production::cancel(&mut w, HouseId(0), jeep, false);
        let i = item(&w, 0, tank).unwrap();
        let want = if held { Hold::Player } else { Hold::None };
        assert_eq!(i.hold, want, "quirk {quirk}, money {money}");
    }
}

// spec: S-0404 claim 10 and Edge cases (quirks.unstartable_queue_head_stalls: a head that cannot start, here by the build-limit answer with the production allowance, is dropped and nothing starts; off: the next that can starts) and S-0407 claim 16 (a queued copy is not started past its limit)
#[test]
fn claim_10_dropped_head() {
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.unstartable_queue_head_stalls = quirk;
        r.sim.vehicles.get_mut("JEEP").unwrap().econ.build_limit = Some(1);
        r.registries.get_mut("vehicles").unwrap().push("VAN".into());
        r.sim.vehicles.insert("VAN".into(), wheel(300));
        let mut w = game(&r);
        building(&mut w, "WARFACT", 0, 0, 0);
        ready(&mut w);
        rich(&mut w);
        let tank = t(&w, Kind::Vehicle, "TANK");
        let jeep = t(&w, Kind::Vehicle, "JEEP");
        let van = t(&w, Kind::Vehicle, "VAN");
        assert!(issue(&mut w, 0, Order::Build { what: tank }));
        assert!(issue(&mut w, 0, Order::Build { what: jeep }));
        assert!(issue(&mut w, 0, Order::Build { what: van }));
        step(&mut w);
        assert_eq!(line(&w, 0, tank).unwrap().queue, vec![jeep, van]);
        // The house comes to own one JEEP: O = L with no JEEP in production.
        glasnost_sim::testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "JEEP",
            0,
            glasnost_sim::testkit::mid(SIZE, 8, 8),
            0,
        );
        production::cancel(&mut w, HouseId(0), tank, false);
        let l = line(&w, 0, tank).unwrap();
        if quirk {
            assert!(l.item.is_none());
            assert_eq!(l.queue, vec![van]);
        } else {
            assert_eq!(l.item.unwrap().t, van);
            assert!(l.queue.is_empty());
        }
    }
}

// spec: S-0404 claim 10 and claim 8 (P10 part C, P8 run 10: a stale prerequisite refusal alone neither drops a queued type nor stops its start; the factory check omits the full tech tree)
#[test]
fn claim_10_stale_prerequisite_starts() {
    let mut r = ruleset();
    add_building(&mut r, "PREQ", plain(100));
    r.sim.vehicles.get_mut("JEEP").unwrap().econ.prerequisite = Some(vec!["PREQ".into()]);
    let mut w = game(&r);
    let f = building(&mut w, "WARFACT", 0, 0, 0);
    let p = building(&mut w, "PREQ", 0, 6, 6);
    ready(&mut w);
    rich(&mut w);
    let tank = t(&w, Kind::Vehicle, "TANK");
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    step(&mut w);
    // The prerequisite goes without a recheck in between (claim 8's check runs on a switch).
    objects::remove(&mut w, p);
    switch(&mut w, f, false);
    let l = line(&w, 0, tank).unwrap();
    assert_eq!(l.item.as_ref().unwrap().hold, Hold::Auto);
    assert_eq!(l.queue, vec![jeep]);
    switch(&mut w, f, true);
    production::cancel(&mut w, HouseId(0), tank, false);
    assert_eq!(item(&w, 0, jeep).unwrap().t, jeep);
}

// spec: S-0404 claim 11 (one creation draw per item started; none at steps, holds, resumes, refunds or completion)
#[test]
fn claim_11_draws() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("JEEP").unwrap().cost = Some(50);
    // A control game without orders, run in step: the draws of the lines are the difference.
    let mk = || {
        let mut w = game(&r);
        building(&mut w, "WARFACT", 0, 0, 0);
        ready(&mut w);
        set_credits(&mut w, 0, 1_000_000);
        w
    };
    let mut w = mk();
    let mut control = mk();
    let tank = t(&w, Kind::Vehicle, "TANK");
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    let mut tick_draws = |w: &mut glasnost_sim::world::World| {
        let (a, c) = (w.rng.taken(), control.rng.taken());
        step(w);
        step(&mut control);
        (w.rng.taken() - a) as i64 - (control.rng.taken() - c) as i64
    };
    assert_eq!(tick_draws(&mut w), 0);
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    assert_eq!(tick_draws(&mut w), 1);
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    assert_eq!(tick_draws(&mut w), 0);
    for _ in 0..40 {
        assert_eq!(tick_draws(&mut w), 0);
    }
    assert!(issue(&mut w, 0, Order::Hold { what: tank }));
    assert_eq!(tick_draws(&mut w), 0);
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    assert_eq!(tick_draws(&mut w), 0);
    // A cancel with a refund starts the queued type: one draw.
    assert!(issue(&mut w, 0, Order::Cancel { what: tank }));
    assert_eq!(tick_draws(&mut w), 1);
    // A cancel with an empty queue: none.
    assert!(issue(&mut w, 0, Order::Cancel { what: jeep }));
    assert_eq!(tick_draws(&mut w), 0);
}

// spec: S-0213 claim 3 (production-start draws across houses: an earlier house's order-caused starts take their values before a later house's, whatever the issue order)
#[test]
fn s0213_claim_3_cross_house_start_draws() {
    let mut w = game(&ruleset());
    building(&mut w, "BARRACKS", 0, 0, 0);
    building(&mut w, "BARRACKS", 1, 0, 8);
    ready(&mut w);
    set_credits(&mut w, 0, 1_000_000);
    set_credits(&mut w, 1, 1_000_000);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    // The later house issues first.
    assert!(issue(&mut w, 1, Order::Build { what: grunt }));
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    step(&mut w);
    let object = |w: &World, h: u8| {
        w.econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == HouseId(h))
            .find_map(|l| l.item.as_ref().filter(|i| i.t == grunt).map(|i| i.object))
            .unwrap()
    };
    assert!(object(&w, 0) < object(&w, 1), "house 0's start drew first");
}

// spec: S-0404 claim 11 (starts take their creation draws at their causing action: several production orders in one tick give successive values in order of application, S-0213 claim 3, not in line order)
#[test]
fn claim_11_creation_draws_in_application_order() {
    for grunt_first in [true, false] {
        let mut w = game(&ruleset());
        building(&mut w, "WARFACT", 0, 0, 0);
        building(&mut w, "BARRACKS", 0, 0, 6);
        ready(&mut w);
        set_credits(&mut w, 0, 1_000_000);
        let grunt = t(&w, Kind::Infantry, "GRUNT");
        let tank = t(&w, Kind::Vehicle, "TANK");
        let order = if grunt_first {
            [grunt, tank]
        } else {
            [tank, grunt]
        };
        for what in order {
            assert!(issue(&mut w, 0, Order::Build { what }));
        }
        step(&mut w);
        // Objects are numbered in creation order, so the earlier id took the earlier draw.
        let object = |w: &World, what| {
            w.econ
                .lines
                .lines
                .iter()
                .find_map(|l| l.item.as_ref().filter(|i| i.t == what).map(|i| i.object))
                .unwrap()
        };
        let (g, k) = (object(&w, grunt), object(&w, tank));
        assert_eq!(g < k, grunt_first, "grunt first {grunt_first}");
    }
}

// spec: S-0404 claim 12 (an AI house produces only through orders)
#[test]
fn claim_12_no_orders_no_production() {
    let mut w = game(&ruleset());
    building(&mut w, "BARRACKS", 1, 0, 0);
    w.houses[1].human = false;
    set_credits(&mut w, 1, 100_000);
    for _ in 0..100 {
        step(&mut w);
    }
    assert!(w.econ.lines.lines.is_empty());
}

// --- Claim 13 and 14: releasing a unit from a factory building ---

use glasnost_map::CellXY;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

/// A new unit of `name` for house 0, off the map (S-0404 claim 11).
fn new_unit(w: &mut World, kind: Kind, name: &str) -> ObjectId {
    let ti = w.rules().index_of(kind, name).unwrap();
    objects::create(w, kind, ti, Some(HouseId(0)))
}

/// The top-left cell of a building.
fn top_left(w: &World, b: ObjectId) -> CellXY {
    w.obj(b).unwrap().pos.cell()
}

fn spawn_at(w: &mut World, kind: Kind, name: &str, c: CellXY) -> ObjectId {
    glasnost_sim::testkit::spawn(w, kind, name, 0, c, 0)
}

fn facing_to(from: Pos, to: Pos) -> u16 {
    let f = glasnost_num::geom::direction(i64::from(to.x - from.x), i64::from(to.y - from.y), true);
    (glasnost_num::geom::n_step(f, 256) * 256) as u16
}

// spec: S-0404 claim 13 (P12 part a: the ground vehicle appears at the top-left cell centre plus the exit displacement, facing 16384, and K starts unloading; with (0, 0, 0) at the foundation centre)
#[test]
fn claim_13_ground_exit_point() {
    let mut r = ruleset();
    r.sim.buildings.get_mut("WARFACT").unwrap().econ.exit_coord = Some([100, 60, 0]);
    let mut w = game(&r);
    let k = building(&mut w, "WARFACT", 0, 0, 0);
    let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    let tl = Pos::centre(top_left(&w, k), 0);
    let o = w.obj(tank).unwrap();
    assert!(o.on_map);
    assert_eq!((o.pos.x, o.pos.y), (tl.x + 100, tl.y + 60));
    assert_eq!(w.body_facing(tank), 16384);
    // spec: S-0417 claim 1 (K queues unloading; it becomes current at a per-tick start that finds the ready mark set)
    assert_eq!(w.obj(k).unwrap().beh.queued, Some(Behaviour::Unload));
    step(&mut w);
    assert_eq!(w.obj(k).unwrap().beh.current, Some(Behaviour::Unload));
    let mut w = game(&ruleset());
    let k = building(&mut w, "WARFACT", 0, 0, 0);
    let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    let c = w.reference_point(k);
    let o = w.obj(tank).unwrap();
    assert_eq!((o.pos.x, o.pos.y), (c.x, c.y));
}

// spec: S-0404 claim 13 (P12 part b: an unloading ground factory passes the release to an idle twin of its type, switched off or not; with none, or only another type, busy)
#[test]
fn claim_13_ground_busy_and_hand_off() {
    let mut r = ruleset();
    add_building(&mut r, "WARFACT2", factory("vehicle"));
    let mut w = game(&r);
    let k = building(&mut w, "WARFACT", 0, 0, 0);
    let other = building(&mut w, "WARFACT2", 0, 6, 0);
    let first = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, first), Outcome::Released);
    unloading_now(&mut w, k);
    let second = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, second), Outcome::Busy);
    assert!(!w.obj(second).unwrap().on_map);
    assert_eq!(w.obj(other).unwrap().beh.current, Some(Behaviour::Guard));
    let twin = building(&mut w, "WARFACT", 0, -6, 0);
    switch(&mut w, twin, false);
    assert_eq!(release::release(&mut w, k, second), Outcome::Released);
    assert_eq!(w.obj(second).unwrap().pos, w.reference_point(twin));
    assert_eq!(w.obj(twin).unwrap().beh.queued, Some(Behaviour::Unload));
}

/// Starts K's queued *unload* as its next per-tick start with the ready
/// mark set would (S-0417 claims 1 and 2).
fn unloading_now(w: &mut World, k: ObjectId) {
    let t = w.tick;
    let o = w.obj_mut(k).unwrap();
    assert_eq!(o.beh.queued, Some(Behaviour::Unload));
    o.beh.force(Behaviour::Unload, t, true);
}

// spec: S-0404 claim 7 (a busy release cancels the item with the refund at once and starts the next queued type; nothing is retried) and S-0407 claim 10 (no ever-built count)
#[test]
fn claim_7_busy_release_cancels() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("JEEP").unwrap().cost = Some(50);
    let mut w = game(&r);
    let k = building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    rich(&mut w);
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let obj = item(&w, 0, jeep).unwrap().object;
    run_to_done(&mut w, 0, jeep, 200).expect("done");
    // K is still letting an earlier vehicle out when the automatic place order applies.
    let earlier = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, earlier), Outcome::Released);
    unloading_now(&mut w, k);
    let before = credits(&w, 0);
    production::place(&mut w, HouseId(0), jeep, None);
    assert!(w.obj(obj).is_none(), "the cancelled unit is deleted");
    assert_eq!(credits(&w, 0), before + 50);
    let i = item(&w, 0, tank).unwrap();
    assert!(!i.done());
    assert_eq!(
        glasnost_sim::econ::tech::ever_built(&w, HouseId(0), jeep),
        0
    );
}

// spec: S-0404 claim 13 (P12 part c: a naval factory's exit cell is the first cell beyond the foundation toward the rally point when it is water and empty; otherwise the nearby-cell search; facing 16384 and a move to the rally point)
#[test]
fn claim_13_naval_exit() {
    let run = |block: bool| {
        let mut w = game(&ruleset());
        let k = building(&mut w, "NAVYARD", 0, 0, 0);
        let tl = top_left(&w, k);
        let rally = Target::Cell(CellXY::new(tl.x + 10, tl.y + 1));
        w.obj_mut(k).unwrap().guard_spot = Some(rally);
        let exit = CellXY::new(tl.x + 2, tl.y + 1);
        w.cells.grid.get_mut(exit).unwrap().land = glasnost_map::terrain::land::WATER;
        if block {
            spawn_at(&mut w, Kind::Vehicle, "TANK", exit);
        }
        let boat = new_unit(&mut w, Kind::Vehicle, "BOAT");
        assert_eq!(release::release(&mut w, k, boat), Outcome::Released);
        let o = w.obj(boat).unwrap();
        assert_eq!(w.body_facing(boat), 16384);
        assert_eq!(o.dest, Some(rally));
        (o.pos.cell(), o.pos.x.rem_euclid(256), exit)
    };
    let (cell, sub, exit) = run(false);
    assert_eq!((cell, sub), (exit, 128));
    let (cell, _, exit) = run(true);
    assert_ne!(cell, exit);
}

/// Gives K a rally point far from it, so that an ordinary released
/// infantry unit walks to its exit cell first (S-0404 claim 13, S-0417
/// claim 10).
fn far_rally(w: &mut World, k: ObjectId) {
    let tl = top_left(w, k);
    w.obj_mut(k).unwrap().guard_spot = Some(Target::Cell(CellXY::new(tl.x + 1, tl.y + 8)));
}

// spec: S-0404 claim 13 (P12 part d: the infantry appears in the foundation cell next to the first free listed cell, facing its rounded direction; with a rally point outside its cell it walks to the exit cell first, S-0417 claim 10; with every listed cell blocked, failed)
#[test]
fn claim_13_infantry_exit_list() {
    let mut w = game(&ruleset());
    let k = building(&mut w, "BARRACKS", 0, 0, 0);
    far_rally(&mut w, k);
    let tl = top_left(&w, k);
    let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
    assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
    let exit = CellXY::new(tl.x, tl.y + 2);
    let inside = CellXY::new(tl.x, tl.y + 1);
    let o = w.obj(grunt).unwrap();
    assert_eq!(o.pos.cell(), inside);
    assert_eq!(o.dest, Some(Target::Cell(exit)));
    assert_eq!(o.contact, Some(k));
    assert_eq!(
        w.body_facing(grunt),
        facing_to(w.reference_point(k), Pos::centre(exit, 0))
    );
    // spec: S-0404 claim 13 (P12 part f: every docking place in contact: busy)
    let second = new_unit(&mut w, Kind::Infantry, "GRUNT");
    assert_eq!(release::release(&mut w, k, second), Outcome::Busy);
    // The next listed cell when the first is blocked.
    let mut w = game(&ruleset());
    let k = building(&mut w, "BARRACKS", 0, 0, 0);
    far_rally(&mut w, k);
    spawn_at(&mut w, Kind::Building, "HUT", exit);
    let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
    assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
    let o = w.obj(grunt).unwrap();
    assert_eq!(o.pos.cell(), CellXY::new(tl.x + 1, tl.y + 1));
    assert_eq!(o.dest, Some(Target::Cell(CellXY::new(tl.x + 1, tl.y + 2))));
    // Every listed cell blocked: failed.
    let mut w = game(&ruleset());
    let k = building(&mut w, "BARRACKS", 0, 0, 0);
    far_rally(&mut w, k);
    for (a, b) in release::exit_list(2, 2, false, true) {
        spawn_at(
            &mut w,
            Kind::Building,
            "HUT",
            CellXY::new(tl.x + a, tl.y + b),
        );
    }
    let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
    assert_eq!(release::release(&mut w, k, grunt), Outcome::Failed);
    assert!(!w.obj(grunt).unwrap().on_map);
}

// spec: S-0404 claim 13 (P12 part e: the door cell is tried first and the exit displacement is added only there)
#[test]
fn claim_13_door_cells() {
    for (door, cell) in [(0, (1, 2)), (1, (2, 2)), (2, (2, 1))] {
        let mut r = ruleset();
        let b = r.sim.buildings.get_mut("BARRACKS").unwrap();
        b.econ.exit_coord = Some([10, 20, 0]);
        match door {
            0 => b.econ.door_first = Some(true),
            1 => b.econ.door_second = Some(true),
            _ => b.econ.door_third = Some(true),
        }
        let mut w = game(&r);
        let k = building(&mut w, "BARRACKS", 0, 0, 0);
        far_rally(&mut w, k);
        let tl = top_left(&w, k);
        let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
        assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
        let exit = CellXY::new(tl.x + cell.0, tl.y + cell.1);
        let inside = Pos::centre(CellXY::new(tl.x + cell.0.min(1), tl.y + cell.1.min(1)), 0);
        let o = w.obj(grunt).unwrap();
        assert_eq!(o.dest, Some(Target::Cell(exit)), "door kind {door}");
        assert_eq!((o.pos.x, o.pos.y), (inside.x + 10, inside.y + 20));
        // With the door cell blocked, the list's first cell, without the displacement.
        let mut w = game(&r);
        let k = building(&mut w, "BARRACKS", 0, 0, 0);
        far_rally(&mut w, k);
        spawn_at(&mut w, Kind::Building, "HUT", exit);
        let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
        assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
        let o = w.obj(grunt).unwrap();
        let first = Pos::centre(CellXY::new(tl.x, tl.y + 1), 0);
        assert_eq!((o.pos.x, o.pos.y), (first.x, first.y));
    }
}

// spec: S-0404 claim 13 (the appearance cell: each exit-cell coordinate adjusted one cell toward the foundation rectangle, at or above x0 + W reduced by 1, below x0 raised by 1, also for the 0 × 0 word) and Edge cases (Q-0195: the 0 × 0 word's exit list is empty with quirks.foundation_exit_lists off as well as on, so only door candidates are tried)
#[test]
fn claim_13_door_release_without_foundation_cells() {
    for exact in [true, false] {
        assert!(release::ring(0, 0).is_empty());
        for (door, cell, at) in [
            (0, (1, 2), (0, 1)),
            (1, (2, 2), (1, 1)),
            (2, (2, 1), (1, 0)),
        ] {
            let mut r = ruleset();
            r.globals.quirks.foundation_exit_lists = exact;
            let b = r.sim.buildings.get_mut("BARRACKS").unwrap();
            b.foundation = Some([0, 0]);
            match door {
                0 => b.econ.door_first = Some(true),
                1 => b.econ.door_second = Some(true),
                _ => b.econ.door_third = Some(true),
            }
            let mut w = game(&r);
            let k = building(&mut w, "BARRACKS", 0, 0, 0);
            let tl = top_left(&w, k);
            let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
            assert_eq!(
                release::release(&mut w, k, grunt),
                Outcome::Released,
                "door {door}"
            );
            let p = Pos::centre(CellXY::new(tl.x + at.0, tl.y + at.1), 0);
            let o = w.obj(grunt).unwrap();
            assert_eq!((o.pos.x, o.pos.y), (p.x, p.y), "door {door} exact {exact}");
            // The door cell blocked: no list cell to fall back on.
            let mut w = game(&r);
            let k = building(&mut w, "BARRACKS", 0, 0, 0);
            spawn_at(
                &mut w,
                Kind::Building,
                "HUT",
                CellXY::new(tl.x + cell.0, tl.y + cell.1),
            );
            let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
            assert_eq!(
                release::release(&mut w, k, grunt),
                Outcome::Failed,
                "door {door} exact {exact}"
            );
        }
    }
}

// spec: S-0404 Edge cases (quirks.foundation_exit_lists: the 4 × 3 list never tries the cells beside the right edge; off, the full ring does)
#[test]
fn claim_13_quirk_foundation_exit_lists() {
    for exact in [true, false] {
        let mut r = ruleset();
        r.sim.buildings.get_mut("BARRACKS").unwrap().foundation = Some([4, 3]);
        r.globals.quirks.foundation_exit_lists = exact;
        let mut w = game(&r);
        let k = building(&mut w, "BARRACKS", 0, 0, 0);
        far_rally(&mut w, k);
        let tl = top_left(&w, k);
        for (a, b) in release::ring(4, 3) {
            if a != 4 || !(0..=2).contains(&b) {
                spawn_at(
                    &mut w,
                    Kind::Building,
                    "HUT",
                    CellXY::new(tl.x + a, tl.y + b),
                );
            }
        }
        let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
        let out = release::release(&mut w, k, grunt);
        if exact {
            assert_eq!(out, Outcome::Failed);
        } else {
            assert_eq!(out, Outcome::Released);
            // The ring's cells beside the right edge, from the bottom row upwards.
            assert_eq!(
                w.obj(grunt).unwrap().dest,
                Some(Target::Cell(CellXY::new(tl.x + 4, tl.y + 2)))
            );
        }
    }
}

// spec: S-0404 claim 13 (the aircraft row: docked at K facing the pose direction × 256 with a rally move; no dock: a dock-bound type fails, any other enters from the local area's edge facing 0 at (Sx + r, Sy + r))
#[test]
fn claim_13_aircraft() {
    let mut r = ruleset();
    r.sim.general.economy.pose_dir = Some(96);
    let mut w = game(&r);
    let k = building(&mut w, "AIRPAD", 0, 0, 0);
    let rally = Target::Cell(CellXY::new(top_left(&w, k).x + 5, top_left(&w, k).y));
    w.obj_mut(k).unwrap().guard_spot = Some(rally);
    let plane = new_unit(&mut w, Kind::Aircraft, "PLANE");
    assert_eq!(release::release(&mut w, k, plane), Outcome::Released);
    assert_eq!(w.body_facing(plane), 96 * 256);
    assert_eq!(w.obj(plane).unwrap().dest, Some(rally));
    // No docking place: the edge entry.
    let plane = new_unit(&mut w, Kind::Aircraft, "PLANE");
    let taken = w.rng.taken();
    assert_eq!(
        release::aircraft(&mut w, k, plane, false),
        Outcome::Released
    );
    let fp = *w.cells.footprint();
    let s = release::edge_start(fp.local, fp.width, w.reference_point(k).cell());
    let c = w.obj(plane).unwrap().pos.cell();
    let d = c.x - s.x;
    assert_eq!(c.y - s.y, d);
    assert!((0..=fp.local[3]).contains(&d));
    assert_eq!(w.body_facing(plane), 0);
    assert_eq!(w.obj(plane).unwrap().dest, Some(rally));
    if fp.local[3] > 0 {
        assert!(w.rng.taken() > taken);
    }
    // A dock-bound type fails.
    let mut r = r.clone();
    r.sim.aircraft.get_mut("PLANE").unwrap().econ.airport_bound = Some(true);
    let mut w = game(&r);
    let k = building(&mut w, "AIRPAD", 0, 0, 0);
    let plane = new_unit(&mut w, Kind::Aircraft, "PLANE");
    assert_eq!(release::aircraft(&mut w, k, plane, false), Outcome::Failed);
}

/// An aircraft factory with `docks` docking places and the given
/// displacements, a pad (Helipad) when `pad`.
fn pad_world(docks: Option<i32>, offsets: Option<Vec<[i32; 3]>>, pad: bool) -> (World, ObjectId) {
    let mut r = ruleset();
    let b = r.sim.buildings.get_mut("AIRPAD").unwrap();
    b.econ.number_of_docks = docks;
    b.econ.docking_offsets = offsets;
    b.helipad = Some(pad);
    let mut w = game(&r);
    let k = building(&mut w, "AIRPAD", 0, 0, 0);
    (w, k)
}

// spec: S-0404 claim 15 (a docked aircraft's position: c, K's foundation centre with its height, plus the displacement of its contact place for a pad; a count of 1 uses the sole displacement; without the flag, c; places without a listed displacement are 0, 0, 0; signed wrapping, not rotated)
#[test]
fn claim_15_pad_position() {
    let offs = vec![[100, -50, 7], [-30, 40, 0]];
    // Two places: the first aircraft takes place 0, the second place 1.
    let (mut w, k) = pad_world(Some(2), Some(offs.clone()), true);
    let c = w.reference_point(k);
    for (i, d) in offs.iter().enumerate() {
        let plane = new_unit(&mut w, Kind::Aircraft, "PLANE");
        assert_eq!(release::aircraft(&mut w, k, plane, true), Outcome::Released);
        let p = w.obj(plane).unwrap().pos;
        assert_eq!(
            (p.x, p.y, p.z),
            (c.x + d[0], c.y + d[1], c.z + d[2]),
            "place {i}"
        );
    }
    // An aircraft already in contact through place 1 keeps it, though place 0 is empty.
    let (mut w, k) = pad_world(Some(2), Some(offs.clone()), true);
    let c = w.reference_point(k);
    let first = new_unit(&mut w, Kind::Aircraft, "PLANE");
    let second = new_unit(&mut w, Kind::Aircraft, "PLANE");
    assert_eq!(release::aircraft(&mut w, k, first, true), Outcome::Released);
    assert_eq!(
        release::aircraft(&mut w, k, second, true),
        Outcome::Released
    );
    w.obj_mut(first).unwrap().contact = None;
    assert_eq!(
        release::aircraft(&mut w, k, second, true),
        Outcome::Released
    );
    let p = w.obj(second).unwrap().pos;
    assert_eq!((p.x, p.y), (c.x + offs[1][0], c.y + offs[1][1]));
    // A count of 1: the sole displacement.
    let (mut w, k) = pad_world(Some(1), Some(vec![[64, 64, 0]]), true);
    let c = w.reference_point(k);
    let plane = new_unit(&mut w, Kind::Aircraft, "PLANE");
    assert_eq!(release::aircraft(&mut w, k, plane, true), Outcome::Released);
    let p = w.obj(plane).unwrap().pos;
    assert_eq!((p.x, p.y), (c.x + 64, c.y + 64));
    // A count of 0, no listed displacement, or no pad flag: c.
    for (docks, offsets, pad) in [
        (Some(0), Some(vec![[64, 64, 0]]), true),
        (Some(2), None, true),
        (Some(2), Some(vec![[64, 64, 0]]), false),
    ] {
        let (mut w, k) = pad_world(docks, offsets.clone(), pad);
        let c = w.reference_point(k);
        let plane = new_unit(&mut w, Kind::Aircraft, "PLANE");
        assert_eq!(release::aircraft(&mut w, k, plane, true), Outcome::Released);
        let p = w.obj(plane).unwrap().pos;
        assert_eq!(
            (p.x, p.y, p.z),
            (c.x, c.y, c.z),
            "{docks:?} {offsets:?} {pad}"
        );
    }
}

// spec: S-0417 claim 14 (an edge-entering aircraft without a rally point: move is forced, with the nearby-cell search's result around K's reference cell as destination; the search takes no draw)
#[test]
fn claim_13_aircraft_edge_entry_without_rally() {
    let mut w = game(&ruleset());
    let k = building(&mut w, "AIRPAD", 0, 0, 0);
    let plane = new_unit(&mut w, Kind::Aircraft, "PLANE");
    assert_eq!(
        release::aircraft(&mut w, k, plane, false),
        Outcome::Released
    );
    let o = w.obj(plane).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Move));
    let Some(Target::Cell(d)) = o.dest else {
        panic!("a nearby cell destination: {:?}", o.dest)
    };
    let b = w.reference_point(k).cell();
    assert!(
        (d.x - b.x).abs() <= 8 && (d.y - b.y).abs() <= 8,
        "{d:?} near {b:?}"
    );
}

// spec: S-0404 claim 14 (P12 part g: each cloning building, in acquisition order, gets a copy released by the infantry row, each with its creation draw; none when the original is busy)
#[test]
fn claim_14_cloning() {
    let mut r = ruleset();
    let mut vat = plain(100);
    vat.econ.cloning = Some(true);
    add_building(&mut r, "VAT", vat);
    let mut w = game(&r);
    let k = building(&mut w, "BARRACKS", 0, 0, 0);
    let v1 = building(&mut w, "VAT", 0, -8, 0);
    let v2 = building(&mut w, "VAT", 0, 8, 0);
    let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
    let taken = w.rng.taken();
    assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
    assert!(w.rng.taken() >= taken + 2);
    let ti = w.rules().index_of(Kind::Infantry, "GRUNT").unwrap();
    let grunts: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Infantry && o.type_index == ti && o.on_map)
        .map(|o| o.id)
        .collect();
    assert_eq!(grunts.len(), 3);
    // The copies, in creation order, leave the first and then the second cloning building.
    let copies: Vec<ObjectId> = grunts.into_iter().filter(|&g| g != grunt).collect();
    assert_eq!(w.obj(copies[0]).unwrap().contact, Some(v1));
    assert_eq!(w.obj(copies[1]).unwrap().contact, Some(v2));
    // The original busy: no copies.
    let before = w.objects.len();
    let again = new_unit(&mut w, Kind::Infantry, "GRUNT");
    assert_eq!(release::release(&mut w, k, again), Outcome::Busy);
    assert_eq!(w.objects.len(), before + 1);
}

// spec: S-0404 claim 13 (the unit keeps K's rally point as its later destination), S-0417 claim 10 (saved as guard spot; the clear report ends contact; the guard-spot row of S-0215 claim 13 gives the rally) and S-0404 claim 13 (with a rally point outside its cell it walks to its exit cell first)
#[test]
fn claim_13_way_out() {
    let mut w = game(&ruleset());
    let k = building(&mut w, "BARRACKS", 0, 0, 0);
    let tl = top_left(&w, k);
    let rally = Target::Cell(CellXY::new(tl.x + 1, tl.y + 6));
    w.obj_mut(k).unwrap().guard_spot = Some(rally);
    let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
    assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
    assert_eq!(w.obj(grunt).unwrap().guard_spot, Some(rally));
    for _ in 0..300 {
        step(&mut w);
        if w.obj(grunt).unwrap().dest == Some(rally) {
            break;
        }
    }
    let o = w.obj(grunt).unwrap();
    assert!(o.econ.exit.is_none());
    assert_eq!(o.contact, None);
    assert_eq!(o.dest, Some(rally));
    // A ground factory's unloading ends once its vehicle is out.
    let mut w = game(&ruleset());
    let k = building(&mut w, "WARFACT", 0, 0, 0);
    let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    step(&mut w);
    assert_eq!(w.obj(k).unwrap().beh.current, Some(Behaviour::Unload));
    for _ in 0..300 {
        step(&mut w);
        if w.obj(k).unwrap().beh.current != Some(Behaviour::Unload) {
            break;
        }
    }
    assert_eq!(w.obj(k).unwrap().beh.current, Some(Behaviour::Guard));
    assert_eq!(w.obj(tank).unwrap().contact, None);
    let cell = w.obj(tank).unwrap().pos.cell();
    let tl = top_left(&w, k);
    assert!(!(tl.x..tl.x + 2).contains(&cell.x) || !(tl.y..tl.y + 2).contains(&cell.y));
}
