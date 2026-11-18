//! S-0009 r47 Edge cases, `quirks.double_start_resolution` off (Q-3214's
//! answer): the header pairs of claim 29 are applied once, at the one
//! resolution, by the early phase's precedence; that resolution comes
//! before the unused-house cleanup of claim 28, after which *After the
//! cleanup* applies and a house that then holds no position is assigned
//! against the same list. Synthetic maps only; the start waypoints are
//! those of `claims_s0009_r45_header_marks.rs`.

mod common;

use common::{Scene, TRACK, unit_type};
use glasnost_map::Header;
use glasnost_sim::world::{HouseRole, HouseSetup, World};

/// Start waypoint markers 0 … 3 whose pairwise distances all differ
/// (scenario M13b): 0–1 is 10, 0–2 30, 0–3 21, 1–2 20, 1–3 16 and 2–3 22
/// cells (truncated).
const WAYPOINTS: [(i16, i16); 4] = [(30, 40), (40, 40), (60, 40), (44, 56)];

const NEUTRAL: usize = 1;
const SPECIAL: usize = 2;

/// Two players, Alpha and Beta, on the four start waypoint markers, with
/// one start resolution.
fn scene() -> Scene {
    let mut s = Scene::new();
    s.vehicle("TANK", unit_type(TRACK, 4, 5));
    s.setup.sandbox = false;
    for (k, &c) in WAYPOINTS.iter().enumerate() {
        s.map.waypoints.insert(k as u16, c);
    }
    s.ruleset.globals.quirks.double_start_resolution = false;
    s
}

/// One human player, Alpha (house 0), on chosen position 0, with the
/// neutral house (house 1) and the special house (house 2), neither
/// passive. No house is an AI house, so the cleanup runs: it removes the
/// neutral house and keeps the special house (claim 28).
fn cleanup_scene(shifts: bool) -> Scene {
    let mut s = scene();
    s.setup.houses.truncate(1);
    s.setup.houses[0].start = Some(0);
    for (name, role) in [
        ("Neutral", HouseRole::Neutral),
        ("Special", HouseRole::Special),
    ] {
        s.register("countries", name);
        s.ruleset.sim.countries.insert(
            name.into(),
            glasnost_rules::sim::Country {
                multiplay_passive: Some(false),
                multiplay_passive_base: Some(false),
                ..Default::default()
            },
        );
        s.setup.houses.push(HouseSetup {
            name: name.into(),
            country: name.into(),
            human: false,
            start: None,
            role,
            ai: false,
        });
    }
    s.ruleset.globals.quirks.cleanup_shifts_start_marks = shifts;
    s
}

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

/// The marks of start positions 0 … 3 (the map's four), after checking that
/// every mark of a position beyond them is −1.
fn marks(w: &World) -> Vec<i32> {
    let m = w.start_marks.clone().expect("the references read marks");
    assert!(m.iter().skip(WAYPOINTS.len()).all(|&k| k == -1), "{m:?}");
    m.into_iter().take(WAYPOINTS.len()).collect()
}

/// spec: S-0009 Edge cases (quirks.double_start_resolution off: the one resolution comes before the unused-house cleanup, which therefore finds the marks it leaves; with quirks.cleanup_shifts_start_marks off the cleanup frees the removed house's marks, and a kept house keeps the position it was given; Q-3214's answer)
#[test]
fn the_one_resolution_comes_before_the_cleanup() {
    // The one resolution: Alpha holds 0; the neutral house takes the
    // farthest, 2; the special house the farthest from 0 and 2, which is 3.
    // The cleanup removes the neutral house and frees its mark.
    let w = load(&cleanup_scene(false));
    assert!(w.houses[NEUTRAL].removed && !w.houses[SPECIAL].removed);
    assert_eq!(starts(&w), [Some(0), None, Some(3)]);
    assert_eq!(marks(&w), [0, -1, -1, 2]);
}

/// spec: S-0009 Edge cases (quirks.double_start_resolution off: after the cleanup, After the cleanup of claim 29 applies to the one resolution's marks; with quirks.cleanup_shifts_start_marks on the kept special house holds the removed neutral house's position)
#[test]
fn after_the_one_resolution_the_kept_special_house_holds_the_neutral_houses_position() {
    let w = load(&cleanup_scene(true));
    assert!(w.houses[NEUTRAL].removed && !w.houses[SPECIAL].removed);
    assert_eq!(starts(&w), [Some(0), None, Some(2)]);
    // Mark 1 (the neutral house's, from the one resolution) names the special
    // house by its current place; mark 2 names no house.
    assert_eq!(marks(&w), [0, -1, 1, 2]);
}

/// spec: S-0009 Edge cases (quirks.double_start_resolution off: the pairs are applied once, at the one resolution before the cleanup, a chosen position's mark being its chooser's house number whatever a pair says; After the cleanup of claim 29 then applies to those marks)
#[test]
fn a_pair_applied_at_the_one_resolution_is_read_after_the_cleanup() {
    // N_h = 9, pair 9 = (−1, 1). The one resolution: Alpha's chosen 0 keeps
    // mark 0; the pair gives position 1 mark 1, so the neutral house holds 1;
    // the special house takes the farthest from 0 and 1, which is 2 (50
    // against 37).
    let mut s = cleanup_scene(true);
    s.map.header = header(9, &[Some([Some(-1), Some(1)])]);
    let w = load(&s);
    assert!(w.houses[NEUTRAL].removed);
    // Switch on: mark 1 now names the special house (current place 1), and
    // mark 2 names no house.
    assert_eq!(starts(&w), [Some(0), None, Some(1)]);
    assert_eq!(marks(&w), [0, 1, 2, -1]);
    // Switch off: the cleanup frees mark 1, and the special house keeps 2.
    let mut s = cleanup_scene(false);
    s.map.header = header(9, &[Some([Some(-1), Some(1)])]);
    let w = load(&s);
    assert_eq!(starts(&w), [Some(0), None, Some(2)]);
    assert_eq!(marks(&w), [0, -1, 2, -1]);
}

/// spec: S-0009 Edge cases (quirks.double_start_resolution off: a non-passive house that holds no position after the cleanup is assigned by claim 19 against the same list, every mark other than −1 counting as taken)
#[test]
fn a_house_left_with_no_position_after_the_cleanup_is_assigned_then() {
    for shifts in [true, false] {
        // Two start positions: Alpha holds 0, the neutral house takes 1, and
        // the special house finds both taken. After the neutral house's
        // removal the special house holds 1 (switch on) or is assigned 1, the
        // one free position (switch off).
        let mut s = cleanup_scene(shifts);
        s.map.waypoints.remove(&2);
        s.map.waypoints.remove(&3);
        let w = load(&s);
        assert!(w.houses[NEUTRAL].removed);
        assert_eq!(starts(&w), [Some(0), None, Some(1)], "shifts {shifts}");
    }
}

/// spec: S-0009 Edge cases (quirks.double_start_resolution off: the pairs are applied once, by the early phase's precedence; an unread component keeps −1, the mark at the start of the resolution; Q-3214's answer)
#[test]
fn an_unread_component_keeps_the_free_mark_in_the_one_resolution() {
    for seed in 1..=3 {
        let mut s = scene();
        s.setup.seed = seed;
        // Pair 9 = (unread, 0): Alpha holds 1; position 0 stays free, so Beta
        // takes the farthest from 1 alone, position 2.
        s.map.header = header(9, &[Some([None, Some(0)])]);
        let w = load(&s);
        assert_eq!(starts(&w), [Some(1), Some(2)], "seed {seed}");
    }
}
