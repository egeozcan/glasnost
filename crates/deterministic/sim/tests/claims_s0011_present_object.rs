//! Claim tests for S-0011 claim 24 and Terms, *Present object* (#216): the
//! change-owner action acts on present objects only, so a carrier whose
//! destruction has begun keeps its owner. Synthetic data: the testkit's
//! invented types, a 200-frame death sequence for WALKER and an invented
//! warhead W1 of infantry death kind 1.

use std::sync::Arc;

use glasnost_map::Map;
use glasnost_map::triggers::{Action, Event, TagDef, TriggerDef, Triggers};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, SequenceEntry, WarheadType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::{HouseId, ObjectId, World};
use glasnost_sim::{testkit, tick, triggers};

fn rules() -> Ruleset {
    let mut r: Ruleset = testkit::ruleset();
    if let Some(t) = r.sim.infantry.get_mut("WALKER") {
        for k in ["stand", "die_1", "die_2"] {
            t.sequences.insert(
                k.into(),
                SequenceEntry {
                    start: 0,
                    count: 200,
                    multiplier: 0,
                    direction: None,
                },
            );
        }
    }
    r.sim.warheads.insert(
        "W1".into(),
        WarheadType {
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            inf_death: Some(1),
            ..Default::default()
        },
    );
    r
}

/// A lethal hit by W1, which gives infantry death kind 1.
fn dying() -> Hit {
    let w1 = rules().sim.warheads.keys().position(|k| k == "W1").unwrap() as u32;
    Hit {
        amount: 100_000,
        warhead: Some(w1),
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    }
}

/// A game whose repeating tag `TAG` has one trigger: `event`, then change
/// owner (action 14) to house value 1 (Beta).
fn game(event: Event) -> World {
    let r = rules();
    let mut m: Map = testkit::empty_map(20);
    m.triggers = Triggers {
        triggers: vec![TriggerDef {
            events: vec![event],
            actions: vec![Action {
                code: 14,
                number: 1,
                ..Default::default()
            }],
            ..TriggerDef::created(b"T".to_vec())
        }],
        tags: vec![TagDef {
            id: b"TAG".to_vec(),
            repeat: 2,
            name: Vec::new(),
            first: Some(0),
            chain: vec![0],
        }],
        ..Default::default()
    };
    glasnost_sim::load::new_game(
        Arc::new(SimRules::new(&r, "synthetic-test")),
        &m,
        testkit::catalogue(),
        &testkit::setup(7),
    )
    .expect("synthetic game loads")
}

fn carrier(w: &mut World, kind: Kind, name: &str, x: i32) -> ObjectId {
    let id = testkit::spawn(w, kind, name, 0, glasnost_map::CellXY::new(x, 3), 0);
    triggers::object_tag(w, id, Some(b"TAG"));
    id
}

fn ev(code: i32, value: i32) -> Event {
    Event {
        code,
        kind: 0,
        value,
        type_name: None,
    }
}

// spec: S-0011 Terms (present object: not an object whose destruction has begun, for example one still showing its death), claims 19 and 24; S-0312 claim 29 (a dying infantry unit keeps its tag but is no present object for claim 24) and scenario D20
#[test]
fn a_poll_leaves_the_owner_of_an_infantry_carrier_showing_its_death() {
    // Variable 0 set: a condition event, judged in a poll.
    let mut w = game(ev(36, 0));
    let man = carrier(&mut w, Kind::Infantry, "WALKER", 3);
    let cart = carrier(&mut w, Kind::Vehicle, "CART", 6);
    damage::apply(&mut w, man, dying());
    let o = w.obj(man).expect("still showing its death");
    assert!(damage::dead(o) && o.on_map, "dead, on the map");
    assert_eq!(o.owner, Some(HouseId(0)));
    assert!(w.triggers.objects.contains_key(&man), "keeps its tag");
    w.triggers.variables[0] = true;
    for _ in 0..20 {
        tick::step(&mut w);
        if w.obj(cart).unwrap().owner == Some(HouseId(1)) {
            break;
        }
    }
    assert_eq!(
        w.obj(cart).unwrap().owner,
        Some(HouseId(1)),
        "the action fired"
    );
    let o = w.obj(man).expect("still showing its death");
    assert!(damage::dead(o), "still dead");
    assert_eq!(
        o.owner,
        Some(HouseId(0)),
        "a carrier showing its death is no present object: its owner stays"
    );
}

// spec: S-0011 Terms (present object) and claim 24; S-0312 claim 29 (the destruction raises come after health reaches 0)
#[test]
fn its_own_destruction_raise_leaves_the_owner_of_the_dying_carrier() {
    let mut w = game(ev(48, 0));
    let man = carrier(&mut w, Kind::Infantry, "WALKER", 3);
    let cart = carrier(&mut w, Kind::Vehicle, "CART", 6);
    damage::apply(&mut w, man, dying());
    assert_eq!(
        w.obj(cart).unwrap().owner,
        Some(HouseId(1)),
        "the action fired"
    );
    let o = w.obj(man).expect("still showing its death");
    assert!(damage::dead(o), "dead");
    assert_eq!(
        o.owner,
        Some(HouseId(0)),
        "the carrier whose destruction has begun keeps its owner"
    );
}
