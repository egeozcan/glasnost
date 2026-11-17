//! Claim tests for S-0409 claim 17 and S-0011 claim 26 on synthetic data:
//! the waypoint reveal action (17) skips each house whose whole-map-revealed
//! mark is set, and (revision 17, M8 A6c) does not count for such a house
//! through an ally.

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_map::triggers::{Action, Event, TagDef, TriggerDef, Triggers};
use glasnost_sim::econ::shroud;
use glasnost_sim::rules::SimRules;
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, World};

const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

/// A game with the lobby shroud setting `shroud` and a one-time trigger
/// whose elapsed-time event (1) fires the reveal action 17 around
/// waypoint 0, at the map's middle cell, which it returns.
fn game(shroud: bool) -> (World, CellXY) {
    let mut r = testkit::ruleset();
    r.sim.general.economy.shroud = Some(shroud);
    game_from(&r, testkit::setup(7))
}

/// As [`game`] with the lobby shroud on, ally reveal on, and Alpha (A) and
/// Beta (B) allied.
fn allied_game() -> (World, CellXY) {
    let mut r = testkit::ruleset();
    r.sim.general.economy.shroud = Some(true);
    r.sim.general.economy.ally_reveal = Some(true);
    let mut setup = testkit::setup(7);
    setup.alliances = vec![(0, 1)];
    game_from(&r, setup)
}

fn game_from(
    r: &glasnost_rules::schema::Ruleset,
    setup: glasnost_sim::world::GameSetup,
) -> (World, CellXY) {
    let rules = Arc::new(SimRules::new(r, "synthetic-test"));
    let mut m = testkit::empty_map(20);
    let cells: Vec<CellXY> = m.cells.iter().map(|(c, _)| c).collect();
    let wp = cells[cells.len() / 2];
    m.waypoints.insert(0, (wp.x as i16, wp.y as i16));
    m.triggers = Triggers {
        triggers: vec![TriggerDef {
            events: vec![Event {
                code: 13,
                kind: 0,
                value: 1,
                type_name: None,
            }],
            actions: vec![Action {
                code: 17,
                number: 0,
                ..Default::default()
            }],
            ..TriggerDef::created(b"T".to_vec())
        }],
        tags: vec![TagDef {
            id: b"TAG".to_vec(),
            repeat: 0,
            name: Vec::new(),
            first: Some(0),
            chain: vec![0],
        }],
        ..Default::default()
    };
    let w = glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &setup)
        .expect("synthetic game loads");
    (w, wp)
}

fn run_to(w: &mut World, t: i32) {
    while w.tick < t {
        tick::step(w);
    }
}

// spec: S-0011 claim 26 and S-0409 claim 17 (the waypoint reveal skips each house whose whole-map-revealed mark is set, tested for each house); S-0409 claim 3 (lobby shroud off: the mark set at start, nothing seen)
#[test]
fn claim_17_reveal_action_skips_a_marked_house() {
    let (mut w, wp) = game(false);
    run_to(&mut w, 14);
    assert!(shroud::whole_map_revealed(&w, A));
    assert!(!shroud::seen(&w, A, wp), "not seen before the action");
    let before = shroud::counts(&w, A, wp);
    run_to(&mut w, 30);
    assert!(shroud::whole_map_revealed(&w, A));
    assert_eq!(
        (shroud::seen(&w, A, wp), shroud::counts(&w, A, wp)),
        (false, before),
        "the action changes nothing for a marked house"
    );
}

// spec: S-0011 claim 26 and S-0409 claim 17 (a house whose mark is clear: the action explores the disc, which stays explored outside hostile gap fields)
#[test]
fn claim_17_reveal_action_reaches_an_unmarked_house() {
    let (mut w, wp) = game(true);
    run_to(&mut w, 14);
    assert!(!shroud::whole_map_revealed(&w, A));
    assert!(!shroud::explored(&w, A, wp));
    run_to(&mut w, 30);
    assert!(shroud::explored(&w, A, wp));
}

// spec: S-0409 claim 17 (revision 17: a waypoint reveal made for a house whose mark is clear does not count for a house whose mark is set, even when that house is allied with it and ally reveal is on; only the marked house is left out, and its unmarked ally still has the reveal made for itself) and claim 2 (revision 17: the waypoint reveal does not count for a marked ally); claim 15 (the defeat reveal sets the mark, explored only)
#[test]
fn claim_17_reveal_action_skips_a_marked_ally() {
    let (mut w, wp) = allied_game();
    run_to(&mut w, 14);
    shroud::defeat_reveal(&mut w, B);
    assert!(shroud::whole_map_revealed(&w, B), "set-up: B is marked");
    assert!(!shroud::whole_map_revealed(&w, A), "set-up: A is not");
    assert!(!shroud::explored(&w, A, wp));
    let before = (shroud::seen(&w, B, wp), shroud::counts(&w, B, wp));
    run_to(&mut w, 30);
    assert!(shroud::whole_map_revealed(&w, B));
    assert_eq!(
        (shroud::seen(&w, B, wp), shroud::counts(&w, B, wp)),
        before,
        "the reveal made for the unmarked ally gives the marked house nothing"
    );
    assert!(
        shroud::explored(&w, A, wp),
        "the unmarked ally still has the reveal made for itself"
    );
}

// spec: S-0409 claim 2 (an area reveal counts for the house it is made for and, with ally reveal, its allies) and claim 17 (revision 17: only the waypoint reveal leaves a marked ally out; another spec's area reveal keeps claim 2's ally rule; S1 row 6 makes the cells seen)
#[test]
fn claim_17_another_specs_area_reveal_still_counts_for_a_marked_ally() {
    let (mut w, wp) = allied_game();
    run_to(&mut w, 2);
    shroud::defeat_reveal(&mut w, B);
    assert!(shroud::whole_map_revealed(&w, B), "set-up: B is marked");
    assert!(!shroud::seen(&w, B, wp), "set-up: explored only, not seen");
    shroud::area_reveal(&mut w, A, glasnost_sim::world::Pos::centre(wp, 0), 2);
    assert!(shroud::seen(&w, A, wp), "made for A");
    assert!(
        shroud::seen(&w, B, wp),
        "counted for the marked ally through claim 2"
    );
}
