//! End-to-end runs of the synthetic scenario: orders move units, and the
//! state hash survives a save/load round trip.

use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::save;
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, Target};

#[test]
fn map_objects_are_placed_and_rest() {
    let mut w = testkit::new_game(7);
    assert_eq!(w.objects.len(), 11, "notes: {:?}", w.notes);
    for _ in 0..5 {
        tick::step(&mut w);
    }
    // spec: S-0215 claim 15 (an armed vehicle with skill level 0 and a threshold of 0 rests in area guard)
    for o in w.objects.values() {
        assert!(o.beh.current.is_some(), "{:?} has no behaviour", o.id);
    }
}

// spec: S-0213 claim 1 (an order issued in tick N is applied after the turn pass of tick N)
// spec: S-0201 claim 14 (orders are blocked during ticks 0 to 3)
#[test]
fn orders_are_blocked_then_applied_between_ticks() {
    let mut w = testkit::new_game(7);
    let id = testkit::house_objects(&w, 0)[0];
    let o = Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(testkit::cell(4, 0))),
    };
    assert!(!orders::issue(&mut w, HouseId(0), o.clone()));
    while w.tick < 4 {
        tick::step(&mut w);
    }
    assert!(orders::issue(&mut w, HouseId(0), o));
    tick::step(&mut w);
    assert_eq!(w.obj(id).unwrap().beh.queued, Some(Behaviour::Move));
}

#[test]
fn a_vehicle_drives_to_its_destination() {
    let mut w = testkit::new_game(7);
    let id = testkit::house_objects(&w, 0)
        .into_iter()
        .find(|&i| w.obj(i).unwrap().kind == Kind::Vehicle)
        .unwrap();
    while w.tick < 4 {
        tick::step(&mut w);
    }
    let dest = testkit::cell(-6, 6);
    orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(dest)),
        },
    );
    for _ in 0..600 {
        tick::step(&mut w);
    }
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos.cell(), dest, "notes: {:?}", w.notes);
}

#[test]
fn an_infantry_unit_walks_to_its_destination() {
    let mut w = testkit::new_game(7);
    let id = testkit::house_objects(&w, 1)
        .into_iter()
        .find(|&i| w.obj(i).unwrap().kind == Kind::Infantry)
        .unwrap();
    while w.tick < 4 {
        tick::step(&mut w);
    }
    let dest = testkit::cell(2, 8);
    orders::issue(
        &mut w,
        HouseId(1),
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(dest)),
        },
    );
    for _ in 0..900 {
        tick::step(&mut w);
    }
    assert_eq!(w.obj(id).unwrap().pos.cell(), dest, "notes: {:?}", w.notes);
}

#[test]
fn save_load_round_trip_keeps_the_hash_and_the_future() {
    let rep = testkit::scripted_replay(11, 400);
    let mut w = rep.run(testkit::rules(), &testkit::map(), testkit::catalogue(), |_| {}).unwrap();
    let h = save::state_hash(&w);
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, testkit::rules(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
    for _ in 0..300 {
        tick::step(&mut w);
        tick::step(&mut l);
        assert_eq!(save::state_hash(&w), save::state_hash(&l), "tick {}", w.tick);
    }
}

#[test]
fn a_replay_runs_the_same_twice() {
    let rep = testkit::scripted_replay(5, 500);
    let bytes = rep.to_bytes();
    let rep2 = save::Replay::from_bytes(&bytes).unwrap();
    let mut a = Vec::new();
    let mut b = Vec::new();
    rep.run(testkit::rules(), &testkit::map(), testkit::catalogue(), |w| a.push(save::state_hash(w))).unwrap();
    rep2.run(testkit::rules(), &testkit::map(), testkit::catalogue(), |w| b.push(save::state_hash(w))).unwrap();
    assert_eq!(a, b);
}
