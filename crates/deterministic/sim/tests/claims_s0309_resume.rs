//! S-0309 claim 23 (b) and (c): resuming an attack move at the start of a
//! tick, and the end of the engaged mark.

use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{AttackMoveMemory, ObjectId, Target, World};

const SIZE: i32 = 32;

fn setup() -> (World, ObjectId, ObjectId) {
    let mut r = testkit::combat_ruleset();
    // No passive searches, so only the resumption changes the behaviour.
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .can_passive_aquire = Some(false);
    let mut w = testkit::game_with(&r, SIZE, 5);
    let a = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 0, 0), 64);
    let e = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 12, 0), 192);
    (w, a, e)
}

// spec: S-0309 claim 23 (b) (engaged: resumed; a destination: move with an immediate start, the memory's destination the object's)
// spec: S-0309 claim 23 (c) (resuming ends the engaged mark)
#[test]
fn engaged_with_a_destination_resumes_the_move() {
    let (mut w, a, _) = setup();
    let c = Target::Cell(mid(SIZE, 8, 0));
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Dest(c));
    o.combat.engaged = true;
    o.beh.force(Behaviour::Attack, t, true);
    testkit::run(&mut w, 1);
    let o = w.obj(a).unwrap();
    assert!(!o.combat.engaged);
    assert_eq!(o.beh.current, Some(Behaviour::Move));
    assert_eq!(o.dest, Some(c));
    assert_eq!(o.attack_move, Some(AttackMoveMemory::Dest(c)));
}

// spec: S-0309 claim 23 (b) (not engaged and in guard: resumed; only a target: attack with an immediate start, the memory's target the object's)
#[test]
fn guard_with_a_target_memory_resumes_the_attack() {
    let (mut w, a, e) = setup();
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Target(Target::Object(e)));
    o.combat.engaged = false;
    testkit::run(&mut w, 1);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
    assert_eq!(o.target, Some(Target::Object(e)));
}

// spec: S-0309 claim 23 (b) (not engaged and not in guard: nothing is resumed)
#[test]
fn not_engaged_outside_guard_is_not_resumed() {
    let (mut w, a, _) = setup();
    let c = Target::Cell(mid(SIZE, 8, 0));
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Dest(c));
    o.beh.force(Behaviour::Stop, t, true);
    testkit::run(&mut w, 1);
    let o = w.obj(a).unwrap();
    assert_ne!(o.beh.current, Some(Behaviour::Move));
    assert_eq!(o.dest, None);
}
