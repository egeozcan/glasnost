//! Our arena's set-up, S-0216 revision 37 claim 8 (design), on test data of
//! our own (M8 step A3 intake 71; in CI): scenario PT7's edge rules (parts
//! a to d) and the synthetic part of scenario PT6 (the order of the draws,
//! the take test, the neighbour order, first come first served). Every
//! type, cost and map here is invented. The records' map part of PT6 is
//! the local lane's `m0006_bunkers_and_placed_cells_in_our_setup` in
//! `tests/arena.rs`. The ledger is `docs/m8-claims/intake71-S-0513-S-0216.md`.

use std::collections::BTreeMap;
use std::sync::Arc;

use glasnost_map::{Cell, CellGrid, CellXY, Footprint};
use glasnost_rules::schema::Ruleset;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};
use glasnost_tools::oracle::{self, Ground, Side};

/// The synthetic combat rules with test costs for CART and WALKER.
fn costs(cart: i32, walker: i32) -> Arc<SimRules> {
    let mut r: Ruleset = testkit::combat_ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().cost = Some(cart);
    r.sim.infantry.get_mut("WALKER").unwrap().cost = Some(walker);
    Arc::new(SimRules::new(&r, "intake71-arena-setup"))
}

/// The synthetic ground of size 40 with test costs.
fn ground(cart: i32, walker: i32) -> Ground {
    let mut g = oracle::synthetic_ground(40);
    g.rules = costs(cart, walker);
    g
}

// --- PT7 parts a and b: the counts, through the set-up ------------------------------

#[test]
fn pt7_a_an_exact_half_goes_to_the_even_count_for_both_kinds() {
    // spec: S-0216 claim 8 (Group counts and Formulas: an exact half goes to the even one of the two counts, the same rule for infantry and vehicle groups) and PT7 part a (B / c = n + 1/2: n when n is even, n + 1 when n is odd)
    // 9000 / 2000 = 4.5 (n = 4, even) and 9000 / 1200 = 7.5 (n = 7, odd).
    for (cost, n) in [(2000, 4), (1200, 8)] {
        let g = ground(cost, cost);
        for t in ["CART", "WALKER"] {
            let (_, count, _) = oracle::group(&g.rules, t).unwrap();
            assert_eq!(count, n, "{t} at cost {cost}");
            let a = oracle::setup(&g, [t, t], 0).unwrap();
            assert_eq!(a.units[0].len(), n as usize, "{t}: side A");
            assert_eq!(a.units[1].len(), n as usize, "{t}: side B");
        }
    }
}

#[test]
fn pt7_b_the_minimum_and_the_infantry_cap() {
    // spec: S-0216 claim 8 (Group counts: the rounded count held to the records' minimum and infantry cap, M-0001 Setup) and PT7 part b
    let g = ground(20000, 20000);
    for t in ["CART", "WALKER"] {
        let a = oracle::setup(&g, [t, t], 0).unwrap();
        assert_eq!(
            a.units[0].len(),
            1,
            "{t}: 0.45 rounds to 0, held to the minimum"
        );
    }
    let g = ground(9000, 100);
    let a = oracle::setup(&g, ["WALKER", "CART"], 0).unwrap();
    assert_eq!(
        a.units[0].len(),
        oracle::INFANTRY_CAP as usize,
        "90 held to the cap"
    );
}

// --- PT7 part c: the error stop -----------------------------------------------------

#[test]
fn pt7_c_a_unit_no_cell_takes_stops_the_set_up_with_an_error() {
    // spec: S-0216 claim 8 (Moving on: if none of the stated cell and its eight neighbours takes the unit, the set-up stops and reports an error) and PT7 part c (for an infantry unit each of the nine cells holds a vehicle; for a vehicle, each holds a unit)
    for (kind, t, bkind, blocker) in [
        (Kind::Infantry, "WALKER", Kind::Vehicle, "CART"),
        (Kind::Vehicle, "CART", Kind::Infantry, "WALKER"),
    ] {
        let mut w = testkit::game_with(&testkit::combat_ruleset(), 30, 3);
        let ti = w.rules().index_of(kind, t).unwrap();
        let stated = testkit::mid(30, 0, 0);
        for c in std::iter::once(stated).chain(oracle::neighbours(stated)) {
            testkit::spawn(&mut w, bkind, blocker, 1, c, 0);
        }
        let on_map = |w: &World| w.objects.values().filter(|o| o.on_map).count();
        let before = on_map(&w);
        let e = oracle::place_unit(&mut w, kind, ti, HouseId(0), stated, 64)
            .err()
            .unwrap_or_else(|| panic!("{t}: placed"));
        assert!(e.contains("nor a neighbouring cell takes it"), "{t}: {e}");
        assert_eq!(on_map(&w), before, "{t}: no unit of that case on the map");
    }
    // `oracle::setup` returns that error, prefixed with the type and the unit's index.
}

// --- PT7 part d: raised cells at the usable area's top and bottom edges ----------------

/// A flat map of 40 × 40 cells whose usable area spans the half-rows s =
/// x + y from 71 to 82 (S-0009 claim 4's plain test: W + 2·Ly < s ≤
/// W + 2 + 2·(Ly + Lh), with W = 40, Ly = 15, Lh = 5), with the arena's W at
/// (40, 41), level-1 flat cells at the two stated cells and level 0
/// elsewhere.
fn edge_ground(raised: &[CellXY]) -> Ground {
    let fp = Footprint::new(40, 40, [0, 15, 40, 5]);
    let mut cells = CellGrid::new(fp);
    let all: Vec<CellXY> = cells.iter().map(|(c, _)| c).collect();
    for c in all {
        if let Some(cell) = cells.get_mut(c) {
            *cell = Cell {
                tile: 0,
                stream: true,
                level: i8::from(raised.contains(&c)),
                ..Cell::BLANK
            };
        }
    }
    let mut g = ground(9000, 9000);
    g.map.footprint = fp;
    g.map.cells = cells;
    g.w = CellXY::new(40, 41);
    let wp = |x: i16, y: i16| (x, y);
    g.map.waypoints = BTreeMap::from([
        (0, wp(35, 41)),
        (1, wp(42, 38)),
        (oracle::W_WAYPOINT, wp(40, 41)),
    ]);
    g
}

#[test]
fn pt7_d_the_height_aware_test_moves_the_top_edge_vehicle_and_keeps_the_bottom_one() {
    // spec: S-0216 claim 8 (The take test: inside the usable area by S-0009 claim 4's height-aware test, as S-0009 claim 5a applies it) and PT7 part d (a level-1 cell in the topmost half-row by the plain test is outside by the height-aware one; a level-1 cell in the half-row below the bottom edge is inside it)
    let probe = edge_ground(&[]);
    let top = oracle::cell_of(probe.w, Side::A, 0);
    let bottom = oracle::cell_of(probe.w, Side::B, 0);
    assert_eq!((top.x + top.y, bottom.x + bottom.y), (71, 83));
    let g = edge_ground(&[top, bottom]);
    let a = oracle::setup(&g, ["CART", "CART"], 0).unwrap();
    let w = &a.world;
    for c in [top, bottom] {
        let cell = w.cells.cell(c).unwrap();
        assert_eq!((cell.level, cell.slope), (1, 0), "{c:?}: flat, raised");
    }
    let fp = *w.cells.footprint();
    assert!(fp.is_usable_with(top, 0) && !w.cells.usable(top));
    assert!(!fp.is_usable_with(bottom, 0) && w.cells.usable(bottom));
    let at = |u| w.obj(u).unwrap().pos.cell();
    // The first neighbour inside the area: (x − 1, y − 1) and (x, y − 1) lie above it.
    assert_eq!(
        at(a.units[0][0]),
        CellXY::new(top.x + 1, top.y - 1),
        "top edge: moved on"
    );
    assert_eq!(at(a.units[1][0]), bottom, "bottom edge: on its stated cell");
}

// --- PT6, synthetic part: the order of the draws and the take test ---------------------

/// The scenario-stream draws one creation of `name` takes.
fn creation_draws(w: &World, kind: Kind, name: &str) -> u64 {
    let mut c = w.clone();
    let ti = c.rules().index_of(kind, name).unwrap();
    let before = c.rng.taken();
    glasnost_sim::objects::create(&mut c, kind, ti, Some(HouseId(0)));
    c.rng.taken() - before
}

#[test]
fn pt6_each_unit_takes_its_creation_draw_then_its_one_placement_draw() {
    // spec: S-0216 claim 8 (Order: each unit created, then tested and placed, before the next is created, so its creation draw is followed by its placement draw; the take test takes no draw, so a refused cell leaves no standing-position draw) and claim 2 (an infantry unit's one draw R(0, 3)); PT6 (exactly one R(0, 3) after each infantry unit's creation draw, none at a refused cell)
    let mut w = testkit::game_with(&testkit::combat_ruleset(), 30, 3);
    let ti = w.rules().index_of(Kind::Infantry, "WALKER").unwrap();
    let k = creation_draws(&w, Kind::Infantry, "WALKER");
    let c = testkit::mid(30, 0, 0);
    let mut ids = Vec::new();
    for n in 0..3 {
        let before = w.rng.taken();
        let (id, cell, why) =
            oracle::place_unit(&mut w, Kind::Infantry, ti, HouseId(0), c, 64).unwrap();
        assert_eq!(
            w.rng.taken() - before,
            k + 1,
            "unit {n}: its creation, then one R(0, 3)"
        );
        // spec: S-0216 claim 8 (first come, first served: an infantry unit can be taken by a cell that holds infantry)
        assert_eq!((cell, why), (c, None), "unit {n}: shares the cell");
        ids.push(id);
    }
    let points: Vec<_> = ids.iter().map(|&i| w.obj(i).unwrap().pos).collect();
    assert!(
        points[0] != points[1] && points[1] != points[2] && points[0] != points[2],
        "three standing positions"
    );
    // A cell that refuses: the take test takes no draw there.
    let d = testkit::mid(30, 5, 5);
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, d, 0);
    let before = w.rng.taken();
    let (_, cell, why) = oracle::place_unit(&mut w, Kind::Infantry, ti, HouseId(0), d, 64).unwrap();
    assert_eq!(w.rng.taken() - before, k + 1, "no draw at the refused cell");
    // spec: S-0216 claim 8 (Moving on: the first of the eight neighbours, in order, that takes it)
    assert_eq!(cell, CellXY::new(d.x - 1, d.y - 1));
    assert!(why.is_some(), "the vehicle's cell refuses");
}

#[test]
fn pt6_a_full_cell_moves_infantry_on_and_the_mission_is_queued() {
    // spec: S-0216 claim 8 (The take test, infantry: at least one of the right, left and bottom standing positions free; Placement: the group's mission queued as a placing step's behaviour, claim 4) and S-0215 claim 7 (queued without being started)
    let mut w = testkit::game_with(&testkit::combat_ruleset(), 30, 3);
    let ti = w.rules().index_of(Kind::Infantry, "WALKER").unwrap();
    let c = testkit::mid(30, 0, 0);
    let mut cells = Vec::new();
    for _ in 0..4 {
        let (id, cell, _) =
            oracle::place_unit(&mut w, Kind::Infantry, ti, HouseId(0), c, 64).unwrap();
        cells.push(cell);
        let o = w.obj(id).unwrap();
        assert_eq!(o.beh.queued, Some(Behaviour::Hunt), "hunt queued");
        assert_ne!(o.beh.current, Some(Behaviour::Hunt), "not started");
    }
    assert_eq!(&cells[..3], [c, c, c], "three side positions");
    assert_eq!(
        cells[3],
        CellXY::new(c.x - 1, c.y - 1),
        "the full cell moves the fourth on"
    );
}
