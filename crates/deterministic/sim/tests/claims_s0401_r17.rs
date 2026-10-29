//! Claim tests for S-0401 revision 17 (a cell holding a terrain object,
//! `quirks.spread_step_lost_on_terrain_object`) and the facts of M-0013
//! (which cells receive spreading ore, scenario E8), on synthetic data:
//! invented tiberiums, overlays, terrain types, periods and seeds, no stock
//! data and none of the record's numbers.

mod harvest_common;

use glasnost_map::terrain::land;
use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{OverlayType, TerrainObjectType};
use glasnost_sim::econ::resource;
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::World;
use harvest_common::*;

/// Overlay list position of WALL, a wall overlay after the 40 resource
/// overlays of the shared ruleset.
const WALL: u8 = 40;

/// The neighbours of `c` in step-direction order.
fn neighbours(c: CellXY) -> Vec<CellXY> {
    glasnost_sim::regions::STEPS
        .iter()
        .map(|&(dx, dy)| CellXY::new(c.x + dx, c.y + dy))
        .collect()
}

/// The shared ruleset with a WALL overlay, a TREE terrain type, a MINE
/// (an animated ore mine that starts at once), an IDLEMINE (an ore mine
/// without the animated flag) and ORE spreading every `s`
/// ticks with spread fraction 1 and no growth; the switch at `on`.
fn rules(on: bool, s: i32) -> Ruleset {
    let mut r = ruleset();
    r.globals.quirks.spread_step_lost_on_terrain_object = on;
    fractions(&mut r, "ORE", "0", "1");
    r.sim.tiberiums.get_mut("ORE").unwrap().spread = Some(s);
    r.registries
        .entry("overlays".into())
        .or_default()
        .push("WALL".into());
    r.sim.overlays.insert(
        "WALL".into(),
        OverlayType {
            wall: Some(true),
            strength: Some(100),
            ..Default::default()
        },
    );
    let terrain = r.registries.entry("terrain_objects".into()).or_default();
    terrain.push("TREE".into());
    terrain.push("MINE".into());
    terrain.push("IDLEMINE".into());
    r.sim.terrain_objects.insert(
        "TREE".into(),
        TerrainObjectType {
            strength: Some(100),
            ..Default::default()
        },
    );
    r.sim.terrain_objects.insert(
        "IDLEMINE".into(),
        TerrainObjectType {
            spawns_tiberium: Some(true),
            ..Default::default()
        },
    );
    r.sim.terrain_objects.insert(
        "MINE".into(),
        TerrainObjectType {
            spawns_tiberium: Some(true),
            is_animated: Some(true),
            animation_rate: Some(2),
            animation_probability: dec("1"),
            image_frames: Some(4),
            ..Default::default()
        },
    );
    r
}

/// A game whose map has a full-grown ORE cell (stage 11) at each of
/// `centres`, and a wall in every neighbour of a centre except the cells of
/// `open`.
fn pens(r: &Ruleset, centres: &[CellXY], open: &[CellXY], seed: u32) -> World {
    let ore: Vec<_> = centres.iter().map(|&c| (c, ORE, 11u8)).collect();
    game_with_map(r, &ore, seed, |m| {
        for &c in centres {
            for n in neighbours(c) {
                if !open.contains(&n) {
                    m.cells.get_mut(n).unwrap().overlay = WALL;
                }
            }
        }
    })
}

fn tree(w: &mut World, c: CellXY) {
    testkit::spawn(w, Kind::Terrain, "TREE", 0, c, 0);
}

fn growth_has(w: &World, c: CellXY) -> bool {
    w.econ.resources.tiberiums[0]
        .growth
        .entries
        .iter()
        .any(|e| e.cell == c)
}

/// What the probe expects of one spread update of a lone spreading cell
/// whose open neighbours are a tree in direction `x` and a free cell in
/// direction `y` (S-0401 claims 9 and 11, F6): the draws, and whether the
/// free cell is filled and whether the tree's cell was chosen.
fn expect_update(p: &mut Rng, half: bool, on: bool, x: u8, y: u8) -> (bool, bool) {
    // spec: S-0401 F6 (one raw draw; n = 1 and s = 1: k clamped to 5, m = (|r| mod 5) + 1)
    let r = p.raw() as i32;
    let m = i64::from(r).abs() % 5 + 1;
    let (mut filled, mut tree_chosen) = (false, false);
    for _ in 0..m {
        // spec: S-0401 claims 9 and 10 (c: the tree's cell counts only with the switch on)
        let c = u8::from(on) + u8::from(!filled);
        if c == 0 {
            break;
        }
        // spec: S-0401 claim 9 (one ranged draw U(0, 7), then the first germinable neighbour from j)
        let j = p.ranged(0, 7, half) as u8;
        let first = (0..8)
            .map(|k| (j + k) % 8)
            .find(|&d| (on && d == x) || (!filled && d == y))
            .unwrap();
        // spec: S-0401 claim 11 (the overlay draw U(0, 11), then the growth-list offer's raw draw: the cell's stage is 0, below 11)
        p.ranged(0, 11, half);
        p.raw();
        if first == y {
            filled = true;
        } else {
            tree_chosen = true;
        }
        // spec: S-0401 claim 9 (c = 1: the entry is not put back)
        if c == 1 {
            break;
        }
    }
    (filled, tree_chosen)
}

// spec: S-0401 claims 9, 10 and 11 (scenario E8 part b; quirks.spread_step_lost_on_terrain_object on: the tree's cell counts in c, a step whose scan reaches it first takes the overlay draw and the growth-list offer and makes nothing, using its attempt; the cell stays germinable)
// spec: S-0401 claim 10 (switch off: the tree's cell is not germinable, c counts only the free cell and every step's scan passes over the tree to it)
#[test]
fn claim_11_tree_cell_chosen_receives_nothing() {
    for on in [true, false] {
        let mut seen_tree = false;
        for seed in 1..=24 {
            let r = rules(on, 10);
            let c = at(0, 0);
            let ns = neighbours(c);
            let (x, y) = (ns[1], ns[5]);
            let mut w = pens(&r, &[c], &[x, y], seed);
            tree(&mut w, x);
            assert_eq!(resource::germinable(&w, x), on, "seed {seed}");
            assert!(resource::germinable(&w, y));
            let half = w.rules().quirks.ranged_draw_half_span;
            let mut p = w.rng.clone();
            let (filled, chosen) = expect_update(&mut p, half, on, 1, 5);
            resource::part(&mut w);
            assert_eq!(w.rng, p, "on {on} seed {seed}: the update's draws");
            assert_eq!(stage(&w, y).is_some(), filled, "on {on} seed {seed}");
            if filled {
                assert_eq!(resource::resource_at(&w, y), Some((0, 3)));
            }
            // spec: S-0401 claim 11 (no overlay, no stage, still germinable; its growth entry was added)
            let cell = w.cells.cell(x).unwrap();
            assert_eq!(cell.overlay, NO_OVERLAY);
            assert_eq!(cell.overlay_data, 0);
            assert_eq!(resource::germinable(&w, x), on);
            assert_eq!(growth_has(&w, x), chosen);
            assert!(on || !chosen, "off: the tree's cell is never chosen");
            seen_tree |= chosen;
        }
        assert_eq!(
            seen_tree, on,
            "on {on}: some seed's scan reaches the tree first"
        );
    }
}

// spec: S-0401 claims 7 and 11 (the growth entry of a tree's cell is dropped when taken while it holds no resource: no re-entry, no draw)
#[test]
fn claim_11_tree_cell_growth_entry_dropped() {
    let mut r = rules(true, 10);
    fractions(&mut r, "ORE", "1", "1");
    let c = at(0, 0);
    let x = neighbours(c)[1];
    let mut w = pens(&r, &[c], &[x], 3);
    tree(&mut w, x);
    // c = 1: the one attempt chooses the tree's cell.
    testkit::run(&mut w, 1);
    assert!(growth_has(&w, x));
    assert_eq!(w.econ.resources.tiberiums[0].growth.entries.len(), 1);
    let p = resource::growth_period(&w, 0);
    testkit::run(&mut w, p);
    assert!(!growth_has(&w, x));
    assert_eq!(w.cells.cell(x).unwrap().overlay, NO_OVERLAY);
}

// spec: S-0401 claims 11 and 15 (an ore mine's step that chooses a tree's cell makes nothing: the overlay draw and the growth-list offer are taken; switch off: no germinable neighbour, only the U(0, 7) draw; the mine never fills its own cell or a wall)
#[test]
fn claim_15_ore_mine_next_to_a_tree() {
    let mut taken = Vec::new();
    for on in [true, false] {
        let r = rules(on, 10);
        let c = at(0, 0);
        let x = neighbours(c)[3];
        let mut w = pens(&r, &[], &[], 5);
        for n in neighbours(c) {
            if n != x {
                w.cells.grid.get_mut(n).unwrap().overlay = WALL;
            }
        }
        tree(&mut w, x);
        let id = testkit::spawn(&mut w, Kind::Terrain, "MINE", 0, c, 0);
        testkit::run(&mut w, 1);
        assert!(w.econ.resources.mines.contains_key(&id), "the mine started");
        while w.econ.resources.mines.contains_key(&id) {
            testkit::run(&mut w, 1);
        }
        taken.push(w.rng.taken());
        assert_eq!(w.cells.cell(x).unwrap().overlay, NO_OVERLAY);
        assert_eq!(w.cells.cell(c).unwrap().overlay, NO_OVERLAY);
        assert!(
            neighbours(c)
                .iter()
                .filter(|&&n| n != x)
                .all(|&n| w.cells.cell(n).unwrap().overlay == WALL)
        );
        assert_eq!(growth_has(&w, x), on);
    }
    // spec: S-0401 claim 11 (on: the overlay draw U(0, 11) and the growth-list raw draw more than off)
    assert_eq!(taken[0], taken[1] + 2);
}

// spec: S-0401 claim 11 and S-0520 claim 24 (a crate's stage increase at a germinable tree's cell takes claim 11's draws and places nothing; switch off: the cell is not germinable and nothing is drawn)
#[test]
fn claim_11_crate_stage_at_a_tree_cell() {
    for on in [true, false] {
        let r = rules(on, 10);
        let mut w = game(&r);
        let x = at(2, 2);
        tree(&mut w, x);
        let before = w.rng.taken();
        resource::crate_stage(&mut w, x, 0);
        assert_eq!(w.rng.taken() - before, if on { 2 } else { 0 });
        assert_eq!(w.cells.cell(x).unwrap().overlay, NO_OVERLAY);
    }
}

// measure: M-0013 (scenario E8 part a: free and infantry cells receive spreading ore; tree, living building, ore mine, high-bridge, rock and wall cells never do; every new cell appears at stage 3; spread gains fall on tick 0 and multiples of the spread field)
// spec: S-0401 claims 6, 9, 10, 11 and 19
#[test]
fn m0013_which_cells_receive_spreading_ore() {
    // Our own field (ticks) and run length, not the record's.
    let s = 7;
    for on in [true, false] {
        for seed in 1..=5 {
            let r = rules(on, s);
            let centres: Vec<CellXY> = (0..7).map(|i| at(-10 + 3 * i, 0)).collect();
            // The open cell of each pen: direction 2 of its centre.
            let open: Vec<CellXY> = centres.iter().map(|&c| neighbours(c)[2]).collect();
            let [free, walker, treed, hut, mine, deck, rock] =
                <[CellXY; 7]>::try_from(open.clone()).unwrap();
            let mut w = pens(&r, &centres, &open, seed);
            testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, walker, 0);
            tree(&mut w, treed);
            place_building(&mut w, "HUT", 1, hut);
            testkit::spawn(&mut w, Kind::Terrain, "IDLEMINE", 0, mine, 0);
            let i = w.cells.idx(deck).unwrap();
            w.cells.deck[i] = true;
            w.cells.grid.get_mut(rock).unwrap().land = land::ROCK;
            let walls: Vec<CellXY> = centres
                .iter()
                .flat_map(|&c| neighbours(c))
                .filter(|n| !open.contains(n))
                .collect();
            assert_eq!(walls.len(), 49);
            assert_eq!(w.tick, 0, "the first game's first tick is tick 0");
            let mut gains: Vec<(CellXY, i32, u8)> = Vec::new();
            for _ in 0..(20 * s) {
                let t = w.tick;
                testkit::run(&mut w, 1);
                for &n in &open {
                    if let Some(d) = stage(&w, n)
                        && !gains.iter().any(|g| g.0 == n)
                    {
                        gains.push((n, t, d));
                    }
                }
            }
            let got: Vec<CellXY> = gains.iter().map(|g| g.0).collect();
            assert!(got.contains(&free), "on {on} seed {seed}: the free cell");
            assert!(
                got.contains(&walker),
                "on {on} seed {seed}: the infantry cell"
            );
            for n in [treed, hut, mine, deck, rock] {
                assert!(!got.contains(&n), "on {on} seed {seed}: {n:?}");
            }
            assert!(
                walls
                    .iter()
                    .all(|&n| w.cells.cell(n).unwrap().overlay == WALL)
            );
            for &(n, t, d) in &gains {
                assert_eq!(d, 3, "on {on} seed {seed}: {n:?} at stage 3");
                assert_eq!(
                    t % s,
                    0,
                    "on {on} seed {seed}: {n:?} on the spread schedule"
                );
            }
        }
    }
}
