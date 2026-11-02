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

// spec: S-0411 claim 14 (production during a sale: a factory queued to be sold keeps its running item paying, gives no new start, queue or release; its removal refunds the item once, in the removal tick, and the later cancellation orders refund nothing more)
// spec: S-0404 claim 8 (the permanent-removal refund)
#[test]
fn claim_14_production_during_a_sale() {
    let (mut w, f, _) = base(&ruleset());
    let tank = t(&w, Kind::Vehicle, "TANK");
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    // The sale is queued, not yet current.
    w.obj_mut(f).unwrap().beh.queued = Some(glasnost_sim::rules::Behaviour::Selling);
    // No body range runs, so no ready mark starts the queued sale (S-0215 claim 11).
    w.obj_mut(f).unwrap().econ.idle_body = None;
    w.obj_mut(f).unwrap().beh.ready_mark = false;
    let (progress, paid) = (item(&w, 0, tank).unwrap().progress, credits(&w, 0));
    for _ in 0..40 {
        step(&mut w);
        assert!(w.houses[0].pending.is_empty());
    }
    assert!(item(&w, 0, tank).unwrap().progress > progress);
    assert!(credits(&w, 0) < paid);
    assert!(sidebar::icons(&w, H).contains(&tank));
    // A queue request is legal (the type is offered) but not applied: the
    // only factory is queued to be sold.
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    step(&mut w);
    assert!(line(&w, 0, tank).unwrap().queue.is_empty());
    // The removal: the refund comes at once, the icon at the next house
    // update, whose cancellation orders refund nothing more.
    let i = item(&w, 0, tank).unwrap();
    let price = glasnost_sim::econ::production::price(&w, H, tank);
    let before = credits(&w, 0);
    objects::remove(&mut w, f);
    assert!(item(&w, 0, tank).is_none());
    assert_eq!(credits(&w, 0), before + price - i.owed);
    let refunded = credits(&w, 0);
    step(&mut w);
    assert!(!sidebar::icons(&w, H).contains(&tank));
    assert_eq!(credits(&w, 0), refunded);
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
    // Place orders with a cell are legal exactly when S-0406's conditions
    // (claims 4 to 6) hold; a cell far from every building fails them.
    let legal = glasnost_sim::econ::placement::place_legal(
        &w,
        glasnost_sim::world::HouseId(0),
        hut,
        testkit_cell(),
    );
    assert_eq!(
        issue(
            &mut w,
            0,
            Order::Place {
                what: hut,
                cell: Some(testkit_cell())
            }
        ),
        legal
    );
    let far = glasnost_sim::testkit::mid(SIZE, 20, -20);
    assert!(!issue(
        &mut w,
        0,
        Order::Place {
            what: hut,
            cell: Some(far)
        }
    ));
}

// spec: S-0411 claim 3 (S3 third run: a stale prerequisite refusal alone does not darken an icon, and its build order starts an item)
// spec: S-0407 claim 13 and quirks.stale_offer_orderable (a stale offer can be ordered and produced; S-0404 claim 3's application test has no full tech-tree test)
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

// spec: S-0411 claim 9 (S7 third run: a failed infantry release cancels the done item with S-0404 claim 6's refund at the automatic order's application boundary; clearing the exit later releases nothing and gives no retry)
// spec: S-0411 claim 8 (the ready label while the unit item is done, until the place order releases or cancels it)
#[test]
fn claim_9_failed_release_cancels() {
    let mut r = ruleset();
    r.sim.infantry.get_mut("GRUNT").unwrap().cost = Some(50);
    let mut w = game(&r);
    let k = building(&mut w, "BARRACKS", 0, 0, 0);
    let tl = w.obj(k).unwrap().pos.cell();
    // Every candidate exit cell of S-0404 claim 13 blocked.
    let huts: Vec<ObjectId> = glasnost_sim::econ::release::exit_list(2, 2, false, true)
        .into_iter()
        .map(|(a, b)| {
            glasnost_sim::testkit::spawn(
                &mut w,
                Kind::Building,
                "HUT",
                0,
                glasnost_map::CellXY::new(tl.x + a, tl.y + b),
                0,
            )
        })
        .collect();
    ready(&mut w);
    set_credits(&mut w, 0, 1_000_000);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    step(&mut w);
    let obj = item(&w, 0, grunt).unwrap().object;
    let done = run_to_done(&mut w, 0, grunt, 200).expect("done");
    assert_eq!(w.tick, done + 1);
    assert!(sidebar::icon_state(&w, H, grunt).unwrap().ready);
    assert_eq!(credits(&w, 0), 1_000_000 - 50);
    // The automatic place order of tick done + 1 is applied at its end.
    step(&mut w);
    assert!(item(&w, 0, grunt).is_none());
    assert!(sidebar::icon_state(&w, H, grunt).is_none());
    assert_eq!(credits(&w, 0), 1_000_000);
    assert!(w.obj(obj).is_none_or(|o| !o.on_map));
    for h in huts {
        objects::remove(&mut w, h);
    }
    for _ in 0..3 {
        step(&mut w);
    }
    assert!(item(&w, 0, grunt).is_none());
    assert!(w.obj(obj).is_none_or(|o| !o.on_map));
    assert_eq!(credits(&w, 0), 1_000_000);
}

// spec: S-0004 claim 11 (rules 3 to 6: category rank, then tech level, then cost when added; a full tie puts the newest first under quirks.sidebar_tie_newest_first)
// spec: S-0411 claim 13 (the icon order is kept for every player house)
#[test]
fn claim_13_icon_order() {
    for newest_first in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.sidebar_tie_newest_first = newest_first;
        // A copy of TANK, registered after it: tied with it on every key.
        r.registries
            .get_mut("vehicles")
            .unwrap()
            .push("TANK2".into());
        let copy = r.sim.vehicles["TANK"].clone();
        r.sim.vehicles.insert("TANK2".into(), copy);
        let mut w = game(&r);
        building(&mut w, "WARFACT", 0, 0, 0);
        building(&mut w, "NAVYARD", 0, 6, 0);
        building(&mut w, "BARRACKS", 0, -6, 6);
        ready(&mut w);
        let v = |n| t(&w, Kind::Vehicle, n);
        let (jeep, tank, tank2, boat) = (v("JEEP"), v("TANK"), v("TANK2"), v("BOAT"));
        let grunt = t(&w, Kind::Infantry, "GRUNT");
        let icons = sidebar::icons(&w, H);
        let vehicles: Vec<TypeRef> = icons
            .iter()
            .copied()
            .filter(|x| x.kind == Kind::Vehicle)
            .collect();
        // JEEP is cheaper; BOAT is naval (rank 3); TANK2 ties TANK.
        if newest_first {
            assert_eq!(vehicles, vec![jeep, tank2, tank, boat]);
        } else {
            assert_eq!(vehicles, vec![jeep, tank, tank2, boat]);
        }
        // The third tab (infantry) comes before the fourth.
        let pos = |x| icons.iter().position(|y| *y == x).unwrap();
        assert!(pos(grunt) < pos(jeep));
    }
}

// spec: S-0004 claim 11 rule 3 (a vehicle that counts as aircraft ranks air, 1: after ground, before naval, ahead of a lower cost)
// spec: S-0004 Import mapping (counts as aircraft = ConsideredAircraft)
#[test]
fn claim_13_icon_order_counts_as_aircraft() {
    let mut r = ruleset();
    r.sim
        .vehicles
        .get_mut("JEEP")
        .unwrap()
        .econ
        .considered_aircraft = Some(true);
    let mut w = game(&r);
    building(&mut w, "WARFACT", 0, 0, 0);
    building(&mut w, "NAVYARD", 0, 6, 0);
    ready(&mut w);
    let v = |n| t(&w, Kind::Vehicle, n);
    let (jeep, tank, boat) = (v("JEEP"), v("TANK"), v("BOAT"));
    let vehicles: Vec<TypeRef> = sidebar::icons(&w, H)
        .into_iter()
        .filter(|x| x.kind == Kind::Vehicle)
        .collect();
    assert_eq!(vehicles, vec![tank, jeep, boat]);
}

// spec: S-0407 claim 17 (new offers of several kinds at one recheck: the kind whose earliest qualifying factory the house acquired later is admitted later, so a full tie puts it first under quirks.sidebar_tie_newest_first; no fixed precedence of kinds)
#[test]
fn s0407_claim_17_kinds_by_factory_acquisition() {
    for vehicles_first in [true, false] {
        let mut r = ruleset();
        r.registries
            .get_mut("vehicles")
            .unwrap()
            .push("VAIR".into());
        let mut vair = r.sim.vehicles["TANK"].clone();
        vair.econ.considered_aircraft = Some(true);
        r.sim.vehicles.insert("VAIR".into(), vair);
        r.sim
            .aircraft
            .get_mut("PLANE")
            .unwrap()
            .econ
            .considered_aircraft = Some(true);
        let mut w = game(&r);
        let order = if vehicles_first {
            ["WARFACT", "AIRPAD"]
        } else {
            ["AIRPAD", "WARFACT"]
        };
        building(&mut w, order[0], 0, 0, 0);
        building(&mut w, order[1], 0, 6, 0);
        ready(&mut w);
        let vair = t(&w, Kind::Vehicle, "VAIR");
        let plane = t(&w, Kind::Aircraft, "PLANE");
        let tied: Vec<TypeRef> = sidebar::icons(&w, H)
            .into_iter()
            .filter(|x| *x == vair || *x == plane)
            .collect();
        if vehicles_first {
            assert_eq!(tied, vec![plane, vair]);
        } else {
            assert_eq!(tied, vec![vair, plane]);
        }
    }
}

// spec: S-0004 r20 claim 11 rule 3 and Import mapping (an aircraft type whose data leaves the counts-as-aircraft key out has the flag, so, unless it also has the naval flag, it ranks as air; open question 2 resolved) and S-0501 Import mapping (ConsideredAircraft absent: yes for an aircraft type, so an aircraft type that leaves the key out ranks as air, after a ground vehicle tied with it on every other key; scenario AC30 part j); the control sets the key to no, and the tie then puts the later offer first (S-0407 claim 17)
#[test]
fn absent_counts_as_aircraft_ranks_an_aircraft_as_air() {
    for flag in [None, Some(false)] {
        let mut r = ruleset();
        r.registries
            .get_mut("vehicles")
            .unwrap()
            .push("VGND".into());
        let vgnd = r.sim.vehicles["TANK"].clone();
        r.sim.vehicles.insert("VGND".into(), vgnd);
        r.sim
            .aircraft
            .get_mut("PLANE")
            .unwrap()
            .econ
            .considered_aircraft = flag;
        let mut w = game(&r);
        building(&mut w, "WARFACT", 0, 0, 0);
        building(&mut w, "AIRPAD", 0, 6, 0);
        ready(&mut w);
        let vgnd = t(&w, Kind::Vehicle, "VGND");
        let plane = t(&w, Kind::Aircraft, "PLANE");
        let pair: Vec<TypeRef> = sidebar::icons(&w, H)
            .into_iter()
            .filter(|x| *x == vgnd || *x == plane)
            .collect();
        if flag.is_none() {
            assert_eq!(pair, vec![vgnd, plane], "air after ground");
        } else {
            assert_eq!(pair, vec![plane, vgnd], "a full tie: the later offer first");
        }
    }
}

fn two_tab_rules() -> Ruleset {
    let mut r = preq_rules();
    r.sim.infantry.get_mut("GRUNT").unwrap().econ.prerequisite = Some(vec!["PREQ".into()]);
    r.sim.buildings.get_mut("HUT").unwrap().econ.prerequisite = Some(vec!["PREQ".into()]);
    r
}

// spec: S-0411 claim 13 (several offers removed at one recheck: the orders in tab order, first to fourth, not registration order; each type's cancel before its cancel-all, and all its orders before the next type's)
#[test]
fn claim_13_tab_order() {
    let mut w = game(&two_tab_rules());
    building(&mut w, "BARRACKS", 0, 0, 0);
    building(&mut w, "CONYARD", 0, -6, 0);
    let p = building(&mut w, "PREQ", 0, 6, 6);
    ready(&mut w);
    set_credits(&mut w, 0, 1_000_000);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    let hut = t(&w, Kind::Building, "HUT");
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    assert!(issue(&mut w, 0, Order::Build { what: hut }));
    step(&mut w);
    objects::remove(&mut w, p);
    glasnost_sim::econ::tech::house_update(&mut w, H);
    // HUT (first tab) before GRUNT (third tab), although infantry types
    // come first in the set.
    assert_eq!(
        w.houses[0].pending,
        vec![
            Order::Cancel { what: hut },
            Order::CancelAll { what: hut },
            Order::Cancel { what: grunt },
            Order::CancelAll { what: grunt },
        ]
    );
}

// spec: S-0411 claim 13 (an item of A, one queued B, both removed: A's icon first gives cancel A, cancel-all A, cancel-all B, and B starts at the first cancellation, takes one creation draw and is refunded 0; B's icon first removes B before cancel A and no B item starts)
// spec: S-0404 claim 11 (one creation draw per item started)
#[test]
fn claim_13_intermediate_start() {
    let mut draws = Vec::new();
    for a_first in [true, false] {
        let mut r = preq_rules();
        let jeep = r.sim.vehicles.get_mut("JEEP").unwrap();
        jeep.econ.prerequisite = Some(vec!["PREQ".into()]);
        // Tech level orders the two icons (S-0004 claim 11 rule 4).
        jeep.tech_level = Some(if a_first { 2 } else { 1 });
        r.sim.vehicles.get_mut("TANK").unwrap().tech_level = Some(if a_first { 1 } else { 2 });
        let setup = |r: &Ruleset| {
            let mut w = game(r);
            building(&mut w, "WARFACT", 0, 0, 0);
            let p = building(&mut w, "PREQ", 0, 6, 6);
            ready(&mut w);
            set_credits(&mut w, 0, 1_000_000);
            (w, p)
        };
        let (mut w, preq) = setup(&r);
        let (mut control, _) = setup(&r);
        let (a, b) = (t(&w, Kind::Vehicle, "TANK"), t(&w, Kind::Vehicle, "JEEP"));
        assert!(issue(&mut w, 0, Order::Build { what: a }));
        assert!(issue(&mut w, 0, Order::Build { what: b }));
        for _ in 0..40 {
            step(&mut w);
            step(&mut control);
        }
        assert!(item(&w, 0, a).is_some());
        assert_eq!(line(&w, 0, a).unwrap().queue, vec![b]);
        let paid = credits(&w, 0);
        objects::remove(&mut w, preq);
        glasnost_sim::econ::tech::house_update(&mut w, H);
        let expected = if a_first {
            vec![
                Order::Cancel { what: a },
                Order::CancelAll { what: a },
                Order::CancelAll { what: b },
            ]
        } else {
            vec![
                Order::CancelAll { what: b },
                Order::Cancel { what: a },
                Order::CancelAll { what: a },
            ]
        };
        assert_eq!(w.houses[0].pending, expected);
        let before = w.rng.taken();
        let control_before = control.rng.taken();
        // The orders are applied at the end of the tick.
        step(&mut w);
        step(&mut control);
        draws.push((w.rng.taken() - before) as i64 - (control.rng.taken() - control_before) as i64);
        assert!(line(&w, 0, a).is_none_or(|l| l.item.is_none() && l.queue.is_empty()));
        // A is refunded what it paid; B paid nothing and is refunded 0.
        assert!(paid < 1_000_000);
        assert_eq!(credits(&w, 0), 1_000_000);
    }
    // One creation draw more when B starts in between.
    assert_eq!(draws[0] - draws[1], 1);
}

/// The production ruleset with `n` more infantry types, copies of GRUNT
/// named F0, F1, …, registered after it; F0 needs PREQ.
fn full_tab_rules(n: usize) -> Ruleset {
    let mut r = preq_rules();
    for i in 0..n {
        let name = format!("F{i}");
        let mut u = r.sim.infantry["GRUNT"].clone();
        if i == 0 {
            u.econ.prerequisite = Some(vec!["PREQ".into()]);
        }
        r.registries.get_mut("infantry").unwrap().push(name.clone());
        r.sim.infantry.insert(name, u);
    }
    r
}

/// The tab of GRUNT's icon and its icons.
fn infantry_tab(w: &World) -> Vec<TypeRef> {
    let grunt = t(w, Kind::Infantry, "GRUNT");
    w.houses[0]
        .econ
        .tech
        .icons
        .iter()
        .find(|tab| tab.iter().any(|i| i.t == grunt))
        .map(|tab| tab.iter().map(|i| i.t).collect())
        .unwrap_or_default()
}

// spec: S-0004 claim 11 and Constants (a tab holds at most 76 items) and S-0411 claim 15 (S10: a suppressed type is absent from the taken-up set, so a build order for it is illegal; admission in increasing registration index within one kind; a vacancy left by a removal is not given to a suppressed type at that recheck, only at the next recheck event)
#[test]
fn claim_15_full_tab() {
    let mut w = game(&full_tab_rules(80));
    let preq = building(&mut w, "PREQ", 0, 6, 0);
    building(&mut w, "BARRACKS", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 1_000_000);
    let tab = infantry_tab(&w);
    assert_eq!(tab.len(), 76);
    let f = |w: &World, i: usize| t(w, Kind::Infantry, &format!("F{i}"));
    // The last registered types are the suppressed ones.
    let suppressed: Vec<TypeRef> = (0..80)
        .map(|i| f(&w, i))
        .filter(|x| !tab.contains(x))
        .collect();
    assert!(!suppressed.is_empty());
    let first_out = suppressed[0];
    assert!(
        suppressed
            .iter()
            .all(|x| x.index > tab.iter().map(|y| y.index).max().unwrap())
    );
    assert!(!glasnost_sim::econ::tech::offered(&w, H, first_out));
    assert!(!issue(&mut w, 0, Order::Build { what: first_out }));
    let f1 = f(&w, 1);
    assert!(issue(&mut w, 0, Order::Build { what: f1 }));
    // PREQ leaves: F0's icon goes, and the vacancy stays empty at that recheck.
    objects::remove(&mut w, preq);
    step(&mut w);
    let tab = infantry_tab(&w);
    assert!(!tab.contains(&f(&w, 0)));
    assert_eq!(tab.len(), 75);
    assert!(!tab.contains(&first_out));
    // S-0411 claim 15 (a vacancy alone schedules no recheck)
    for _ in 0..5 {
        step(&mut w);
    }
    assert_eq!(infantry_tab(&w).len(), 75);
    // The next recheck event admits the first suppressed type.
    glasnost_sim::econ::tech::recheck_due(&mut w, H);
    step(&mut w);
    let tab = infantry_tab(&w);
    assert_eq!(tab.len(), 76);
    assert!(tab.contains(&first_out));
    assert!(glasnost_sim::econ::tech::offered(&w, H, first_out));
}

// spec: S-0411 claim 15 (capacity is judged before the removals of that recheck: an icon the same recheck removes still occupies a place for admission)
#[test]
fn claim_15_capacity_before_removals() {
    // A full tab: GRUNT, MEDIC and F0 to F73. PREQ's loss and the arrival of
    // H0 and H1 (both needing HUT) come in one recheck.
    let mut r = full_tab_rules(74);
    for name in ["H0", "H1"] {
        let mut u = r.sim.infantry["GRUNT"].clone();
        u.econ.prerequisite = Some(vec!["HUT".into()]);
        r.registries.get_mut("infantry").unwrap().push(name.into());
        r.sim.infantry.insert(name.into(), u);
    }
    let mut w = game(&r);
    let preq = building(&mut w, "PREQ", 0, 6, 0);
    building(&mut w, "BARRACKS", 0, 0, 0);
    ready(&mut w);
    let before = infantry_tab(&w);
    assert_eq!(before.len(), 76);
    let f0 = t(&w, Kind::Infantry, "F0");
    let h0 = t(&w, Kind::Infantry, "H0");
    assert!(before.contains(&f0) && !before.contains(&h0));
    // In one tick: PREQ leaves (F0 out) and a HUT comes (H0 and H1 eligible).
    objects::remove(&mut w, preq);
    building(&mut w, "HUT", 0, -6, 0);
    step(&mut w);
    let after = infantry_tab(&w);
    assert!(!after.contains(&f0));
    assert!(
        !after.contains(&h0),
        "F0's place still counted at admission"
    );
    assert_eq!(after.len(), 75);
}
