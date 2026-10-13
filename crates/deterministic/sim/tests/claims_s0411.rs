//! Claim tests for the simulation-side rules of S-0411 (sidebar behaviour)
//! on synthetic data: the click tables, icon states, legality, and the
//! orders the sidebar gives by itself for every player house.

mod econ_common;

use econ_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::sidebar::{self, Click, ClickOutcome};
use glasnost_sim::objects;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, World};

const H: HouseId = HouseId(0);

fn click(w: &World, ty: TypeRef, b: Click, shift: bool) -> ClickOutcome {
    sidebar::click(w, H, ty, b, shift)
}

fn order(o: Order) -> ClickOutcome {
    ClickOutcome::Order(o)
}

fn cheap(r: &mut Ruleset) {
    r.sim.vehicles.get_mut("JEEP").unwrap().cost = Some(50);
    r.sim.buildings.get_mut("PLANT").unwrap().cost = Some(50);
}

fn base(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let f = building(&mut w, "WARFACT", 0, 0, 0);
    let c = building(&mut w, "CONYARD", 0, -6, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 1_000_000);
    (w, f, c)
}

// spec: S-0411 claim 1 (each line fed independently: icons of one tab that feed different lines can be busy together)
#[test]
fn claim_1_tabs_and_lines() {
    let (mut w, _, _) = base(&ruleset());
    building(&mut w, "NAVYARD", 0, 6, 0);
    step(&mut w);
    let tank = t(&w, Kind::Vehicle, "TANK");
    let boat = t(&w, Kind::Vehicle, "BOAT");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    assert!(issue(&mut w, 0, Order::Build { what: boat }));
    step(&mut w);
    assert!(item(&w, 0, tank).is_some() && item(&w, 0, boat).is_some());
}

fn preq_rules() -> Ruleset {
    let mut r = ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("PREQ".into());
    r.sim.buildings.insert("PREQ".into(), plain(100));
    r.sim.vehicles.get_mut("TANK").unwrap().econ.prerequisite = Some(vec!["PREQ".into()]);
    r
}

// spec: S-0411 claim 2 (a recheck removes the type from the taken-up set: the icon goes, a cancel order then a cancel-all order are given in that house update)
// spec: S-0404 claim 9 (the orders for every player house, issued in the house update of the recheck)
#[test]
fn claim_2_removal_orders() {
    let mut w = game(&preq_rules());
    building(&mut w, "WARFACT", 0, 0, 0);
    let p = building(&mut w, "PREQ", 0, 6, 6);
    ready(&mut w);
    set_credits(&mut w, 0, 1_000_000);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(sidebar::icons(&w, H).contains(&tank));
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    // The prerequisite leaves the map (as in a turn pass): a recheck event.
    objects::remove(&mut w, p);
    assert!(w.houses[0].econ.tech.recheck_due);
    glasnost_sim::econ::tech::house_update(&mut w, H);
    assert_eq!(
        w.houses[0].pending,
        vec![
            Order::Cancel { what: tank },
            Order::CancelAll { what: tank }
        ]
    );
    assert!(!sidebar::icons(&w, H).contains(&tank));
    step(&mut w);
    assert!(line(&w, 0, tank).is_none());
    assert_eq!(credits(&w, 0), 1_000_000);
}

// spec: S-0411 claim 2 (a destruction in the turn pass of tick n: recheck, removal and orders in tick n, applied at its end)
#[test]
fn claim_2_same_tick() {
    let mut w = game(&preq_rules());
    building(&mut w, "WARFACT", 0, 0, 0);
    let p = building(&mut w, "PREQ", 0, 6, 6);
    ready(&mut w);
    set_credits(&mut w, 0, 1_000_000);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    // Removed between ticks, as by the turn pass of the next tick.
    objects::remove(&mut w, p);
    step(&mut w);
    assert!(line(&w, 0, tank).is_none());
    assert!(!sidebar::icons(&w, H).contains(&tank));
}

// spec: S-0411 claim 2 (the start of a sale alone removes no icon and gives no orders; the icon goes at the first house update after the building's removal)
// spec: S-0407 claim 12 and quirks.sale_start_no_recheck (T14 run 7: the start of a sale is no event)
#[test]
fn claim_2_sale_keeps_icon() {
    let (mut w, f, _) = base(&ruleset());
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    // The sale of the only vehicle factory starts (its behaviour becomes
    // selling, S-0407 Terms): the type is no longer buildable.
    w.obj_mut(f).unwrap().beh.current = Some(glasnost_sim::rules::Behaviour::Selling);
    assert!(!glasnost_sim::econ::tech::buildable(&w, H, tank));
    let progress = item(&w, 0, tank).unwrap().progress;
    for _ in 0..40 {
        step(&mut w);
        assert!(w.houses[0].pending.is_empty());
    }
    // The icon stays through the sale, and nothing cancels the item.
    assert!(sidebar::icons(&w, H).contains(&tank));
    assert!(item(&w, 0, tank).unwrap().progress > progress);
    // In the span the click table still gives the order of its row
    // (claim 4): a right click holds the running item.
    assert_eq!(
        click(&w, tank, Click::Right, false),
        order(Order::Hold { what: tank })
    );
    // The sale's removal: the icon goes at the next house update.
    objects::remove(&mut w, f);
    step(&mut w);
    assert!(!sidebar::icons(&w, H).contains(&tank));
    assert!(line(&w, 0, tank).is_none());
}

// spec: S-0411 claim 2 (a type that stops being buildable without a recheck event keeps its icon; a later recheck event removes it)
#[test]
fn claim_2_stale_until_recheck() {
    let mut w = game(&preq_rules());
    building(&mut w, "WARFACT", 0, 0, 0);
    let p = building(&mut w, "PREQ", 0, 6, 6);
    ready(&mut w);
    set_credits(&mut w, 0, 1_000_000);
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    // The prerequisite goes without any event of S-0407 claim 12 (taken
    // out of the object table directly).
    w.objects.remove(&p);
    for _ in 0..10 {
        step(&mut w);
    }
    assert!(sidebar::icons(&w, H).contains(&tank));
    assert!(item(&w, 0, tank).is_some());
    // An unrelated building placed: event (a), the recheck shows the change.
    building(&mut w, "HUT", 0, -6, 6);
    step(&mut w);
    assert!(!sidebar::icons(&w, H).contains(&tank));
    assert!(line(&w, 0, tank).is_none());
}

// spec: S-0411 claim 3 (darkened: a building line with an item, no switched-on factory, at limit; never the item's own icon)
#[test]
fn claim_3_darkened() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("JEEP").unwrap().econ.build_limit = Some(1);
    let (mut w, f, _) = base(&r);
    let hut = t(&w, Kind::Building, "HUT");
    let plant = t(&w, Kind::Building, "PLANT");
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(!sidebar::darkened(&w, H, hut));
    assert!(issue(&mut w, 0, Order::Build { what: hut }));
    step(&mut w);
    assert!(!sidebar::darkened(&w, H, hut));
    assert!(sidebar::darkened(&w, H, plant));
    objects::create(&mut w, Kind::Vehicle, jeep.index, Some(H));
    assert!(sidebar::darkened(&w, H, jeep));
    assert!(!sidebar::darkened(&w, H, tank));
    switch(&mut w, f, false);
    assert!(sidebar::darkened(&w, H, tank));
}

// spec: S-0411 claim 4 (the left-click table: new build, refusal on a busy building line, at limit, resume, done items)
#[test]
fn claim_4_left_click() {
    let mut r = ruleset();
    cheap(&mut r);
    r.sim.vehicles.get_mut("TANK").unwrap().econ.build_limit = Some(1);
    let (mut w, f, _) = base(&r);
    let hut = t(&w, Kind::Building, "HUT");
    let plant = t(&w, Kind::Building, "PLANT");
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    let tank = t(&w, Kind::Vehicle, "TANK");
    // Last row: a build order.
    assert_eq!(
        click(&w, plant, Click::Left, false),
        order(Order::Build { what: plant })
    );
    assert!(issue(&mut w, 0, Order::Build { what: plant }));
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    step(&mut w);
    // Row 6: another building while the line has an item.
    assert_eq!(click(&w, hut, Click::Left, false), ClickOutcome::Refused);
    // Row 7: at limit.
    objects::create(&mut w, Kind::Vehicle, tank.index, Some(H));
    assert_eq!(click(&w, tank, Click::Left, false), ClickOutcome::Nothing);
    // Row 5: a held item resumes.
    assert!(issue(&mut w, 0, Order::Hold { what: jeep }));
    step(&mut w);
    assert_eq!(
        click(&w, jeep, Click::Left, false),
        order(Order::Build { what: jeep })
    );
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    // Row 4: a done building starts placement mode.
    run_to_done(&mut w, 0, plant, 200).expect("done");
    assert_eq!(
        click(&w, plant, Click::Left, false),
        ClickOutcome::PlacementMode
    );
    let _ = f;
}

// spec: S-0411 claim 4 row 2 (a done unit with a factory: a place order without a cell)
#[test]
fn claim_4_done_unit() {
    let mut r = ruleset();
    cheap(&mut r);
    let (mut w, f, _) = base(&r);
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    run_to_done(&mut w, 0, jeep, 200).expect("done");
    // Switched off for the automatic order, then on again: the item waits for a click.
    switch(&mut w, f, false);
    step(&mut w);
    switch(&mut w, f, true);
    assert!(item(&w, 0, jeep).unwrap().done());
    assert_eq!(
        click(&w, jeep, Click::Left, false),
        order(Order::Place {
            what: jeep,
            cell: None
        })
    );
}

// spec: S-0411 claim 4 rows 1 and 3 (a done item and no factory could make it: a cancel order)
#[test]
fn claim_4_done_without_factory() {
    let mut r = ruleset();
    cheap(&mut r);
    let (mut w, _, c) = base(&r);
    let plant = t(&w, Kind::Building, "PLANT");
    assert!(issue(&mut w, 0, Order::Build { what: plant }));
    run_to_done(&mut w, 0, plant, 200).expect("done");
    // The factory starts its sale: it no longer counts (S-0404 Terms).
    w.obj_mut(c).unwrap().beh.current = Some(glasnost_sim::rules::Behaviour::Selling);
    assert_eq!(
        click(&w, plant, Click::Left, false),
        order(Order::Cancel { what: plant })
    );
}

// spec: S-0411 claim 4 Legality and claim 6 (a starting or queuing build order is legal when offered, not at limit, and for a building line with no item; the resume is exempt)
#[test]
fn claim_4_6_legality() {
    let mut r = ruleset();
    r.sim.buildings.get_mut("HUT").unwrap().econ.build_limit = Some(1);
    let (mut w, _, _) = base(&r);
    let hut = t(&w, Kind::Building, "HUT");
    let plant = t(&w, Kind::Building, "PLANT");
    assert!(issue(&mut w, 0, Order::Build { what: hut }));
    step(&mut w);
    assert!(!issue(&mut w, 0, Order::Build { what: plant }));
    assert!(issue(&mut w, 0, Order::Hold { what: hut }));
    step(&mut w);
    // The resume of the held item is legal whatever its limit and the busy line.
    assert!(issue(&mut w, 0, Order::Build { what: hut }));
    assert!(!issue(&mut w, 0, Order::Build { what: plant }));
    // Place orders with a cell ask S-0406 (a stub until it is implemented: legal).
    assert!(issue(
        &mut w,
        0,
        Order::Place {
            what: hut,
            cell: Some(testkit_cell())
        }
    ));
}

// spec: S-0411 claim 3 (S3 third run: a stale prerequisite refusal alone does not darken an icon, and its build order starts an item)
// spec: S-0407 claim 13 and quirks.stale_offer_orderable (a stale offer can be ordered and produced; Q-0122)
#[test]
fn claim_3_stale_refusal_lit() {
    let mut r = ruleset();
    r.sim.general.economy.prerequisite_proc_alternate = Some("JEEP".into());
    r.sim.infantry.get_mut("GRUNT").unwrap().econ.prerequisite = Some(vec!["PROC".into()]);
    let mut w = game(&r);
    building(&mut w, "BARRACKS", 0, -6, 0);
    let v = glasnost_sim::testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "JEEP",
        0,
        glasnost_sim::testkit::mid(SIZE, 4, 4),
        0,
    );
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    assert!(sidebar::icons(&w, H).contains(&grunt));
    // The vehicle's end is no recheck event (S-0407 claim 12 (o)).
    objects::remove(&mut w, v);
    step(&mut w);
    assert_eq!(
        glasnost_sim::econ::tech::answer(&w, H, grunt),
        glasnost_sim::econ::tech::Answer::Refused
    );
    assert!(!sidebar::darkened(&w, H, grunt));
    assert_eq!(
        click(&w, grunt, Click::Left, false),
        order(Order::Build { what: grunt })
    );
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    step(&mut w);
    assert!(item(&w, 0, grunt).is_some());
}

// spec: S-0411 claim 4 Legality (S4 b2: the resume click of a held item at its limit is legal; the application-time limit allowance decides)
// spec: S-0407 claim 9 (the infantry exception holds only at O = L)
#[test]
fn claim_4_resume_at_limit() {
    for extra in [false, true] {
        let mut r = ruleset();
        r.sim.infantry.get_mut("GRUNT").unwrap().econ.build_limit = Some(1);
        let mut w = game(&r);
        building(&mut w, "BARRACKS", 0, -6, 0);
        ready(&mut w);
        set_credits(&mut w, 0, 100_000);
        let grunt = t(&w, Kind::Infantry, "GRUNT");
        assert!(issue(&mut w, 0, Order::Build { what: grunt }));
        step(&mut w);
        assert!(issue(&mut w, 0, Order::Hold { what: grunt }));
        step(&mut w);
        if extra {
            // An extra infantry of the type on the map: O = 2 > L.
            glasnost_sim::testkit::spawn(
                &mut w,
                Kind::Infantry,
                "GRUNT",
                0,
                glasnost_sim::testkit::mid(SIZE, 4, 4),
                0,
            );
        }
        assert_eq!(
            click(&w, grunt, Click::Left, false),
            order(Order::Build { what: grunt })
        );
        assert!(issue(&mut w, 0, Order::Build { what: grunt }));
        step(&mut w);
        let i = item(&w, 0, grunt).unwrap();
        // Without the extra object the item resumes; with it, it stays held
        // and nothing queues.
        assert_eq!(i.hold == glasnost_sim::econ::production::Hold::None, !extra);
        assert!(line(&w, 0, grunt).unwrap().queue.is_empty());
    }
}

fn testkit_cell() -> glasnost_map::CellXY {
    glasnost_sim::testkit::mid(SIZE, 8, 8)
}

// spec: S-0411 claim 5 (right click: hold a running item, cancel a held or done one, Shift for cancel-all, a queued copy, else nothing)
#[test]
fn claim_5_right_click() {
    let (mut w, _, _) = base(&ruleset());
    let tank = t(&w, Kind::Vehicle, "TANK");
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    assert_eq!(click(&w, tank, Click::Right, false), ClickOutcome::Nothing);
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    step(&mut w);
    assert_eq!(
        click(&w, tank, Click::Right, false),
        order(Order::Hold { what: tank })
    );
    assert_eq!(
        click(&w, jeep, Click::Right, false),
        order(Order::Cancel { what: jeep })
    );
    assert_eq!(
        click(&w, jeep, Click::Right, true),
        order(Order::CancelAll { what: jeep })
    );
    assert!(issue(&mut w, 0, Order::Hold { what: tank }));
    step(&mut w);
    assert_eq!(
        click(&w, tank, Click::Right, false),
        order(Order::Cancel { what: tank })
    );
    assert_eq!(
        click(&w, tank, Click::Right, true),
        order(Order::CancelAll { what: tank })
    );
}

// spec: S-0411 claim 7 (the queue count: shown while a copy is queued; counts the item of the type)
#[test]
fn claim_7_queue_count() {
    let mut r = ruleset();
    r.sim.general.economy.maximum_queued_objects = Some(5);
    let mut w = game(&r);
    building(&mut w, "BARRACKS", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 1_000_000);
    let a = t(&w, Kind::Infantry, "GRUNT");
    let b = t(&w, Kind::Infantry, "MEDIC");
    for ty in [a, a, a, b] {
        assert!(issue(&mut w, 0, Order::Build { what: ty }));
    }
    step(&mut w);
    assert_eq!(sidebar::queue_count(&w, H, a), Some(3));
    assert_eq!(sidebar::queue_count(&w, H, b), Some(1));
    assert!(issue(&mut w, 0, Order::Cancel { what: a }));
    assert!(issue(&mut w, 0, Order::Cancel { what: a }));
    step(&mut w);
    assert_eq!(sidebar::queue_count(&w, H, a), None);
}

// spec: S-0411 claim 8 (clock stage = progress + 1; ready when done; on hold while held; nothing while waiting for money)
#[test]
fn claim_8_icon_state() {
    let (mut w, _, _) = base(&ruleset());
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let s = sidebar::icon_state(&w, H, tank).unwrap();
    assert_eq!((s.stage, s.ready, s.on_hold), (1, false, false));
    for _ in 0..40 {
        step(&mut w);
    }
    let p = item(&w, 0, tank).unwrap().progress;
    assert_eq!(sidebar::icon_state(&w, H, tank).unwrap().stage, p + 1);
    set_credits(&mut w, 0, 0);
    for _ in 0..20 {
        step(&mut w);
    }
    let s = sidebar::icon_state(&w, H, tank).unwrap();
    assert!(!s.on_hold && !s.ready);
    assert!(issue(&mut w, 0, Order::Hold { what: tank }));
    step(&mut w);
    assert!(sidebar::icon_state(&w, H, tank).unwrap().on_hold);
}

// spec: S-0411 claim 9 (the automatic place order in tick n + 1 for a unit done in tick n; given once with quirks.release_order_once)
#[test]
fn claim_9_automatic_release() {
    for once in [true, false] {
        let mut r = ruleset();
        cheap(&mut r);
        r.globals.quirks.release_order_once = once;
        let (mut w, f, _) = base(&r);
        let jeep = t(&w, Kind::Vehicle, "JEEP");
        assert!(issue(&mut w, 0, Order::Build { what: jeep }));
        let done = run_to_done(&mut w, 0, jeep, 200).expect("done");
        assert_eq!(w.tick, done + 1);
        // No releasing building at the order of tick done + 1.
        switch(&mut w, f, false);
        step(&mut w);
        switch(&mut w, f, true);
        for _ in 0..3 {
            step(&mut w);
        }
        // Once: the item stays done; off: the order is repeated and the unit leaves.
        assert_eq!(item(&w, 0, jeep).is_some(), once);
    }
}

// spec: S-0411 claim 9 (the unit leaves at the end of the tick after its done tick)
#[test]
fn claim_9_release_tick() {
    let mut r = ruleset();
    cheap(&mut r);
    let (mut w, _, _) = base(&r);
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    step(&mut w);
    let obj = item(&w, 0, jeep).unwrap().object;
    let done = run_to_done(&mut w, 0, jeep, 200).expect("done");
    assert_eq!(w.tick, done + 1);
    assert!(!w.obj(obj).unwrap().on_map);
    step(&mut w);
    assert!(w.obj(obj).unwrap().on_map);
}
