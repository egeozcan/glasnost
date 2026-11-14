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
    assert_eq!(w.objects.len(), 22, "notes: {:?}", w.notes);
    for _ in 0..5 {
        tick::step(&mut w);
    }
    // spec: S-0215 claims 14 and 15 (every mobile object gets a resting behaviour from its placement fall-back)
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
    // Save in the middle of the replay; the saved and the loaded game then
    // take the replay's remaining orders and stay identical.
    let mut rep = testkit::scripted_replay(11, 400);
    let mut w = rep
        .run(
            testkit::rules(),
            &testkit::map(),
            testkit::catalogue(),
            |_| {},
        )
        .unwrap();
    let h = save::state_hash(&w);
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, testkit::rules(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
    rep.ticks = 1200;
    let mut a = Vec::new();
    let mut b = Vec::new();
    rep.run_on(&mut w, rep.ticks, |w| a.push(save::state_hash(w)));
    rep.run_on(&mut l, rep.ticks, |w| b.push(save::state_hash(w)));
    assert_eq!(a.len(), 800);
    assert_eq!(a, b);
}

/// A replay hands each tick's effects to its callback and then takes them,
/// as a drawing driver does (#398): the callback sees what a driver that
/// drains the list after every tick sees, and the list never piles up.
#[test]
fn a_replay_takes_each_ticks_effects() {
    let ticks = 400;
    let rep = testkit::combat_replay(7, ticks);
    let mut seen = Vec::new();
    let mut hashes = Vec::new();
    let w = rep
        .run(
            testkit::combat_rules(),
            &testkit::map(),
            testkit::catalogue(),
            |w| {
                seen.push(w.effects.len());
                hashes.push(save::state_hash(w));
            },
        )
        .unwrap();
    assert!(
        w.effects.is_empty(),
        "the world keeps no effect after the run"
    );
    // The same game stepped by hand, draining after every tick.
    let mut v = testkit::combat_game(7);
    let mut want = Vec::new();
    let mut want_hashes = Vec::new();
    while tick::ticks_run(&v) < ticks && !v.econ.end.finished {
        rep.issue_due(&mut v);
        tick::step(&mut v);
        want.push(v.effects.len());
        want_hashes.push(save::state_hash(&v));
        v.effects.clear();
    }
    assert_eq!(hashes, want_hashes);
    assert_eq!(seen, want);
    assert!(seen.iter().sum::<usize>() > 0, "the fight makes effects");
}

#[test]
fn save_load_mid_combat_keeps_the_hash_and_the_future() {
    // Save while shots are in flight and reload timers run; the saved and the
    // loaded game then fight on identically.
    let mut rep = testkit::combat_replay(17, 140);
    let mut w = rep
        .run(
            testkit::combat_rules(),
            &testkit::map(),
            testkit::catalogue(),
            |_| {},
        )
        .unwrap();
    let mut extra = 0;
    while w.projectiles.is_empty() && extra < 200 {
        tick::step(&mut w);
        extra += 1;
    }
    assert!(!w.projectiles.is_empty(), "no shot in flight to save");
    let h = save::state_hash(&w);
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, testkit::combat_rules(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
    rep.ticks = 1400;
    let mut a = Vec::new();
    let mut b = Vec::new();
    rep.run_on(&mut w, rep.ticks, |w| a.push(save::state_hash(w)));
    rep.run_on(&mut l, rep.ticks, |w| b.push(save::state_hash(w)));
    assert!(!a.is_empty());
    assert_eq!(a, b);
    assert!(
        w.objects.len() < testkit::combat_game(17).objects.len(),
        "nobody died in the combat replay"
    );
}

// spec: S-0210 claim 3 (a crusher crushes at the centre of the cell it reaches)
// spec: S-0210 claim 4 (a crushed object is removed on the same tick)
#[test]
fn the_crusher_crushes_the_buggy() {
    let rep = testkit::scripted_replay(11, 900);
    let w0 = testkit::new_game(11);
    let buggy = w0
        .objects
        .values()
        .find(|o| o.kind == Kind::Vehicle && o.type_index == 2)
        .unwrap()
        .id;
    let w = rep
        .run(
            testkit::rules(),
            &testkit::map(),
            testkit::catalogue(),
            |_| {},
        )
        .unwrap();
    assert!(
        w.obj(buggy).is_none(),
        "the buggy survived; notes: {:?}",
        w.notes
    );
}

#[test]
fn a_replay_runs_the_same_twice() {
    let rep = testkit::scripted_replay(5, 500);
    let bytes = rep.to_bytes();
    let rep2 = save::Replay::from_bytes(&bytes).unwrap();
    let mut a = Vec::new();
    let mut b = Vec::new();
    rep.run(
        testkit::rules(),
        &testkit::map(),
        testkit::catalogue(),
        |w| a.push(save::state_hash(w)),
    )
    .unwrap();
    rep2.run(
        testkit::rules(),
        &testkit::map(),
        testkit::catalogue(),
        |w| b.push(save::state_hash(w)),
    )
    .unwrap();
    assert_eq!(a, b);
}
