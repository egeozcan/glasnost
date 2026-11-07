//! Claim tests of S-0309 revision 120 (Q-2712 answered), claim 30: a holder
//! that changes owner drops its target inside the change, whatever the
//! cause, with no targeting-timer draw; for the causes the claim lists the
//! same change clears its destination and requests *guard* as S-0601 claim
//! 9 gives for a release. And S-0508 claim 7 (r13): a civilian garrison
//! building's owner change comes after its behaviour processing, its
//! per-tick search and all its firing in its turn. Built on scenario A19
//! and S-0508 scenario G20, on the synthetic rules of `structures_common`:
//! invented types and numbers, no game data.

mod structures_common;

use glasnost_map::triggers::{Action, Event, TagDef, TriggerDef, Triggers};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::garrison;
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use structures_common::*;

/// Gives `holder` the target `t`, a destination and a targeting timer 30
/// ticks ahead (more than claim 27's 10 ticks left, so a claim 27 drop
/// would draw).
fn arm(w: &mut World, holder: ObjectId, t: ObjectId) {
    let tick = w.tick;
    let o = w.obj_mut(holder).unwrap();
    o.target = Some(Target::Object(t));
    if o.kind != Kind::Building {
        o.dest = Some(Target::Cell(at(5, 5)));
    }
    o.targeting_timer = tick + 30;
}

/// What claim 30 leaves on `holder`: its target, destination, current and
/// queued behaviour and targeting timer.
type Left = (
    Option<Target>,
    Option<Target>,
    Option<Behaviour>,
    Option<Behaviour>,
    i32,
);

fn left(w: &World, holder: ObjectId) -> Left {
    let o = w.obj(holder).unwrap();
    (o.target, o.dest, o.beh.current, o.beh.queued, o.targeting_timer)
}

fn guarded(l: &Left) -> bool {
    l.2 == Some(Behaviour::Guard) || l.3 == Some(Behaviour::Guard)
}

// spec: S-0309 claim 30 (an engineer capture, S-0509 claim 8: the building drops its target inside the change, with no targeting-timer draw, and the timer counts on from its value; the change also requests guard as S-0601 claim 9 gives); scenario A19 parts a and c
#[test]
fn a_captured_building_drops_its_target_with_no_draw() {
    let mut w = game();
    let tower = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 3, 0);
    arm(&mut w, tower, foe);
    let timer = w.obj(tower).unwrap().targeting_timer;
    let rng = w.rng.clone();
    ownership::change(&mut w, tower, Some(B), Cause::Capture);
    let l = left(&w, tower);
    assert_eq!(l.0, None, "the target is dropped");
    assert_eq!(l.4, timer, "the timer goes on from its value");
    assert_eq!(w.rng, rng, "no draw");
    assert!(guarded(&l), "guard requested: {l:?}");
}

// spec: S-0309 claim 30 (the drop does not depend on the target: a target the change makes an object of the holder's own house is dropped too); glossary building ownership change (a change to the house it already belongs to is no change and drops nothing)
#[test]
fn the_drop_is_whatever_the_target_and_none_without_a_change() {
    let mut w = game();
    let tower = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 3, 0);
    arm(&mut w, tower, foe);
    ownership::change(&mut w, tower, Some(A), Cause::Capture);
    assert_eq!(
        w.obj(tower).unwrap().target,
        Some(Target::Object(foe)),
        "no change, no drop"
    );
    // The change makes the target one of its own house's objects.
    ownership::change(&mut w, tower, Some(B), Cause::Trigger);
    assert_eq!(w.obj(tower).unwrap().target, None);
}

// spec: S-0309 claim 30 (the causes the claim lists, S-0508 claims 7 and 8, S-0509 claim 8 and S-0011 claim 24, clear the destination and request guard; the vehicle-like takeover of S-0509 claim 17 is not listed: drop only, design: Q-2747 interim)
#[test]
fn listed_causes_request_guard_the_takeover_drops_only() {
    for (cause, listed) in [
        (Cause::Capture, true),
        (Cause::Trigger, true),
        (Cause::Occupation, true),
        (Cause::Takeover, false),
    ] {
        let mut w = game();
        let tower = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
        let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 3, 0);
        arm(&mut w, tower, foe);
        let before = left(&w, tower);
        ownership::change(&mut w, tower, Some(B), cause);
        let l = left(&w, tower);
        assert_eq!(l.0, None, "{cause:?}: dropped");
        if listed {
            assert!(guarded(&l), "{cause:?}: guard requested");
        } else {
            assert_eq!((l.2, l.3), (before.2, before.3), "{cause:?}: behaviours kept");
        }
    }
}

/// A map whose tag `TAG` changes its carriers' owner to Beta (house value
/// 1) when variable-free event 8 (any event) fires.
fn change_owner_triggers() -> Triggers {
    let t = TriggerDef {
        events: vec![Event {
            code: 8,
            kind: 0,
            value: 0,
            type_name: None,
        }],
        actions: vec![Action {
            code: 14,
            number: 1,
            ..Default::default()
        }],
        ..TriggerDef::created(b"CHOWN".to_vec())
    };
    Triggers {
        triggers: vec![t],
        tags: vec![TagDef {
            id: b"TAG".to_vec(),
            repeat: 0,
            name: Vec::new(),
            first: Some(0),
            chain: vec![0],
        }],
        ..Default::default()
    }
}

// spec: S-0309 claim 30 (a trigger's change-owner action, S-0011 claim 24: an infantry unit and a vehicle drop their targets inside the change, with no draw, lose their destinations and are given guard as S-0601 claim 9 gives); scenario A19 part b
#[test]
fn a_trigger_owner_change_drops_target_and_destination() {
    for kind in [Kind::Infantry, Kind::Vehicle] {
        let mut w = game_on(&ruleset(), change_owner_triggers());
        let name = if kind == Kind::Infantry {
            "WALKER"
        } else {
            "CRUSHER"
        };
        let holder = put(&mut w, kind, name, 0, 0, 0);
        let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 3, 0);
        tag(&mut w, holder, "TAG");
        arm(&mut w, holder, foe);
        let timer = w.obj(holder).unwrap().targeting_timer;
        let rng = w.rng.clone();
        let tag_id = *w.triggers.objects.get(&holder).unwrap();
        glasnost_sim::triggers::raise(
            &mut w,
            tag_id,
            glasnost_sim::triggers::Raise {
                code: Some(8),
                causer: None,
                carrier: Some(glasnost_sim::triggers::Carrier::Object(holder)),
            },
        );
        assert_eq!(owner(&w, holder), Some(B), "{kind:?}: the action ran");
        let l = left(&w, holder);
        assert_eq!(l.0, None, "{kind:?}: the target is dropped");
        assert_eq!(l.1, None, "{kind:?}: the destination is cleared");
        assert!(guarded(&l), "{kind:?}: guard requested: {l:?}");
        assert_eq!(l.4, timer, "{kind:?}: the timer goes on");
        assert_eq!(w.rng, rng, "{kind:?}: no draw");
    }
}

// spec: S-0309 claim 30 (every cause drops the target, with no draw: an owner change by any other rule, here the plain change of `econ::events::owner_changed`, drops it too; the destination and guard belong to the listed causes only)
#[test]
fn any_other_owner_change_drops_the_target_only() {
    let mut w = game();
    let holder = put(&mut w, Kind::Vehicle, "CRUSHER", 0, 0, 0);
    let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 3, 0);
    arm(&mut w, holder, foe);
    let before = left(&w, holder);
    let rng = w.rng.clone();
    w.obj_mut(holder).unwrap().owner = Some(B);
    glasnost_sim::econ::events::owner_changed(&mut w, holder, Some(A));
    let l = left(&w, holder);
    assert_eq!(l.0, None);
    assert_eq!(l.1, before.1, "the destination is kept");
    assert_eq!(w.rng, rng, "no draw");
}

// spec: S-0508 claim 7 (r13: the owner change comes after the building's behaviour processing, its per-tick search and all its firing in its turn) and S-0309 claim 30 (the occupation's change drops the target the building's search took as the civilian house's); scenario G20 and play note 49
#[test]
fn an_occupied_civilian_building_drops_its_target_at_the_change() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "HUT", 2, 0, 0);
    let mut own = Vec::new();
    for k in 0..3 {
        own.push(put(&mut w, Kind::Infantry, "WALKER", 0, 3 + k, 3));
    }
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, -6);
    assert!(garrison::attempt(&mut w, u, b));
    let health: Vec<i32> = own.iter().map(|&x| w.obj(x).unwrap().health).collect();
    testkit::run(&mut w, 1);
    assert_eq!(owner(&w, b), Some(A), "the change at the building's turn");
    assert!(
        w.obj(b)
            .unwrap()
            .target
            .is_none_or(|t| !matches!(t, Target::Object(x) if own.contains(&x))),
        "no target of its new house's after the change"
    );
    testkit::run(&mut w, 300);
    let after: Vec<i32> = own.iter().map(|&x| w.obj(x).unwrap().health).collect();
    assert_eq!(after, health, "it never fires on its own house's infantry");
    assert_eq!(
        w.obj(b).unwrap().combat.experience,
        Default::default(),
        "no experience from allied kills"
    );
}

// spec: S-0309 claim 30 (the drop is inside the change itself: the owner change that S-0601 claims 6 and 9 make, through its every consumer, drops the target before the takeover's guard reset or the release's orders)
#[test]
fn a_mind_control_owner_change_drops_the_target_inside_the_change() {
    let mut w = game();
    let holder = put(&mut w, Kind::Vehicle, "CRUSHER", 0, 0, 0);
    let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 3, 0);
    arm(&mut w, holder, foe);
    let rng = w.rng.clone();
    glasnost_sim::mind::set_owner(&mut w, holder, Some(HouseId(1)));
    assert_eq!(w.obj(holder).unwrap().target, None, "dropped inside the change");
    assert_eq!(w.rng, rng, "no draw");
}

// spec: S-0309 claim 30 (a building whose animation-delayed shot, S-0301 claim 14, is pending at the change makes no firing check and no shot in the turn that ends the delay when it then holds no target, and the shot is no longer pending)
#[test]
fn a_pending_delayed_shot_ends_with_the_drop() {
    let mut w = game();
    let tower = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 3, 0);
    arm(&mut w, tower, foe);
    let t = w.tick;
    w.obj_mut(tower).unwrap().combat.delayed_shot = Some(t + 2);
    ownership::change(&mut w, tower, Some(B), Cause::Capture);
    // spec: S-0508 claim 7 (r13: a building's pending delayed shot stays pending across its owner change)
    assert_eq!(w.obj(tower).unwrap().combat.delayed_shot, Some(t + 2), "pending after the change");
    let health = w.obj(foe).unwrap().health;
    testkit::run(&mut w, 3);
    assert_eq!(w.obj(tower).unwrap().combat.delayed_shot, None, "no longer pending");
    assert_eq!(w.obj(foe).unwrap().health, health, "no shot");
}
