//! Claim tests of S-0011 revision 29 with S-0312 revision 54 (Q-2716
//! answered): a carrier keeps its tag, and its place in the tag's carrier
//! count, when it is destroyed, leaves play or is removed, and the place is
//! not given back when it stops existing (claim 2,
//! `quirks.removed_carrier_keeps_tag`). So an all-carriers tag that has lost
//! a carrier in this way does not fire at its last carrier's raise (claim
//! 10, scenario T6 variant H); an infantry unit still showing its death
//! keeps its tag through its removal (S-0312 claim 29); and a leftover still
//! carries its tag, so the defeat sweep's killing hit on a leftover with
//! health left makes S-0312 claim 29's raises for a killing hit with no
//! attacker (S-0203 claim 13 (g)). Both positions of the switch. On
//! synthetic maps and the testkit's invented types; no game data.

use std::sync::Arc;

use glasnost_map::Map;
use glasnost_map::triggers::{Action, Event, TagDef, TriggerDef, Triggers};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, SequenceEntry, WarheadType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::{HouseId, ObjectId, World};
use glasnost_sim::{leftover, testkit, tick, triggers};

fn ev(code: i32, value: i32) -> Event {
    Event {
        code,
        kind: 0,
        value,
        type_name: None,
    }
}

/// Action 56: set variable `v` (the observable).
fn set(v: i32) -> Action {
    Action {
        code: 56,
        number: v,
        ..Default::default()
    }
}

fn trig(events: Vec<Event>, actions: Vec<Action>) -> TriggerDef {
    TriggerDef {
        events,
        actions,
        ..TriggerDef::created(b"T".to_vec())
    }
}

fn tag(repeat: i32) -> TagDef {
    TagDef {
        id: b"TAG".to_vec(),
        repeat,
        name: Vec::new(),
        first: Some(0),
        chain: vec![0],
    }
}

/// The testkit's rules with the switch set, die-1 and die-2 sequences of 4
/// frames for WALKER and warhead W1 of infantry death kind 1 (a die-1
/// request, S-0107 claim 17). Invented values.
fn rules(keeps: bool) -> Ruleset {
    let mut r: Ruleset = testkit::ruleset();
    r.globals.quirks.removed_carrier_keeps_tag = keeps;
    if let Some(t) = r.sim.infantry.get_mut("WALKER") {
        for k in ["stand", "die_1", "die_2"] {
            t.sequences.insert(
                k.into(),
                SequenceEntry {
                    start: 0,
                    count: 4,
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

/// A killing hit with warhead W1 (a death sequence for WALKER), no
/// attacker and no attacking house.
fn dying() -> Hit {
    let w1 = rules(true)
        .sim
        .warheads
        .keys()
        .position(|k| k == "W1")
        .unwrap() as u32;
    Hit {
        warhead: Some(w1),
        ignore_defences: false,
        ..lethal()
    }
}

fn game(def: TriggerDef, repeat: i32, keeps: bool) -> World {
    let r = rules(keeps);
    let mut m: Map = testkit::empty_map(20);
    m.triggers = Triggers {
        triggers: vec![def],
        tags: vec![tag(repeat)],
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

fn carriers(w: &World) -> i32 {
    w.triggers.tags[&0].carriers
}

/// A killing hit with no attacker and no attacking house.
fn lethal() -> Hit {
    Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

fn step_to(w: &mut World, t: i32) {
    while w.tick < t {
        tick::step(w);
    }
}

/// Scenario T6 variant H: an all-carriers tag on vehicles A and B, whose
/// trigger has a game-time event (value 2: from tick 30, claim 16) first
/// and destroyed-by-anything second, and sets variable 0. A is destroyed in
/// tick `a`, B in tick `b`; whether the variable was set.
fn variant_h(keeps: bool, a: i32, b: i32) -> bool {
    let mut w = game(trig(vec![ev(47, 2), ev(48, 0)], vec![set(0)]), 1, keeps);
    let va = carrier(&mut w, Kind::Vehicle, "CART", 3);
    let vb = carrier(&mut w, Kind::Vehicle, "CART", 6);
    assert_eq!(carriers(&w), 2);
    step_to(&mut w, a);
    damage::apply(&mut w, va, lethal());
    assert!(w.obj(va).is_none(), "A is removed at its destruction");
    step_to(&mut w, b);
    damage::apply(&mut w, vb, lethal());
    w.triggers.variables[0]
}

// spec: S-0011 claim 2 (a carrier that stops existing while it carries the tag keeps its place in the carrier count, quirks.removed_carrier_keeps_tag) and claim 10 (so the other carrier's later raise finds two carriers and only detaches it); scenario T6 variant H, run 1
#[test]
fn t6_h_run_1_a_carrier_destroyed_before_the_trigger_is_satisfied_still_counts() {
    assert!(
        !variant_h(true, 10, 40),
        "no firing: B's raise finds two carriers"
    );
}

// spec: S-0011 Edge cases (quirks.removed_carrier_keeps_tag off: an object loses its tag when it is removed, as at a detachment, so the count drops by one); scenario T6 variant H, run 1, the run that tells the two settings apart
#[test]
fn t6_h_run_1_switch_off_the_removal_detaches_and_b_fires() {
    assert!(variant_h(false, 10, 40), "B's raise finds one carrier");
}

// spec: S-0011 claim 10 and State and ordering (A's satisfied raise detaches A under either setting, so B's raise finds one carrier and fires once); scenario T6 variant H, run 2
#[test]
fn t6_h_run_2_both_destroyed_after_the_game_time_fire_once_in_both_positions() {
    for keeps in [true, false] {
        assert!(variant_h(keeps, 35, 40), "keeps {keeps}: one firing at B");
    }
}

// spec: S-0011 claim 2 (being removed does not detach the tag: the entry and the place in the count stay for the rest of the game; off: the removal detaches it)
#[test]
fn a_removed_carrier_keeps_its_tag_and_its_place() {
    for keeps in [true, false] {
        let mut w = game(trig(vec![ev(48, 0)], vec![set(0)]), 2, keeps);
        let v = carrier(&mut w, Kind::Vehicle, "CART", 3);
        glasnost_sim::objects::remove(&mut w, v);
        assert_eq!(
            w.triggers.objects.contains_key(&v),
            keeps,
            "keeps {keeps}: the tag"
        );
        assert_eq!(carriers(&w), i32::from(keeps), "keeps {keeps}: the count");
        for _ in 0..3 {
            tick::step(&mut w);
        }
        assert_eq!(
            carriers(&w),
            i32::from(keeps),
            "keeps {keeps}: not given back when it stops existing"
        );
        assert!(!w.triggers.variables[0], "a removal makes no raise");
    }
}

// spec: S-0312 claim 29 (r54, Infantry still showing death: starting the death sequence does not detach the tag; its removal does not detach it under quirks.removed_carrier_keeps_tag, and detaches it with the switch off; completion and removal make no new destroyed-by-anything raise)
#[test]
fn an_infantry_carrier_keeps_its_tag_through_its_death_sequence_and_removal() {
    for keeps in [true, false] {
        // A repeating tag whose trigger waits for a game time far ahead:
        // its destruction raises are not satisfied, so no raise detaches
        // anything and nothing fires.
        let mut w = game(trig(vec![ev(47, 1000)], vec![set(0)]), 2, keeps);
        let man = carrier(&mut w, Kind::Infantry, "WALKER", 3);
        let other = carrier(&mut w, Kind::Vehicle, "CART", 6);
        assert_eq!(carriers(&w), 2);
        let fired = w.triggers.fired;
        damage::apply(&mut w, man, dying());
        assert!(w.obj(man).is_some(), "still showing its death");
        assert!(
            w.triggers.objects.contains_key(&man),
            "the start detaches nothing"
        );
        assert_eq!(carriers(&w), 2);
        for _ in 0..400 {
            if w.obj(man).is_none() {
                break;
            }
            tick::step(&mut w);
        }
        assert!(w.obj(man).is_none(), "removed at the end of its death");
        assert_eq!(
            carriers(&w),
            if keeps { 2 } else { 1 },
            "keeps {keeps}: the removal"
        );
        assert_eq!(w.triggers.fired, fired, "no firing at all");
        assert!(w.obj(other).is_some());
    }
}

/// A repeating tag whose only trigger is destroyed-by-anything with no
/// action, carried by a vehicle of house 0 that is removed with `health`
/// left and so kept as a leftover (S-0203 claim 13); the firings of the
/// defeat sweep's hit on it (S-0203 claim 13 (g)).
fn sweep_firings(keeps: bool, health: i32) -> u64 {
    let mut w = game(trig(vec![ev(48, 0)], vec![]), 2, keeps);
    let v = carrier(&mut w, Kind::Vehicle, "CART", 3);
    w.obj_mut(v).unwrap().health = health;
    glasnost_sim::objects::remove(&mut w, v);
    assert!(leftover::object(&w, v).is_some(), "kept as a leftover");
    let f0 = w.triggers.fired;
    leftover::sweep_hit(&mut w, v, HouseId(0));
    w.triggers.fired - f0
}

// spec: S-0203 claim 13 (g) and S-0312 claim 29 (a leftover with health left takes a killing hit, and the raises of a killing hit with no attacker and no attacking house come again for a tag that it still carries: four firings of a fresh repeating destroyed-by-anything trigger, S-0011 claim 19); S-0011 claim 2 (a leftover still carries its tag, quirks.removed_carrier_keeps_tag)
#[test]
fn the_sweep_hit_on_a_tagged_leftover_raises_its_tag() {
    assert_eq!(sweep_firings(true, 50), 4);
}

// spec: S-0011 Edge cases (quirks.removed_carrier_keeps_tag off: a leftover carries no tag, so the sweep's hit raises nothing); S-0203 claim 13 (c) (a leftover with no health takes a hit of no amount, which is no killing hit and makes no raise, in both positions)
#[test]
fn no_raise_from_the_sweep_with_the_switch_off_or_on_a_leftover_with_no_health() {
    assert_eq!(sweep_firings(false, 50), 0);
    for keeps in [true, false] {
        assert_eq!(sweep_firings(keeps, 0), 0, "keeps {keeps}: no health left");
    }
}
