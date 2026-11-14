//! Claim tests of S-0309 revision 140 (Q-3061 answered, #163): claim 27a,
//! a put-aside object that leaves play, and claim 23 (d), the attack-move
//! memory converted at the loss of its remembered target. On the synthetic
//! rules of `cloak_common`: invented types and numbers, no game data.

mod cloak_common;

use cloak_common::*;
use glasnost_sim::objects;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::world::{AttackMoveMemory, ObjectId, Target, World};

/// How the put-aside object leaves play in a test.
#[derive(Clone, Copy, Debug)]
enum Leave {
    Kill,
    OffMap,
    Remove,
}

fn leave(w: &mut World, x: ObjectId, how: Leave) {
    match how {
        Leave::Kill => kill(w, x),
        Leave::OffMap => objects::take_inside(w, x),
        Leave::Remove => objects::remove(w, x),
    }
}

/// Gives `h` an override in effect: *attack* current with `target`,
/// *guard* suspended, `aside` put aside.
fn overridden(
    w: &mut World,
    h: ObjectId,
    target: Option<Target>,
    aside: (Option<Target>, Option<Target>),
) {
    let o = w.obj_mut(h).unwrap();
    o.target = target;
    o.beh.current = Some(Behaviour::Attack);
    o.beh.queued = None;
    o.beh.suspended = Some(Behaviour::Guard);
    o.beh.aside = Some(aside);
}

// spec: S-0309 claim 27a (r140, Q-3061: Every other holder, one whose current target is another object, loses the object from its put-aside target inside the action in which it is destroyed, goes off the map or is removed; a put-aside destination that is a cell is kept; no timer draw; the override stays in effect)
#[test]
fn claim_27a_put_aside_target_lost_while_the_override_goes_on() {
    let r = ruleset();
    for how in [Leave::Kill, Leave::OffMap, Leave::Remove] {
        let mut w = game(&r);
        let x = spawn(&mut w, "PLAIN", BETA, 4, 0);
        let y = spawn(&mut w, "CART", BETA, 0, 4);
        let h = spawn(&mut w, "PLAIN", ALPHA, 0, 0);
        let cell = Target::Cell(at(2, 2));
        overridden(
            &mut w,
            h,
            Some(Target::Object(y)),
            (Some(Target::Object(x)), Some(cell)),
        );
        let timer = w.obj(h).unwrap().targeting_timer;
        let taken = w.rng.taken();
        leave(&mut w, x, how);
        let o = w.obj(h).unwrap();
        assert_eq!(o.beh.aside, Some((None, Some(cell))), "{how:?}");
        assert_eq!(o.target, Some(Target::Object(y)), "{how:?}: still holds Y");
        assert_eq!(o.beh.suspended, Some(Behaviour::Guard), "{how:?}: no revert");
        assert_eq!(o.targeting_timer, timer, "{how:?}: no timer draw");
        if matches!(how, Leave::OffMap) {
            assert_eq!(w.rng.taken(), taken, "no draw at all");
        }
    }
}

// spec: S-0309 claim 27a (r140, Q-3061: a put-aside destination that is the object is lost too; a put-aside target that is another object is kept)
#[test]
fn claim_27a_put_aside_destination_that_is_the_object() {
    let r = ruleset();
    for how in [Leave::Kill, Leave::OffMap, Leave::Remove] {
        let mut w = game(&r);
        let x = spawn(&mut w, "PLAIN", BETA, 4, 0);
        let y = spawn(&mut w, "CART", BETA, 0, 4);
        let z = spawn(&mut w, "CART", BETA, -4, 0);
        let h = spawn(&mut w, "PLAIN", ALPHA, 0, 0);
        overridden(
            &mut w,
            h,
            Some(Target::Object(y)),
            (Some(Target::Object(z)), Some(Target::Object(x))),
        );
        leave(&mut w, x, how);
        let o = w.obj(h).unwrap();
        assert_eq!(o.beh.aside, Some((Some(Target::Object(z)), None)), "{how:?}");
    }
}

// spec: S-0309 claim 27a (r140, Q-3061: Order within the action: a holder whose current target is the object has claim 27's drop and revert first, so the revert gives the object back as its target; a put-aside destination that is the object becomes its destination again and is lost from there later in the same action, so the holder is left with no destination)
#[test]
fn claim_27a_order_within_the_action() {
    let r = ruleset();
    let mut w = game(&r);
    let x = spawn(&mut w, "PLAIN", BETA, 4, 0);
    let h = spawn(&mut w, "PLAIN", ALPHA, 0, 0);
    overridden(
        &mut w,
        h,
        Some(Target::Object(x)),
        (Some(Target::Object(x)), Some(Target::Object(x))),
    );
    objects::take_inside(&mut w, x);
    let o = w.obj(h).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "reverted");
    assert_eq!(o.beh.suspended, None);
    assert_eq!(o.target, Some(Target::Object(x)), "claim 27: given back");
    assert_eq!(o.dest, None, "the restored destination is lost");
    assert_eq!(o.beh.aside, None);
}

// spec: S-0309 claim 23 (d) (r140, Q-3061: the loss of a remembered target, at the killing hit, going off the map or removal, turns the memory into a remembered destination at the cell of the target's position at that loss) and claim 27a (A holder's attack-move memory)
#[test]
fn claim_23d_memory_converted_at_each_loss() {
    let r = ruleset();
    for how in [Leave::Kill, Leave::OffMap, Leave::Remove] {
        let mut w = game(&r);
        let x = spawn(&mut w, "PLAIN", BETA, 4, 0);
        let h = spawn(&mut w, "PLAIN", ALPHA, 0, 0);
        let cell = w.obj(x).unwrap().pos.cell();
        {
            let o = w.obj_mut(h).unwrap();
            o.attack_move = Some(AttackMoveMemory::Target(Target::Object(x)));
            o.combat.engaged = true;
        }
        leave(&mut w, x, how);
        let o = w.obj(h).unwrap();
        assert_eq!(
            o.attack_move,
            Some(AttackMoveMemory::Dest(Target::Cell(cell))),
            "{how:?}"
        );
        assert!(o.combat.engaged, "{how:?}: the engaged mark is kept");
    }
}

/// A SHADE of Beta, a PLAIN holder of Alpha with an override in effect
/// whose put-aside target and destination are the SHADE, and, when
/// `sensed`, a SEEKER of Alpha whose sensors cover the SHADE's cell. Runs
/// the cloak turns (the SEEKER's, then the SHADE's) in which the SHADE
/// starts cloaking, and no other part of a tick: the unarmed holder's own
/// processing would let go of its target.
fn cloak_moment(sensed: bool) -> (World, ObjectId, ObjectId, ObjectId) {
    let r = ruleset();
    let mut w = game(&r);
    let s = spawn(&mut w, "SHADE", BETA, 0, 0);
    let y = spawn(&mut w, "CART", BETA, 9, 0);
    let h = spawn(&mut w, "PLAIN", ALPHA, -6, 0);
    if sensed {
        let k = spawn(&mut w, "SEEKER", ALPHA, 0, 2);
        glasnost_sim::cloak::turn(&mut w, k);
    }
    overridden(
        &mut w,
        h,
        Some(Target::Object(y)),
        (Some(Target::Object(s)), Some(Target::Object(s))),
    );
    glasnost_sim::cloak::turn(&mut w, s);
    assert_eq!(state(&w, s), glasnost_sim::cloak::CloakState::Cloaking);
    (w, s, y, h)
}

// spec: S-0309 claim 27a (r140, Q-3061: at the moments of claim 28's drops a holder whose override goes on loses a put-aside target that is that object unless its house senses the object's cell, and a put-aside destination that is that object whatever senses its cell)
#[test]
fn claim_27a_cloak_moments() {
    let (w, s, y, h) = cloak_moment(false);
    let o = w.obj(h).unwrap();
    assert_eq!(o.target, Some(Target::Object(y)));
    assert_eq!(o.beh.suspended, Some(Behaviour::Guard));
    assert_eq!(o.beh.aside, Some((None, None)), "not sensed: both lost");
    let _ = s;
    let (w, s, _, h) = cloak_moment(true);
    let o = w.obj(h).unwrap();
    assert_eq!(
        o.beh.aside,
        Some((Some(Target::Object(s)), None)),
        "sensed: the target is kept, the destination lost"
    );
}
