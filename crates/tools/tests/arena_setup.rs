//! Our arena's set-up, S-0216 revision 59 claim 8 (design), on test data of
//! our own (M8 step A3 intakes 71 and 77; in CI): scenario PT7's edge rules
//! (parts a to d) and the synthetic part of scenario PT6 (the order of the
//! groups and of the draws, the draws of refused cells, the take test, the
//! neighbour order, first come first served). Every type, cost and map
//! here is invented. The records' map part of PT6 is the local lane's
//! `m0006_bunkers_and_placed_cells_in_our_setup` and
//! `m0019_tick_0_placements_in_our_setup` in `tests/arena.rs`. The ledgers
//! are `docs/m8-claims/intake71-S-0513-S-0216.md` and
//! `docs/m8-claims/intake77.md`.

use std::collections::BTreeMap;
use std::sync::Arc;

use glasnost_map::{Cell, CellGrid, CellXY, Footprint};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::UnitType;
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseRole, ObjectId, World};
use glasnost_tools::oracle::{self, Ground, Side};

/// The synthetic arena rules ([`oracle::synthetic_ruleset`]: the combat
/// rules with a base-unit type and the neutral and special countries) with
/// test costs for CART and WALKER.
fn costs(cart: i32, walker: i32) -> Arc<SimRules> {
    let mut r: Ruleset = oracle::synthetic_ruleset();
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
            let a = oracle::setup(&g, [t, t], Side::A, 0).unwrap();
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
        let a = oracle::setup(&g, [t, t], Side::A, 0).unwrap();
        assert_eq!(
            a.units[0].len(),
            1,
            "{t}: 0.45 rounds to 0, held to the minimum"
        );
    }
    let g = ground(9000, 100);
    let a = oracle::setup(&g, ["WALKER", "CART"], Side::A, 0).unwrap();
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
        // The human player house's start (S-0216 claim 8, Game start), well away from W.
        (oracle::HUMAN_START as u16, wp(30, 46)),
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
    let a = oracle::setup(&g, ["CART", "CART"], Side::A, 0).unwrap();
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
    // spec: S-0216 claim 8 (Order: each unit created, then tested and placed, before the next is created, so its creation draw is followed by its placement draw; r59, Draws of a refused cell: a refused cell costs one try, its creation draw and, for infantry, one R(0, 3)) and claim 2 (an infantry unit's one draw R(0, 3)); PT6 (exactly one R(0, 3) after each placed infantry unit's creation draws; for each refused cell, the try's creation draws and one R(0, 3))
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
    // A cell that refuses: its try's creation draw and one R(0, 3), then the
    // next cell's try.
    let d = testkit::mid(30, 5, 5);
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, d, 0);
    let before = w.rng.taken();
    let (_, cell, why) = oracle::place_unit(&mut w, Kind::Infantry, ti, HouseId(0), d, 64).unwrap();
    assert_eq!(
        w.rng.taken() - before,
        2 * (k + 1),
        "the refused try's draws, then the placed unit's"
    );
    // spec: S-0216 claim 8 (Moving on: the first of the eight neighbours, in order, that takes it)
    assert_eq!(cell, CellXY::new(d.x - 1, d.y - 1));
    assert!(why.is_some(), "the vehicle's cell refuses");
}

/// A world on the 30-cell synthetic map whose stated cell `c` and its
/// first `p − 1` neighbours in claim 8's order each hold a vehicle of house
/// 1, so that the `p`-th cell of the order is the first that takes a unit.
fn blocked(r: &Ruleset, c: CellXY, p: usize) -> World {
    let mut w = testkit::game_with(r, 30, 3);
    for b in std::iter::once(c).chain(oracle::neighbours(c)).take(p) {
        testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, b, 0);
    }
    w
}

/// The objects of house 0.
fn own(w: &World) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(0)))
        .map(|o| o.id)
        .collect()
}

#[test]
fn pt6_a_unit_on_the_p_th_cell_follows_p_refused_tries_and_keeps_no_try() {
    // spec: S-0216 claim 8 (Draws of a refused cell, r59: a unit placed on the p-th cell of the order is preceded by p refused tries, 2p draws for infantry and p for a vehicle whose creation makes no other object; each try reserves its creation number; no object of a try is kept, a documented departure with no switch)
    let r = testkit::combat_ruleset();
    let c = testkit::mid(30, 0, 0);
    for (kind, t, per_try) in [(Kind::Infantry, "WALKER", 2), (Kind::Vehicle, "CART", 1)] {
        for p in [1usize, 3, 8] {
            let mut w = blocked(&r, c, p);
            let ti = w.rules().index_of(kind, t).unwrap();
            let half = w.rules().quirks.ranged_draw_half_span;
            let mut probe = w.rng.clone();
            let (before, first) = (w.rng.taken(), w.next_id);
            let (id, cell, _) = oracle::place_unit(&mut w, kind, ti, HouseId(0), c, 64).unwrap();
            let order: Vec<CellXY> = std::iter::once(c).chain(oracle::neighbours(c)).collect();
            assert_eq!(cell, order[p], "{t}, p = {p}: the p-th cell");
            let placement = u64::from(kind == Kind::Infantry);
            assert_eq!(
                w.rng.taken() - before,
                per_try * p as u64 + 1 + placement,
                "{t}, p = {p}: the tries' draws, then the unit's"
            );
            // spec: S-0216 claim 8 (r59: a refused try reserves one creation number, right after its creation draw)
            assert_eq!(
                id.0,
                first + p as u32,
                "{t}, p = {p}: the tries' numbers reserved"
            );
            assert_eq!(own(&w), vec![id], "{t}, p = {p}: no object of a try kept");
            // The placed unit's creation draw is the raw value after the tries' draws.
            for _ in 0..p {
                probe.raw();
                if kind == Kind::Infantry {
                    probe.ranged(0, 3, half);
                }
            }
            let want = (probe.raw() & 0xFFFF) as u16;
            assert_eq!(
                w.obj(id).unwrap().report_value,
                want,
                "{t}, p = {p}: its creation draw follows the tries'"
            );
        }
    }
}

/// The combat ruleset with an aircraft type DRN, an infantry type SLV, an
/// infantry spawner NEST (DRN ×2) and a vehicle slave master HERD (SLV ×2):
/// invented types whose creation makes other objects (S-0506 claim 1,
/// S-0402 claim 19).
fn makers() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push(&mut r, "aircraft", "DRN");
    push(&mut r, "infantry", "SLV");
    push(&mut r, "infantry", "NEST");
    push(&mut r, "vehicles", "HERD");
    let walker = r.sim.infantry["WALKER"].clone();
    let cart = r.sim.vehicles["CART"].clone();
    let s = &mut r.sim;
    s.aircraft.insert(
        "DRN".into(),
        UnitType {
            strength: Some(50),
            speed: Some(10),
            ..Default::default()
        },
    );
    s.infantry.insert("SLV".into(), walker.clone());
    let mut nest = walker;
    nest.combat.spawns = Some("DRN".into());
    nest.combat.spawns_number = Some(2);
    s.infantry.insert("NEST".into(), nest);
    let mut herd = cart;
    herd.econ.enslaves = Some("SLV".into());
    herd.econ.slaves_number = Some(2);
    s.vehicles.insert("HERD".into(), herd);
    r
}

#[test]
fn pt6_a_refused_try_makes_its_whole_creation_set_and_keeps_none_of_it() {
    // spec: S-0216 claim 8 (Draws of a refused cell, r59: the try's creation draws as in Order, the unit's own then those of the objects its creation makes, then, for an infantry unit, one R(0, 3), so a spawner's slot objects' draws fall between its creation draw and that R(0, 3); each object of the try reserves its creation number right after its own creation draw; none of them is kept)
    let r = makers();
    let c = testkit::mid(30, 0, 0);
    for (kind, t, made) in [(Kind::Infantry, "NEST", 2u32), (Kind::Vehicle, "HERD", 2)] {
        let mut w = blocked(&r, c, 1);
        let ti = w.rules().index_of(kind, t).unwrap();
        let half = w.rules().quirks.ranged_draw_half_span;
        let mut probe = w.rng.clone();
        let first = w.next_id;
        let (id, cell, why) = oracle::place_unit(&mut w, kind, ti, HouseId(0), c, 64).unwrap();
        assert_eq!(cell, oracle::neighbours(c)[0], "{t}: the first neighbour");
        assert!(why.is_some(), "{t}: moved on");
        // The try: 1 + made creation draws, then R(0, 3) for infantry.
        for _ in 0..=made {
            probe.raw();
        }
        if kind == Kind::Infantry {
            probe.ranged(0, 3, half);
        }
        // spec: S-0216 claim 8 (r59: one creation number for the try's unit and one for each object its creation makes)
        assert_eq!(
            id.0,
            first + 1 + made,
            "{t}: the try's set reserved its numbers"
        );
        for n in 0..=made {
            let x = glasnost_sim::world::ObjectId(id.0 + n);
            assert_eq!(
                w.obj(x).unwrap().report_value,
                (probe.raw() & 0xFFFF) as u16,
                "{t}: object {n} of the placed unit's set"
            );
        }
        let mut kept = own(&w);
        kept.sort_unstable();
        let want: Vec<ObjectId> = (0..=made).map(|n| ObjectId(id.0 + n)).collect();
        assert_eq!(kept, want, "{t}: only the placed unit's set");
        if kind == Kind::Infantry {
            // spec: S-0216 claim 2 (the placed unit's own R(0, 3), after its set's creation draws)
            probe.ranged(0, 3, half);
        }
        assert_eq!(w.rng, probe, "{t}: no other draw");
        assert!(
            (first..first + 1 + made).all(|n| w.obj(ObjectId(n)).is_none()),
            "{t}: no object of the try kept"
        );
    }
}

#[test]
fn pt6_the_row_types_group_is_created_first_on_whichever_side_it_plays() {
    // spec: S-0216 claim 8 (Order, r59: the group of the pairing's row type first, on whichever side it plays, then the other group, each in increasing unit index; measure: M-0019, creation order puts the row type's units first)
    let g = ground(1500, 1500);
    for (row_side, first) in [(Side::A, 0), (Side::B, 1)] {
        let a = oracle::setup(&g, ["CART", "WALKER"], row_side, 2).unwrap();
        let (lead, next) = (&a.units[first], &a.units[1 - first]);
        assert!(!lead.is_empty() && !next.is_empty());
        assert!(
            lead.windows(2).all(|p| p[0] < p[1]) && next.windows(2).all(|p| p[0] < p[1]),
            "{row_side:?}: each group in increasing index"
        );
        assert!(
            lead.last() < next.first(),
            "{row_side:?}: the row side's group is created first"
        );
        // spec: S-0216 claim 8 (Acceptance, r59: the first object's creation draw takes the scenario stream's raw value at the place the set-up starts from, counted from the seeding)
        assert!(
            a.moved.iter().all(|m| m.side != row_side || m.index > 0),
            "the first unit stands on its stated cell"
        );
        // spec: S-0216 claim 8 (Seeds, r69: seed number 2's stream is seeded with 1000 + 2)
        let mut fresh = glasnost_sim::rng::Rng::new(oracle::game_seed(2).unwrap());
        for _ in 0..a.first_raw {
            fresh.raw();
        }
        assert_eq!(
            a.world.obj(lead[0]).unwrap().report_value,
            (fresh.raw() & 0xFFFF) as u16,
            "{row_side:?}: the first creation draw is at place first_raw"
        );
    }
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

// --- PT6, synthetic part: the game start (r67, Q-2336) ---------------------------------

/// The base unit of house `h` in `w`: its only object.
fn base_unit_of(w: &World, h: HouseId) -> ObjectId {
    let mine: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.owner == Some(h))
        .map(|o| o.id)
        .collect();
    assert_eq!(mine.len(), 1, "house {h:?} owns its base unit only");
    mine[0]
}

#[test]
fn pt6_game_start_houses_in_claim_19s_order() {
    // spec: S-0216 claim 8 (Game start: a human player house at start 3, a player and not an observer, then the two computer houses, then the neutral and the special house, in S-0009 claim 19's order)
    let g = ground(1500, 1500);
    let a = oracle::setup(&g, ["CART", "WALKER"], Side::A, 0).unwrap();
    let w = &a.world;
    let roles: Vec<HouseRole> = w.houses.iter().map(|h| h.role).collect();
    assert_eq!(
        roles,
        [
            HouseRole::Player,
            HouseRole::Player,
            HouseRole::Player,
            HouseRole::Neutral,
            HouseRole::Special
        ]
    );
    let human = &w.houses[usize::from(oracle::HUMAN.0)];
    assert!(human.human && human.idle_test_house);
    assert!(!human.econ.setup.observer, "a player, not an observer");
    assert_eq!(human.start, Some(oracle::HUMAN_START));
    assert_eq!(a.houses, oracle::SIDE_HOUSES);
    for h in oracle::SIDE_HOUSES {
        let house = &w.houses[usize::from(h.0)];
        assert!(
            !house.human && !house.idle_test_house,
            "{h:?}: a computer house"
        );
    }
    // spec: S-0216 claim 8 (Seeds and the human player house's country, r70: the human player house is British) and measure: M-0001 to M-0003 (Setup)
    let country =
        |h: HouseId| w.rules().country_names[w.houses[usize::from(h.0)].country as usize].clone();
    assert_eq!(country(oracle::HUMAN), "British");
    assert_ne!(country(oracle::HUMAN), g.countries[0]);
    assert_eq!(country(oracle::SIDE_HOUSES[0]), g.countries[0]);
    // spec: S-0216 claim 8 (Game start: crates off)
    assert_eq!(w.econ.settings.crates, Some(false));
}

#[test]
fn pt6_rules_without_the_british_country_fail_with_a_clear_error() {
    // spec: S-0216 claim 8 (Seeds and the human player house's country, r70: the human player house is British; no silent fall back when the loaded data has no such country)
    let mut r = oracle::synthetic_ruleset();
    r.registries
        .get_mut("countries")
        .unwrap()
        .retain(|n| n != oracle::HUMAN_COUNTRY);
    let mut g = ground(1500, 1500);
    g.rules = Arc::new(SimRules::new(&r, "intake91-no-british"));
    let e = oracle::setup(&g, ["CART", "WALKER"], Side::A, 0)
        .err()
        .expect("no British country, no game");
    assert!(e.contains("British"), "{e}");
    assert!(oracle::human_country(&g.rules).is_err());
    // With the country present the lookup returns the rules' own spelling.
    assert_eq!(
        oracle::human_country(&ground(1500, 1500).rules).as_deref(),
        Ok("British")
    );
}

#[test]
fn pt6_seed_number_s_seeds_the_stream_with_1000_plus_s() {
    // spec: S-0216 claim 8 (Seeds and the human player house's country, r69, Q-2447 answered: the arena's game for seed number s seeds S-0202's generator with 1000 + s; the Acceptance item, PT6 and every per-seed comparison use that seed)
    assert_eq!(oracle::game_seed(0), Some(1000));
    assert_eq!(oracle::game_seed(5), Some(1005));
    assert_eq!(oracle::game_seed(u32::MAX), None);
    let g = ground(1500, 1500);
    for seed in oracle::SEEDS {
        let a = oracle::setup(&g, ["CART", "WALKER"], Side::A, seed).unwrap();
        let first = a.units[0][0];
        // The first set-up object's creation draw takes the raw value at
        // place first_raw of the stream seeded with 1000 + s, not of the
        // stream seeded with s.
        let at = |s: u32| {
            let mut r = Rng::new(s);
            for _ in 0..a.first_raw {
                r.raw();
            }
            (r.raw() & 0xFFFF) as u16
        };
        let got = a.world.obj(first).unwrap().report_value;
        assert_eq!(got, at(1000 + seed), "seed number {seed}");
        assert_ne!(
            got,
            at(seed),
            "seed number {seed}: not the stream seeded with {seed}"
        );
    }
}

#[test]
fn pt6_game_start_two_house_rounds_three_base_units_then_the_final_draw() {
    // spec: S-0216 claim 8 (Game start: five houses created in each of the two rounds of S-0202 claim 14, three base units created, each with its creation draw and put on its base cell with no draw, no budget unit, then the final R(0, 65535); Removal before the groups: no draw) and PT6 (Expected, rules)
    let g = ground(1500, 1500);
    for seed in [0, 1, 2, 5] {
        let a = oracle::setup(&g, ["CART", "WALKER"], Side::A, seed).unwrap();
        let w = &a.world;
        let half = w.rules().quirks.ranged_draw_half_span;
        let k = creation_draws(w, Kind::Vehicle, oracle::SYNTHETIC_BASE_UNIT);
        // spec: S-0216 claim 8 (Seeds, r69: seed number s's stream is seeded with 1000 + s)
        let mut want = Rng::new(oracle::game_seed(seed).unwrap());
        for _ in 0..2 * w.houses.len() {
            // spec: S-0202 claim 14 (one R(450, 1800) per house and round)
            let _ = want.ranged(450, 1800, half);
        }
        let rounds = want.taken();
        for _ in 0..3 * k {
            want.raw();
        }
        // spec: S-0009 claim 20 and Constants (the final R(0, 65535) after the last house)
        let _ = want.ranged(0, 65535, half);
        assert_eq!(
            a.first_raw,
            want.taken(),
            "seed {seed}: no other draw before the first group (houses {}, rounds' raw values {rounds}, creation draws per base unit {k})",
            w.houses.len()
        );
    }
}

#[test]
fn pt6_computer_base_units_are_taken_off_and_the_humans_stays() {
    // spec: S-0216 claim 8 (Removal before the groups: the two computer houses' base units taken off the map, none of them at the next tick, their creation numbers taken; the human player house's base unit on its base cell; the base units' creation numbers before every set-up object's)
    let g = ground(1500, 1500);
    let a = oracle::setup(&g, ["CART", "WALKER"], Side::A, 0).unwrap();
    let mut w = a.world.clone();
    assert_eq!(a.removed.len(), 2, "the two computer houses' base units");
    let human = base_unit_of(&w, oracle::HUMAN);
    let o = w.obj(human).unwrap();
    let base = ground_cell(&g, oracle::HUMAN_START);
    assert_eq!(
        w.rules()
            .index_of(Kind::Vehicle, oracle::SYNTHETIC_BASE_UNIT),
        Some(o.type_index)
    );
    assert_eq!(o.pos.cell(), base, "the human's base unit on its base cell");
    let health = o.health;
    // The three base units in house order, before every set-up object.
    let first_unit = a.units.iter().flatten().min().copied().unwrap();
    assert!(human < a.removed[0] && a.removed[0] < a.removed[1]);
    assert!(a.removed[1] < first_unit);
    for id in &a.removed {
        assert!(w.obj(*id).is_none(), "{id:?} is out of the game");
    }
    // spec: S-0216 claim 8 (Removal before the groups: the human's base unit stays on its base cell for the whole game; no object targets or damages it)
    for _ in 0..600 {
        glasnost_sim::tick::step(&mut w);
        let o = w.obj(human).expect("the human's base unit stays");
        assert_eq!((o.pos.cell(), o.health), (base, health), "tick {}", w.tick);
        for id in &a.removed {
            assert!(w.obj(*id).is_none(), "{id:?} never comes back");
        }
    }
}

/// The cell of start waypoint `i` of `g`'s map.
fn ground_cell(g: &Ground, i: u32) -> CellXY {
    let (x, y) = g.map.waypoints[&(i as u16)];
    CellXY::new(i32::from(x), i32::from(y))
}

#[test]
fn pt6_the_removal_takes_no_draw_starts_no_animation_and_frees_the_cells() {
    // spec: S-0216 claim 8 (Removal before the groups: every vehicle the human player house does not own, the neutral house's included, taken off with no random draw and no animation, its cells freed for the take test, its creation number kept; the neutral house's infantry stay)
    let r = oracle::synthetic_ruleset();
    let mut w = testkit::game_with_non_players(&r, 30, 3);
    let neutral = HouseId(2);
    let human = HouseId(0);
    let c = |dx| testkit::mid(30, dx, 0);
    let mine = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, c(-6), 0);
    let theirs = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(0), 0);
    let neutral_vehicle = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", neutral.0, c(3), 0);
    let neutral_man = testkit::spawn(&mut w, Kind::Infantry, "WALKER", neutral.0, c(6), 0);
    let (rng, anims, next) = (w.rng.clone(), w.anims.clone(), w.next_id);
    let gone = oracle::take_off_before_the_groups(&mut w, human);
    assert_eq!(gone, vec![theirs, neutral_vehicle]);
    assert_eq!(w.rng, rng, "no random draw");
    assert_eq!(w.anims, anims, "no animation");
    assert_eq!(w.next_id, next, "the creation numbers stay taken");
    assert!(w.obj(mine).is_some() && w.obj(neutral_man).is_some());
    // The take test no longer sees them: a new vehicle stands on each freed cell.
    let ti = w.rules().index_of(Kind::Vehicle, "CART").unwrap();
    for at in [c(0), c(3)] {
        let (_, cell, why) = oracle::place_unit(&mut w, Kind::Vehicle, ti, human, at, 64).unwrap();
        assert_eq!((cell, why), (at, None), "{at:?} is free");
    }
}

#[test]
fn pt6_no_object_targets_or_damages_the_idle_house() {
    // spec: S-0216 claim 8 (Removal before the groups: no house treats the human player house as an enemy; no object targets or damages its objects; design) — with the flag clear, the same scene fights, so the flag is what keeps it out
    for idle in [true, false] {
        let mut w = testkit::game_with(&testkit::combat_ruleset(), 30, 3);
        w.houses[0].idle_test_house = idle;
        let quiet = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, testkit::mid(30, 0, 0), 0);
        let gun = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, testkit::mid(30, 3, 0), 0);
        let full = w.obj(quiet).unwrap().health;
        // A hit with the combat ruleset's first warhead.
        let hit = glasnost_sim::combat::damage::Hit {
            amount: 50,
            warhead: Some(0),
            distance: 0,
            attacker: Some(gun),
            house: Some(HouseId(1)),
            ignore_defences: false,
            no_escape: false,
        };
        let result = glasnost_sim::combat::damage::apply(&mut w, quiet, hit);
        if idle {
            assert_eq!(result, glasnost_sim::combat::damage::HitResult::Unaffected);
            assert_eq!(w.obj(quiet).unwrap().health, full, "a hit does no damage");
        } else {
            assert!(
                w.obj(quiet).unwrap().health < full,
                "control: the hit damages"
            );
        }
        let before = w.obj(quiet).unwrap().health;
        let mut targeted = false;
        for _ in 0..300 {
            glasnost_sim::tick::step(&mut w);
            targeted |= w.obj(gun).and_then(|o| o.target)
                == Some(glasnost_sim::world::Target::Object(quiet));
        }
        let after = w.obj(quiet).map_or(0, |o| o.health);
        if idle {
            assert!(!targeted, "nothing targets it");
            assert_eq!(after, before, "nothing damages it");
            assert_ne!(
                w.houses[1].combat.enemy,
                Some(HouseId(0)),
                "no house makes it its enemy house"
            );
        } else {
            assert!(targeted && after < before, "control: it is fought");
        }
    }
}
