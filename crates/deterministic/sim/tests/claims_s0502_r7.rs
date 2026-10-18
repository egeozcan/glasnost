//! Claim tests of S-0502 revision 7 (movement of jumpjets) on synthetic data:
//! repeated assignments of jumpjet infantry (claim 24, answers Q-1030) and
//! balloon-hover destination retention (claim 28, answers Q-1031), with the
//! orders of S-0213 claims 8, 10 and 21. Every name and number is invented;
//! no game data.

mod jump_common;

use glasnost_sim::econ::assign_destination;
use glasnost_sim::jump::{self, JumpState, Phase};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use jump_common::*;

fn j(w: &World, id: ObjectId) -> &JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn unblock(w: &mut World) {
    while w.tick <= w.orders_blocked_through {
        step(w);
    }
}

/// Issues an order in the name of house 0 and runs the tick that applies it.
fn order(w: &mut World, o: Order) {
    unblock(w);
    assert!(orders::issue(w, HouseId(0), o));
    step(w);
}

/// The raw draws an order costs beyond the tick that applies it: the same
/// setup is run once with the order and once with a bare tick.
fn extra_draws(setup: impl Fn() -> (World, ObjectId), ord: impl Fn(ObjectId) -> Order) -> u64 {
    let (mut with, u) = setup();
    let a = with.rng.taken();
    order(&mut with, ord(u));
    let (mut without, _) = setup();
    let b = without.rng.taken();
    unblock(&mut without);
    step(&mut without);
    (with.rng.taken() - a) - (without.rng.taken() - b)
}

/// A unit cruising towards a far cell with a movement commitment.
fn cruising(w: &mut World, name: &str, kind: Kind, dx: i32) -> ObjectId {
    let u = put(w, name, kind, 0, dx, 0);
    go(w, u, at(dx + 8, 0));
    jump::start_in_flight(w, u, Phase::Cruising, 500);
    u
}

// spec: S-0502 claim 24 (revision 7, answers Q-1030) and S-0213 claim 21: an infantry unit of a jumpjet type has no unchanged-destination suppression: assigning the same nonempty destination again repeats the movement-point adjustment and its one ranged draw, while the vehicle control ignores the repeat
#[test]
fn claim_24_infantry_repeats_a_nonempty_assignment() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let t0 = w.rng.taken();
    go(&mut w, u, at(6, 0));
    let first = w.rng.taken() - t0;
    assert!(first > 0, "the adjustment takes its standing-position draw");
    let t1 = w.rng.taken();
    go(&mut w, u, at(6, 0));
    assert_eq!(w.rng.taken() - t1, first, "the repeat draws again");
    // The same object as destination repeats as well.
    let target = put(&mut w, "JV", Kind::Vehicle, 1, 7, 0);
    assign_destination(&mut w, u, Some(Target::Object(target)));
    let t2 = w.rng.taken();
    assign_destination(&mut w, u, Some(Target::Object(target)));
    assert!(w.rng.taken() > t2, "the same object repeats too");
    // Control: a vehicle ignores the unchanged destination.
    let v = put(&mut w, "JV", Kind::Vehicle, 0, 0, 3);
    go(&mut w, v, at(6, 3));
    let kept = j(&w, v).dest;
    let t3 = w.rng.taken();
    go(&mut w, v, at(6, 3));
    assert_eq!(w.rng.taken(), t3);
    assert_eq!(j(&w, v).dest, kept);
}

// spec: S-0502 claim 24 (revision 7) and S-0215 claim 13: an empty-to-empty assignment of a committed jumpjet-type infantry unit is an effective clear and reroutes each time, with the adjustment's draw; the vehicle control reroutes once and suppresses the repeat
#[test]
fn claim_24_infantry_empty_to_empty_reroutes_every_time() {
    let mut w = game();
    let u = cruising(&mut w, "JJ", Kind::Infantry, -4);
    let t0 = w.rng.taken();
    assign_destination(&mut w, u, None);
    let one = w.rng.taken() - t0;
    assert!(one > 0, "a reroute with its draw");
    assert!(w.obj(u).unwrap().dest.is_none());
    assert!(j(&w, u).moving, "the commitment survives the reroute");
    let t1 = w.rng.taken();
    assign_destination(&mut w, u, None);
    assert_eq!(
        w.rng.taken() - t1,
        one,
        "the second empty assignment reroutes too"
    );
    let v = cruising(&mut w, "JV", Kind::Vehicle, -4);
    assign_destination(&mut w, v, None);
    let t2 = w.rng.taken();
    assign_destination(&mut w, v, None);
    assert_eq!(w.rng.taken(), t2, "the vehicle's repeat is suppressed");
}

// spec: S-0502 claim 24 (revision 7) and S-0213 claim 8: a stop order is one clear for an ordinary infantry unit and two for a balloon-hover type, with target clearing between them; infantry has no vehicle equality suppression, so both reroute while the commitment lasts (the rule is per assignment, not per order)
#[test]
fn claim_24_stop_of_balloon_hover_infantry_reroutes_twice() {
    let make = |name: &'static str| {
        move || {
            let mut w = game();
            let u = cruising(&mut w, name, Kind::Infantry, -4);
            (w, u)
        }
    };
    let one = extra_draws(make("JJ"), |actor| Order::Stop { actor });
    assert!(one > 0, "one reroute with its draw");
    let two = extra_draws(make("JJBAL"), |actor| Order::Stop { actor });
    assert_eq!(two, 2 * one, "two effective clears");
}

/// A balloon-hover vehicle cruising towards a far destination with an enemy
/// target set; the turret faces `turret_offset` facing units from the
/// direction of the target.
fn balloon_with_target(w: &mut World, turret_offset: i32) -> (ObjectId, ObjectId) {
    let u = cruising(w, "BALLOON", Kind::Vehicle, -4);
    let enemy = put(w, "JV", Kind::Vehicle, 1, 0, 6);
    w.obj_mut(enemy).unwrap().health = 1000;
    let up = pos(w, u);
    let ep = pos(w, enemy);
    let dir = glasnost_num::geom::direction(
        i64::from(ep.x - up.x),
        i64::from(ep.y - up.y),
        w.rules().quirks.table_direction,
    );
    let face = (i32::from(dir) + turret_offset).rem_euclid(65536) as u16;
    let o = w.obj_mut(u).unwrap();
    o.turret = Turning::at(face);
    o.target = Some(Target::Object(enemy));
    (u, enemy)
}

fn attack_order(w: &mut World, u: ObjectId, enemy: ObjectId) {
    order(
        w,
        Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(enemy)),
            dest: None,
        },
    );
}

// spec: S-0502 claim 28 (revision 7, answers Q-1031): a balloon-hover vehicle with a nonempty destination and a target rejects an empty assignment when the target direction is at most 16384 facing units from the turret facing (inclusive; 16383, 16384 retain and 16385 does not); the rejection keeps destination, point, commitment and reservation, makes no draw and makes attack current
#[test]
fn claim_28_the_angle_bound_is_inclusive() {
    for (offset, retained) in [
        (0, true),
        (16383, true),
        (16384, true),
        (-16384, true),
        (16385, false),
        (-16385, false),
        (32768, false),
    ] {
        let mut w = game();
        let (u, enemy) = balloon_with_target(&mut w, offset);
        let point = j(&w, u).dest;
        let dest = w.obj(u).unwrap().dest;
        w.obj_mut(u).unwrap().beh.current = Some(Behaviour::Move);
        let draws = extra_draws(
            || {
                let mut w = game();
                let (u, _) = balloon_with_target(&mut w, offset);
                (w, u)
            },
            |actor| Order::Behaviour {
                actor,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(enemy)),
                dest: None,
            },
        );
        attack_order(&mut w, u, enemy);
        if retained {
            assert_eq!(
                w.obj(u).unwrap().dest,
                dest,
                "offset {offset}: destination kept"
            );
            assert_eq!(j(&w, u).dest, point, "offset {offset}: point kept");
            assert!(j(&w, u).moving, "offset {offset}: commitment kept");
            assert_eq!(draws, 0, "offset {offset}: no draw");
            assert_eq!(w.obj(u).unwrap().beh.current, Some(Behaviour::Attack));
        } else {
            assert!(w.obj(u).unwrap().dest.is_none(), "offset {offset}: cleared");
        }
    }
}

// spec: S-0502 claim 28 (revision 7): the identity rows retain whatever the turret faces: the destination is the target object itself, or a cell whose first ground-layer listed object is the target; another cell does not (and an empty destination or an absent target cannot be retained)
#[test]
fn claim_28_identity_rows_and_the_controls() {
    let away = 32768; // the turret faces away from the target
    // Row 1: the destination is the target object.
    let mut w = game();
    let (u, enemy) = balloon_with_target(&mut w, away);
    assign_destination(&mut w, u, Some(Target::Object(enemy)));
    attack_order(&mut w, u, enemy);
    assert_eq!(w.obj(u).unwrap().dest, Some(Target::Object(enemy)));
    // Row 2: a cell whose first ground-layer listed object is the target.
    let mut w = game();
    let (u, enemy) = balloon_with_target(&mut w, away);
    let c = pos(&w, enemy).cell();
    assign_destination(&mut w, u, Some(Target::Cell(c)));
    attack_order(&mut w, u, enemy);
    assert_eq!(w.obj(u).unwrap().dest, Some(Target::Cell(c)));
    // Neither row, and the turret faces away: the clear is effective.
    let mut w = game();
    let (u, enemy) = balloon_with_target(&mut w, away);
    attack_order(&mut w, u, enemy);
    assert!(w.obj(u).unwrap().dest.is_none());
    // An already empty destination cannot be retained, target or not.
    let mut w = game();
    let (u, _) = balloon_with_target(&mut w, 0);
    w.obj_mut(u).unwrap().dest = None;
    assert!(!jump::clear_rejected(&mut w, u));
    // A nonempty request bypasses retention altogether.
    let mut w = game();
    let (u, _) = balloon_with_target(&mut w, 0);
    assign_destination(&mut w, u, Some(Target::Cell(at(-2, 5))));
    assert_eq!(w.obj(u).unwrap().dest, Some(Target::Cell(at(-2, 5))));
    // No target: nothing is retained.
    let mut w = game();
    let (u, _) = balloon_with_target(&mut w, 0);
    w.obj_mut(u).unwrap().target = None;
    assert!(!jump::clear_rejected(&mut w, u));
    // An ordinary jumpjet vehicle never rejects.
    let mut w = game();
    let jv = cruising(&mut w, "JV", Kind::Vehicle, -4);
    w.obj_mut(jv).unwrap().target = Some(Target::Object(jv));
    assert!(!jump::clear_rejected(&mut w, jv));
}

// spec: S-0502 claim 28 (revision 7): a rejection leaves attack alone when attack is already the current behaviour, and makes it current (queued behaviour emptied) otherwise; with no current behaviour the queued one is what counts
#[test]
fn claim_28_the_behaviour_result() {
    let mut w = game();
    let (u, _) = balloon_with_target(&mut w, 0);
    w.obj_mut(u).unwrap().beh.current = Some(Behaviour::Attack);
    w.obj_mut(u).unwrap().beh.queued = Some(Behaviour::Guard);
    assert!(jump::clear_rejected(&mut w, u));
    assert_eq!(
        w.obj(u).unwrap().beh.queued,
        Some(Behaviour::Guard),
        "left alone"
    );
    let mut w = game();
    let (u, _) = balloon_with_target(&mut w, 0);
    w.obj_mut(u).unwrap().beh.current = None;
    w.obj_mut(u).unwrap().beh.queued = Some(Behaviour::Attack);
    assert!(jump::clear_rejected(&mut w, u));
    assert_eq!(w.obj(u).unwrap().beh.current, None, "queued attack counts");
    let mut w = game();
    let (u, _) = balloon_with_target(&mut w, 0);
    w.obj_mut(u).unwrap().beh.current = Some(Behaviour::Move);
    w.obj_mut(u).unwrap().beh.queued = Some(Behaviour::Guard);
    assert!(jump::clear_rejected(&mut w, u));
    let b = &w.obj(u).unwrap().beh;
    assert_eq!((b.current, b.queued), (Some(Behaviour::Attack), None));
}

// spec: S-0502 claim 28 and S-0213 claim 8 (revision 7): stop requests the clear while the target is present and, for a balloon-hover type, again after it is cleared: a qualifying first request is rejected (attack can survive), the second is effective and reroutes, and stop ends with an empty destination and no target
#[test]
fn claim_28_stop_ends_empty_after_a_rejection() {
    let mut w = game();
    let (u, _) = balloon_with_target(&mut w, 0);
    w.obj_mut(u).unwrap().beh.current = Some(Behaviour::Move);
    let taken = w.rng.taken();
    order(&mut w, Order::Stop { actor: u });
    let o = w.obj(u).unwrap();
    assert!(o.dest.is_none(), "stop ends with an empty destination");
    assert!(o.target.is_none());
    assert!(
        w.rng.taken() > taken,
        "the second request is effective: one reroute"
    );
    assert!(j(&w, u).moving, "the commitment survives the reroute");
    assert_eq!(
        j(&w, u).dest.map(|d| d.cell()),
        Some(pos(&w, u).cell()),
        "a replacement point in the current cell"
    );
    assert_eq!(
        w.obj(u).unwrap().beh.seen(),
        Some(Behaviour::Attack),
        "the interim attack survives"
    );
}

// spec: S-0502 claim 28 and S-0213 claim 8 (revision 7): when the first stop request is effective (no qualifying row), the balloon-hover vehicle's second, unchanged empty assignment is suppressed: one reroute in all
#[test]
fn claim_28_stop_without_a_rejection_reroutes_once() {
    let mut w = game();
    let (u, _) = balloon_with_target(&mut w, 32768);
    let taken = w.rng.taken();
    order(&mut w, Order::Stop { actor: u });
    assert!(w.obj(u).unwrap().dest.is_none());
    // A vehicle's reroute takes no standing-position draw: the point is the plain cell centre; the suppression is shown by the kept point.
    let kept = j(&w, u).dest;
    assert!(kept.is_some());
    let _ = taken;
    let p = pos(&w, u);
    w.obj_mut(u).unwrap().pos = Pos::new(p.x + 256 * 3, p.y, p.z);
    assign_destination(&mut w, u, None);
    assert_eq!(
        j(&w, u).dest,
        kept,
        "another empty assignment is suppressed"
    );
}

// spec: S-0502 claim 28, S-0517 claim 5 and S-0213 claim 10 (revisions 7, 8 and 36): deploy makes a single clear request, before the target is cleared and unload is queued, and no retry: a qualifying balloon-hover vehicle keeps its nonempty destination, point and commitment, ends with no target and attack made current before unload is queued
#[test]
fn claim_28_deploy_keeps_the_destination_without_a_retry() {
    let mut w = game();
    let u = cruising(&mut w, "BDEPN", Kind::Vehicle, -4);
    let enemy = put(&mut w, "JV", Kind::Vehicle, 1, 0, 6);
    let up = pos(&w, u);
    let ep = pos(&w, enemy);
    let dir = glasnost_num::geom::direction(
        i64::from(ep.x - up.x),
        i64::from(ep.y - up.y),
        w.rules().quirks.table_direction,
    );
    let o = w.obj_mut(u).unwrap();
    o.turret = Turning::at(dir);
    o.target = Some(Target::Object(enemy));
    o.beh.current = Some(Behaviour::Move);
    let dest = o.dest;
    let point = j(&w, u).dest;
    let mut bare = w.clone();
    let taken = w.rng.taken();
    order(&mut w, Order::Deploy { actor: u });
    let spent = w.rng.taken() - taken;
    let taken = bare.rng.taken();
    unblock(&mut bare);
    step(&mut bare);
    assert_eq!(
        spent,
        bare.rng.taken() - taken,
        "the deploy clear costs no draw"
    );
    let o = w.obj(u).unwrap();
    assert_eq!(o.dest, dest, "the nonempty destination survives");
    assert!(o.target.is_none(), "the target is cleared all the same");
    assert_eq!(j(&w, u).dest, point);
    assert!(j(&w, u).moving);
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Attack),
        "attack was made current"
    );
    assert_eq!(
        o.beh.queued,
        Some(Behaviour::Unload),
        "then unload is queued"
    );
}

// spec: S-0213 claim 10 (revision 36): deploy of a balloon-hover vehicle with no qualifying row clears the destination at once, as for any vehicle
#[test]
fn claim_28_deploy_without_a_qualifying_row_clears() {
    let mut w = game();
    let u = cruising(&mut w, "BDEPN", Kind::Vehicle, -4);
    order(&mut w, Order::Deploy { actor: u });
    assert!(w.obj(u).unwrap().dest.is_none());
}

// spec: S-0215 claim 13 (revision 53) and S-0502 claim 28: an override to attack installs its target and requests the destination clear; a qualifying balloon-hover vehicle keeps its destination
#[test]
fn claim_28_behaviour_processing_clear_can_be_rejected() {
    let mut w = game();
    let (u, enemy) = balloon_with_target(&mut w, 0);
    let dest = w.obj(u).unwrap().dest;
    glasnost_sim::combat::acquire::override_attack(&mut w, u, enemy);
    assert_eq!(w.obj(u).unwrap().dest, dest);
    let mut w = game();
    let (u, enemy) = balloon_with_target(&mut w, 32768);
    glasnost_sim::combat::acquire::override_attack(&mut w, u, enemy);
    assert!(w.obj(u).unwrap().dest.is_none());
}
