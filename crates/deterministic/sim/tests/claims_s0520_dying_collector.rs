//! Claim test for S-0520 claim 7, sim issues S15 (#281): a tagged
//! collector that the event-49 raise kills into its death sequence cancels
//! the pickup. Synthetic data of `crate_common`; every name and number is
//! invented. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S15*.

#[path = "crate_common/mod.rs"]
mod crate_common;

use std::sync::Arc;

use crate_common::*;
use glasnost_map::triggers::{Action, Event, Owner, TagDef, TriggerDef, Triggers};
use glasnost_rules::sim::SequenceEntry;
use glasnost_sim::combat::damage;
use glasnost_sim::crates;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::testkit;

// spec: S-0520 claim 7 (if the collector is no longer alive after the raise, nothing more happens: the crate stays, the any-crate mark is unchanged by this attempt and a ground step is blocked); State and ordering table (pickup blocked by a dying tagged collector: crate remains, mover stops)
#[test]
fn issue_281_a_collector_killed_by_the_raise_into_its_death_sequence_cancels_the_pickup() {
    let mut r = ruleset();
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.strength = Some(10);
    for k in [
        "stand", "walk", "go_prone", "prone", "crawl", "get_up", "die_1", "die_2",
    ] {
        walker.sequences.insert(
            k.into(),
            SequenceEntry {
                start: 0,
                count: 4,
                multiplier: 0,
                direction: None,
            },
        );
    }
    r.sim.warheads.get_mut("DEMO").unwrap().inf_death = Some(1);

    let def = TriggerDef {
        owner: Owner::None,
        events: vec![Event {
            code: 49,
            kind: 0,
            value: 0,
            type_name: None,
        }],
        actions: vec![Action {
            code: 63,
            number: 3,
            ..Default::default()
        }],
        ..TriggerDef::created(b"T".to_vec())
    };
    let tr = Triggers {
        tags: vec![TagDef {
            id: def.id.clone(),
            repeat: 0,
            name: def.id.clone(),
            first: Some(0),
            chain: vec![0],
        }],
        triggers: vec![def],
        ..Default::default()
    };
    let mut m = testkit::empty_map(SIZE);
    m.triggers = tr;
    let home = at(0, 0);
    m.waypoints.insert(3, (home.x as i16, home.y as i16));
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &testkit::setup(7))
        .expect("synthetic game loads");
    w.econ.settings.crates = Some(false);

    let inf = spawn(&mut w, Kind::Infantry, "WALKER", 0, home);
    glasnost_sim::triggers::object_tag(&mut w, inf, Some(b"T"));
    assert!(
        w.triggers.objects.contains_key(&inf),
        "set-up: the collector is tagged"
    );
    let c = at(1, 0);
    crate_at(&mut w, c, TRIGGER, 0);
    let money = credits(&w, 0);
    let fired = w.triggers.fired;

    let blocked = crates::pickup(&mut w, inf, c);

    assert_eq!(
        w.triggers.fired,
        fired + 1,
        "set-up: the event-49 trigger fired"
    );
    let o = w.obj(inf).expect("still in play, in its death sequence");
    assert!(damage::dead(o), "set-up: the raise killed the collector");
    assert!(crates::is_crate(&w, c), "the crate stays");
    assert!(!w.crates.any_picked, "the any-crate mark is unchanged");
    assert_eq!(credits(&w, 0), money, "no power-up is given");
    assert!(blocked, "a ground step is blocked");
}
