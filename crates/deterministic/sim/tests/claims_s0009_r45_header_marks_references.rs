//! S-0009 r45 claim 29, References: with a header count of 9 or more,
//! "player at position X" is the house whose number is mark X (#368).
//! Synthetic maps only; the test shapes are those of scenario M13b. The
//! scene matches `claims_s0009_r45_header_marks.rs`.

mod common;

use common::{Scene, TRACK, unit_type};
use glasnost_map::Header;
use glasnost_sim::mp_triggers::resolve_house;
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, World};

/// "Player at position A"; B … H follow it.
const POSITION_A: i32 = 4475; // spec: S-0011 claim 23 (4475 … 4482: the player at position A … H)

/// Start waypoint markers 0 … 3 whose pairwise distances all differ
/// (scenario M13b).
const WAYPOINTS: [(i16, i16); 4] = [(30, 40), (40, 40), (60, 40), (44, 56)];

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

/// "Player at position" `k` (0 for A).
fn at(w: &World, k: i32) -> Option<HouseId> {
    resolve_house(w, POSITION_A + k, None)
}

/// spec: S-0009 claim 29 (References: the house whose number is mark X, not the house that starts there; scenario M13b variant E) and S-0011 claim 23
#[test]
fn variant_e_references_follow_the_marks() {
    let mut s = scene();
    s.map.header = header(9, &[pair(0, 0)]);
    let w = load(&s);
    assert_eq!(starts(&w), [Some(1), Some(2)]);
    // House 0 starts at position 1 but holds position 0 too.
    assert_eq!(at(&w, 0), Some(HouseId(0)));
    assert_eq!(at(&w, 1), Some(HouseId(0)));
    assert_eq!(at(&w, 2), Some(HouseId(1)));
    assert_eq!(at(&w, 3), None);
}

/// spec: S-0009 claim 29 (References: a mark that names no house gives an empty reference; scenario M13b variant F) and S-0011 claim 23
#[test]
fn variant_f_a_mark_that_names_no_house_is_empty() {
    let mut s = scene();
    s.map.header = header(9, &[pair(5, 5)]);
    let w = load(&s);
    assert_eq!(starts(&w), [Some(2), Some(3)]);
    assert_eq!(at(&w, 0), None);
    assert_eq!(at(&w, 1), None);
    assert_eq!(at(&w, 2), Some(HouseId(0)));
    assert_eq!(at(&w, 3), Some(HouseId(1)));
}

/// spec: S-0009 claim 29 (References read the late-phase marks: the pair wins over the early chooser)
#[test]
fn references_read_the_late_marks() {
    let mut s = scene();
    s.setup.houses[0].start = Some(0);
    s.map.header = header(9, &[pair(1, 0)]);
    let w = load(&s);
    // Early: positions 0 and 1 are house 0's, house 1 takes W* (2). Late:
    // position 0 is house 1's again; house 1 starts at 2, the highest it holds.
    assert_eq!(starts(&w), [Some(1), Some(2)]);
    assert_eq!(at(&w, 0), Some(HouseId(1)));
    assert_eq!(at(&w, 1), Some(HouseId(0)));
    assert_eq!(at(&w, 2), Some(HouseId(1)));
}

/// spec: S-0009 claim 29 (Q-3215's interim rule: a mark at a position numbered N or above is not kept, so the reference there is empty) and S-0011 claim 23
#[test]
fn references_at_positions_n_and_above_are_empty() {
    let mut s = scene();
    // Pair 11 marks positions 4 and 5 of a four-position game.
    s.map.header = header(11, &[None, None, pair(0, 1)]);
    let w = load(&s);
    assert_eq!(at(&w, 4), None);
    assert_eq!(at(&w, 5), None);
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

/// spec: S-0009 claim 29 (References: the house named by the mark may be a passive house) and S-0011 claim 23
#[test]
fn a_reference_may_name_a_passive_house() {
    let mut s = scene();
    with_neutral(&mut s, true, true);
    // An AI house is a computer player, so the cleanup does not run (claim 28).
    s.setup.houses[0].human = false;
    s.setup.houses[0].ai = true;
    s.map.header = header(9, &[pair(2, -1)]);
    let w = load(&s);
    assert_eq!(w.houses[2].start, None, "a passive house takes no position");
    assert_eq!(starts(&w)[..2], [Some(2), Some(3)]);
    assert_eq!(at(&w, 0), Some(HouseId(2)));
}

/// spec: S-0009 claims 28 and 29 (Q-3213's interim rule: a removed house is no house for References, though a late pair names it)
#[test]
fn a_removed_house_is_no_reference() {
    let mut s = scene();
    with_neutral(&mut s, false, false);
    // No computer player: the cleanup removes the neutral house (claim 28).
    s.map.header = header(9, &[pair(2, -1)]);
    let w = load(&s);
    assert!(w.houses[2].removed);
    // Early: the neutral house holds position 0; the players take 2 and 3.
    assert_eq!(starts(&w), [Some(2), Some(3), None]);
    assert_eq!(at(&w, 0), None);
}

/// spec: S-0009 claim 29 with quirks.header_pairs_as_start_marks off (Edge cases: the importer reads at most 8 pairs, so no mark applies; each position's reference is the house that holds it, as claim 21 gives it) and S-0011 claim 23
#[test]
fn eight_pairs_read_keep_the_holders() {
    for seed in 1..=5 {
        let mut s = scene();
        s.setup.seed = seed;
        s.map.header = header(12, &[]);
        let w = load(&s);
        for k in 0..4 {
            let holder = w
                .houses
                .iter()
                .position(|h| h.start == Some(k as u32))
                .map(|i| HouseId(i as u8));
            assert_eq!(at(&w, k), holder, "seed {seed}, position {k}");
        }
    }
}

/// spec: S-0009 claim 29 (the late-phase marks are game state: the state hash survives a save/load round trip, and so does the reference)
#[test]
fn marks_survive_a_save_round_trip() {
    let mut s = scene();
    s.map.header = header(9, &[pair(0, 0)]);
    let w = load(&s);
    let bytes = glasnost_sim::save::save(&w);
    let back = glasnost_sim::save::load(
        &bytes,
        s.sim_rules(),
        std::sync::Arc::new(s.catalogue.clone()),
    )
    .expect("loads");
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
    assert_eq!(at(&back, 0), Some(HouseId(0)));
}
