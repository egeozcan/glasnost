//! Claim tests for S-0402 (the harvest cycle) on synthetic data: invented
//! tiberiums, refineries, harvesters and values, no stock data. Houses
//! Alpha (0) and Beta (1) of the testkit's sandbox game, both human; the
//! neutral (2) and special (3) houses where a test needs them.

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_sim::econ::harvest::{self, Counter, DockState, HarvestState, Phase};
use glasnost_sim::econ::sell::Stage;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use harvest_common::*;

/// The refinery's top-left cell: REFN covers (−6 … −4, −1 … 1) from the
/// middle; its docking cell, (3, 1) from the top-left cell (S-0402 claim
/// 10), is (−3, 0).
fn refinery_cell() -> CellXY {
    at(-6, -1)
}

fn dock_cell() -> CellXY {
    at(-3, 0)
}

/// A 3 × 3 ORE field of stage `d` with its top-left cell at (x, y).
fn field(x: i32, y: i32, d: u8) -> Vec<(CellXY, u8, u8)> {
    let mut v = Vec::new();
    for dy in 0..3 {
        for dx in 0..3 {
            v.push((at(x + dx, y + dy), ORE, d));
        }
    }
    v
}

fn harvest_now(w: &mut World, h: ObjectId) {
    let t = w.tick;
    w.obj_mut(h).unwrap().beh.force(Behaviour::Harvest, t, true);
}

/// Runs tick by tick until `f` holds, returning the tick count after the
/// step in which it first held.
fn until(w: &mut World, max: i32, mut f: impl FnMut(&World) -> bool) -> Option<i32> {
    for _ in 0..max {
        testkit::run(w, 1);
        if f(w) {
            return Some(w.tick);
        }
    }
    None
}

/// Runs until orders are accepted (S-0213 claim 1: the order block at game start).
fn unblock(w: &mut World) {
    while w.tick <= w.orders_blocked_through {
        testkit::run(w, 1);
    }
}

fn current(w: &World, h: ObjectId) -> Option<Behaviour> {
    w.obj(h).unwrap().beh.current
}

// spec: S-0402 claim 1 and claim 15 (each bale adds to the amount of its cell's tiberium; the amounts are kept apart)
#[test]
fn claim_1_15_load_per_tiberium() {
    let r = ruleset();
    let (g, o) = (at(0, 0), at(1, 0));
    let mut w = game_with_ore(&r, &[(g, GEM, 2), (o, ORE, 5)], 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, g, 0);
    harvest_now(&mut w, h);
    until(&mut w, 400, |w| harvest::state_of(w, h).total() == CAPACITY).expect("full");
    // GEM stage 2: three bales, the last lost (S-0401 claim 3), so 2 GEM; then ORE.
    assert_eq!(harvest::load(&w, h), vec![3, 2]);
}

// spec: S-0402 claim 1 and claim 16 (capacity 0: the ratio is not a number, never full by the ratio; the cycle carries no bale)
#[test]
fn claim_1_16_capacity_zero() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(0, 0, 5), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "EMPTYH", 0, at(1, 1), 0);
    assert!(!harvest::full(&w, h));
    harvest_now(&mut w, h);
    until(&mut w, 400, |w| harvest::phase(w, h) == Phase::Returning).expect("returning");
    assert_eq!(harvest::state_of(&w, h).total(), 0);
    assert_eq!(stage(&w, at(1, 1)), Some(5));
}

// spec: S-0402 Terms and claim 1 (quirks.zero_capacity_harvester_cycles off: imported capacity 0 is effective
// C = 1 bale and the ordinary positive-capacity cycle: one bale loaded, then full and returning)
#[test]
fn claim_1_quirk_zero_capacity_off_counts_one_bale() {
    let mut r = ruleset();
    r.globals.quirks.zero_capacity_harvester_cycles = false;
    let mut w = game_with_ore(&r, &field(0, 0, 5), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "EMPTYH", 0, at(1, 1), 0);
    assert_eq!(harvest::capacity(&w, h), 1);
    assert!(!harvest::full(&w, h));
    harvest_now(&mut w, h);
    until(&mut w, 400, |w| harvest::phase(w, h) == Phase::Returning).expect("returning");
    assert_eq!(harvest::state_of(&w, h).total(), 1);
    assert!(harvest::full(&w, h));
}

// spec: S-0402 claim 2 (a harvest order on a cell gives harvest with the cell as destination)
#[test]
fn claim_2_harvest_order() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(4, -1, 5), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    let c = at(5, 0);
    unblock(&mut w);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Harvest,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c)),
        }
    ));
    until(&mut w, 400, |w| harvest::state_of(w, h).total() > 0).expect("a bale");
    assert_eq!(current(&w, h), Some(Behaviour::Harvest));
    assert_eq!(w.obj(h).unwrap().pos.cell(), c);
}

// spec: S-0402 claim 2 (stopping a harvester in harvest gives it guard at once, S-0213 table T1)
#[test]
fn claim_2_stop_order() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(4, -1, 5), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    harvest_now(&mut w, h);
    testkit::run(&mut w, 3);
    unblock(&mut w);
    assert!(orders::issue(&mut w, HouseId(0), Order::Stop { actor: h }));
    until(&mut w, 30, |w| current(w, h) == Some(Behaviour::Guard)).expect("guard");
}

// spec: S-0402 claim 2 and claim 10 (an enter order on a refinery starts docking directly; it ends in unload)
#[test]
fn claim_2_10_enter_order() {
    let r = ruleset();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 2), 0);
    unblock(&mut w);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: Some(Target::Object(refn)),
            dest: Some(Target::Object(refn)),
        }
    ));
    until(&mut w, 600, |w| current(w, h) == Some(Behaviour::Unload)).expect("unload");
    let o = w.obj(h).unwrap();
    assert_eq!(o.contact, Some(refn));
    assert_eq!(o.pos.cell(), dock_cell());
    // spec: S-0402 claim 10 rows 3 and 4 (turned on the docking cell to exactly 16384 before unload is queued)
    assert_eq!(w.body_facing(h), 16384);
    assert!(harvest::state_of(&w, h).tethered);
}

// spec: S-0402 claim 10 (Tag notifications) and S-0524 claim 14 (ordinary refinery arrival and docking in enter add neither an entered-by raise on the refinery's tag nor a destroyed-by-anything raise on the harvester's own tag; docking does not consume the harvester; C16)
#[test]
fn claim_10_refinery_docking_raises_no_tag() {
    let r = ruleset();
    let mut w = game_with_map(&r, &[], 7, |m| m.triggers = testkit::entry_triggers());
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 2), 0);
    glasnost_sim::triggers::object_tag(&mut w, refn, Some(b"ENT"));
    glasnost_sim::triggers::object_tag(&mut w, h, Some(b"DIE"));
    unblock(&mut w);
    let f0 = w.triggers.fired;
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: Some(Target::Object(refn)),
            dest: Some(Target::Object(refn)),
        }
    ));
    until(&mut w, 600, |w| current(w, h) == Some(Behaviour::Unload)).expect("unload");
    testkit::run(&mut w, 200);
    assert_eq!(w.triggers.fired, f0, "no raise reached a firing");
    assert!(!w.triggers.variables[0] && !w.triggers.variables[1]);
    assert!(
        w.obj(h).is_some_and(|o| o.on_map),
        "docking does not consume the harvester"
    );
}

// spec: S-0402 claim 3 and claim 13 (no building of a dock type: guard queued, 1 tick)
#[test]
fn claim_3_13_no_dock_building_gives_guard() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(0, 0, 5), 7);
    let h = place_vehicle(&mut w, "HARV", 0, at(1, 1), 0);
    harvest_now(&mut w, h);
    until(&mut w, 30, |w| current(w, h) == Some(Behaviour::Guard)).expect("guard");
    assert_eq!(harvest::state_of(&w, h).total(), 0);
}

// spec: S-0402 claim 13 (the dock test is on every building the owner owns: #392 looks at the house's acquired list first and walks every object when none of it passes, so the list's contents never change the answer)
#[test]
fn claim_13_dock_test_does_not_depend_on_the_acquired_list() {
    let r = ruleset();
    // A dock building missing from the acquired list still counts.
    let mut w = game_with_ore(&r, &field(0, 0, 5), 7);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(1, 1), 0);
    assert!(w.houses[0].econ.acquired.contains(&refn));
    w.houses[0].econ.acquired.retain(|&b| b != refn);
    harvest_now(&mut w, h);
    assert_eq!(w.obj(h).unwrap().beh.queued, None);
    harvest::process_harvest(&mut w, h);
    assert_ne!(w.obj(h).unwrap().beh.queued, Some(Behaviour::Guard));
    // A listed building that fails the test does not count: no health left.
    let mut w = game_with_ore(&r, &field(0, 0, 5), 7);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(1, 1), 0);
    assert!(w.houses[0].econ.acquired.contains(&refn));
    w.obj_mut(refn).unwrap().health = 0;
    harvest_now(&mut w, h);
    harvest::process_harvest(&mut w, h);
    assert_eq!(w.obj(h).unwrap().beh.queued, Some(Behaviour::Guard));
}

// spec: S-0402 claim 3 and claim 13 (nothing found: idle, the no-field record set; 105 ticks later guard)
#[test]
fn claim_3_13_idle_and_the_no_field_record() {
    let r = ruleset();
    let mut w = game(&r);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(4, 4), 0);
    assert!(!w.houses[0].econ.harvest.no_field);
    harvest_now(&mut w, h);
    let idle = until(&mut w, 60, |w| harvest::phase(w, h) == Phase::Idle).expect("idle");
    assert!(w.houses[0].econ.harvest.no_field);
    // spec: S-0402 Constants (idle interval 105 ticks)
    let guard = until(&mut w, 200, |w| current(w, h) == Some(Behaviour::Guard)).expect("guard");
    assert!(guard - idle >= 105, "{idle} {guard}");
    // In guard a harvester of a human-controlled owner stays.
    testkit::run(&mut w, 60);
    assert_eq!(current(&w, h), Some(Behaviour::Guard));
}

// spec: S-0402 claim 3 (seeking with a remembered cell: drives there and forgets it)
#[test]
fn claim_3_remembered_cell() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(4, -1, 5), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    let c = at(6, 1);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load: vec![0, 0],
        remembered: Some(c),
        ..Default::default()
    });
    harvest_now(&mut w, h);
    testkit::run(&mut w, 2);
    assert_eq!(harvest::state_of(&w, h).remembered, None);
    assert_eq!(w.obj(h).unwrap().dest, Some(Target::Cell(c)));
}

// spec: S-0402 claim 4 (the own cell when its land is tiberium; else the first ring with an acceptable cell, greatest worth)
#[test]
fn claim_4_ring_search_rings_and_worth() {
    let r = ruleset();
    let mut w = game_with_ore(
        &r,
        &[(at(1, 0), ORE, 0), (at(-1, 1), ORE, 2), (at(2, 2), GEM, 11)],
        7,
    );
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    // Ring 1 wins over the richer ring-2 cell; in ring 1 the greater worth.
    assert_eq!(
        harvest::ring_search(&mut w, h, at(0, 0), 32),
        Some(at(-1, 1))
    );
    // R ≤ 1: nothing.
    assert_eq!(harvest::ring_search(&mut w, h, at(0, 0), 1), None);
    // Range 2 reaches ring 1 only; from (3, 3) ring 1 holds (2, 2).
    assert_eq!(harvest::ring_search(&mut w, h, at(3, 3), 2), Some(at(2, 2)));
    // The own cell, whatever it holds.
    assert_eq!(
        harvest::ring_search(&mut w, h, at(1, 0), 32),
        Some(at(1, 0))
    );
}

// spec: S-0402 claim 4 (ties go to the smallest tie key: top (a, 0), bottom (a, 1), left (b, 2), right (b, 3))
#[test]
fn claim_4_ring_search_tie_key() {
    let r = ruleset();
    // (−2, −2): top (−2, 0) and left (−2, 2) → (−2, 0); (2, −2): top (2, 0), right (−2, 3) → (−2, 3).
    let mut w = game_with_ore(&r, &[(at(-2, -2), ORE, 3), (at(2, -2), ORE, 3)], 7);
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    assert_eq!(
        harvest::ring_search(&mut w, h, at(0, 0), 32),
        Some(at(-2, -2))
    );
    // (0, 2): bottom (0, 1); (−2, 0): left (0, 2) → the bottom cell wins.
    let mut w = game_with_ore(&r, &[(at(0, 2), ORE, 3), (at(-2, 0), ORE, 3)], 7);
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    assert_eq!(
        harvest::ring_search(&mut w, h, at(0, 0), 32),
        Some(at(0, 2))
    );
}

// spec: S-0402 claim 4 (acceptable only when the harvester could enter it now: an occupied cell is skipped)
#[test]
fn claim_4_ring_search_entry_test() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &[(at(1, 0), ORE, 9), (at(0, 2), ORE, 1)], 7);
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    place_vehicle(&mut w, "BUGGY", 1, at(1, 0), 0);
    assert_eq!(
        harvest::ring_search(&mut w, h, at(0, 0), 32),
        Some(at(0, 2))
    );
}

// spec: S-0402 claim 5 and F1 (first bale 19 ticks after loading starts, then every 9L + 1 ticks; returning C(9L + 1) after the start + 19)
#[test]
fn claim_5_f1_load_timing() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(0, 0, 11), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(1, 1), 0);
    harvest_now(&mut w, h);
    let t0 = until(&mut w, 40, |w| harvest::phase(w, h) == Phase::Loading).expect("loading");
    let l = 2; // spec: S-0402 Import mapping (HarvesterLoadRate absent: 2)
    let mut last = t0;
    for j in 1..=CAPACITY {
        let t = until(&mut w, 100, |w| harvest::state_of(w, h).total() == j).expect("bale");
        let expect = if j == 1 { 19 } else { 9 * l + 1 };
        assert_eq!(t - last, expect, "bale {j}");
        last = t;
    }
    let ret = until(&mut w, 100, |w| harvest::phase(w, h) == Phase::Returning).expect("returning");
    assert_eq!(ret - t0, 19 + CAPACITY * (9 * l + 1));
    // spec: S-0402 F1 (r18: that processing changes it to returning; refinery selection occurs no earlier than its following processing)
    let o = w.obj(h).unwrap();
    assert_eq!((o.dest, o.contact), (None, None));
    let chosen = until(&mut w, 100, |w| {
        let o = w.obj(h).unwrap();
        o.dest.is_some() || o.contact.is_some()
    })
    .expect("a refinery chosen");
    assert!(chosen > ret);
}

// spec: S-0402 claim 5 and F6 (rises: p > 0 at a + kp, k ≥ 1; p < 0 every tick; p = 0 none; counted from r to t − 1)
#[test]
fn f6_step_counter() {
    let c = Counter {
        reset: 10,
        base: 0,
        period: 2,
        start: 10,
    };
    // ⌊(t − 1 − t0) / 2⌋, first 9 in tick t0 + 19.
    assert_eq!(c.value(28, true), 8);
    assert_eq!(c.value(29, true), 9);
    let neg = Counter { period: -1, ..c };
    assert_eq!(neg.value(19, true), 9);
    let zero = Counter { period: 0, ..c };
    assert_eq!(zero.value(100, true), 0);
    // spec: S-0402 F6 (quirks.zero_load_period_stalls off: a period of 0 rises like a negative one)
    assert_eq!(zero.value(19, false), 9);
}

// spec: S-0402 claim 5 and Edge cases (quirks.zero_load_period_stalls: L = 0 loads one bale and stalls; off: bales 9 ticks apart)
#[test]
fn claim_5_quirk_zero_load_period_stalls() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.zero_load_period_stalls = on;
        r.sim.general.economy.harvester_load_rate = Some(0);
        let mut w = game_with_ore(&r, &field(0, 0, 11), 7);
        place_building(&mut w, "REFN", 0, refinery_cell());
        let h = place_vehicle(&mut w, "HARV", 0, at(1, 1), 0);
        harvest_now(&mut w, h);
        let first = until(&mut w, 100, |w| harvest::state_of(w, h).total() == 1).expect("bale");
        let second = until(&mut w, 200, |w| harvest::state_of(w, h).total() == 2);
        if on {
            assert_eq!(second, None);
        } else {
            assert_eq!(second.map(|t| t - first), Some(9));
        }
    }
}

// spec: S-0402 claim 6 (full: ring search with the short scan range, the result remembered; returning)
#[test]
fn claim_6_full_remembers_a_cell() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(0, 0, 11), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(1, 1), 0);
    harvest_now(&mut w, h);
    until(&mut w, 300, |w| harvest::phase(w, h) == Phase::Returning).expect("returning");
    // The own cell is still tiberium land: the search gives it.
    assert_eq!(harvest::state_of(&w, h).remembered, Some(at(1, 1)));
}

// spec: S-0402 claim 7 (exhausted, not full: short-scan search; nothing found and no destination: returning with what it carries)
#[test]
fn claim_7_exhausted_cell() {
    let r = ruleset();
    // A lone cell of stage 1: two bales, the last lost, so one is delivered.
    let mut w = game_with_ore(&r, &[(at(0, 0), ORE, 1)], 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    harvest_now(&mut w, h);
    until(&mut w, 300, |w| harvest::phase(w, h) == Phase::Returning).expect("returning");
    assert_eq!(harvest::load(&w, h), vec![1, 0]);
    assert_eq!(harvest::state_of(&w, h).remembered, None);
    // With a neighbour cell it drives on and keeps loading.
    let mut w = game_with_ore(&r, &[(at(0, 0), ORE, 1), (at(2, 0), ORE, 5)], 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    harvest_now(&mut w, h);
    until(&mut w, 300, |w| w.obj(h).unwrap().pos.cell() == at(2, 0)).expect("drives on");
    assert_eq!(harvest::phase(&w, h), Phase::Loading);
}

// spec: S-0402 claim 8 (the nearest candidate; a primary building wins the level from its place on)
#[test]
fn claim_8_refinery_choice() {
    let r = ruleset();
    let mut w = game(&r);
    let near = place_building(&mut w, "REFN", 0, at(-6, -1));
    let far = place_building(&mut w, "REFN", 0, at(5, -1));
    let h = place_vehicle(&mut w, "HARV", 0, at(-1, 0), 0);
    assert_eq!(harvest::choose_refinery(&mut w, h, false), Some(near));
    w.obj_mut(far).unwrap().econ.building.primary = true;
    assert_eq!(harvest::choose_refinery(&mut w, h, false), Some(far));
    // A primary building earlier in holding order than a nearer one: the nearer one wins.
    w.obj_mut(far).unwrap().econ.building.primary = false;
    w.obj_mut(near).unwrap().econ.building.primary = true;
    let h2 = place_vehicle(&mut w, "HARV", 0, at(3, 0), 0);
    assert_eq!(harvest::choose_refinery(&mut w, h2, false), Some(far));
}

// spec: S-0402 claim 8 (two levels: a type's best counts as primary only if it is one; across types by the dock list)
#[test]
fn claim_8_two_levels() {
    let r = ruleset();
    let mut w = game(&r);
    // REFN (first dock type): a primary far one, then a near one (its best is the near one, not primary).
    let p = place_building(&mut w, "REFN", 0, at(8, -1));
    let near = place_building(&mut w, "REFN", 0, at(-6, -1));
    // REF2 (second dock type): one not primary, between the two.
    let mid = place_building(&mut w, "REF2", 0, at(4, 3));
    w.obj_mut(p).unwrap().econ.building.primary = true;
    let h = place_vehicle(&mut w, "HARV", 0, at(-1, 0), 0);
    // Type REFN: P = p, nearest among p and after it is `near`; across types: no primary best, nearest wins.
    assert_eq!(harvest::choose_refinery(&mut w, h, false), Some(near));
    let _ = mid;
}

// spec: S-0402 claim 8 and F5 (quirks.refinery_distance_wraps: dx² + dy² wraps as a signed 32-bit integer; off: exact)
#[test]
fn claim_8_quirk_refinery_distance_wraps() {
    let d = 46341;
    assert_eq!(
        harvest::squared_distance(d, 0, true),
        i64::from((d * d) as i32)
    );
    assert!(harvest::squared_distance(d, 0, true) < 0);
    assert_eq!(harvest::squared_distance(d, 0, false), d * d);
    assert_eq!(harvest::squared_distance(300, 400, true), 250000);
}

// spec: S-0402 claim 8, F5 and Edge cases (r18: with quirks.refinery_distance_wraps off only the squared distance changes; both choice levels, the primary-building overrides and the tie rules are the same)
#[test]
fn claim_8_quirk_off_keeps_primary_and_levels() {
    let mut r = ruleset();
    r.globals.quirks.refinery_distance_wraps = false;
    let mut w = game(&r);
    let near = place_building(&mut w, "REFN", 0, at(-6, -1));
    let far = place_building(&mut w, "REFN", 0, at(5, -1));
    let h = place_vehicle(&mut w, "HARV", 0, at(-1, 0), 0);
    assert_eq!(harvest::choose_refinery(&mut w, h, false), Some(near));
    // A primary building still defeats a nearer one.
    w.obj_mut(far).unwrap().econ.building.primary = true;
    assert_eq!(harvest::choose_refinery(&mut w, h, false), Some(far));
    // A primary building earlier in holding order than a nearer one: the nearer one wins.
    w.obj_mut(far).unwrap().econ.building.primary = false;
    w.obj_mut(near).unwrap().econ.building.primary = true;
    let h2 = place_vehicle(&mut w, "HARV", 0, at(3, 0), 0);
    assert_eq!(harvest::choose_refinery(&mut w, h2, false), Some(far));
    // Two levels: the type's best is not primary, so across types the nearest wins.
    let mut w = game(&r);
    let p = place_building(&mut w, "REFN", 0, at(8, -1));
    let near = place_building(&mut w, "REFN", 0, at(-6, -1));
    place_building(&mut w, "REF2", 0, at(4, 3));
    w.obj_mut(p).unwrap().econ.building.primary = true;
    let h = place_vehicle(&mut w, "HARV", 0, at(-1, 0), 0);
    assert_eq!(harvest::choose_refinery(&mut w, h, false), Some(near));
}

// spec: S-0402 claim 9 (within the too-far distance: contact and requesting; else the queueing cell)
#[test]
fn claim_9_reserve_or_queue() {
    let r = ruleset();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REF2", 0, refinery_cell());
    // Within 5 cells of the reference point.
    let h = place_vehicle(&mut w, "HARV", 0, at(-2, 2), 0);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        phase: Phase::Returning,
        ..Default::default()
    });
    harvest_now(&mut w, h);
    until(&mut w, 30, |w| harvest::phase(w, h) == Phase::Requesting).expect("requesting");
    assert_eq!(w.obj(h).unwrap().contact, Some(refn));
    // Far away, the slot held: it drives to the queueing cell (top-left + (−2, 1)).
    let h2 = place_vehicle(&mut w, "HARV", 0, at(8, 0), 0);
    w.obj_mut(h2).unwrap().econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        phase: Phase::Returning,
        ..Default::default()
    });
    harvest_now(&mut w, h2);
    let q = CellXY::new(refinery_cell().x - 2, refinery_cell().y + 1);
    until(&mut w, 30, |w| w.obj(h2).unwrap().dest.is_some()).expect("a destination");
    assert_eq!(w.obj(h2).unwrap().dest, Some(Target::Cell(q)));
    assert_eq!(w.obj(h2).unwrap().contact, None);
}

// spec: S-0402 claim 9 (an ordinary harvester at most 768 leptons from the waiting choice stays where it is)
#[test]
fn claim_9_wait_near_a_busy_refinery() {
    let r = ruleset();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let holder = place_vehicle(&mut w, "HARV", 0, dock_cell(), 16384);
    w.obj_mut(holder).unwrap().contact = Some(refn);
    w.obj_mut(refn).unwrap().econ.dock = Some(DockState {
        holders: vec![holder],
    });
    // Two cells south of the reference point (the foundation's middle): 512 leptons.
    let h = place_vehicle(&mut w, "HARV", 0, at(-5, 2), 0);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        phase: Phase::Returning,
        ..Default::default()
    });
    harvest_now(&mut w, h);
    testkit::run(&mut w, 40);
    assert_eq!(harvest::phase(&w, h), Phase::Returning);
    assert_eq!(w.obj(h).unwrap().dest, None);
    assert_eq!(w.obj(h).unwrap().pos.cell(), at(-5, 2));
}

// spec: S-0402 claim 10 (a refinery refuses when its slots are held by others, when switched off, when being sold)
#[test]
fn claim_10_acceptance() {
    let r = ruleset();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let a = place_vehicle(&mut w, "HARV", 0, dock_cell(), 16384);
    let b = place_vehicle(&mut w, "HARV", 0, at(0, 3), 0);
    assert_eq!(harvest::choose_refinery(&mut w, b, false), Some(refn));
    w.obj_mut(a).unwrap().contact = Some(refn);
    w.obj_mut(refn).unwrap().econ.dock = Some(DockState { holders: vec![a] });
    // One slot, held by a: refused in the normal choice, not in the waiting choice.
    assert_eq!(harvest::choose_refinery(&mut w, b, false), None);
    assert_eq!(harvest::choose_refinery(&mut w, b, true), Some(refn));
    // The holder itself is still accepted.
    assert_eq!(harvest::choose_refinery(&mut w, a, false), Some(refn));
    w.obj_mut(a).unwrap().contact = None;
    w.obj_mut(refn).unwrap().econ.building.switched_off = true;
    assert_eq!(harvest::choose_refinery(&mut w, b, true), None);
    w.obj_mut(refn).unwrap().econ.building.switched_off = false;
    let t = w.tick;
    w.obj_mut(refn)
        .unwrap()
        .beh
        .force(Behaviour::Selling, t, true);
    assert_eq!(harvest::choose_refinery(&mut w, b, true), None);
}

/// A harvester docked at REFN (contact, slot held) with `load`, facing
/// `facing`, in unload.
fn docked(w: &mut World, load: Vec<i32>, facing: u16) -> (ObjectId, ObjectId) {
    let refn = place_building(w, "REFN", 0, refinery_cell());
    let h = place_vehicle(w, "HARV", 0, dock_cell(), facing);
    let o = w.obj_mut(h).unwrap();
    o.contact = Some(refn);
    o.econ.harvest = Some(HarvestState {
        load,
        ..Default::default()
    });
    w.obj_mut(refn).unwrap().econ.dock = Some(DockState { holders: vec![h] });
    let t = w.tick;
    w.obj_mut(h).unwrap().beh.force(Behaviour::Unload, t, true);
    (refn, h)
}

// spec: S-0402 claim 11, claim 15 and F2 (turn to east, start, one payment per tiberium ⌈900 × D⌉ apart, then finishing and harvest)
#[test]
fn claim_11_15_f2_unloading() {
    let r = ruleset();
    let mut w = game(&r);
    let (_, h) = docked(&mut w, vec![4, 3], 49152);
    let c0 = w.houses[0].credits;
    let u0 = until(&mut w, 100, |w| harvest::state_of(w, h).unloading).expect("unloading");
    assert!((16256..=16511).contains(&w.body_facing(h)));
    let p1 = until(&mut w, 100, |w| w.houses[0].credits != c0).expect("first payment");
    // spec: S-0402 F2 (first payment no earlier than u0 + 1 + ⌈14.4⌉)
    assert!(p1 - u0 >= 16, "{u0} {p1}");
    // spec: S-0402 claim 11 (the lowest registration index first: ORE)
    assert_eq!(w.houses[0].credits - c0, 4 * ORE_VALUE);
    assert_eq!(harvest::load(&w, h), vec![0, 3]);
    let p2 = until(&mut w, 100, |w| harvest::load(w, h) == vec![0, 0]).expect("second payment");
    // spec: S-0402 F2 (each later payment ⌈900 × D⌉ = 15 ticks after the previous one)
    assert_eq!(p2 - p1, 15);
    assert_eq!(w.houses[0].credits - c0, 4 * ORE_VALUE + 3 * GEM_VALUE);
    let fin = until(&mut w, 100, |w| harvest::state_of(w, h).finishing).expect("finishing");
    assert_eq!(fin - p2, 15);
    let back = until(&mut w, 10, |w| current(w, h) == Some(Behaviour::Harvest)).expect("harvest");
    assert_eq!(back - fin, 1);
    assert_eq!(w.obj(h).unwrap().contact, None);
}

// spec: S-0402 claim 11 and F2 (r18: with a dump rate of 0 or below the first payment comes at the first processing after the start row, then each payment and the change to finishing max(1, ⌈900 × D⌉) = 1 tick apart; never two payments in one processing)
#[test]
fn claim_11_f2_zero_and_negative_dump_rate() {
    for d in ["0", "-0.02"] {
        let mut r = ruleset();
        r.sim.general.economy.harvester_dump_rate = dec(d);
        let mut w = game(&r);
        let (_, h) = docked(&mut w, vec![4, 3], 49152);
        let c0 = w.houses[0].credits;
        let u0 = until(&mut w, 100, |w| harvest::state_of(w, h).unloading).expect("unloading");
        let (_, start_delay) = w.obj(h).unwrap().beh.last.expect("the start row");
        let p1 = until(&mut w, 100, |w| w.houses[0].credits != c0).expect("first payment");
        assert_eq!(
            p1 - u0,
            start_delay,
            "{d}: the first processing after the start row"
        );
        assert_eq!(
            w.houses[0].credits - c0,
            4 * ORE_VALUE,
            "{d}: one tiberium per payment"
        );
        let p2 = until(&mut w, 10, |w| harvest::load(w, h) == vec![0, 0]).expect("second payment");
        assert_eq!(p2 - p1, 1, "{d}");
        let fin = until(&mut w, 10, |w| harvest::state_of(w, h).finishing).expect("finishing");
        assert_eq!(fin - p2, 1, "{d}");
    }
}

// spec: S-0402 claim 11 (unloading with no building in the neighbouring cell: breaks off contact, harvest at once)
#[test]
fn claim_11_no_building_west() {
    let r = ruleset();
    let mut w = game(&r);
    let (refn, h) = docked(&mut w, vec![4, 0], 16384);
    until(&mut w, 100, |w| harvest::state_of(w, h).unloading).expect("unloading");
    // The refinery leaves: no building west of the harvester.
    glasnost_sim::objects::remove(&mut w, refn);
    until(&mut w, 30, |w| current(w, h) == Some(Behaviour::Harvest)).expect("harvest");
    assert_eq!(w.obj(h).unwrap().contact, None);
    assert_eq!(harvest::load(&w, h), vec![4, 0]);
}

// spec: S-0402 claim 11 row 1 and Edge cases (quirks.unload_without_contact_stalls: contact lost while unloading, nothing queued: it stays in unload with its load; off: it falls back as if unload were not current and starts at once)
#[test]
fn claim_11_quirk_unload_without_contact_stalls() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.unload_without_contact_stalls = on;
        let mut w = game(&r);
        let (_, h) = docked(&mut w, vec![4, 0], 16384);
        until(&mut w, 100, |w| harvest::state_of(w, h).unloading).expect("unloading");
        w.obj_mut(h).unwrap().contact = None;
        testkit::run(&mut w, 30);
        assert!(!harvest::state_of(&w, h).unloading);
        assert_eq!(harvest::load(&w, h), vec![4, 0]);
        if on {
            assert_eq!(current(&w, h), Some(Behaviour::Unload));
            assert_eq!(w.obj(h).unwrap().beh.queued, None);
        } else {
            assert_ne!(current(&w, h), Some(Behaviour::Unload));
        }
    }
}

// spec: S-0402 claim 11 row 1 and Edge cases (Unload without contact stalls: on, the harvester stays in unload for good; off, it falls back as if unload were not current and starts at once); M8 step C7 part 3: the play-fixes layer sets the switch off, read from the layer file, so a harvester whose contact is lost while it unloads goes on working
#[test]
fn play_fixes_layer_frees_a_harvester_unloading_without_contact() {
    for layered in [false, true] {
        let mut r = ruleset();
        if layered {
            r.globals.quirks = glasnost_rules::layers::Layers::PLAY_FIXES
                .apply_quirks(r.globals.quirks)
                .unwrap();
        }
        let mut w = game(&r);
        let (_, h) = docked(&mut w, vec![4, 0], 16384);
        until(&mut w, 100, |w| harvest::state_of(w, h).unloading).expect("unloading");
        w.obj_mut(h).unwrap().contact = None;
        testkit::run(&mut w, 30);
        assert_eq!(
            current(&w, h) == Some(Behaviour::Unload),
            !layered,
            "stalled in unload only without the layer"
        );
    }
}

// spec: S-0402 claim 11 (an order while unloading breaks off contact and frees the slot in the same tick; the order starts at the next processing of unload; the unpaid load is kept)
#[test]
fn claim_11_order_while_unloading() {
    let r = ruleset();
    let mut w = game(&r);
    unblock(&mut w);
    let (refn, h) = docked(&mut w, vec![4, 3], 16384);
    until(&mut w, 100, |w| harvest::load(w, h) == vec![0, 3]).expect("first payment");
    let other = place_vehicle(&mut w, "HARV", 0, at(0, 4), 0);
    assert_eq!(harvest::choose_refinery(&mut w, other, false), None);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(4, 4))),
        }
    ));
    // The order is carried out in the next tick: contact and slot go at once.
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(h).unwrap().contact, None);
    assert!(harvest::docked(&w, refn).is_empty());
    assert_eq!(harvest::choose_refinery(&mut w, other, false), Some(refn));
    until(&mut w, 5, |w| current(w, h) == Some(Behaviour::Move)).expect("move");
    assert!(!harvest::state_of(&w, h).unloading);
    assert_eq!(harvest::load(&w, h), vec![0, 3]);
}

/// The ruleset with REFN given a build-up, so that it can be sold.
fn sellable_ruleset() -> glasnost_rules::schema::Ruleset {
    let mut r = ruleset();
    r.sim.buildings.get_mut("REFN").unwrap().econ.buildup = Some(true);
    r
}

// spec: S-0402 claim 11 (refinery sold while unloading: leaves the unloading state, scatters, harvest started, the unpaid load kept, contact broken off)
#[test]
fn claim_11_refinery_sold_while_unloading() {
    let r = sellable_ruleset();
    let mut w = game(&r);
    unblock(&mut w);
    let (refn, h) = docked(&mut w, vec![4, 3], 16384);
    until(&mut w, 100, |w| harvest::load(w, h) == vec![0, 3]).expect("first payment");
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Sell { actor: refn }
    ));
    until(&mut w, 10, |w| w.obj(h).unwrap().contact.is_none()).expect("contact broken off");
    assert_eq!(current(&w, h), Some(Behaviour::Harvest));
    assert!(!harvest::state_of(&w, h).unloading);
    assert_eq!(harvest::load(&w, h), vec![0, 3]);
}

// spec: S-0402 claim 11 (refinery sold while the harvester is still in enter: guard queued, it scatters, contact broken off)
#[test]
fn claim_11_refinery_sold_while_entering() {
    let r = sellable_ruleset();
    let mut w = game(&r);
    unblock(&mut w);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 4), 0);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        ..Default::default()
    });
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: Some(Target::Object(refn)),
            dest: Some(Target::Object(refn)),
        }
    ));
    until(&mut w, 30, |w| w.obj(h).unwrap().contact == Some(refn)).expect("contact");
    assert_eq!(current(&w, h), Some(Behaviour::Enter));
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Sell { actor: refn }
    ));
    until(&mut w, 10, |w| w.obj(h).unwrap().contact.is_none()).expect("contact broken off");
    let o = w.obj(h).unwrap();
    assert!(
        o.beh.queued == Some(Behaviour::Guard) || o.beh.current == Some(Behaviour::Guard),
        "{:?}",
        o.beh
    );
    assert_eq!(harvest::load(&w, h), vec![5, 0]);
}

fn sale_stage(w: &World, b: ObjectId) -> Option<Stage> {
    w.obj(b).and_then(|o| o.econ.sale.as_ref()).map(|s| s.stage)
}

/// Sells `refn` and runs to the tick of the sale's first processing.
fn sell_to_first_processing(w: &mut World, refn: ObjectId) -> i32 {
    assert!(orders::issue(w, HouseId(0), Order::Sell { actor: refn }));
    until(w, 10, |w| sale_stage(w, refn) == Some(Stage::Begun)).expect("first processing")
}

// spec: S-0402 claim 11 (H12 part a2: harvest queued and started first, then the scatter request while it has no destination; the contact holds through the sale's first processing and ends at its second, 1 tick later, tethered or not) and Changes for other specs (S-0408 claim 6: the tethered harvester holds back the build-down, not the end of its contact)
#[test]
fn claim_11_sale_breaks_contact_at_its_second_processing() {
    let r = sellable_ruleset();
    let mut w = game(&r);
    unblock(&mut w);
    let (refn, h) = docked(&mut w, vec![4, 3], 16384);
    w.obj_mut(h)
        .unwrap()
        .econ
        .harvest
        .as_mut()
        .unwrap()
        .tethered = true;
    until(&mut w, 100, |w| harvest::load(w, h) == vec![0, 3]).expect("first payment");
    let first = sell_to_first_processing(&mut w, refn);
    let o = w.obj(h).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Harvest));
    assert!(!harvest::state_of(&w, h).unloading);
    assert_eq!(
        o.contact,
        Some(refn),
        "contact kept at the first processing"
    );
    assert!(
        matches!(o.dest, Some(Target::Cell(_))),
        "the scatter destination comes while the contact holds: {:?}",
        o.dest
    );
    assert_eq!(harvest::load(&w, h), vec![0, 3]);
    testkit::run(&mut w, 1);
    assert_eq!(w.tick, first + 1);
    assert_eq!(w.obj(h).unwrap().contact, None, "broken at the second");
    assert!(harvest::docked(&w, refn).is_empty());
    assert_eq!(
        sale_stage(&w, refn),
        Some(Stage::Begun),
        "a tethered harvester holds back the build-down"
    );
    let down = until(&mut w, 10, |w| {
        matches!(sale_stage(w, refn), Some(Stage::BuildDown(_)))
    })
    .expect("build-down");
    assert!(down > first + 1);
}

// spec: S-0402 claim 11 and Changes for other specs (a contact without a tether does not hold up the release: with no unit docked, the build-down begins at the sale's second processing, S-0408 claim 6)
#[test]
fn claim_11_untethered_contact_does_not_hold_the_sale() {
    let r = sellable_ruleset();
    let mut w = game(&r);
    unblock(&mut w);
    let (refn, h) = docked(&mut w, vec![4, 3], 16384);
    until(&mut w, 100, |w| harvest::load(w, h) == vec![0, 3]).expect("first payment");
    assert!(!harvest::state_of(&w, h).tethered);
    sell_to_first_processing(&mut w, refn);
    assert_eq!(w.obj(h).unwrap().contact, Some(refn));
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(h).unwrap().contact, None);
    assert!(matches!(sale_stage(&w, refn), Some(Stage::BuildDown(_))));
}

// spec: S-0402 claim 11 (an unloading harvester whose harvest could not start: the scatter request is refused, since unload is still current (S-0207 claim 16), and it gets no scatter destination)
#[test]
fn claim_11_sale_scatter_refused_when_harvest_cannot_start() {
    let r = sellable_ruleset();
    let mut w = game(&r);
    unblock(&mut w);
    let (refn, h) = docked(&mut w, vec![4, 3], 16384);
    until(&mut w, 100, |w| harvest::load(w, h) == vec![0, 3]).expect("first payment");
    // The sale's first processing, with a body turn still running.
    let t = w.tick;
    w.obj_mut(h).unwrap().body.turn_to(0, t, 1);
    harvest::refinery_sold(&mut w, refn);
    let o = w.obj(h).unwrap();
    assert!(o.body.turning(w.tick));
    assert_eq!(o.contact, Some(refn));
    assert_eq!(o.beh.current, Some(Behaviour::Unload));
    assert_eq!(o.beh.queued, Some(Behaviour::Harvest));
    assert!(!harvest::state_of(&w, h).unloading);
    assert_eq!(o.dest, None, "no scatter destination");
    assert_eq!(harvest::load(&w, h), vec![0, 3]);
}

// spec: S-0402 claim 11 (H12 part a3: a harvester in enter on the docking cell, still turning: guard queued, a destination naming the refinery removed, the scatter request refused by the turning body; it is in guard soon after and never docks)
#[test]
fn claim_11_sale_while_entering_on_the_docking_cell_still_turning() {
    let r = sellable_ruleset();
    let mut w = game(&r);
    unblock(&mut w);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, dock_cell(), 0);
    // In enter on the docking cell, tethered, its body turning towards the
    // dock facing (claim 10 row 3), with a destination naming the refinery.
    let t = w.tick;
    let o = w.obj_mut(h).unwrap();
    o.contact = Some(refn);
    o.dest = Some(Target::Object(refn));
    o.econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        tethered: true,
        ..Default::default()
    });
    o.beh.force(Behaviour::Enter, t, true);
    o.body.turn_to(16384, t, 1);
    w.obj_mut(refn).unwrap().econ.dock = Some(DockState { holders: vec![h] });
    sell_to_first_processing(&mut w, refn);
    let o = w.obj(h).unwrap();
    assert!(o.body.turning(w.tick));
    assert_eq!(o.beh.queued, Some(Behaviour::Guard));
    assert_eq!(
        o.dest, None,
        "the refinery destination removed, no scatter cell"
    );
    for _ in 0..150 {
        testkit::run(&mut w, 1);
        assert_ne!(current(&w, h), Some(Behaviour::Unload), "never docks");
    }
    assert_eq!(current(&w, h), Some(Behaviour::Guard));
    assert_eq!(w.obj(h).unwrap().contact, None);
    assert_eq!(harvest::load(&w, h), vec![5, 0]);
}

// spec: S-0402 claim 11 (H12 part a4: a harvester in enter, in contact and driving to the docking cell: guard queued, the docking-cell destination kept, no scatter request; it never docks)
#[test]
fn claim_11_sale_while_driving_to_the_docking_cell_keeps_it() {
    let r = sellable_ruleset();
    let mut w = game(&r);
    unblock(&mut w);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(4, 4), 0);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        ..Default::default()
    });
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: Some(Target::Object(refn)),
            dest: Some(Target::Object(refn)),
        }
    ));
    until(&mut w, 30, |w| {
        let o = w.obj(h).unwrap();
        o.contact == Some(refn) && o.dest == Some(Target::Cell(dock_cell()))
    })
    .expect("in contact, driving to the docking cell");
    assert_ne!(w.obj(h).unwrap().pos.cell(), dock_cell());
    sell_to_first_processing(&mut w, refn);
    let o = w.obj(h).unwrap();
    assert_eq!(o.beh.queued.or(o.beh.current), Some(Behaviour::Guard));
    assert_eq!(o.dest, Some(Target::Cell(dock_cell())), "kept, no request");
    for _ in 0..150 {
        testkit::run(&mut w, 1);
        assert_ne!(current(&w, h), Some(Behaviour::Unload), "never docks");
    }
    assert_eq!(w.obj(h).unwrap().contact, None);
    assert_eq!(harvest::load(&w, h), vec![5, 0]);
}

/// A harvester resting on the docking cell of a refinery, and a second one
/// at `from`, loaded and ordered to enter the refinery (S-0402 claim 10a).
fn occupied_dock(w: &mut World, from: CellXY) -> (ObjectId, ObjectId, ObjectId) {
    unblock(w);
    let refn = place_building(w, "REFN", 0, refinery_cell());
    let a = place_vehicle(w, "HARV", 0, dock_cell(), 0);
    let b = place_vehicle(w, "HARV", 0, from, 0);
    w.obj_mut(b).unwrap().econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        ..Default::default()
    });
    assert!(orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor: b,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: Some(Target::Object(refn)),
            dest: Some(Target::Object(refn)),
        }
    ));
    (refn, a, b)
}

// spec: S-0402 claim 10a (Q-1870: within the close-enough distance no substitute is adopted; the look at the first cell sends the resting occupant a forced, move-anywhere scatter request, the contacting harvester never gives up the move, keeps its contact and reservation and docks once the cell is clear; the occupant stays in guard)
#[test]
fn claim_10a_occupant_scattered_and_the_contacting_harvester_docks() {
    let r = ruleset();
    let mut w = game(&r);
    let (refn, a, b) = occupied_dock(&mut w, at(0, 0));
    until(&mut w, 600, |w| current(w, b) == Some(Behaviour::Unload))
        .expect("the contacting harvester docks");
    assert_ne!(
        w.obj(a).unwrap().pos.cell(),
        dock_cell(),
        "the occupant left"
    );
    assert_eq!(w.obj(b).unwrap().pos.cell(), dock_cell());
    assert_eq!(w.obj(b).unwrap().contact, Some(refn));
    assert_eq!(current(&w, a), Some(Behaviour::Guard));
}

/// `occupied_dock` on a base raised to cell level `level` everywhere.
fn raised_occupied_dock(flat: bool, level: i8) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut r = ruleset();
    r.globals.quirks.substitution_distance_flat_destination = flat;
    let mut w = game_with_map(&r, &[], 7, |m| {
        for dy in -20..=20 {
            for dx in -20..=20 {
                if let Some(c) = m.cells.get_mut(at(dx, dy)) {
                    c.level = level;
                }
            }
        }
    });
    let (refn, a, b) = occupied_dock(&mut w, at(0, 0));
    (w, refn, a, b)
}

// spec: S-0402 claim 10a (quirks.substitution_distance_flat_destination: on a base raised so high that the harvester's world height alone exceeds the close-enough distance, every offer is substituted, it never gets a path onto the docking cell, never sends the request and keeps its contact and reservation; with the switch off the true distance decides, so it scatters the occupant and docks)
#[test]
fn claim_10a_raised_base_substitutes_every_offer() {
    let (mut w, refn, a, b) = raised_occupied_dock(true, 8);
    let r = w.rules_arc();
    assert!(
        w.obj(b).unwrap().pos.z > r.general.close_enough,
        "raised high enough"
    );
    for _ in 0..600 {
        testkit::run(&mut w, 1);
        assert_ne!(current(&w, b), Some(Behaviour::Unload), "never docks");
    }
    assert_eq!(
        w.obj(a).unwrap().pos.cell(),
        dock_cell(),
        "the occupant stays"
    );
    assert_eq!(w.obj(b).unwrap().contact, Some(refn), "contact kept");
    assert!(harvest::docked(&w, refn).contains(&b), "reservation kept");
    let (mut w, _, a, b) = raised_occupied_dock(false, 8);
    until(&mut w, 600, |w| current(w, b) == Some(Behaviour::Unload))
        .expect("switch off: the true distance decides and it docks");
    assert_ne!(w.obj(a).unwrap().pos.cell(), dock_cell());
}

// spec: S-0402 claim 11 (finishing with a destination and a queued behaviour other than harvest: leaves the unloading state, starts it, keeps contact)
#[test]
fn claim_11_finishing_with_destination() {
    let r = ruleset();
    let mut w = game(&r);
    let (refn, h) = docked(&mut w, vec![0, 0], 16384);
    until(&mut w, 100, |w| harvest::state_of(w, h).finishing).expect("finishing");
    let o = w.obj_mut(h).unwrap();
    o.dest = Some(Target::Cell(at(4, 4)));
    o.beh.queue(Some(Behaviour::Move));
    until(&mut w, 5, |w| current(w, h) == Some(Behaviour::Move)).expect("move");
    assert!(!harvest::state_of(&w, h).unloading);
    assert_eq!(w.obj(h).unwrap().contact, Some(refn));
}

// spec: S-0402 claim 10 and Constants (the docking cell is (3, 1) from the top-left cell, independent of the foundation size)
#[test]
fn claim_10_docking_cell_of_another_foundation() {
    let mut r = ruleset();
    r.sim.buildings.get_mut("REFN").unwrap().foundation = Some([2, 2]);
    let mut w = game(&r);
    let tl = refinery_cell();
    let refn = place_building(&mut w, "REFN", 0, tl);
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 4), 0);
    unblock(&mut w);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: Some(Target::Object(refn)),
            dest: Some(Target::Object(refn)),
        }
    ));
    until(&mut w, 600, |w| current(w, h) == Some(Behaviour::Unload)).expect("unload");
    assert_eq!(
        w.obj(h).unwrap().pos.cell(),
        CellXY::new(tl.x + 3, tl.y + 1)
    );
}

// spec: S-0402 claim 10 and Edge cases (a refinery that is not a dock-unload refinery agrees to contact but never directs the harvester: it stays in enter, unpaid)
#[test]
fn claim_10_not_a_dock_unload_refinery() {
    let mut r = ruleset();
    r.sim.buildings.get_mut("REFN").unwrap().econ.dock_unload = None;
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(-2, 2), 0);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        phase: Phase::Returning,
        ..Default::default()
    });
    harvest_now(&mut w, h);
    let c0 = w.houses[0].credits;
    until(&mut w, 300, |w| current(w, h) == Some(Behaviour::Enter)).expect("enter");
    testkit::run(&mut w, 400);
    assert_eq!(current(&w, h), Some(Behaviour::Enter));
    assert_eq!(w.obj(h).unwrap().contact, Some(refn));
    assert_eq!(w.houses[0].credits, c0);
    assert_eq!(harvest::load(&w, h), vec![5, 0]);
}

// spec: S-0402 claims 8 and 10 and Edge cases (quirks.non_unloading_refinery_accepts off: a building without the
// dock-unload flag is excluded from the choice and refuses contact, an enter order included)
#[test]
fn claims_8_10_quirk_non_unloading_refinery_off() {
    let mut r = ruleset();
    r.globals.quirks.non_unloading_refinery_accepts = false;
    let mut plain = r.sim.buildings.get("REFN").unwrap().clone();
    plain.econ.dock_unload = None;
    r.sim.buildings.insert("REFN".into(), plain);
    // Without any dock-unload refinery: no contact through the choice.
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(-2, 2), 0);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        phase: Phase::Returning,
        ..Default::default()
    });
    harvest_now(&mut w, h);
    testkit::run(&mut w, 400);
    assert_ne!(w.obj(h).unwrap().contact, Some(refn));
    assert_ne!(current(&w, h), Some(Behaviour::Enter));
    // An explicit enter order is refused too: contact is never taken.
    let t = w.tick;
    w.obj_mut(h).unwrap().dest = Some(Target::Object(refn));
    w.obj_mut(h).unwrap().beh.force(Behaviour::Enter, t, true);
    testkit::run(&mut w, 60);
    assert_ne!(w.obj(h).unwrap().contact, Some(refn));
    assert_eq!(harvest::load(&w, h), vec![5, 0]);
}

// spec: S-0402 claim 10 row 2 (revision 26) and S-0207 claim 27: the docking cell is offered at each processing of enter; a changed offer, or an unchanged one under pending refresh, reaches the dock test, and for a qualified teleport-capable harvester that is not teleporting the fallback installs the order-level cell destination without its movement point and leaves refresh pending; an ordinary harvester takes the destination and its point and docks
// spec: S-0207 claim 27 (No suspended predecessor, including an ordinary driving locomotor: stop the active locomotion) and claim 28 (An occupied docking cell: a vehicle with no suspended predecessor whose current locomotor is not teleporting gets claim 27's fallback at every qualified docking assignment and does not jump)
#[test]
fn claim_10_row_2_offers_reach_the_dock_test() {
    let mut r = ruleset();
    // CHRONO here drives (its type has the teleport-capable flag, its own locomotor is driving: no suspended predecessor).
    r.sim.vehicles.get_mut("CHRONO").unwrap().econ.dock = Some(vec!["REFN".into()]);
    let returning = |w: &mut World, u: ObjectId| {
        w.obj_mut(u).unwrap().econ.harvest = Some(HarvestState {
            load: vec![5, 0],
            phase: Phase::Returning,
            ..Default::default()
        });
        harvest_now(w, u);
    };
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "CHRONO", 0, at(-2, 5), 0);
    returning(&mut w, h);
    until(&mut w, 300, |w| {
        w.obj(h).unwrap().econ.refresh && w.obj(h).unwrap().contact == Some(refn)
    })
    .expect("the fallback");
    let o = w.obj(h).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(dock_cell())));
    assert_eq!(o.drive.as_ref().unwrap().goal, None, "its point is skipped");
    let at_fallback = o.pos;
    // Every later offer is an effective assignment under refresh and tests again: refresh stays, the point stays
    // skipped, and the stopped miner does not creep towards the dock.
    for _ in 0..600 {
        testkit::run(&mut w, 1);
        let o = w.obj(h).unwrap();
        assert!(o.econ.refresh);
        assert_eq!(o.drive.as_ref().unwrap().goal, None);
        assert_eq!(o.pos, at_fallback, "it stays stopped");
        assert_ne!(current(&w, h), Some(Behaviour::Unload));
    }
    // The ordinary harvester docks.
    let mut w = game(&r);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let ordinary = place_vehicle(&mut w, "HARV", 0, at(2, 5), 0);
    returning(&mut w, ordinary);
    until(&mut w, 800, |w| {
        current(w, ordinary) == Some(Behaviour::Unload)
    })
    .expect("unload");
    assert!(!w.obj(ordinary).unwrap().econ.refresh);
}

// spec: S-0402 claim 10 row 1 (switched off: refused; not tethered, contact broken off and rest; tethered, contact kept)
#[test]
fn claim_10_row_1_refusal() {
    for tethered in [false, true] {
        let r = ruleset();
        let mut w = game(&r);
        let refn = place_building(&mut w, "REFN", 0, refinery_cell());
        let cell = if tethered { dock_cell() } else { at(0, 4) };
        let h = place_vehicle(&mut w, "HARV", 0, cell, 0);
        w.obj_mut(h).unwrap().contact = Some(refn);
        w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
            load: vec![5, 0],
            ..Default::default()
        });
        w.obj_mut(refn).unwrap().econ.dock = Some(DockState { holders: vec![h] });
        let t = w.tick;
        w.obj_mut(h).unwrap().beh.force(Behaviour::Enter, t, true);
        if tethered {
            until(&mut w, 60, |w| harvest::state_of(w, h).tethered).expect("tethered");
        }
        w.obj_mut(refn).unwrap().econ.building.switched_off = true;
        until(&mut w, 60, |w| current(w, h) != Some(Behaviour::Enter)).expect("enter ends");
        if tethered {
            // A tethered harvester keeps its contact and goes on to the other rows.
            assert_eq!(w.obj(h).unwrap().contact, Some(refn));
            assert_eq!(current(&w, h), Some(Behaviour::Unload));
        } else {
            assert_eq!(w.obj(h).unwrap().contact, None);
        }
    }
}

// spec: S-0402 claim 8 (connected: the harvester's cell and the refinery's reference-point cell in the same reach region; a refinery on a plateau it cannot reach is no candidate)
#[test]
fn claim_8_connectivity() {
    let r = ruleset();
    let west = at(-2, 0).x;
    let mut w = game_with_map(&r, &[], 7, |m| {
        for y in 0..=2 * SIZE {
            for x in 0..=2 * SIZE {
                if let Some(c) = m.cells.get_mut(CellXY::new(x, y))
                    && x < west
                {
                    c.level = 4;
                }
            }
        }
    });
    let near = place_building(&mut w, "REFN", 0, refinery_cell());
    let far = place_building(&mut w, "REFN", 0, at(8, -1));
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    assert_eq!(harvest::choose_refinery(&mut w, h, false), Some(far));
    // One on the plateau is chosen.
    let up = place_vehicle(&mut w, "HARV", 0, at(-8, 3), 0);
    assert_eq!(harvest::choose_refinery(&mut w, up, false), Some(near));
}

// spec: S-0402 claim 9 (the queueing search: with the default shift the queueing cell is a foundation cell, so the result is (x0, y0 − 1) on open ground)
#[test]
fn claim_9_queueing_search_default_shift() {
    let r = ruleset();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let holder = place_vehicle(&mut w, "HARV", 0, dock_cell(), 16384);
    w.obj_mut(holder).unwrap().contact = Some(refn);
    w.obj_mut(refn).unwrap().econ.dock = Some(DockState {
        holders: vec![holder],
    });
    let h = place_vehicle(&mut w, "HARV", 0, at(8, 0), 0);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        phase: Phase::Returning,
        ..Default::default()
    });
    harvest_now(&mut w, h);
    until(&mut w, 30, |w| w.obj(h).unwrap().dest.is_some()).expect("a destination");
    let tl = refinery_cell();
    assert_eq!(
        w.obj(h).unwrap().dest,
        Some(Target::Cell(CellXY::new(tl.x, tl.y - 1)))
    );
}

// spec: S-0402 claim 13 (the idle step-off: off the refinery-flag building's cell to the nearby-cell search's cell, then guard)
#[test]
fn claim_13_idle_step_off() {
    let r = ruleset();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    // A harvester standing on the refinery's own cell (as one left there would).
    let on = CellXY::new(refinery_cell().x + 1, refinery_cell().y + 1);
    let h = place_vehicle(&mut w, "HARV", 0, on, 0);
    let _ = refn;
    harvest_now(&mut w, h);
    let start = w.obj(h).unwrap().beh.start_tick;
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        phase: Phase::Idle,
        harvest_start: Some(start),
        ..Default::default()
    });
    // spec: S-0402 claim 13 (the destination: the search's cell around the reference point's cell, outside the building)
    harvest::process_harvest(&mut w, h);
    assert_eq!(w.obj(h).unwrap().beh.queued, Some(Behaviour::Guard));
    let Some(Target::Cell(d)) = w.obj(h).unwrap().dest else {
        panic!("a cell destination");
    };
    let tl = refinery_cell();
    assert!(!(tl.x..tl.x + 3).contains(&d.x) || !(tl.y..tl.y + 3).contains(&d.y));
    assert!((d.x - (tl.x + 1)).abs() <= 2 && (d.y - (tl.y + 1)).abs() <= 2);
}

// spec: S-0402 claim 12 (virtual purifiers: none for player houses; the neutral and special houses take the second entry, difficulty index 1; a short list counts 0)
#[test]
fn claim_12_virtual_purifiers() {
    let mut r = ruleset();
    r.sim.general.economy.ai_virtual_purifiers = Some(vec![3, 2, 1]);
    let w = testkit::game_with_non_players(&r, SIZE, 7);
    for (h, n) in [(0, 0), (1, 0), (2, 2), (3, 2)] {
        assert_eq!(harvest::purifier_count(&w, HouseId(h)), n);
    }
    r.sim.general.economy.ai_virtual_purifiers = Some(vec![3]);
    let w = testkit::game_with_non_players(&r, SIZE, 7);
    assert_eq!(harvest::purifier_count(&w, HouseId(2)), 0);
}

/// The ruleset with FACT (2 × 2, cost 1000, free unit HARV of cost 400).
fn free_unit_ruleset() -> glasnost_rules::schema::Ruleset {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("HARV").unwrap().cost = Some(400);
    let mut b = building(2, 2);
    b.cost = Some(1000);
    b.econ.free_unit = Some("HARV".into());
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("FACT".into());
    r.sim.buildings.insert("FACT".into(), b);
    r
}

fn vehicles(w: &World) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.kind == glasnost_sim::rules::Kind::Vehicle)
        .map(|o| o.id)
        .collect()
}

// spec: S-0402 claim 18 (the grand opening at the end of the build-up gives the free unit; a pre-placed building gets none)
#[test]
fn claim_18_free_unit_at_the_grand_opening() {
    let r = free_unit_ruleset();
    let mut w = game(&r);
    let f = place_building(&mut w, "FACT", 0, at(0, 0));
    testkit::run(&mut w, 5);
    assert!(vehicles(&w).is_empty(), "pre-placed: none");
    let t = w.tick;
    w.obj_mut(f)
        .unwrap()
        .beh
        .force(Behaviour::Construction, t, true);
    until(&mut w, 400, |w| !vehicles(w).is_empty()).expect("the free unit");
    assert_eq!(vehicles(&w).len(), 1);
}

// spec: S-0402 claim 18 and Constants (first place: south of the reference point's cell, facing 49152; blocked: the search's cell, facing 40960; harvest started at once)
#[test]
fn claim_18_free_unit_places() {
    let r = free_unit_ruleset();
    let mut w = game(&r);
    let f = place_building(&mut w, "FACT", 0, at(0, 0));
    let rc = w.reference_point(f).cell();
    let (dx, dy) = glasnost_sim::regions::STEPS[4];
    let first = CellXY::new(rc.x + dx, rc.y + dy);
    harvest::grand_opening(&mut w, f);
    let v = vehicles(&w);
    assert_eq!(v.len(), 1);
    assert_eq!(w.obj(v[0]).unwrap().pos.cell(), first);
    assert_eq!(w.body_facing(v[0]), 49152);
    assert_eq!(current(&w, v[0]), Some(Behaviour::Harvest));
    assert_eq!(w.obj(v[0]).unwrap().owner, Some(HouseId(0)));
    // The first place blocked: a searched place.
    let mut w = game(&r);
    let f = place_building(&mut w, "FACT", 0, at(0, 0));
    place_building(&mut w, "HUT", 1, first);
    harvest::grand_opening(&mut w, f);
    let v = vehicles(&w);
    assert_eq!(v.len(), 1);
    assert_ne!(w.obj(v[0]).unwrap().pos.cell(), first);
    assert_eq!(w.body_facing(v[0]), 40960);
    // spec: S-0402 claim 18 (the second place: the search with the no-overlay condition, which passes over the ore nearer the top-left cell)
    let mut ore = Vec::new();
    for dy in -2..=3 {
        for dx in -2..=3 {
            if !((0..2).contains(&dx) && (0..2).contains(&dy)) {
                ore.push((at(dx, dy), ORE, 5));
            }
        }
    }
    let mut w = game_with_ore(&r, &ore, 7);
    let f = place_building(&mut w, "FACT", 0, at(0, 0));
    place_building(&mut w, "HUT", 1, first);
    harvest::grand_opening(&mut w, f);
    let v = vehicles(&w);
    assert_eq!(v.len(), 1);
    let c = w.obj(v[0]).unwrap().pos.cell();
    assert_eq!(stage(&w, c), None, "{c:?}");
    assert_eq!(w.body_facing(v[0]), 40960);
}

// spec: S-0402 claim 18 (no place accepts the unit: it is removed and the owner gets the refund with the percentage taken as 1)
#[test]
fn claim_18_free_unit_refund() {
    let r = free_unit_ruleset();
    // A small map whose every other cell holds a hut of Beta: no place is clear.
    let mut w = testkit::game_with(&r, 6, 7);
    let f = testkit::spawn(
        &mut w,
        glasnost_sim::rules::Kind::Building,
        "FACT",
        0,
        testkit::mid(6, 0, 0),
        0,
    );
    let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for c in cells {
        if w.cells
            .list(c, glasnost_sim::world::Layer::Ground)
            .is_empty()
        {
            testkit::spawn(&mut w, glasnost_sim::rules::Kind::Building, "HUT", 1, c, 0);
        }
    }
    let c0 = w.houses[0].credits;
    let n = w.objects.len();
    harvest::grand_opening(&mut w, f);
    assert!(vehicles(&w).is_empty());
    assert_eq!(w.objects.len(), n);
    assert_eq!(w.houses[0].credits - c0, 400);
}

// spec: S-0402 claim 18 (a produced building of a player house whose start price is not 0 and not above C gets none; above C, or 0, or a neutral owner: it does)
#[test]
fn claim_18_free_unit_start_price() {
    let r = free_unit_ruleset();
    // C = 1000 − 400 = 600 (S-0408 F2).
    for (owner, price, expect) in [
        (0, Some(600), 0),
        (0, Some(1), 0),
        (0, Some(601), 1),
        (0, Some(0), 1),
        (0, None, 1),
        (2, Some(600), 1),
    ] {
        let mut w = testkit::game_with_non_players(&r, SIZE, 7);
        let f = place_building(&mut w, "FACT", owner, at(0, 0));
        w.obj_mut(f).unwrap().econ.start_price = price;
        harvest::grand_opening(&mut w, f);
        assert_eq!(vehicles(&w).len(), expect, "{owner} {price:?}");
    }
}

// spec: S-0402 claim 12 and F3 (B′ = trunc(RZ(RZ(RZ(V × M) × a) + B)); the bonus x = RZ₂₄(RZ(nₚ × b) × a))
#[test]
fn claim_12_f3_payment() {
    let r = ruleset();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    w.houses[0].credits = 0;
    harvest::pay(&mut w, refn, 0, 10);
    assert_eq!(w.houses[0].credits, 10 * ORE_VALUE);
    // One purifier, b absent (the single nearest 0.25): x = 2.5, bonus trunc(25 × 2.5) = 62.
    place_building(&mut w, "PURI", 0, at(5, 5));
    assert_eq!(harvest::purifier_count(&w, HouseId(0)), 1);
    w.houses[0].credits = 0;
    harvest::pay(&mut w, refn, 0, 10);
    assert_eq!(w.houses[0].credits, 250 + 62);
    // No storage caps the balance.
    w.houses[0].credits = 2_000_000_000;
    harvest::pay(&mut w, refn, 1, 1);
    assert!(w.houses[0].credits > 2_000_000_000);
}

// spec: S-0402 claim 12 (the income factor M of the owner's country, a single)
#[test]
fn claim_12_income_factor() {
    let mut r = ruleset();
    r.sim
        .countries
        .get_mut("Alphaland")
        .unwrap()
        .econ
        .income_mult = dec("1.5");
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    w.houses[0].credits = 0;
    harvest::pay(&mut w, refn, 0, 10);
    assert_eq!(w.houses[0].credits, 375);
}

// spec: S-0402 claim 13 (the computer-controlled guard rule for the neutral and special houses; the no-field record stops it; human houses stay)
#[test]
fn claim_13_guard_rule() {
    let r = ruleset();
    let mut w = testkit::game_with_non_players(&r, SIZE, 7);
    let h = place_vehicle(&mut w, "HARV", 2, at(0, 0), 0);
    // No refinery of its dock types: no rule.
    assert!(!harvest::guard_rule(&mut w, h));
    place_building(&mut w, "REFN", 2, refinery_cell());
    assert!(harvest::guard_rule(&mut w, h));
    assert_eq!(w.obj(h).unwrap().beh.queued, Some(Behaviour::Harvest));
    w.houses[2].econ.harvest.no_field = true;
    w.obj_mut(h).unwrap().beh.queue(None);
    assert!(!harvest::guard_rule(&mut w, h));
    // A human player's harvester: no rule.
    place_building(&mut w, "REFN", 0, at(5, 5));
    let p = place_vehicle(&mut w, "HARV", 0, at(0, 4), 0);
    assert!(!harvest::guard_rule(&mut w, p));
}

// spec: S-0402 claim 14 and F5 (the wake-up rule: k + s_m < t, at rest, and computer / on tiberium / k + s_g < t with a scan hit)
#[test]
fn claim_14_wake_rule() {
    let wake = |k, sm, t, rest, comp, ore, sg, wraps, scan: bool| {
        harvest::wake_rule(k, sm, t, rest, comp, ore, sg, wraps, || scan)
    };
    assert!(wake(10, 5, 16, true, false, true, 0, true, false));
    assert!(!wake(10, 5, 15, true, false, true, 0, true, false));
    assert!(!wake(10, 5, 16, false, true, true, 0, true, true));
    assert!(wake(10, 5, 16, true, false, false, 5, true, true));
    assert!(!wake(10, 5, 16, true, false, false, 6, true, true));
    assert!(!wake(10, 5, 16, true, false, false, 5, true, false));
}

// spec: S-0402 claim 14 and F5 (quirks.slave_kick_delay_wraps: 2147483647 + s wraps for s ≥ 1, passing; s = 0 fails; off: exact)
#[test]
fn claim_14_quirk_slave_kick_delay_wraps() {
    let k = i32::MAX;
    assert!(harvest::wake_rule(
        k,
        1,
        100,
        true,
        true,
        false,
        0,
        true,
        || false
    ));
    assert!(!harvest::wake_rule(
        k,
        0,
        100,
        true,
        true,
        false,
        0,
        true,
        || false
    ));
    assert!(!harvest::wake_rule(
        k,
        1,
        100,
        true,
        true,
        false,
        0,
        false,
        || false
    ));
}

// spec: S-0402 claim 14 (a slave miner: gatherer, destination and an enslaves field; its group, of no slaves here, is at rest, and with the wrapping kick delay a start tick of 0 fails the rule)
#[test]
fn claim_14_slave_miner() {
    let r = ruleset();
    let mut w = game(&r);
    let m = place_vehicle(&mut w, "SLAVEM", 0, at(0, 0), 0);
    let h = place_vehicle(&mut w, "HARV", 0, at(3, 0), 0);
    assert!(harvest::is_slave_miner(&w, m));
    assert!(!harvest::is_slave_miner(&w, h));
    assert!(glasnost_sim::seams::is_slave_master(&w, m));
    assert!(!glasnost_sim::seams::slave_wake_rule_holds(&mut w, m));
}

// spec: S-0402 claim 16 and F4 (full when total / C equals 1.0: exactly total = C)
#[test]
fn claim_16_f4_fullness() {
    let r = ruleset();
    let mut w = game(&r);
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    for (load, full) in [(vec![4, 0], false), (vec![2, 3], true), (vec![0, 5], true)] {
        w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
            load,
            ..Default::default()
        });
        assert_eq!(harvest::full(&w, h), full);
    }
}

// spec: S-0402 claim 17 (a chrono miner in guard next to its owner's refinery queues harvest again)
#[test]
fn claim_17_chrono_guard_rule() {
    let r = ruleset();
    let mut w = game(&r);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let c = place_vehicle(&mut w, "CHRONO", 0, dock_cell(), 0);
    let far = place_vehicle(&mut w, "CHRONO", 0, at(5, 5), 0);
    assert!(harvest::guard_rule(&mut w, c));
    assert!(!harvest::guard_rule(&mut w, far));
}

// spec: S-0402 claim 17 (the chrono miner's too-far field: it reserves from further away)
#[test]
fn claim_17_chrono_too_far() {
    let r = ruleset();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    for (name, reserves) in [("CHRONO", true), ("HARV", false)] {
        let h = place_vehicle(&mut w, name, 0, at(8, 4), 0);
        w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
            load: vec![5, 0],
            phase: Phase::Returning,
            ..Default::default()
        });
        harvest_now(&mut w, h);
        testkit::run(&mut w, 3);
        assert_eq!(w.obj(h).unwrap().contact == Some(refn), reserves, "{name}");
        // Free the slot for the next one.
        w.obj_mut(h).unwrap().contact = None;
        glasnost_sim::objects::remove(&mut w, h);
    }
}

// The full cycle from a free unit: S-0215 claim 15 row 4 gives a harvester placed by a building harvest, and it earns credits.
// spec: S-0402 claims 3 to 12 (the cycle end to end)
#[test]
fn cycle_end_to_end() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(4, -1, 5), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    w.obj_mut(h).unwrap().beh.queue(None);
    w.obj_mut(h).unwrap().beh.current = None;
    // spec: S-0215 claim 15 row 4 (the placement fall-back of a harvester queues harvest)
    glasnost_sim::process::fall_back(&mut w, h, true);
    assert_eq!(w.obj(h).unwrap().beh.queued, Some(Behaviour::Harvest));
    let c0 = w.houses[0].credits;
    until(&mut w, 2000, |w| {
        w.houses[0].credits >= c0 + CAPACITY * ORE_VALUE
    })
    .expect("paid");
    assert_eq!(w.houses[0].credits, c0 + CAPACITY * ORE_VALUE);
}

// The harvest state survives a save and load round trip.
#[test]
fn harvest_state_round_trip() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(4, -1, 5), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    harvest_now(&mut w, h);
    testkit::run(&mut w, 300);
    let bytes = glasnost_sim::save::save(&w);
    let mut back =
        glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
    testkit::run(&mut w, 300);
    testkit::run(&mut back, 300);
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
}

/// An idle harvester standing on REFN's cell (S-0402 claim 13), of type
/// `name`, after one idle processing; returns the world and the harvester.
fn idle_on_refinery(r: &glasnost_rules::schema::Ruleset, name: &str) -> (World, ObjectId) {
    let mut w = game(r);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let on = CellXY::new(refinery_cell().x + 1, refinery_cell().y + 1);
    let h = place_vehicle(&mut w, name, 0, on, 0);
    harvest_now(&mut w, h);
    let start = w.obj(h).unwrap().beh.start_tick;
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        phase: Phase::Idle,
        harvest_start: Some(start),
        ..Default::default()
    });
    w.obj_mut(h).unwrap().dest = Some(Target::Cell(at(5, 5)));
    harvest::process_harvest(&mut w, h);
    (w, h)
}

// spec: S-0402 claim 13 and Edge cases (Q-0194: a failed step-off search replaces the destination with the off-map cell (0, 0), quirks.harvester_failed_step_off_origin; off, it clears it; both queue guard without a start, keep facing and contact, and take only the standard delay's draw)
#[test]
fn claim_13_failed_step_off() {
    let mut r = ruleset();
    // A harvester that floats: no land cell around the refinery qualifies.
    let mut boat = harvester(CAPACITY);
    boat.locomotion_class = Some(glasnost_rules::sim::LocomotionClass::Float);
    boat.movement_zone = Some(glasnost_rules::sim::MovementZone::Water);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("BOATH".into());
    r.sim.vehicles.insert("BOATH".into(), boat);
    for row in r.sim.land_types.values_mut() {
        row.float = dec("0");
    }
    let (ok, hk) = idle_on_refinery(&r, "HARV");
    let Some(Target::Cell(d)) = ok.obj(hk).unwrap().dest else {
        panic!("an ordinary step-off destination");
    };
    assert_ne!(d, CellXY::new(0, 0));
    for on in [true, false] {
        r.globals.quirks.harvester_failed_step_off_origin = on;
        let (w, h) = idle_on_refinery(&r, "BOATH");
        let o = w.obj(h).unwrap();
        let want = on.then_some(Target::Cell(CellXY::new(0, 0)));
        assert_eq!(o.dest, want, "quirk {on}");
        assert_eq!(o.beh.queued, Some(Behaviour::Guard));
        assert_eq!(
            o.beh.current,
            Some(Behaviour::Harvest),
            "guard queued without a start"
        );
        assert_eq!(w.body_facing(h), 0);
        assert_eq!(o.contact, None);
        // The search takes no draw, failed or not: the same stream as the
        // ordinary step-off, which takes only the standard delay's draw.
        assert_eq!(w.rng, ok.rng, "only the standard delay's draw");
    }
}

/// A tethered harvester in *enter* holding contact with REFN, its next
/// processing far off; returns it.
fn tethered_enter(w: &mut World, refn: ObjectId, c: CellXY, facing: u16) -> ObjectId {
    let h = place_vehicle(w, "HARV", 0, c, facing);
    w.obj_mut(h).unwrap().contact = Some(refn);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        tethered: true,
        ..Default::default()
    });
    w.obj_mut(refn).unwrap().econ.dock = Some(DockState { holders: vec![h] });
    let t = w.tick;
    let o = w.obj_mut(h).unwrap();
    o.beh.force(Behaviour::Enter, t, true);
    o.beh.expired = false;
    o.beh.last = Some((t, 100_000));
    h
}

// spec: S-0402 claim 10 (second route, Q-0197: reaching a track target in a cell whose step-direction-0 neighbour is the contacted refinery queues unload without a start, with no facing or stopped test, and also when the refinery is switched off; crossing into a cell mid-track does not)
#[test]
fn claim_10_second_route() {
    let tl = refinery_cell();
    // South of the foundation's middle column: its map-north neighbour is a REFN cell.
    let arrive = CellXY::new(tl.x + 1, tl.y + 3);
    for off in [false, true] {
        let r = ruleset();
        let mut w = game(&r);
        let refn = place_building(&mut w, "REFN", 0, tl);
        w.obj_mut(refn).unwrap().econ.building.switched_off = off;
        let h = tethered_enter(&mut w, refn, CellXY::new(tl.x + 1, tl.y + 6), 0);
        w.obj_mut(h).unwrap().dest = Some(Target::Cell(arrive));
        let mut queued_at = None;
        for _ in 0..200 {
            testkit::run(&mut w, 1);
            let o = w.obj(h).unwrap();
            if o.beh.queued == Some(Behaviour::Unload) || o.beh.current == Some(Behaviour::Unload) {
                queued_at = Some(o.pos.cell());
                // spec: S-0402 claim 10 (queued, not started by the report; facing map north, not 16384)
                // spec: S-0215 claim 11 (revision 57: the stopped vehicle is ready at its late start, after the movement that queued unload, so unload is current at the end of that update, not yet processed; claim 3b)
                assert_eq!(o.beh.current, Some(Behaviour::Unload));
                assert_eq!(o.beh.queued, None);
                assert!(o.beh.expired, "started, not processed");
                assert_ne!(w.body_facing(h), 16384);
                break;
            }
        }
        assert_eq!(queued_at, Some(arrive), "switched off {off}");
        assert_eq!(w.obj(h).unwrap().contact, Some(refn));
    }
    // Not in contact with the building to the north: no report.
    let r = ruleset();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, tl);
    let h = tethered_enter(&mut w, refn, arrive, 0);
    w.obj_mut(h).unwrap().contact = None;
    harvest::track_arrival(&mut w, h);
    assert_eq!(w.obj(h).unwrap().beh.queued, None);
}

// spec: S-0402 claim 10 (second route refused by a refinery in selling: a forced vehicle scatter request without a threat point and not move-anywhere, S-0207 claim 16; no draw; contact, tether and behaviours kept; a destination is kept)
#[test]
fn claim_10_second_route_refused() {
    let tl = refinery_cell();
    let arrive = CellXY::new(tl.x + 1, tl.y + 3);
    for with_dest in [false, true] {
        let r = ruleset();
        let mut w = game(&r);
        let refn = place_building(&mut w, "REFN", 0, tl);
        let t = w.tick;
        let b = w.obj_mut(refn).unwrap();
        b.beh.current = Some(Behaviour::Selling);
        b.beh.start_tick = t;
        let h = tethered_enter(&mut w, refn, arrive, 0);
        let far = Target::Cell(at(8, 8));
        if with_dest {
            w.obj_mut(h).unwrap().dest = Some(far);
        }
        let rng = w.rng.clone();
        harvest::track_arrival(&mut w, h);
        let o = w.obj(h).unwrap();
        assert_eq!(w.rng, rng, "no draw");
        assert_eq!(o.beh.current, Some(Behaviour::Enter));
        assert_eq!(o.beh.queued, None);
        assert_eq!(o.contact, Some(refn));
        assert!(harvest::state_of(&w, h).tethered);
        if with_dest {
            assert_eq!(o.dest, Some(far), "a destination refuses the scatter");
        } else {
            let Some(Target::Cell(d)) = o.dest else {
                panic!("the nearby-cell result around the arrival cell");
            };
            assert!((d.x - arrive.x).abs() <= 2 && (d.y - arrive.y).abs() <= 2);
            assert_ne!(d, arrive);
        }
    }
}

// spec: S-0402 claim 3b and F1a (Q-0900; H27), S-0207 claim 26 and S-0215 claim 2 (a seeking harvester that ends its last track in the field cell still holds the order-level destination at the due processing before the next locomotion update, so it loads only at a processing strictly after the destination's removal tick T + 1; the first bale comes 19 ticks after loading starts), from several starting distances
#[test]
fn claim_3b_loading_starts_after_the_destination_is_removed() {
    let r = ruleset();
    let mut seen = 0;
    for (k, x) in [-1, 0, 1, 2, 3, 5, 6, 8].into_iter().enumerate() {
        let mut w = game_with_ore(&r, &field(4, -1, 5), 7 + k as u32);
        place_building(&mut w, "REFN", 0, refinery_cell());
        let h = place_vehicle(&mut w, "HARV", 0, at(x, 6 + k as i32 % 3), 0);
        harvest_now(&mut w, h);
        let mut removed = None; // T + 1: the tick whose locomotion update removed the destination
        let mut with_dest = false;
        let mut start = None;
        let mut bale = None;
        for _ in 0..400 {
            testkit::run(&mut w, 1);
            let dest = w.obj(h).unwrap().dest.is_some();
            let phase = harvest::phase(&w, h);
            if phase == Phase::Seeking && with_dest && !dest {
                removed = Some(w.tick);
            }
            if phase == Phase::Loading && start.is_none() {
                start = Some(w.tick);
            }
            if start.is_some() && bale.is_none() && harvest::state_of(&w, h).total() > 0 {
                bale = Some(w.tick);
                break;
            }
            with_dest = dest;
        }
        let (s, b) = (start.expect("loading"), bale.expect("bale"));
        if let Some(t1) = removed {
            // No loading start at or before the removal tick; the processing in T + 1 stays seeking.
            assert!(s > t1, "loading at {s}, destination removed at {t1}");
            seen += 1;
        }
        assert_eq!(b - s, 19, "the first bale 19 ticks after loading starts");
    }
    assert!(seen >= 4, "the arrivals with a removal observed: {seen}");
}

/// The harvest ruleset with a refinery smoke type on REFN: the smoke
/// system PLUME (smoke kind, PUFF, a long emission interval) at one set
/// smoke point, with a long refinery-smoke time.
fn smoky() -> glasnost_rules::schema::Ruleset {
    use glasnost_rules::sim::{ParticleSystemType, ParticleType};
    let mut r = ruleset();
    r.registries
        .entry("particles".into())
        .or_default()
        .push("PUFF".into());
    r.sim.particles.insert(
        "PUFF".into(),
        ParticleType {
            behaves_like: Some("smoke".into()),
            max_ec: Some(1000),
            ..Default::default()
        },
    );
    r.registries
        .entry("particle_systems".into())
        .or_default()
        .push("PLUME".into());
    r.sim.particle_systems.insert(
        "PLUME".into(),
        ParticleSystemType {
            behaves_like: Some("smoke".into()),
            holds_what: Some("PUFF".into()),
            spawn_frames: Some(1000),
            spawn_cutoff: dec("100000.0"),
            ..Default::default()
        },
    );
    let refn = r.sim.buildings.get_mut("REFN").unwrap();
    refn.refinery_smoke_particle_system = Some("PLUME".into());
    refn.refinery_smoke_offsets = Some(vec![[0, 0, 0], [40, 0, 0]]);
    r.sim.general.refinery_smoke_frames = Some(5000);
    r
}

/// The ticks of the smoke events of one unloading of `load` at REFN, each
/// with whether credits changed and whether finishing began in that tick.
fn smoke_events(load: Vec<i32>) -> Vec<(i32, bool, bool)> {
    let r = smoky();
    let mut w = game(&r);
    let (refn, h) = docked(&mut w, load, 16384);
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for _ in 0..200 {
        let c0 = w.houses[0].credits;
        let f0 = harvest::state_of(&w, h).finishing;
        testkit::run(&mut w, 1);
        let new: Vec<_> = w
            .gas
            .systems
            .iter()
            .filter(|(id, s)| s.owner_object == Some(refn) && !seen.contains(*id))
            .map(|(id, _)| *id)
            .collect();
        for id in &new {
            seen.insert(*id);
            out.push((
                w.tick,
                w.houses[0].credits != c0,
                !f0 && harvest::state_of(&w, h).finishing,
            ));
        }
        if current(&w, h) == Some(Behaviour::Harvest) {
            break;
        }
    }
    out
}

// spec: S-0402 claim 11 (revision 32) and S-0616 claim 1 (Q-1401: each payment row and the empty-load row that starts finishing make one smoke event on the building in the neighbouring cell, before the payment of that processing; k tiberiums make k + 1 events, an empty dock one; one system per set smoke point)
#[test]
fn claim_11_refinery_smoke_events() {
    // Two tiberiums: two payment rows and the finishing row.
    let e = smoke_events(vec![4, 3]);
    assert_eq!(e.len(), 3, "{e:?}");
    assert!(e[0].1 && e[1].1, "made in the tick of each payment: {e:?}");
    assert!(
        !e[2].1 && e[2].2,
        "the last in the tick finishing begins: {e:?}"
    );
    assert!(e[0].0 < e[1].0 && e[1].0 < e[2].0);
    // One tiberium: 2; docking empty: 1.
    let e = smoke_events(vec![4, 0]);
    assert_eq!(e.len(), 2, "{e:?}");
    let e = smoke_events(vec![0, 0]);
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].2, "the empty-load row");
}
