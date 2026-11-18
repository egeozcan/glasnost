//! S-0009 r45 claim 29, the late phase: the header pairs are applied again
//! over the early-phase marks, and a house that holds no position then is
//! assigned anew against list 3 (#368). Synthetic maps only; the test
//! shapes are those of scenario M13b. The scene matches
//! `claims_s0009_r45_header_marks.rs`.

mod common;

use common::{Scene, TRACK, unit_type};
use glasnost_map::{CellXY, Header};
use glasnost_sim::world::{HouseRole, HouseSetup, World};

/// Start waypoint markers 0 … 3 whose pairwise distances all differ
/// (scenario M13b): 0–1 is 10, 0–2 30, 0–3 21, 1–2 20, 1–3 16 and 2–3 22
/// cells (truncated).
const WAYPOINTS: [(i16, i16); 4] = [(30, 40), (40, 40), (60, 40), (44, 56)];

/// For each position p, the free position farthest from p alone (S-0009
/// claim 19 on [`WAYPOINTS`]).
const FARTHEST_FROM: [u32; 4] = [2, 2, 0, 2];

/// A skirmish of the two human players Alpha (house 0) and Beta (house 1)
/// on "random", with the four start waypoint markers and no header block.
fn scene() -> Scene {
    let mut s = Scene::new();
    s.vehicle("TANK", unit_type(TRACK, 4, 5));
    s.setup.sandbox = false;
    for (k, &c) in WAYPOINTS.iter().enumerate() {
        s.map.waypoints.insert(k as u16, c);
    }
    s
}

/// A header block with marker count `count` whose pairs 1 … 8 are (0, 0)
/// and whose pairs from the ninth on are `marks`.
fn header(count: i32, marks: &[Option<[Option<i32>; 2]>]) -> Header {
    let mut pairs: Vec<Option<[Option<i32>; 2]>> = vec![Some([Some(0), Some(0)]); 8];
    pairs.extend_from_slice(marks);
    Header {
        count,
        origin: (0, 0),
        extent: (10, 10),
        pairs,
    }
}

fn pair(a: i32, b: i32) -> Option<[Option<i32>; 2]> {
    Some([Some(a), Some(b)])
}

fn load(s: &Scene) -> World {
    glasnost_sim::load::new_game(
        s.sim_rules(),
        &s.map,
        std::sync::Arc::new(s.catalogue.clone()),
        &s.setup,
    )
    .expect("loads")
}

fn starts(w: &World) -> Vec<Option<u32>> {
    w.houses.iter().map(|h| h.start).collect()
}

fn wp(k: usize) -> CellXY {
    let (x, y) = WAYPOINTS[k];
    CellXY::new(i32::from(x), i32::from(y))
}

fn base(w: &World, h: usize) -> Option<CellXY> {
    w.houses[h].base.start
}

/// spec: S-0009 claim 29 (late phase: an applied pair of (−1, −1) frees two chosen positions; the first house is assigned anew by a random-position draw, the second by the farthest-sum rule against list 3; scenario M13b variant B and its control)
#[test]
fn variant_b_pair_9_frees_chosen_positions_in_the_late_phase() {
    let mut moved = false;
    for seed in 1..=8 {
        let mut s = scene();
        s.setup.seed = seed;
        s.setup.houses[0].start = Some(0);
        s.setup.houses[1].start = Some(1);
        let mut control = scene();
        control.setup = s.setup.clone();
        // Control: N_h = 8, so no pair beyond the eighth.
        control.map.header = header(8, &[]);
        let c = load(&control);
        assert_eq!(starts(&c), [Some(0), Some(1)]);
        s.map.header = header(9, &[pair(-1, -1)]);
        let w = load(&s);
        let first = w.houses[0].start.expect("assigned anew");
        let second = w.houses[1].start.expect("assigned anew");
        assert_eq!(second, FARTHEST_FROM[first as usize], "seed {seed}");
        assert_eq!(base(&w, 0), Some(wp(first as usize)));
        // One ranged draw R(0, 3), one raw value, more than the control.
        assert_eq!(w.rng.taken(), c.rng.taken() + 1, "seed {seed}");
        moved |= first != 0;
    }
    assert!(
        moved,
        "the late draw places the first house elsewhere than its choice"
    );
}

/// spec: S-0009 claim 29 and Import mapping (late phase: an unread component keeps the early-phase mark, so the two phases complete it differently; S-0003 claim 9, a departure)
#[test]
fn an_unread_component_keeps_the_early_mark_late() {
    let mut s = scene();
    s.setup.houses[0].start = Some(0);
    s.setup.houses[1].start = Some(1);
    let c = load(&s);
    // Pair 9 = (unread, −1) over chosen positions 0 and 1. Position 0 keeps its
    // early mark (house 0), position 1 is freed, so house 1 is assigned anew by
    // the farthest-sum rule (position 2), with no draw.
    s.map.header = header(9, &[Some([None, Some(-1)])]);
    let w = load(&s);
    assert_eq!(starts(&w), [Some(0), Some(FARTHEST_FROM[0])]);
    assert_eq!(base(&w, 1), Some(wp(2)));
    assert_eq!(w.rng, c.rng, "no random-position draw");
}

/// Adds the neutral house (house 2) with the given passive fields: the
/// early phase's (claim 33, the base rules') and the late phase's.
fn with_neutral(s: &mut Scene, early_passive: bool, late_passive: bool) {
    s.register("countries", "Neutral");
    s.ruleset.sim.countries.insert(
        "Neutral".into(),
        glasnost_rules::sim::Country {
            multiplay_passive: Some(late_passive),
            multiplay_passive_base: Some(early_passive),
            ..Default::default()
        },
    );
    s.setup.houses.push(HouseSetup {
        name: "Neutral".into(),
        country: "Neutral".into(),
        human: false,
        start: None,
        role: HouseRole::Neutral,
        ai: false,
    });
}

/// spec: S-0009 claims 28, 29 and 33 (a house the early phase saw as passive holds a position by a mark in the late phase, where it is not passive, and starts there with no assignment)
#[test]
fn a_house_passive_early_holds_its_mark_late() {
    let mut s = scene();
    with_neutral(&mut s, true, false);
    // An AI house is a computer player, so the cleanup does not run (claim 28).
    s.setup.houses[0].human = false;
    s.setup.houses[0].ai = true;
    s.map.header = header(9, &[pair(2, -1)]);
    let w = load(&s);
    // Early: position 0 is taken by no player (the neutral house is passive then):
    // house 0 takes 2, house 1 takes 3. Late: the neutral house holds 0.
    assert_eq!(starts(&w), [Some(2), Some(3), Some(0)]);
    assert_eq!(base(&w, 2), Some(wp(0)));
}

/// spec: S-0009 claims 28 and 29 and Edge cases (quirks.cleanup_shifts_start_marks off, the fixed behaviour that Q-3213's answer makes of S27's interim rule: the cleanup frees a removed house's marks, so the position it held is free in the late phase; the switch's on position is in claims_s0009_r47_cleanup_marks.rs)
#[test]
fn a_removed_house_frees_its_position() {
    let mut s = scene();
    s.ruleset.globals.quirks.cleanup_shifts_start_marks = false;
    with_neutral(&mut s, false, false);
    // No computer player: the cleanup removes the neutral house (claim 28).
    // Pair 9 = (−1, −1) over chosen positions 0 and 1 frees both late; the
    // neutral house held position 2 early (farthest from 0 and 1).
    s.setup.houses[0].start = Some(0);
    s.setup.houses[1].start = Some(1);
    s.map.header = header(9, &[pair(-1, -1)]);
    for seed in 1..=8 {
        s.setup.seed = seed;
        let w = load(&s);
        assert!(w.houses[2].removed);
        assert_eq!(w.houses[2].start, None);
        let first = w.houses[0].start.expect("assigned anew");
        // Only house 0's position is taken when house 1's turn comes: the
        // removed house's position 2 is free again.
        assert_eq!(
            w.houses[1].start,
            Some(FARTHEST_FROM[first as usize]),
            "seed {seed}"
        );
    }
}

/// spec: S-0009 claim 29 with quirks.double_start_resolution off (Q-3214's interim rule: one resolution by the early-phase rule against the one list)
#[test]
fn one_resolution_applies_the_pairs_once() {
    for seed in 1..=5 {
        let mut s = scene();
        s.setup.seed = seed;
        s.ruleset.globals.quirks.double_start_resolution = false;
        s.map.header = header(9, &[pair(1, 0)]);
        let w = load(&s);
        assert_eq!(starts(&w), [Some(1), Some(0)], "seed {seed}");
    }
    let mut s = scene();
    s.ruleset.globals.quirks.double_start_resolution = false;
    s.setup.houses[0].start = Some(0);
    s.map.header = header(9, &[pair(1, 0)]);
    let w = load(&s);
    // The chooser wins position 0 and holds 1 by the pair; house 1 takes W*.
    assert_eq!(starts(&w), [Some(1), Some(2)]);
}
