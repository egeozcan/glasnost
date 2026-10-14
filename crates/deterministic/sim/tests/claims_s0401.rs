//! Claim tests for S-0401 (ore and gems) on synthetic data: invented
//! tiberiums, overlays and values, no stock data. Houses Alpha (0) and
//! Beta (1) of the testkit's sandbox game.

mod harvest_common;

use glasnost_map::NO_OVERLAY;
use glasnost_map::terrain::land;
use glasnost_rules::sim::TerrainObjectType as TerrainType;
use glasnost_sim::econ::resource;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::World;
use harvest_common::*;

fn growth_cells(w: &World, t: usize) -> Vec<glasnost_map::CellXY> {
    w.econ.resources.tiberiums[t]
        .growth
        .entries
        .iter()
        .map(|e| e.cell)
        .collect()
}

fn spread_cells(w: &World, t: usize) -> Vec<glasnost_map::CellXY> {
    w.econ.resources.tiberiums[t]
        .spread
        .entries
        .iter()
        .map(|e| e.cell)
        .collect()
}

/// The neighbours of `c` in step-direction order.
fn neighbours(c: glasnost_map::CellXY) -> Vec<glasnost_map::CellXY> {
    glasnost_sim::regions::STEPS
        .iter()
        .map(|&(dx, dy)| glasnost_map::CellXY::new(c.x + dx, c.y + dy))
        .collect()
}

// spec: S-0401 claim 1 (a resource cell holds one tiberium, by the image run its overlay falls in, at a stage)
#[test]
fn claim_1_tiberium_and_stage_of_a_cell() {
    let r = ruleset();
    let (a, b) = (at(0, 0), at(3, 0));
    let w = game_with_ore(&r, &[(a, ORE + 5, 4), (b, GEM + 15, 2)], 7);
    assert_eq!(resource::resource_at(&w, a), Some((0, 4)));
    assert_eq!(resource::resource_at(&w, b), Some((1, 2)));
    assert_eq!(resource::resource_at(&w, at(1, 1)), None);
    assert_eq!(resource::tiberium_of_overlay(&w, NO_OVERLAY), None);
}

// spec: S-0401 claim 2 and F1 (d + 1 bales; worth V × (d + 1), 0 without resource)
#[test]
fn claim_2_f1_worth() {
    let r = ruleset();
    let (a, b) = (at(0, 0), at(3, 0));
    let w = game_with_ore(&r, &[(a, ORE, 3), (b, GEM, 0)], 7);
    assert_eq!(resource::worth(&w, a), i64::from(ORE_VALUE) * 4);
    assert_eq!(resource::worth(&w, b), i64::from(GEM_VALUE));
    assert_eq!(resource::worth(&w, at(1, 1)), 0);
}

// spec: S-0401 claim 3 (n ≤ d: stage d − n and n delivered; n ≥ d + 1: emptied, d delivered; n ≤ 0: nothing)
#[test]
fn claim_3_taking_bales_and_the_lost_last_bale() {
    let r = ruleset();
    let c = at(0, 0);
    let mut w = game_with_ore(&r, &[(c, ORE, 5)], 7);
    assert_eq!(resource::take_bales(&mut w, c, 0), 0);
    assert_eq!(stage(&w, c), Some(5));
    assert_eq!(resource::take_bales(&mut w, c, 2), 2);
    assert_eq!(stage(&w, c), Some(3));
    // spec: S-0401 claim 3 (quirks.last_bale_lost: the stage-0 bale yields nothing)
    assert_eq!(resource::take_bales(&mut w, c, 9), 3);
    assert_eq!(stage(&w, c), None);
    assert_eq!(resource::take_bales(&mut w, c, 1), 0);
}

// spec: S-0401 claims 3 and 16c (a 6-bale take: stages 0 and 5 are emptied, stage 6 keeps its overlay at stage 0, stage 7 at stage 1)
#[test]
fn claim_16c_six_bale_take_by_stage() {
    let r = ruleset();
    let cells = [(at(0, 0), 0), (at(3, 0), 5), (at(6, 0), 6), (at(9, 0), 7)];
    let ore: Vec<_> = cells.iter().map(|&(c, d)| (c, ORE, d)).collect();
    let mut w = game_with_ore(&r, &ore, 7);
    for &(c, _) in &cells {
        resource::take_bales(&mut w, c, 6); // spec: S-0401 claim 16c (6 bales)
    }
    assert_eq!(stage(&w, cells[0].0), None);
    assert_eq!(stage(&w, cells[1].0), None);
    assert_eq!(stage(&w, cells[2].0), Some(0));
    assert_eq!(stage(&w, cells[3].0), Some(1));
}

// spec: S-0401 claim 3 (quirks.last_bale_lost off: all d + 1 bales delivered)
#[test]
fn claim_3_quirk_last_bale_lost_off() {
    let mut r = ruleset();
    r.globals.quirks.last_bale_lost = false;
    let c = at(0, 0);
    let mut w = game_with_ore(&r, &[(c, ORE, 3)], 7);
    assert_eq!(resource::take_bales(&mut w, c, 9), 4);
    assert_eq!(stage(&w, c), None);
}

// spec: S-0401 claim 4 (the map growth switch: off, no update runs and the lists are rebuilt empty for growth)
#[test]
fn claim_4_map_growth_switch() {
    let mut r = ruleset();
    fractions(&mut r, "ORE", "1", "0");
    let c = at(0, 0);
    let mut w = game_with_map(&r, &[(c, ORE, 3)], 7, |m| {
        m.basic.tiberium_growth = Some(false);
    });
    assert!(w.econ.resources.growth_off);
    assert!(growth_cells(&w, 0).is_empty());
    testkit::run(&mut w, 40);
    assert_eq!(stage(&w, c), Some(3));
    assert_eq!(w.econ.resources.tiberiums[0].growth_last, None);
    // With the switch on the same map grows.
    let mut w = game_with_ore(&r, &[(c, ORE, 3)], 7);
    assert!(!w.econ.resources.growth_off);
    testkit::run(&mut w, 1);
    assert!(stage(&w, c).unwrap() > 3);
}

// spec: S-0401 claim 5 (emptying: overlay removed, stage 0, land worked out again, unmarked, neighbours offered to the spread list)
#[test]
fn claim_5_emptying_a_cell() {
    let mut r = ruleset();
    fractions(&mut r, "ORE", "0", "1");
    let (c, n) = (at(0, 0), at(1, 0));
    let mut w = game_with_ore(&r, &[(c, ORE, 0), (n, ORE, 6)], 7);
    // n is in the spread list from the start; drop it to see the offer.
    w.econ.resources.tiberiums[0].spread = Default::default();
    assert_eq!(w.cells.cell(c).unwrap().land, land::TIBERIUM);
    resource::take_bales(&mut w, c, 1);
    let cell = w.cells.cell(c).unwrap();
    assert_eq!(cell.overlay, NO_OVERLAY);
    assert_eq!(cell.overlay_data, 0);
    assert_ne!(cell.land, land::TIBERIUM);
    assert_eq!(spread_cells(&w, 0), vec![n]);
    assert!(w.econ.resources.tiberiums[0].spread.marks.contains(&n));
}

// spec: S-0401 claim 5 (quirks.spread_offer_by_emptied_type: neighbours join the emptied cell's tiberium's list; off: their own)
#[test]
fn claim_5_quirk_spread_offer_by_emptied_type() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.spread_offer_by_emptied_type = on;
        fractions(&mut r, "ORE", "0", "1");
        fractions(&mut r, "GEM", "0", "1");
        let (c, n) = (at(0, 0), at(1, 0));
        let mut w = game_with_ore(&r, &[(c, ORE, 0), (n, GEM, 6)], 7);
        w.econ.resources.tiberiums[1].spread = Default::default();
        resource::take_bales(&mut w, c, 1);
        let (ore, gem) = (spread_cells(&w, 0), spread_cells(&w, 1));
        if on {
            assert_eq!((ore, gem), (vec![n], vec![]));
        } else {
            assert_eq!((ore, gem), (vec![], vec![n]));
        }
    }
}

// spec: S-0401 claim 6 and F3 (growth period trunc(G × 0.3); spread period S; first updates in the first tick, then every period)
#[test]
fn claim_6_f3_f4_timers() {
    let r = ruleset();
    let mut w = game(&r);
    // spec: S-0401 F3 (the double nearest 0.3 is below 0.3: G = 100 gives 29, not 30)
    assert_eq!(resource::growth_period(&w, 0), 29);
    assert_eq!(resource::spread_period(&w, 0), 100);
    let t0 = w.tick;
    testkit::run(&mut w, 1);
    assert_eq!(w.econ.resources.tiberiums[0].growth_last, Some(t0));
    assert_eq!(w.econ.resources.tiberiums[0].spread_last, Some(t0));
    testkit::run(&mut w, 28);
    assert_eq!(w.econ.resources.tiberiums[0].growth_last, Some(t0));
    testkit::run(&mut w, 1);
    assert_eq!(w.econ.resources.tiberiums[0].growth_last, Some(t0 + 29));
    assert_eq!(w.econ.resources.tiberiums[0].spread_last, Some(t0));
}

// spec: S-0401 claim 7 (growth steps of 1; a lone cell's re-entry is taken again, up to m ≤ 5 steps; at 11 no re-entry; stage-11 cells stay out)
#[test]
fn claim_7_growth_update() {
    let mut r = ruleset();
    fractions(&mut r, "ORE", "1", "0");
    for seed in 1..6 {
        let (a, b) = (at(0, 0), at(4, 0));
        let mut w = game_with_ore(&r, &[(a, ORE, 3), (b, ORE, 11)], seed);
        assert_eq!(growth_cells(&w, 0), vec![a]);
        testkit::run(&mut w, 1);
        // n = 1: k = clamp(trunc(1 × 1.0), 5, 50) = 5, so m is 1 … 5.
        let d = stage(&w, a).unwrap();
        assert!((4..=8).contains(&d), "seed {seed}: {d}");
        assert_eq!(growth_cells(&w, 0), vec![a]);
        assert_eq!(stage(&w, b), Some(11));
    }
    let a = at(0, 0);
    let mut w = game_with_ore(&r, &[(a, ORE, 10)], 7);
    testkit::run(&mut w, 1);
    assert_eq!(stage(&w, a), Some(11));
    assert!(growth_cells(&w, 0).is_empty());
}

// spec: S-0401 claim 8 (no object in the cell's ground layer; a marked cell is not added twice)
#[test]
fn claim_8_spread_list_offers() {
    let mut r = ruleset();
    fractions(&mut r, "ORE", "0", "1");
    let (c, n, m) = (at(0, 0), at(1, 0), at(-1, 0));
    let mut w = game_with_ore(&r, &[(c, ORE, 1), (n, ORE, 6), (m, ORE, 6)], 7);
    w.econ.resources.tiberiums[0].spread = Default::default();
    place_vehicle(&mut w, "HARV", 0, m, 0);
    resource::take_bales(&mut w, c, 9);
    // m holds a vehicle: not added; n added and marked.
    assert_eq!(spread_cells(&w, 0), vec![n]);
    let e = w.econ.resources.tiberiums[0].spread.count;
    // A second emptying next to n does not add it again.
    let c2 = at(2, 0);
    put(&mut w, c2, ORE, 0);
    resource::take_bales(&mut w, c2, 1);
    assert_eq!(spread_cells(&w, 0), vec![n]);
    assert_eq!(w.econ.resources.tiberiums[0].spread.count, e);
}

// spec: S-0401 F2 (key = t + (|r| mod 50), one raw draw per added entry)
#[test]
fn f2_offer_keys() {
    let mut r = ruleset();
    fractions(&mut r, "ORE", "0", "1");
    let c = at(0, 0);
    let ns: Vec<_> = neighbours(c);
    let mut ore: Vec<_> = ns.iter().map(|&n| (n, ORE, 6u8)).collect();
    ore.push((c, ORE, 0));
    let mut w = game_with_ore(&r, &ore, 7);
    w.econ.resources.tiberiums[0].spread = Default::default();
    testkit::run(&mut w, 3);
    let t = i64::from(w.tick);
    let draws = w.rng.taken();
    resource::take_bales(&mut w, c, 1);
    assert_eq!(w.rng.taken() - draws, 8);
    let l = &w.econ.resources.tiberiums[0].spread;
    assert_eq!(l.entries.len(), 8);
    assert!(l.entries.iter().all(|e| e.key >= t && e.key < t + 50));
    // Taking order: keys never fall.
    assert!(
        l.entries
            .iter()
            .zip(l.entries.iter().skip(1))
            .all(|(a, b)| a.key <= b.key)
    );
}

// spec: S-0401 claim 9 and claim 11 (a spread step makes a germinable neighbour a stage-3 cell of the spreading tiberium)
#[test]
fn claim_9_spread_update() {
    let mut r = ruleset();
    fractions(&mut r, "ORE", "0", "1");
    let c = at(0, 0);
    let mut w = game_with_ore(&r, &[(c, ORE, 6)], 7);
    assert_eq!(spread_cells(&w, 0), vec![c]);
    testkit::run(&mut w, 1);
    let new: Vec<_> = neighbours(c)
        .into_iter()
        .filter(|&n| stage(&w, n).is_some())
        .collect();
    // n = 1: k = 5, m = 1 … 5; with key 0 the cell is taken again until its attempts run out.
    assert!((1..=5).contains(&new.len()));
    assert!(
        new.iter()
            .all(|&n| resource::resource_at(&w, n) == Some((0, 3)))
    );
    // c kept at least 2 germinable neighbours: it is added again and stays marked.
    assert!(w.econ.resources.tiberiums[0].spread.marks.contains(&c));
}

// spec: S-0401 claim 9 (quirks.spread_mark_kept_on_last_neighbour: a cell dropped with one germinable neighbour stays marked; off: unmarked)
#[test]
fn claim_9_quirk_spread_mark_kept_on_last_neighbour() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.spread_mark_kept_on_last_neighbour = on;
        fractions(&mut r, "ORE", "0", "1");
        let c = at(0, 0);
        let ns = neighbours(c);
        // Seven neighbours hold stage-0 resource (not germinable, not spreading); one is free.
        let mut ore: Vec<_> = ns[1..].iter().map(|&n| (n, ORE, 0u8)).collect();
        ore.push((c, ORE, 6));
        let mut w = game_with_ore(&r, &ore, 7);
        assert_eq!(spread_cells(&w, 0), vec![c]);
        testkit::run(&mut w, 1);
        assert_eq!(stage(&w, ns[0]), Some(3));
        assert!(spread_cells(&w, 0).is_empty());
        assert_eq!(w.econ.resources.tiberiums[0].spread.marks.contains(&c), on);
    }
}

// spec: S-0401 claim 9 (quirks.spread_attempts_count_failures: a failing cell with 2+ germinable neighbours comes back with key 0 and uses an attempt each time; off: it is dropped and counts none)
#[test]
fn claim_9_quirk_spread_attempts_count_failures() {
    for seed in 1..6 {
        let mut grown = Vec::new();
        for on in [true, false] {
            let mut r = ruleset();
            r.globals.quirks.spread_attempts_count_failures = on;
            fractions(&mut r, "ORE", "0", "1");
            let (a, b) = (at(-5, 0), at(5, 0));
            let mut w = game_with_ore(&r, &[(a, ORE, 6), (b, ORE, 6)], seed);
            assert_eq!(spread_cells(&w, 0), vec![a, b]);
            // A vehicle on a: a fails claim 8's conditions when it is taken.
            place_vehicle(&mut w, "HARV", 0, a, 0);
            testkit::run(&mut w, 1);
            grown.push(
                neighbours(b)
                    .into_iter()
                    .filter(|&n| stage(&w, n).is_some())
                    .count(),
            );
            assert_eq!(w.econ.resources.tiberiums[0].spread.marks.contains(&a), on);
        }
        // Same m in both runs: on, a and b alternate (a first), so b gets ⌊m/2⌋ attempts; off, b gets all m.
        assert!(grown[1] >= 1, "seed {seed}");
        assert_eq!(grown[0], grown[1] / 2, "seed {seed}");
    }
}

// spec: S-0401 claim 10 (germinable: usable, buildable land, no overlay, flat, no living building, the tileset allows it)
#[test]
fn claim_10_germinable_cells() {
    let mut r = ruleset();
    r.sim
        .land_types
        .get_mut(&glasnost_rules::sim::LandType::Road)
        .unwrap()
        .buildable = Some(false);
    let (slope, ore, hut) = (at(2, 0), at(4, 0), at(6, 0));
    let mut w = game_with_map(&r, &[(ore, ORE, 2)], 7, |m| {
        m.cells.get_mut(slope).unwrap().tile = 1;
    });
    place_building(&mut w, "HUT", 0, hut);
    assert!(resource::germinable(&w, at(0, 0)));
    assert!(!resource::germinable(&w, slope));
    assert!(!resource::germinable(&w, ore));
    assert!(!resource::germinable(&w, hut));
    // Road is not buildable in this ruleset; a cell of the road land type is not germinable.
    let mut w2 = game(&r);
    let c = at(0, 0);
    w2.cells.grid.get_mut(c).unwrap().land = land::ROAD;
    assert!(!resource::germinable(&w2, c));
}

// spec: S-0401 claim 11 (a new cell: overlay P + U(0, 11) on flat ground, stage 3, land type tiberium)
#[test]
fn claim_11_new_resource_cell() {
    let mut r = ruleset();
    fractions(&mut r, "GEM", "0", "1");
    let c = at(0, 0);
    let mut w = game_with_ore(&r, &[(c, GEM, 6)], 9);
    testkit::run(&mut w, 1);
    let n = neighbours(c)
        .into_iter()
        .find(|&n| stage(&w, n).is_some())
        .expect("a new cell");
    let cell = w.cells.cell(n).unwrap();
    assert!((GEM..GEM + 12).contains(&cell.overlay));
    assert_eq!(cell.overlay_data, 3);
    assert_eq!(cell.land, land::TIBERIUM);
}

// spec: S-0401 claim 12 (resource on a sloped cell is in neither list)
#[test]
fn claim_12_sloped_resource_never_grows_or_spreads() {
    let mut r = ruleset();
    fractions(&mut r, "ORE", "1", "1");
    let c = at(0, 0);
    let mut w = game_with_map(&r, &[(c, ORE + 12, 6)], 7, |m| {
        m.cells.get_mut(c).unwrap().tile = 1;
    });
    assert!(growth_cells(&w, 0).is_empty());
    assert!(spread_cells(&w, 0).is_empty());
    testkit::run(&mut w, 40);
    assert_eq!(stage(&w, c), Some(6));
}

// spec: S-0401 claim 13 and claim 14 (game-start rebuild: key 0 in the visiting order, E the number of entries; equal keys keep their order)
#[test]
fn claim_13_14_rebuild_at_game_start() {
    let mut r = ruleset();
    fractions(&mut r, "ORE", "1", "1");
    let cells = [at(3, 3), at(-4, 1), at(0, -2), at(5, -5)];
    let ore: Vec<_> = cells.iter().map(|&c| (c, ORE, 7u8)).collect();
    let w = game_with_ore(&r, &ore, 7);
    let order: Vec<_> = w
        .cells
        .grid
        .visiting_order(w.rules().quirks.narrow_map_partial_pass)
        .into_iter()
        .filter(|c| cells.contains(c))
        .collect();
    let g = &w.econ.resources.tiberiums[0].growth;
    assert_eq!(growth_cells(&w, 0), order);
    assert!(g.entries.iter().all(|e| e.key == 0));
    assert_eq!(g.count, 4);
    assert_eq!(spread_cells(&w, 0), order);
}

fn mine_rules(animated: bool, probability: &str) -> glasnost_rules::schema::Ruleset {
    let mut r = ruleset();
    r.registries
        .entry("terrain_objects".into())
        .or_default()
        .push("MINE".into());
    r.sim.terrain_objects.insert(
        "MINE".into(),
        TerrainType {
            spawns_tiberium: Some(true),
            is_animated: Some(animated),
            animation_rate: Some(3),
            animation_probability: dec(probability),
            image_frames: Some(4),
            ..Default::default()
        },
    );
    r
}

// spec: S-0401 claim 15 and F8 (a mine starts with probability p and, R × ⌊N/2⌋ ticks later, makes a stage-3 cell next to it)
#[test]
fn claim_15_f8_ore_mine() {
    let r = mine_rules(true, "1");
    let mut w = game(&r);
    let c = at(0, 0);
    let id = testkit::spawn(&mut w, Kind::Terrain, "MINE", 0, c, 0);
    testkit::run(&mut w, 1);
    let t0 = *w.econ.resources.mines.get(&id).expect("the mine started");
    // spec: S-0401 claim 15 (it stops in tick t0 + R × ⌊N/2⌋ = t0 + 6)
    while w.tick <= t0 + 5 {
        testkit::run(&mut w, 1);
        assert!(neighbours(c).iter().all(|&n| stage(&w, n).is_none()));
    }
    testkit::run(&mut w, 1);
    let made: Vec<_> = neighbours(c)
        .into_iter()
        .filter(|&n| stage(&w, n).is_some())
        .collect();
    assert_eq!(made.len(), 1);
    assert_eq!(resource::resource_at(&w, made[0]), Some((0, 3)));
}

// spec: S-0401 claim 15 (a mine without the animated flag, or with p = 0, never makes resource)
#[test]
fn claim_15_idle_mines() {
    for (animated, p) in [(false, "1"), (true, "0")] {
        let r = mine_rules(animated, p);
        let mut w = game(&r);
        let c = at(0, 0);
        testkit::spawn(&mut w, Kind::Terrain, "MINE", 0, c, 0);
        testkit::run(&mut w, 60);
        assert!(w.econ.resources.mines.is_empty());
        assert!(neighbours(c).iter().all(|&n| stage(&w, n).is_none()));
    }
}

// spec: S-0401 claim 16 (b) (taking every bale empties the cell)
#[test]
fn claim_16_take_all() {
    let r = ruleset();
    let c = at(0, 0);
    let mut w = game_with_ore(&r, &[(c, ORE, 7)], 7);
    resource::take_all(&mut w, c);
    assert_eq!(stage(&w, c), None);
}

// spec: S-0401 claim 16 (f) (quirks.thinning_hits_waypoint_only: 109 takes from the waypoint cell; off: one bale from each disc cell)
#[test]
fn claim_16f_quirk_thinning_hits_waypoint_only() {
    let wp = at(0, 0);
    let (near, far) = (at(5, 3), at(6, 0));
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.thinning_hits_waypoint_only = on;
        let mut w = game_with_ore(&r, &[(wp, ORE, 11), (near, ORE, 11), (far, ORE, 11)], 7);
        resource::thin_around_waypoint(&mut w, wp);
        if on {
            assert_eq!(stage(&w, wp), None);
            assert_eq!(stage(&w, near), Some(11));
        } else {
            assert_eq!(stage(&w, wp), Some(10));
            // 25 + 9 = 34 ≤ 35: inside the disc.
            assert_eq!(stage(&w, near), Some(10));
        }
        assert_eq!(stage(&w, far), Some(11));
    }
}

// spec: S-0401 claim 16 (a) and S-0303 claim 17 (a detonation takes trunc(D / 10) bales by claim 3's rule)
#[test]
fn claim_16a_detonation_takes_bales() {
    // The area-damage code calls take_bales(c, D / 10); its taking rule is claim 3's.
    let r = ruleset();
    let c = at(0, 0);
    let mut w = game_with_ore(&r, &[(c, ORE, 8)], 7);
    let damage = 45;
    assert_eq!(resource::take_bales(&mut w, c, damage / 10), 4);
    assert_eq!(stage(&w, c), Some(4));
}

// spec: S-0401 claim 17 (quirks.spread_stage_from_index: stage above ⌊i/2⌋ spreads; off: above 5)
#[test]
fn claim_17_quirk_spread_stage_from_index() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.spread_stage_from_index = on;
        fractions(&mut r, "ORE", "0", "1");
        let (low, high) = (at(0, 0), at(5, 0));
        let w = game_with_ore(&r, &[(low, ORE, 1), (high, ORE, 6)], 7);
        let cells = spread_cells(&w, 0);
        assert_eq!(cells.contains(&low), on);
        assert!(cells.contains(&high));
    }
}

// spec: S-0401 claim 18 (quirks.full_cell_no_regrowth: a harvested stage-11 cell is not offered to growth; off: it is)
#[test]
fn claim_18_quirk_full_cell_no_regrowth() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.full_cell_no_regrowth = on;
        fractions(&mut r, "ORE", "1", "0");
        let c = at(0, 0);
        let mut w = game_with_ore(&r, &[(c, ORE, 11)], 7);
        assert!(growth_cells(&w, 0).is_empty());
        resource::take_bales(&mut w, c, 1);
        assert_eq!(stage(&w, c), Some(10));
        assert_eq!(growth_cells(&w, 0).contains(&c), !on);
    }
}

// spec: S-0401 claim 19 (quirks.resource_timers_carry_over: the timers of the previous game carry over; off: they restart)
#[test]
fn claim_19_quirk_resource_timers_carry_over() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.resource_timers_carry_over = on;
        let mut first = game(&r);
        testkit::run(&mut first, 5);
        let previous = first.econ.resources.clone();
        let mut next = game(&r);
        resource::carry_over(&mut next, &previous);
        let expect = if on {
            previous.tiberiums[0].growth_last
        } else {
            None
        };
        assert_eq!(next.econ.resources.tiberiums[0].growth_last, expect);
    }
}

// spec: S-0401 F5 and F7 (a growth update takes at most k = clamp(trunc(n × g), 5, 50) entries; re-entries count in E)
#[test]
fn f5_growth_count_bounds() {
    let mut r = ruleset();
    fractions(&mut r, "ORE", "1", "0");
    let ore: Vec<_> = (0..20).map(|i| (at(i - 10, 4), ORE, 1u8)).collect();
    let mut w = game_with_ore(&r, &ore, 7);
    testkit::run(&mut w, 1);
    let grown = ore
        .iter()
        .filter(|&&(c, _, _)| stage(&w, c) == Some(2))
        .count();
    // k = clamp(trunc(20 × 1.0), 5, 50) = 20; m = (|r| mod 20) + 1.
    assert!((1..=20).contains(&grown));
    let g = &w.econ.resources.tiberiums[0].growth;
    assert_eq!(g.entries.len(), 20);
    assert_eq!(g.count, 20 + grown as i64);
}

// The resource state survives a save and load round trip.
#[test]
fn resource_state_round_trip() {
    let mut r = ruleset();
    fractions(&mut r, "ORE", "1", "1");
    let mut w = game_with_ore(&r, &[(at(0, 0), ORE, 6), (at(3, 3), ORE, 9)], 7);
    testkit::run(&mut w, 40);
    let bytes = glasnost_sim::save::save(&w);
    let back = glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(back.econ.resources, w.econ.resources);
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
}

// spec: S-0303 claim 17 (revision 5: a cell loses trunc(D / 10) bales only when its resource overlay's type has the chain-reaction flag and the warhead has the resource flag; either missing: no loss)
#[test]
fn s0303_claim_17_resource_loss_needs_both_flags() {
    for (chain, flag, want) in [(true, true, 4), (false, true, 8), (true, false, 8)] {
        let mut r = ruleset();
        r.sim.overlays.get_mut("OV00").unwrap().chain_reaction = Some(chain);
        r.sim.warheads.insert(
            "ORESHOT".into(),
            glasnost_rules::sim::WarheadType {
                tiberium: Some(flag),
                ..Default::default()
            },
        );
        let c = at(0, 0);
        let mut w = game_with_ore(&r, &[(c, ORE, 8)], 7);
        let h = w
            .rules()
            .combat
            .warheads
            .iter()
            .position(|x| x.name == "ORESHOT")
            .unwrap() as u32;
        glasnost_sim::combat::area::detonate(
            &mut w,
            glasnost_sim::combat::area::Detonation {
                point: glasnost_sim::world::Pos::centre(c, 0),
                damage: 45,
                warhead: Some(h),
                source: None,
                house: None,
                resources: true,
            },
        );
        assert_eq!(stage(&w, c), Some(want), "chain {chain}, flag {flag}");
    }
}
