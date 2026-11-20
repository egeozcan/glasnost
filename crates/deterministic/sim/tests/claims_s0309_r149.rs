//! Claim tests of S-0309 revision 149 (Q-3298 answered, #453), claim 32: a
//! target that changes owner is dropped by every holder, at that moment,
//! inside the change, whatever the holder's house and however it came to
//! hold it; an overridden holder reverts first; no targeting-timer draw;
//! behaviour, destination, put-aside slots and attack-move memory are kept;
//! an attacker whose strike target is the living object keeps it. The
//! mind-control parts (scenario A20 parts a, e, f and g) are in
//! `claims_s0601.rs`. On the synthetic rules of `structures_common`:
//! invented types and numbers, no game data. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S44*.

mod structures_common;

use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::entry;
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::world::{AttackMoveMemory, ObjectId, Target, World};
use structures_common::*;

/// Gives `holder` the target `x`, the behaviour `b`, a destination cell for
/// a mobile holder and a targeting timer 30 ticks ahead (more than claim
/// 27's 10 ticks left, so a claim 27 drop would draw).
fn arm(w: &mut World, holder: ObjectId, x: ObjectId, b: Behaviour) {
    let tick = w.tick;
    let o = w.obj_mut(holder).unwrap();
    o.beh.force(b, tick, false);
    o.target = Some(Target::Object(x));
    if o.kind != Kind::Building {
        o.dest = Some(Target::Cell(at(5, 5)));
    }
    o.targeting_timer = tick + 30;
}

fn target(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).and_then(|o| o.target)
}

/// Holders of every kind and of three houses (Alpha, Beta and the neutral
/// civilian house), each armed with `x`.
fn holders(w: &mut World, x: ObjectId) -> Vec<ObjectId> {
    let mut v = Vec::new();
    for (i, house) in [0u8, 1, 2].into_iter().enumerate() {
        let row = 6 + i as i32;
        v.push(put(w, Kind::Infantry, "WALKER", house, -6, row));
        v.push(put(w, Kind::Vehicle, "CRUSHER", house, -3, row));
    }
    v.push(put(w, Kind::Building, "TOWER", 1, -8, -6));
    for &h in &v {
        arm(w, h, x, Behaviour::Guard);
    }
    v
}

// spec: S-0309 r149 claim 32 (every infantry unit, vehicle and building that holds it drops it at that moment, inside the change, whatever the holder's owner: of the new owner, of the old owner, of any other house, human- or computer-controlled; *No timer draw*: the timer counts on and nothing is drawn; *Behaviour and destination*: neither changes); scenario A20 part d for a trigger's change
#[test]
fn every_holder_drops_a_target_that_changes_owner() {
    let mut w = game();
    let x = put(&mut w, Kind::Infantry, "WALKER", 2, 3, 0);
    let hs = holders(&mut w, x);
    let before: Vec<_> = hs
        .iter()
        .map(|&h| {
            let o = w.obj(h).unwrap();
            (o.targeting_timer, o.dest, o.beh.current, o.beh.queued)
        })
        .collect();
    let rng = w.rng.clone();
    glasnost_sim::mind::set_owner(&mut w, x, Some(B));
    assert_eq!(owner(&w, x), Some(B), "set-up: changed");
    for (&h, b) in hs.iter().zip(before) {
        let o = w.obj(h).unwrap();
        let case = format!("{:?} of {:?}", o.kind, o.owner);
        assert_eq!(o.target, None, "{case}: dropped");
        assert_eq!(
            (o.targeting_timer, o.dest, o.beh.current, o.beh.queued),
            b,
            "{case}: timer, destination and behaviours kept"
        );
    }
    assert_eq!(w.rng, rng, "no draw");
}

// spec: S-0309 r149 claim 32 (a change to the house the object already belongs to is no change and drops nothing)
#[test]
fn no_change_drops_nothing() {
    let mut w = game();
    let x = put(&mut w, Kind::Building, "TARGET", 1, 3, 0);
    let hs = holders(&mut w, x);
    ownership::change(&mut w, x, Some(B), Cause::Trigger);
    for &h in &hs {
        assert_eq!(target(&w, h), Some(Target::Object(x)), "{h:?}");
    }
}

// spec: S-0309 r149 claim 32 (an engineer capture of a building, S-0509 claim 8: every holder of the building drops it, the capturer's house's, the old owner's and a third house's alike); scenario A20 part c
#[test]
fn an_engineer_capture_drops_the_building_for_every_holder() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    testkit_run(&mut w, 8);
    let hs = holders(&mut w, b);
    let timers: Vec<i32> = hs
        .iter()
        .map(|&h| w.obj(h).unwrap().targeting_timer)
        .collect();
    let c = w.obj(b).unwrap().pos.cell();
    let e = glasnost_sim::testkit::spawn(&mut w, Kind::Infantry, "ENGI", 0, c, 0);
    let tick = w.tick;
    let o = w.obj_mut(e).unwrap();
    o.dest = Some(Target::Object(b));
    o.beh.force(Behaviour::Capture, tick, true);
    assert!(entry::arrival(&mut w, e), "set-up: the engineer goes in");
    assert_eq!(owner(&w, b), Some(A), "set-up: captured");
    for (&h, timer) in hs.iter().zip(timers) {
        let o = w.obj(h).unwrap();
        assert_eq!(o.target, None, "{:?} of {:?}: dropped", o.kind, o.owner);
        assert_eq!(o.targeting_timer, timer, "no draw");
    }
}

/// A holder of Beta with an override in effect (S-0215 claim 10): *attack*
/// on `x` current, *guard* suspended, `aside` put aside as its target and
/// no destination put aside.
fn overridden(w: &mut World, x: ObjectId, aside: Option<Target>) -> ObjectId {
    let h = put(w, Kind::Vehicle, "CRUSHER", 1, -3, 0);
    arm(w, h, x, Behaviour::Attack);
    let o = w.obj_mut(h).unwrap();
    o.beh.suspended = Some(Behaviour::Guard);
    o.beh.aside = Some((aside, None));
    h
}

// spec: S-0309 r149 claim 32 (*Override in effect*: the holder first reverts, as S-0215 claim 10's owner-change cause gives, then drops the object if the revert has left it as its target; a revert that gives back another target keeps that target); scenario A20 part b for the revert
#[test]
fn an_overridden_holder_reverts_then_drops() {
    let mut w = game();
    let x = put(&mut w, Kind::Infantry, "WALKER", 2, 3, 0);
    let y = put(&mut w, Kind::Infantry, "WALKER", 2, 3, 3);
    let none = overridden(&mut w, x, None);
    let other = overridden(&mut w, x, Some(Target::Object(y)));
    let same = overridden(&mut w, x, Some(Target::Object(x)));
    glasnost_sim::mind::set_owner(&mut w, x, Some(A));
    for (h, kept, case) in [
        (none, None, "nothing put aside"),
        (other, Some(Target::Object(y)), "another target put aside"),
        (same, None, "the object itself put aside"),
    ] {
        let o = w.obj(h).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Guard), "{case}: reverted");
        assert_eq!(o.beh.suspended, None, "{case}: the override ended");
        assert_eq!(o.target, kept, "{case}");
    }
}

// spec: S-0309 r149 claim 32 (*Override in effect*: a holder whose current target is another object and that has this object only as its put-aside target drops nothing; the change takes nothing out of the put-aside slots, claim 27a, so a later revert gives the object back)
#[test]
fn a_put_aside_target_is_kept() {
    let mut w = game();
    let x = put(&mut w, Kind::Infantry, "WALKER", 2, 3, 0);
    let y = put(&mut w, Kind::Infantry, "WALKER", 2, 3, 3);
    let h = overridden(&mut w, y, Some(Target::Object(x)));
    glasnost_sim::mind::set_owner(&mut w, x, Some(A));
    let o = w.obj(h).unwrap();
    assert_eq!(o.target, Some(Target::Object(y)), "the current target kept");
    assert_eq!(o.beh.current, Some(Behaviour::Attack), "no revert");
    assert_eq!(
        o.beh.aside,
        Some((Some(Target::Object(x)), None)),
        "slot kept"
    );
}

// spec: S-0309 r149 claim 32 (*Memory*: the change leaves every attack-move memory as it is; a memory whose remembered target is the object still names it)
#[test]
fn an_attack_move_memory_is_kept() {
    let mut w = game();
    let x = put(&mut w, Kind::Infantry, "WALKER", 2, 3, 0);
    let h = put(&mut w, Kind::Vehicle, "CRUSHER", 1, -3, 0);
    arm(&mut w, h, x, Behaviour::Attack);
    let memory = Some(AttackMoveMemory::Target(Target::Object(x)));
    w.obj_mut(h).unwrap().attack_move = memory;
    glasnost_sim::mind::set_owner(&mut w, x, Some(A));
    assert_eq!(target(&w, h), None, "dropped");
    assert_eq!(w.obj(h).unwrap().attack_move, memory, "the memory kept");
}

// spec: S-0309 r149 claim 32 (*Strike target*: an attacker whose strike target, S-0519 Terms, is the object keeps it while the object is alive, as S-0215 claim 10 gives for the revert; S-0215 claim 10: the exception also holds after a strike ends, as it keeps its strike target)
#[test]
fn a_strike_target_is_kept_while_alive() {
    let mut w = game();
    let x = put(&mut w, Kind::Infantry, "WALKER", 2, 3, 0);
    let d = put(&mut w, Kind::Infantry, "WALKER", 1, -3, 0);
    let plain = put(&mut w, Kind::Infantry, "WALKER", 1, -3, 2);
    arm(&mut w, d, x, Behaviour::Attack);
    arm(&mut w, plain, x, Behaviour::Attack);
    w.strike.strikes.insert(
        d,
        glasnost_sim::strike::Strike {
            target: Some(x),
            ended: true,
            ..Default::default()
        },
    );
    glasnost_sim::mind::set_owner(&mut w, x, Some(A));
    assert_eq!(
        target(&w, d),
        Some(Target::Object(x)),
        "the designator keeps it"
    );
    assert_eq!(target(&w, plain), None, "another attacker drops it");
}

fn testkit_run(w: &mut World, n: i32) {
    glasnost_sim::testkit::run(w, n);
}
