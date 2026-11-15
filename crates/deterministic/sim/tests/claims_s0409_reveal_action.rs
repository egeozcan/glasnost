//! Claim tests for S-0409 claim 17 and S-0011 claim 26 on synthetic data:
//! the waypoint reveal action (17) skips each house whose whole-map-revealed
//! mark is set.

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_map::triggers::{Action, Event, TagDef, TriggerDef, Triggers};
use glasnost_sim::econ::shroud;
use glasnost_sim::rules::SimRules;
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, World};

const A: HouseId = HouseId(0);

/// A game with the lobby shroud setting `shroud` and a one-time trigger
/// whose elapsed-time event (1) fires the reveal action 17 around
/// waypoint 0, at the map's middle cell, which it returns.
fn game(shroud: bool) -> (World, CellXY) {
    let mut r = testkit::ruleset();
    r.sim.general.economy.shroud = Some(shroud);
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
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
    let w = glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &testkit::setup(7))
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
