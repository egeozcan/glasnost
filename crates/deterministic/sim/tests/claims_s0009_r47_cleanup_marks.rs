//! S-0009 r47 claim 29, *After the cleanup*: the unused-house cleanup
//! (claim 28) changes no start position mark, and with
//! `quirks.cleanup_shifts_start_marks` on every house after a removed one
//! has a current place one lower (Terms, *current place*), by which the
//! marks are read in the late phase and for "player at position X" (claims
//! 21 and 29). Off, current places never change and the cleanup frees a
//! removed house's marks. Synthetic maps only; the start waypoints are
//! those of `claims_s0009_r45_header_marks.rs`.

mod common;

use common::{Scene, TRACK, unit_type};
use glasnost_map::{CellXY, Header};
use glasnost_sim::mp_triggers::resolve_house;
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, World};

/// "Player at position A"; B … H follow it.
const POSITION_A: i32 = 4475; // spec: S-0011 claim 23 (4475 … 4482: the player at position A … H)

/// Start waypoint markers 0 … 3 whose pairwise distances all differ
/// (scenario M13b): 0–1 is 10, 0–2 30, 0–3 21, 1–2 20, 1–3 16 and 2–3 22
/// cells (truncated).
const WAYPOINTS: [(i16, i16); 4] = [(30, 40), (40, 40), (60, 40), (44, 56)];

const ALPHA: usize = 0;
const NEUTRAL: usize = 1;
const SPECIAL: usize = 2;

/// A skirmish of one human player, Alpha (house 0), with the neutral house
/// (house 1) and the special house (house 2) after it, on the four start
/// waypoint markers. No house is an AI house, so the cleanup runs (claim
/// 28). Each country's passive field is given for the early phase (claim
/// 33, the base rules') and for the late phase.
fn scene(neutral: (bool, bool), special: (bool, bool), on: bool) -> Scene {
    let mut s = Scene::new();
    s.vehicle("TANK", unit_type(TRACK, 4, 5));
    s.setup.sandbox = false;
    for (k, &c) in WAYPOINTS.iter().enumerate() {
        s.map.waypoints.insert(k as u16, c);
    }
    s.setup.houses.truncate(1);
    for (name, role, (early, late)) in [
        ("Neutral", HouseRole::Neutral, neutral),
        ("Special", HouseRole::Special, special),
    ] {
        s.register("countries", name);
        s.ruleset.sim.countries.insert(
            name.into(),
            glasnost_rules::sim::Country {
                multiplay_passive: Some(late),
                multiplay_passive_base: Some(early),
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
    s.ruleset.globals.quirks.cleanup_shifts_start_marks = on;
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

/// "Player at position" `k` (0 for A).
fn at(w: &World, k: i32) -> Option<HouseId> {
    resolve_house(w, POSITION_A + k, None)
}

fn house(h: usize) -> Option<HouseId> {
    Some(HouseId(h as u8))
}

/// The first four marks in force from the late phase on.
fn marks(w: &World) -> Option<Vec<i32>> {
    w.start_marks.as_ref().map(|m| m[..4].to_vec())
}

/// spec: S-0009 claim 29 (After the cleanup, its table: the neutral house removed and the special house kept; with quirks.cleanup_shifts_start_marks on, a mark equal to the neutral house's number names the special house, which holds that position and starts there, and the position it took early names no house) and claim 28 (its exception for a special house kept after the neutral house's removal); no header pairs beyond the eighth
#[test]
fn a_kept_special_house_holds_the_removed_neutral_houses_position() {
    // Early: Alpha on chosen 0; the neutral house takes the farthest, 2; the
    // special house the farthest from 0 and 2, which is 3. The cleanup then
    // removes the neutral house and keeps the special house.
    let mut s = scene((false, false), (false, false), true);
    s.setup.houses[ALPHA].start = Some(0);
    let w = load(&s);
    assert!(w.houses[NEUTRAL].removed && !w.houses[SPECIAL].removed);
    assert_eq!(starts(&w), [Some(0), None, Some(2)]);
    assert_eq!(base(&w, SPECIAL), Some(wp(2)));
    // The cleanup changes no mark.
    assert_eq!(marks(&w), Some(vec![0, -1, 1, 2]));
    assert_eq!(at(&w, 0), house(ALPHA));
    assert_eq!(at(&w, 1), None);
    assert_eq!(at(&w, 2), house(SPECIAL));
    assert_eq!(at(&w, 3), None);
}

/// spec: S-0009 Edge cases (quirks.cleanup_shifts_start_marks off: current places never change; the cleanup frees every mark equal to a removed house's number, and a kept special house keeps the position it took in the early phase)
#[test]
fn switch_off_a_kept_special_house_keeps_its_own_position() {
    let mut s = scene((false, false), (false, false), false);
    s.setup.houses[ALPHA].start = Some(0);
    let w = load(&s);
    assert!(w.houses[NEUTRAL].removed && !w.houses[SPECIAL].removed);
    assert_eq!(starts(&w), [Some(0), None, Some(3)]);
    assert_eq!(base(&w, SPECIAL), Some(wp(3)));
    assert_eq!(at(&w, 2), None);
    assert_eq!(at(&w, 3), house(SPECIAL));
}

/// spec: S-0009 claim 29 (After the cleanup: with no position marked with the neutral house's number, the kept special house holds none and is assigned anew against list 3, every position marked one higher counting as taken, and the position it is given carries its current place) and claim 28 (the neutral house passive in the early phase, claim 33, but not in the late phase)
#[test]
fn a_kept_special_house_with_no_mark_of_its_place_is_assigned_anew() {
    // Early: Alpha on chosen 0; the neutral house is passive then, so it takes
    // nothing; the special house takes 2, the farthest from 0.
    let mut s = scene((true, false), (false, false), true);
    s.setup.houses[ALPHA].start = Some(0);
    let w = load(&s);
    assert!(w.houses[NEUTRAL].removed && !w.houses[SPECIAL].removed);
    // Late: no mark is the special house's current place 1, so it is assigned
    // anew with positions 0 and 2 taken: position 3 (43 against 30).
    assert_eq!(starts(&w), [Some(0), None, Some(3)]);
    assert_eq!(base(&w, SPECIAL), Some(wp(3)));
    assert_eq!(marks(&w), Some(vec![0, -1, 2, 1]));
    assert_eq!(at(&w, 2), None, "mark 2 names no house");
    assert_eq!(at(&w, 3), house(SPECIAL));
    // Switch off: the special house holds its early position 2 by its number.
    let mut off = scene((true, false), (false, false), false);
    off.setup.houses[ALPHA].start = Some(0);
    let w = load(&off);
    assert_eq!(starts(&w), [Some(0), None, Some(2)]);
    assert_eq!(at(&w, 2), house(SPECIAL));
    assert_eq!(at(&w, 3), None);
}

/// spec: S-0009 claim 29 (After the cleanup: the marks that the late phase's applied pairs give are read by current places too, so a late pair giving the neutral house's number puts the kept special house there; with quirks.cleanup_shifts_start_marks off, the position is taken by no player and the special house is assigned anew)
#[test]
fn a_late_pair_giving_the_removed_houses_number_names_the_next_house() {
    // N_h = 9, pair 9 = (−1, 1). Early: the neutral house holds 1; Alpha
    // takes 2 (farthest from 1); the special house 0 (40 against 38).
    let mut s = scene((false, false), (false, false), true);
    s.map.header = header(9, &[pair(-1, 1)]);
    let w = load(&s);
    assert!(w.houses[NEUTRAL].removed);
    // Late: the pair frees 0 and gives 1 again, which names the special house.
    assert_eq!(starts(&w), [Some(2), None, Some(1)]);
    assert_eq!(base(&w, SPECIAL), Some(wp(1)));
    assert_eq!(at(&w, 0), None);
    assert_eq!(at(&w, 1), house(SPECIAL));
    assert_eq!(at(&w, 2), house(ALPHA));
    // Switch off: position 1 is taken by no player, and the special house is
    // assigned anew with 1 and 2 taken: position 0, which then carries its
    // number 2.
    let mut off = scene((false, false), (false, false), false);
    off.map.header = header(9, &[pair(-1, 1)]);
    let w = load(&off);
    assert_eq!(starts(&w), [Some(2), None, Some(0)]);
    assert_eq!(marks(&w), Some(vec![2, 1, 0, -1]));
    assert_eq!(at(&w, 0), house(SPECIAL));
    assert_eq!(at(&w, 1), None);
}

/// spec: S-0009 claim 29 (After the cleanup: when the house a mark names after the shift is passive, that position is taken by no player and "player at position X" on it names that passive house) and claim 21 (once the cleanup has removed a house the References rule replaces claim 21 on any map)
#[test]
fn a_shifted_mark_may_name_a_passive_house() {
    // The special house is not passive early (it takes 3) but passive late,
    // so the cleanup removes only the neutral house (which took 2).
    let mut s = scene((false, false), (false, true), true);
    s.setup.houses[ALPHA].start = Some(0);
    let w = load(&s);
    assert!(w.houses[NEUTRAL].removed && !w.houses[SPECIAL].removed);
    assert!(w.start_marks.is_some(), "the References rule is in force");
    assert_eq!(at(&w, 2), house(SPECIAL));
    assert_eq!(at(&w, 3), None);
    // Switch off: the neutral house's mark is freed, and the special house's
    // own mark names it.
    let mut off = scene((false, false), (false, true), false);
    off.setup.houses[ALPHA].start = Some(0);
    let w = load(&off);
    assert_eq!(at(&w, 2), None);
    assert_eq!(at(&w, 3), house(SPECIAL));
}

/// spec: S-0009 claim 29 (After the cleanup: a removed house has no current place and no mark names it; the last house in house order shifts no other, so its early position stays taken by no player in the late phase) and claim 28
#[test]
fn a_removed_last_houses_position_stays_taken() {
    // Alpha and Beta on chosen 0 and 1, the neutral house last (house 2),
    // N_h = 9, pair 9 = (−1, −1). Early: the neutral house takes 2.
    let mut s = Scene::new();
    s.vehicle("TANK", unit_type(TRACK, 4, 5));
    s.setup.sandbox = false;
    for (k, &c) in WAYPOINTS.iter().enumerate() {
        s.map.waypoints.insert(k as u16, c);
    }
    s.register("countries", "Neutral");
    s.ruleset.sim.countries.insert(
        "Neutral".into(),
        glasnost_rules::sim::Country {
            multiplay_passive: Some(false),
            multiplay_passive_base: Some(false),
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
    s.setup.houses[0].start = Some(0);
    s.setup.houses[1].start = Some(1);
    s.map.header = header(9, &[pair(-1, -1)]);
    for seed in 1..=4 {
        s.setup.seed = seed;
        let w = load(&s);
        assert!(w.houses[2].removed);
        // Late: 0 and 1 are freed; 2 is taken by no player, so Alpha takes the
        // farthest from 2 (position 0) with no draw, and Beta the farthest from
        // 0 and 2 (position 3).
        assert_eq!(starts(&w), [Some(0), Some(3), None], "seed {seed}");
        assert_eq!(at(&w, 2), None);
    }
}

/// spec: S-0009 claims 28 and 29 (After the cleanup's table: the special house removed, the neutral house passive; and quirks.cleanup_skips_next_house off, both removed: no mark names a removed house, in either position of quirks.cleanup_shifts_start_marks)
#[test]
fn marks_of_removed_houses_name_no_house() {
    for on in [true, false] {
        let mut s = scene((true, true), (false, false), on);
        s.setup.houses[ALPHA].start = Some(0);
        let w = load(&s);
        assert!(!w.houses[NEUTRAL].removed && w.houses[SPECIAL].removed);
        assert_eq!(starts(&w), [Some(0), None, None], "on {on}");
        assert_eq!(at(&w, 2), None, "on {on}");
        let mut both = scene((false, false), (false, false), on);
        both.ruleset.globals.quirks.cleanup_skips_next_house = false;
        both.setup.houses[ALPHA].start = Some(0);
        let w = load(&both);
        assert!(w.houses[NEUTRAL].removed && w.houses[SPECIAL].removed);
        assert_eq!(starts(&w), [Some(0), None, None], "on {on}");
        assert_eq!((at(&w, 2), at(&w, 3)), (None, None), "on {on}");
    }
}
