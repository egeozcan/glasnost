//! S-0519 claim 4 and S-0203 claim 6 (c) (#384): an infantry carrier that
//! its own timed bomb kills, and that stays in play while its death
//! sequence plays, takes no more of its living turn: no walking step and
//! no boarding. Synthetic rules (`tests/terror_common`).

#[path = "terror_common/mod.rs"]
mod terror_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{LocomotionClass, Locomotor, SequenceEntry, UnitType};
use glasnost_sim::bomb::Bomb;
use glasnost_sim::combat::damage;
use glasnost_sim::infantry::Seq;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::transport;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use terror_common::*;

/// DOOMED: a walker of strength 50 with a die-1 sequence; the bomb warhead's
/// infantry-death kind is 1, so a DOOMED killed by a bomb stays in play while
/// its death sequence plays. HAULER: a transport for four.
fn tweak(r: &mut Ruleset) {
    infantry(r, "DOOMED", |t| {
        t.strength = Some(50);
        for k in ["stand", "walk", "die_1", "die_2"] {
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
    });
    r.sim.warheads.get_mut("BOOM").unwrap().inf_death = Some(1);
    register(r, "vehicles", "HAULER");
    r.sim.vehicles.insert(
        "HAULER".into(),
        UnitType {
            strength: Some(100_000),
            speed: Some(6),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            passengers: Some(4),
            size_limit: Some(d("2")),
            ..Default::default()
        },
    );
}

/// A bomb with no planter that goes off in the carrier's turn of the next
/// step.
fn arm(w: &mut World, id: ObjectId) {
    let planted = w.tick - 51;
    w.obj_mut(id).unwrap().combat.bomb = Some(Bomb {
        planter: None,
        house: None,
        planted,
    });
}

fn dying(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        damage::dead(o) && o.infantry.as_ref().and_then(|s| s.seq) == Some(Seq::Die1)
    })
}

// spec: S-0519 claim 4 (the bomb is checked in its carrier's own turn) and S-0203 claim 6 (c) (a killed unit that stays in play only plays out its death sequence)
#[test]
fn a_carrier_killed_by_its_own_bomb_takes_no_walking_step_after_it() {
    let mut w = world(tweak);
    let v = spawn(&mut w, Kind::Infantry, "DOOMED", 0, at(0, 0));
    testkit::run(&mut w, 5);
    let order = Order::Behaviour {
        actor: v,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(at(8, 0))),
    };
    assert!(orders::issue(&mut w, HouseId(0), order));
    // Walk until it has been moving for a few ticks.
    let start = w.obj(v).unwrap().pos;
    for _ in 0..200 {
        testkit::run(&mut w, 1);
        let p = w.obj(v).unwrap().pos;
        if (p.x - start.x).abs() + (p.y - start.y).abs() > 300 {
            break;
        }
    }
    let a = w.obj(v).unwrap().pos;
    testkit::run(&mut w, 1);
    let b = w.obj(v).unwrap().pos;
    assert_ne!(a, b, "it is walking");
    arm(&mut w, v);
    testkit::run(&mut w, 1);
    assert!(dying(&w, v), "killed by its bomb, in its death sequence");
    let c = w.obj(v).unwrap().pos;
    // The bomb check comes before the movement in the turn, so any change of
    // position in this step was made after the kill.
    assert_eq!(b, c, "the dead unit took a walking step in its kill turn");
}

// spec: S-0519 claim 4 and S-0203 claim 6 (c) (a killed unit in its death sequence leaves play on the tick the sequence completes: it does not board in its kill turn)
#[test]
fn a_carrier_killed_by_its_own_bomb_does_not_board_in_that_turn() {
    let mut w = world(tweak);
    let t = spawn(&mut w, Kind::Vehicle, "HAULER", 0, at(0, 0));
    let p = spawn(&mut w, Kind::Infantry, "DOOMED", 0, at(3, 0));
    testkit::run(&mut w, 5);
    let r = targeting::click(
        &w,
        HouseId(0),
        &[p],
        ClickOn::Object(t),
        Modifiers::default(),
    );
    assert_eq!(r.orders.len(), 1, "an enter order");
    for o in r.orders.clone() {
        orders::issue(&mut w, HouseId(0), o);
    }
    // Find the step in which the living unit boards.
    let mut steps = None;
    let mut probe = w.clone();
    for n in 0..600 {
        if transport::riding(&probe, p) == Some(t) {
            steps = Some(n);
            break;
        }
        testkit::run(&mut probe, 1);
    }
    let steps = steps.expect("the living unit boards");
    testkit::run(&mut w, steps - 1);
    assert!(transport::riding(&w, p).is_none());
    arm(&mut w, p);
    testkit::run(&mut w, 1);
    let o = w.obj(p).expect("still in play");
    assert!(damage::dead(o), "killed by its bomb");
    assert_eq!(
        transport::riding(&w, p),
        None,
        "the dead unit boarded the transport in its kill turn"
    );
    testkit::run(&mut w, 300);
    assert!(w.obj(p).is_none(), "its death sequence completed");
    assert_eq!(transport::count(&w, t), 0, "no seat is held");
}
