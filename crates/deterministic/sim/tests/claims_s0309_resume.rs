//! S-0309 claim 23 (b) to (d): the attack-move refresh among the common
//! effects of a fall-back to rest, the end of the engaged mark, arrival and
//! the loss of a remembered target.

use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{AttackMoveMemory, ObjectId, Target, World};
use glasnost_sim::{objects, process};

const SIZE: i32 = 32;

/// A CART of house 0 at (0, 0) and an enemy CART at (`ex`, 0).
fn setup(ex: i32) -> (World, ObjectId, ObjectId) {
    let mut r = testkit::combat_ruleset();
    // No passive searches, so only the refresh changes the behaviour.
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .can_passive_aquire = Some(false);
    let mut w = testkit::game_with(&r, SIZE, 5);
    let a = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 0, 0), 64);
    let e = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, ex, 0), 192);
    // The placement fall-back used this tick's common effects; let a test's own fall-back use them.
    w.obj_mut(a).unwrap().beh.last_common_effects = None;
    (w, a, e)
}

// spec: S-0309 claim 23 (b) (engaged: resumed at a fall-back to rest; a destination: move with an immediate start, the memory's destination the object's)
// spec: S-0309 claim 23 (c) (resuming ends the engaged mark)
#[test]
fn engaged_with_a_destination_resumes_the_move() {
    let (mut w, a, _) = setup(12);
    let c = Target::Cell(mid(SIZE, 8, 0));
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Dest(c));
    o.combat.engaged = true;
    o.beh.force(Behaviour::Attack, t, true);
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert!(!o.combat.engaged);
    assert_eq!(o.beh.current, Some(Behaviour::Move));
    assert_eq!(o.dest, Some(c));
    assert_eq!(o.attack_move, Some(AttackMoveMemory::Dest(c)));
}

// spec: S-0309 claim 23 (b) (not engaged and in guard: resumed; only a target: attack with an immediate start, the memory's target the object's)
#[test]
fn guard_with_a_target_memory_resumes_the_attack() {
    let (mut w, a, e) = setup(12);
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Target(Target::Object(e)));
    o.combat.engaged = false;
    o.beh.force(Behaviour::Guard, t, true);
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
    assert_eq!(o.target, Some(Target::Object(e)));
}

// spec: S-0309 claim 23 (b) (not engaged and not in guard: the memory is removed, and the rest table judges the state after the refresh)
// spec: S-0215 claim 15 row 1 (applies only if the memory survives)
#[test]
fn not_engaged_outside_guard_removes_the_memory() {
    let (mut w, a, _) = setup(12);
    let c = Target::Cell(mid(SIZE, 8, 0));
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Dest(c));
    o.beh.force(Behaviour::Stop, t, true);
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.attack_move, None);
    assert!(!o.combat.engaged);
    assert_eq!(o.dest, None);
    assert!(
        matches!(o.beh.queued, Some(Behaviour::Guard | Behaviour::AreaGuard)),
        "the ordinary rest rules queue guard or area guard: {:?}",
        o.beh.queued
    );
}

// spec: S-0309 claim 23 (b) (the refresh at most once per actor per tick: a second fall-back on the same tick leaves the memory)
// spec: S-0215 claim 13 (the common effects at most once per actor per tick)
#[test]
fn refresh_at_most_once_per_tick() {
    let (mut w, a, _) = setup(12);
    let c = Target::Cell(mid(SIZE, 8, 0));
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Dest(c));
    o.beh.force(Behaviour::Stop, t, true);
    o.beh.last_common_effects = Some(t);
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.attack_move, Some(AttackMoveMemory::Dest(c)));
    assert_eq!(o.beh.queued, None, "row 1: nothing changes");
}

// spec: S-0309 claim 23 (c) (an engagement survives later tick starts: the refresh is not part of the per-tick start)
#[test]
fn engagement_survives_tick_starts() {
    let (mut w, a, e) = setup(3);
    let c = Target::Cell(mid(SIZE, 8, 0));
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Dest(c));
    o.combat.engaged = true;
    o.target = Some(Target::Object(e));
    o.beh.force(Behaviour::Attack, t, true);
    testkit::run(&mut w, 3);
    let o = w.obj(a).unwrap();
    assert!(o.combat.engaged);
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
    assert_eq!(o.attack_move, Some(AttackMoveMemory::Dest(c)));
}

// spec: S-0309 claim 23 (d) (arrival: the next move processing falls back to rest, the memory is removed and guard or area guard is queued; the object does not stay in move with the memory)
#[test]
fn arrival_removes_the_memory() {
    let (mut w, a, _) = setup(30);
    let here = Target::Cell(mid(SIZE, 0, 0));
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Dest(here));
    o.dest = None;
    o.beh.force(Behaviour::Move, t, true);
    o.beh.queued = None;
    let mut removed = false;
    for _ in 0..8 {
        testkit::run(&mut w, 1);
        if w.obj(a).unwrap().attack_move.is_none() {
            removed = true;
            break;
        }
    }
    assert!(removed, "the memory is removed at the arrival fall-back");
    testkit::run(&mut w, 2);
    let o = w.obj(a).unwrap();
    assert_eq!(o.attack_move, None);
    assert!(
        matches!(o.beh.current, Some(Behaviour::Guard | Behaviour::AreaGuard))
            || matches!(o.beh.queued, Some(Behaviour::Guard | Behaviour::AreaGuard)),
        "rests in guard or area guard: {:?} {:?}",
        o.beh.current,
        o.beh.queued
    );
}

// spec: S-0309 claim 23 (d) (removal of a remembered target turns the memory into a destination, the cell of the target's last position; the engaged mark is kept)
#[test]
fn removed_target_becomes_its_last_cell() {
    let (mut w, a, e) = setup(6);
    let last = w.obj(e).unwrap().pos.cell();
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Target(Target::Object(e)));
    o.combat.engaged = true;
    objects::remove(&mut w, e);
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.attack_move,
        Some(AttackMoveMemory::Dest(Target::Cell(last)))
    );
    assert!(o.combat.engaged);
    // A fall-back while engaged resumes move to that cell.
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Move));
    assert_eq!(o.dest, Some(Target::Cell(last)));
}

/// A WALKER of house 0 at (0, 0) with a guard spot two cells away, and an
/// enemy CART at (12, 0).
fn infantry_setup() -> (World, ObjectId, ObjectId, Target) {
    let r = testkit::combat_ruleset();
    let mut w = testkit::game_with(&r, SIZE, 5);
    let a = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 64);
    let e = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 12, 0), 192);
    let spot = Target::Cell(mid(SIZE, 0, 2));
    let o = w.obj_mut(a).unwrap();
    o.beh.last_common_effects = None;
    o.guard_spot = Some(spot);
    (w, a, e, spot)
}

// spec: S-0215 claim 13 (Q-0202: the refresh precedes the guard-spot row, which replaces a resumed destination; a resumed move that started stays current, now to the guard spot, and the guard spot is forgotten)
#[test]
fn guard_spot_row_replaces_a_resumed_destination() {
    let (mut w, a, _, spot) = infantry_setup();
    let c = Target::Cell(mid(SIZE, 8, 0));
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Dest(c));
    o.combat.engaged = true;
    o.beh.force(Behaviour::Attack, t, true);
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Move));
    assert_eq!(o.beh.queued, None, "move is already current (claim 7)");
    assert_eq!(o.dest, Some(spot));
    assert_eq!(o.guard_spot, None);
    assert_eq!(o.attack_move, Some(AttackMoveMemory::Dest(c)));
}

// spec: S-0215 claim 13 (Q-0202: with target-only memory the resumed attack target is kept; an already-started attack stays current with move queued)
#[test]
fn guard_spot_row_after_a_resumed_attack_queues_move() {
    let (mut w, a, e, spot) = infantry_setup();
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Target(Target::Object(e)));
    o.combat.engaged = false;
    o.beh.force(Behaviour::Guard, t, true);
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
    assert_eq!(o.beh.queued, Some(Behaviour::Move));
    assert_eq!(o.target, Some(Target::Object(e)));
    assert_eq!(o.dest, Some(spot));
    assert_eq!(o.guard_spot, None);
}

// Sim issues S11, #229: the resumption's immediate start depends on
// readiness.

// spec: S-0309 claim 23 (b) (a destination: move queued with an immediate start) and S-0215 claim 7 ("with an immediate start" is the readiness-dependent start); S-0215 claim 11 (a vehicle moving outside guard is not ready)
#[test]
fn issue_229_a_vehicle_that_is_not_ready_keeps_the_resumed_move_queued() {
    let (mut w, a, e) = setup(12);
    let c = Target::Cell(mid(SIZE, 8, 0));
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Dest(c));
    o.combat.engaged = true;
    o.target = Some(Target::Object(e));
    o.beh.force(Behaviour::Attack, t, true);
    // Its body is still turning: moving, in attack with a target.
    o.body.t0 = t;
    o.body.duration = 8;
    o.body.step = 1;
    assert!(!process::ready(&w, a), "set-up: the vehicle is not ready");
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(c), "the memory's destination is the object's");
    assert_eq!(
        (o.beh.current, o.beh.queued),
        (Some(Behaviour::Attack), Some(Behaviour::Move)),
        "attack stays current and move stays queued"
    );
}

// spec: S-0309 claim 23 (b) (only a target: attack queued with an immediate start) and S-0215 claim 13 (Q-0202: the resumed attack is current if it could start, queued otherwise); S-0215 r118 claim 11 (infantry in the middle of firing is not ready)
#[test]
fn issue_229_an_infantry_unit_that_is_not_ready_keeps_the_resumed_attack_queued() {
    let r = testkit::combat_ruleset();
    let mut w = testkit::game_with(&r, SIZE, 5);
    let a = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 64);
    let e = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 12, 0), 192);
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.beh.last_common_effects = None;
    o.attack_move = Some(AttackMoveMemory::Target(Target::Object(e)));
    o.combat.engaged = false;
    o.beh.force(Behaviour::Guard, t, true);
    o.infantry.as_mut().unwrap().attack = true;
    assert!(!process::ready(&w, a), "set-up: the unit is not ready");
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, Some(Target::Object(e)));
    assert_eq!(
        (o.beh.current, o.beh.queued),
        (Some(Behaviour::Guard), Some(Behaviour::Attack)),
        "guard stays current and attack stays queued"
    );
}

// spec: S-0215 claim 13 (Q-0202: if readiness prevented the resumed start, move remains queued and the previous behaviour remains current; the guard-spot row then replaces the destination and the guard spot is forgotten)
#[test]
fn issue_229_guard_spot_row_after_a_resumed_move_that_did_not_start() {
    let (mut w, a, _, spot) = infantry_setup();
    let c = Target::Cell(mid(SIZE, 8, 0));
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.attack_move = Some(AttackMoveMemory::Dest(c));
    o.combat.engaged = true;
    o.beh.force(Behaviour::Attack, t, true);
    o.infantry.as_mut().unwrap().attack = true;
    assert!(!process::ready(&w, a), "set-up: the unit is not ready");
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(
        (o.beh.current, o.beh.queued),
        (Some(Behaviour::Attack), Some(Behaviour::Move)),
        "the previous behaviour stays current and move stays queued"
    );
    assert_eq!(o.dest, Some(spot));
    assert_eq!(o.guard_spot, None);
}
