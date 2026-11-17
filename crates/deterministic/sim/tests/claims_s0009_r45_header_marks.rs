//! S-0009 r45 claim 29, the early phase: header pairs beyond the eighth act
//! as start position marks (#368). Synthetic maps only; the test shapes are
//! those of scenario M13b. The late phase is in
//! `claims_s0009_r45_header_marks_late.rs`, the References rule in
//! `claims_s0009_r45_header_marks_references.rs`.

mod common;

use common::{Scene, TRACK, unit_type};
use glasnost_map::{CellXY, Header, Unit};
use glasnost_sim::world::World;

/// Start waypoint markers 0 … 3 whose pairwise distances all differ
/// (scenario M13b): 0–1 is 10, 0–2 30, 0–3 21, 1–2 20, 1–3 16 and 2–3 22
/// cells (truncated). From positions 0 and 1 the distance sums are 50 for
/// position 2 and 37 for position 3, so W* is position 2.
const WAYPOINTS: [(i16, i16); 4] = [(30, 40), (40, 40), (60, 40), (44, 56)];

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
/// and whose pairs from the ninth on are `marks` (pair 9 first; `None` for
/// a pair the map does not hold, a `None` component for one that cannot be
/// read), as the importer gives it.
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

/// A present pair with two readable numbers.
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

/// The cell of start waypoint marker `k`.
fn wp(k: usize) -> CellXY {
    let (x, y) = WAYPOINTS[k];
    CellXY::new(i32::from(x), i32::from(y))
}

/// House `h`'s base cell (S-0009 claim 20: its position's entry in list 3;
/// S-0215 claim 38's base-start cell).
fn base(w: &World, h: usize) -> Option<CellXY> {
    w.houses[h].base.start
}

/// spec: S-0009 claim 29 (early phase: an applied pair's marks place the houses it names, with no random-position draw; scenario M13b variant A)
#[test]
fn variant_a_pair_9_places_both_players() {
    for seed in 1..=5 {
        let mut s = scene();
        s.setup.seed = seed;
        s.map.header = header(9, &[pair(1, 0)]);
        let w = load(&s);
        assert_eq!(starts(&w), [Some(1), Some(0)], "seed {seed}");
        assert_eq!((base(&w, 0), base(&w, 1)), (Some(wp(1)), Some(wp(0))));
        // The same game with the two positions chosen: no position draw either.
        let mut control = scene();
        control.setup.seed = seed;
        control.setup.houses[0].start = Some(1);
        control.setup.houses[1].start = Some(0);
        let c = load(&control);
        assert_eq!(starts(&c), starts(&w));
        assert_eq!(w.rng, c.rng, "seed {seed}: no random-position draw");
    }
}

/// spec: S-0009 claims 25 and 29 (a pair numbered above N_h is not applied, and an absent pair leaves its marks; scenario M13b variant C)
#[test]
fn variant_c_pairs_above_the_count_are_not_applied() {
    for seed in 1..=5 {
        let mut plain = scene();
        plain.setup.seed = seed;
        let p = load(&plain);
        let mut s = scene();
        s.setup.seed = seed;
        // N_h = 9: pair 9 absent; pair 10 = (1, 0) is held but above N_h.
        s.map.header = header(9, &[None, pair(1, 0)]);
        let w = load(&s);
        assert_eq!(starts(&w), starts(&p), "seed {seed}");
        assert_eq!(w.rng, p.rng, "seed {seed}");
    }
}

/// spec: S-0009 claim 29 (pair 10 sets the marks of positions 2 and 3; scenario M13b variant D)
#[test]
fn variant_d_pair_10_marks_positions_2_and_3() {
    for seed in 1..=5 {
        let mut s = scene();
        s.setup.seed = seed;
        s.map.header = header(10, &[None, pair(1, 0)]);
        let w = load(&s);
        assert_eq!(starts(&w), [Some(3), Some(2)], "seed {seed}");
        assert_eq!((base(&w, 0), base(&w, 1)), (Some(wp(3)), Some(wp(2))));
    }
}

/// spec: S-0009 claim 29 (a house holding several positions starts at the highest-numbered, and every marked position counts as taken for the farthest-sum rule; scenario M13b variant E)
#[test]
fn variant_e_the_highest_position_held() {
    for seed in 1..=3 {
        let mut s = scene();
        s.setup.seed = seed;
        s.map.header = header(9, &[pair(0, 0)]);
        let w = load(&s);
        // House 0 holds positions 0 and 1 and starts at 1; house 1 at W* (position 2).
        assert_eq!(starts(&w), [Some(1), Some(2)], "seed {seed}");
        assert_eq!((base(&w, 0), base(&w, 1)), (Some(wp(1)), Some(wp(2))));
    }
}

/// spec: S-0009 claim 29 (a mark that names no house: the position is taken by no player; scenario M13b variant F)
#[test]
fn variant_f_a_mark_that_names_no_house() {
    for seed in 1..=3 {
        let mut s = scene();
        s.setup.seed = seed;
        s.map.header = header(9, &[pair(5, 5)]);
        let w = load(&s);
        // Positions 0 and 1 are taken: house 0 gets W* (2), house 1 the other (3).
        assert_eq!(starts(&w), [Some(2), Some(3)], "seed {seed}");
        assert_eq!((base(&w, 0), base(&w, 1)), (Some(wp(2)), Some(wp(3))));
    }
}

/// spec: S-0009 claim 29 (early phase: a chosen position's mark is its chooser's number whatever a pair says, and a house takes the highest-numbered position it holds)
#[test]
fn a_chooser_wins_over_the_pair_early() {
    let mut s = scene();
    s.setup.houses[0].start = Some(0);
    s.map.header = header(9, &[pair(1, 0)]);
    let w = load(&s);
    // Position 0 is house 0's (its choice), position 1 house 0's (the pair):
    // house 0 starts at 1, and house 1 takes W* (2).
    assert_eq!(starts(&w), [Some(1), Some(2)]);
}

/// spec: S-0009 claim 29 and Import mapping (early phase: an unread component keeps the mark as it stands, −1; S-0003 claim 9, a departure)
#[test]
fn an_unread_component_keeps_the_free_mark_early() {
    // Pair 9 = (1, unread): position 0 is house 1's, position 1 stays free.
    let mut s = scene();
    s.map.header = header(9, &[Some([Some(1), None])]);
    let w = load(&s);
    // House 0 is the farthest from position 0: position 2.
    assert_eq!(starts(&w), [Some(2), Some(0)]);
}

/// spec: S-0009 claim 29 (a mark at a position numbered N or above is ignored, a departure: pairs 11 and 13 mark positions 4, 5, 8 and 9 of a four-position game; our engine notes it)
#[test]
fn marks_at_positions_n_and_above_are_ignored() {
    for seed in 1..=5 {
        let mut plain = scene();
        plain.setup.seed = seed;
        let p = load(&plain);
        let mut s = scene();
        s.setup.seed = seed;
        s.map.header = header(16, &[None, None, pair(0, 1), None, pair(1, 0)]);
        let w = load(&s);
        assert_eq!(starts(&w), starts(&p), "seed {seed}");
        assert_eq!(w.rng, p.rng, "seed {seed}");
        assert!(
            w.notes
                .iter()
                .any(|n| n.contains("S-0009 claim 29") && n.contains("N or above")),
            "{:?}",
            w.notes
        );
    }
}

/// A scene whose houses each receive a base unit (S-0009 claim 45).
fn base_unit_scene() -> Scene {
    let mut s = scene();
    let mut t = unit_type(TRACK, 4, 5);
    t.cost = Some(500);
    t.tech_level = Some(1);
    t.owner = Some(vec!["Alphaland".into(), "Betaland".into()]);
    s.vehicle("MCV", t);
    s.ruleset.sim.general.base_unit = Some(vec!["MCV".into()]);
    s.setup.bases = true;
    s
}

/// spec: S-0009 claim 29 (a house that needs a position by the farthest-sum rule when every position below N is taken gets no start position and no starting units, a departure; our engine notes it) and claim 27 with quirks.skirmish_drops_country_owned_objects off (an object of "player at position A" is dropped when no house is there)
#[test]
fn every_position_taken_gives_no_start() {
    let mut s = base_unit_scene();
    s.map.waypoints.remove(&2);
    s.map.waypoints.remove(&3);
    s.ruleset
        .globals
        .quirks
        .skirmish_drops_country_owned_objects = false;
    s.map.objects.units.push(Unit {
        placed: glasnost_map::objects::Placed {
            key: b"k".to_vec(),
            owner: b"<Player @ A>".to_vec(),
            type_name: b"TANK".to_vec(),
            health: 256,
            cell: common::cell(-6, 0),
            facing: 0,
            raw: Vec::new(),
        },
    });
    let control = load(&s);
    assert!(control.houses.iter().all(|h| h.start.is_some()));
    assert!(!control.objects.is_empty(), "the object goes to the player at A");
    // N = 2, and pair 9 = (5, 5) takes both positions for no house.
    s.map.header = header(9, &[pair(5, 5)]);
    let w = load(&s);
    assert_eq!(starts(&w), [None, None]);
    assert_eq!((base(&w, 0), base(&w, 1)), (None, None));
    assert!(w.objects.is_empty(), "no starting units, and the object is dropped");
    assert!(
        w.notes
            .iter()
            .any(|n| n.contains("S-0009 claim 29") && n.contains("no start position")),
        "{:?}",
        w.notes
    );
}

/// spec: S-0009 claims 19 and 20 with quirks.header_pairs_as_start_marks off (Edge cases: the importer reads at most 8 pairs, so no mark applies)
#[test]
fn eight_pairs_read_change_nothing() {
    for seed in 1..=5 {
        let mut plain = scene();
        plain.setup.seed = seed;
        let p = load(&plain);
        let mut s = scene();
        s.setup.seed = seed;
        s.map.header = header(12, &[]);
        let w = load(&s);
        assert_eq!(starts(&w), starts(&p), "seed {seed}");
        assert_eq!(w.rng, p.rng, "seed {seed}");
    }
}
