//! A storm cloud made by a trigger action after the turn pass first
//! updates in the next tick, which delays its first advance for a stage
//! delay of 1 (#352; S-0513 claim 46, S-0109 claim 5). On the synthetic
//! superweapon rules of `sw_common` (cloud types of 7 image frames, no rate
//! field) and invented triggers, no game data.

mod sw_common;

use std::sync::Arc;

use glasnost_map::triggers::{Action, Event, Owner, TagDef, TriggerDef, Triggers};
use glasnost_sim::rules::SimRules;
use glasnost_sim::superweapons::storm;
use glasnost_sim::testkit;
use glasnost_sim::world::World;
use sw_common::{SIZE, at, step};

fn ev(code: i32, value: i32) -> Event {
    Event {
        code,
        kind: 0,
        value,
        type_name: None,
    }
}

fn def(id: &str, events: Vec<Event>, actions: Vec<Action>) -> TriggerDef {
    TriggerDef {
        owner: Owner::Name(b"Alphaland".to_vec()),
        events,
        actions,
        ..TriggerDef::created(id.as_bytes().to_vec())
    }
}

fn game(tr: Triggers) -> World {
    let r = sw_common::ruleset();
    let mut m = testkit::empty_map(SIZE);
    m.triggers = tr;
    let c = at(0, 0);
    m.waypoints.insert(0, (c.x as i16, c.y as i16));
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &testkit::setup(7))
        .expect("synthetic game loads")
}

/// Steps until the one cloud has struck and gone; returns (strike tick,
/// gone tick).
fn strike_and_gone(w: &mut World) -> (Option<i32>, Option<i32>) {
    let (mut strike, mut gone) = (None, None);
    for _ in 0..50 {
        let t = w.tick;
        step(w);
        if strike.is_none() && w.sw.clouds.first().is_none_or(|c| !c.pending) {
            strike = Some(t);
        }
        if w.sw.clouds.is_empty() {
            gone = Some(t);
            break;
        }
    }
    (strike, gone)
}

/// Tag S (registered first): any-event and variable 5 set; action 90
/// (lightning strike) at waypoint 0. Tag V (registered second): any-event;
/// action 56 sets variable 5. In tick 0 the poll (registration order) finds
/// variable 5 clear for S, then V sets it; the house raise of tick 0, after
/// the turn pass, then fires S.
fn late_triggers() -> Triggers {
    let defs = vec![
        def(
            "S",
            vec![ev(8, 0), ev(36, 5)],
            vec![Action {
                code: 90,
                waypoint: 0,
                ..Default::default()
            }],
        ),
        def(
            "V",
            vec![ev(8, 0)],
            vec![Action {
                code: 56,
                number: 5,
                ..Default::default()
            }],
        ),
    ];
    let tags = defs
        .iter()
        .enumerate()
        .map(|(i, t)| TagDef {
            id: t.id.clone(),
            repeat: 0,
            name: t.id.clone(),
            first: Some(i as u32),
            chain: vec![i as u32],
        })
        .collect();
    Triggers {
        triggers: defs,
        tags,
        ..Default::default()
    }
}

// spec: S-0513 claim 46 (created before the turn pass, u = c: F = 7, d = 1, strike in c + (⌊F/2⌋ + 1)d + 1, gone in c + (F − 1)d + 1)
#[test]
fn issue_352_control_a_cloud_made_before_the_turn_pass() {
    let mut w = game(Triggers::default());
    let c = w.tick;
    storm::cloud(&mut w, at(0, 0));
    assert_eq!(w.sw.clouds[0].delay, 1);
    assert_eq!(strike_and_gone(&mut w), (Some(c + 5), Some(c + 7)));
}

// spec: S-0513 claim 46 and S-0109 claim 5 (created after the turn pass: u = c + 1, a = max(c + d, u + 1) = c + 2; strike in a + ⌊F/2⌋d + 1 = c + 6, gone in a + (F − 2)d + 1 = c + 8) and S-0011 claim 35 (the house raise near the end of the tick's game logic)
#[test]
fn issue_352_a_cloud_made_after_the_turn_pass_with_delay_1() {
    let tr = late_triggers();
    // The poll alone makes no cloud: only V fires in it.
    let mut p = game(tr.clone());
    glasnost_sim::triggers::poll(&mut p);
    assert_eq!((p.triggers.fired, p.sw.clouds.len()), (1, 0), "set-up");
    let mut w = game(tr);
    let c = w.tick;
    assert_eq!(c % 8, 0, "set-up: a house-raise tick");
    step(&mut w);
    assert_eq!(w.triggers.fired, 2, "set-up");
    assert_eq!(w.sw.clouds.len(), 1, "set-up: S fired in the house raise of tick c");
    assert_eq!((w.sw.clouds[0].created, w.sw.clouds[0].delay), (c, 1));
    assert_eq!(strike_and_gone(&mut w), (Some(c + 6), Some(c + 8)));
}
