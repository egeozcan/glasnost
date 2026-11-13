//! Claim tests for S-0502 r24 (Q-3042, #189): the empty destination
//! assignment at a jumpjet's crash start, F4's ground hold for a negative
//! desired height, and the held-up crash with `quirks.jumpjet_crash_lookahead`
//! (scenario J31). Synthetic rules (`jump_common` plus JJDEF, a crashable
//! jumpjet infantry type whose climb and crash rates are equal); every name
//! and number is invented.

mod jump_common;

use glasnost_num::F64;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::jump::{self, JumpState, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use jump_common::*;

fn j(w: &World, id: ObjectId) -> &JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn kill(w: &mut World, id: ObjectId) {
    let health = w.obj(id).unwrap().health;
    damage::apply(
        w,
        id,
        Hit {
            amount: health * 10,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
}

/// The jumpjet fixture plus JJDEF: a crashable infantry jumpjet with equal
/// climb and crash rates (c = k = 5) and target height `ht`; `lookahead`
/// sets `quirks.jumpjet_crash_lookahead`.
fn game_def(ht: i32, lookahead: bool) -> World {
    let mut r = ruleset();
    let mut def = r.sim.infantry.get("JJCRASH").unwrap().clone();
    def.jumpjet_climb = Some(d("5.0"));
    def.jumpjet_crash = Some(d("5.0"));
    def.jumpjet_height = Some(ht);
    def.jumpjet_no_wobbles = Some(true);
    r.sim.infantry.insert("JJDEF".into(), def);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("JJDEF".into());
    r.globals.quirks.jumpjet_crash_lookahead = lookahead;
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

/// A JJDEF at (0, 0), with (when `shared`) a second infantry unit with no
/// order standing in the same cell, ordered to (10, 0) and run during its
/// ascent until at height `kill_at` or more. Returns the unit and the other
/// unit.
fn shot_down(w: &mut World, shared: bool, kill_at: i32) -> (ObjectId, Option<ObjectId>) {
    let u = put(w, "JJDEF", Kind::Infantry, 0, 0, 0);
    let other = shared.then(|| put(w, "JJ", Kind::Infantry, 0, 0, 0));
    go(w, u, at(10, 0));
    assert!(until(w, 60, |w| height(w, u) >= kill_at));
    assert_eq!(phase(w, u), Phase::Ascending);
    (u, other)
}

// spec: S-0502 claim 18 (r24, Q-3042: starting a crash makes one empty destination assignment at once, inside the hit; a jumpjet infantry unit with a movement commitment reroutes under claim 17, a nearby search around its current cell, then standing-position adjustment with its ranged draw over 0 to 3, claim 23; the replacement point lies in another cell when the search rejects the current one, as for another unit's mark there); S-0312 claim 16 (r77)
#[test]
fn j31_a_the_crash_start_clears_the_destination_and_reroutes() {
    let mut w = game_def(400, true);
    let (u, _) = shot_down(&mut w, true, 50);
    let here = pos(&w, u).cell();
    let before = j(&w, u).dest.expect("a movement point");
    assert_eq!(before.cell(), at(10, 0));
    kill(&mut w, u);
    assert!(jump::crashing(&w, u), "it crashes");
    let o = w.obj(u).unwrap();
    assert_eq!(o.dest, None, "the order-level destination is empty");
    let p = j(&w, u).dest.expect("the commitment survives");
    assert_ne!(p.cell(), here, "the point is not in the current cell");
    assert_ne!(p.cell(), at(10, 0), "the point was rerouted");
    let (dx, dy) = (p.cell().x - here.x, p.cell().y - here.y);
    assert!(dx.abs() <= 1 && dy.abs() <= 1, "a nearby cell: {dx}, {dy}");
}

// spec: S-0502 claim 18 (r24, Q-3042: held-up crash: while its movement point lies in another cell an ordinary crashing jumpjet's reference is the look-ahead one; at maximum speed 0 only its own cell's clearance counts, terrain + 85 with another unit listed; with c = k it keeps its kill height in every update, stays crashing, has current speed 0 at the end of each update and does not move sideways); claim 8 and F4 (r24: the ground hold for a negative desired height holds exactly when u ≤ h_d)
#[test]
fn j31_a_a_crash_over_a_listed_unit_is_held_up() {
    let mut w = game_def(400, true);
    let (u, _) = shot_down(&mut w, true, 50);
    kill(&mut w, u);
    let z = pos(&w, u).z;
    let (x, y) = (pos(&w, u).x, pos(&w, u).y);
    for t in 0..3000 {
        step(&mut w);
        let o = w.obj(u).expect("still in play");
        assert_eq!(o.pos.z, z, "the kill height is kept (tick {t})");
        assert_eq!((o.pos.x, o.pos.y), (x, y), "no sideways move (tick {t})");
        assert_eq!(phase(&w, u), Phase::Crashing);
        assert_eq!(j(&w, u).vc, F64::ZERO, "speed 0 (tick {t})");
    }
}

// spec: S-0502 claim 18 (r24, Q-3042: a held-up crash ends when g falls far enough, as when the listed object leaves the cell, by the ordinary crash end)
#[test]
fn j31_b_the_held_up_crash_ends_once_the_other_unit_leaves() {
    let mut w = game_def(400, true);
    let (u, other) = shot_down(&mut w, true, 50);
    kill(&mut w, u);
    for _ in 0..30 {
        step(&mut w);
    }
    assert!(jump::crashing(&w, u), "held up");
    glasnost_sim::objects::remove(&mut w, other.unwrap());
    assert!(
        until(&mut w, 30, |w| w.obj(u).is_none_or(|_| !jump::crashing(w, u))),
        "the crash ends within a few ticks"
    );
}

// spec: S-0502 claim 18 (r24, Q-3042: control: alone in its cell the crash ends within a few ticks of the kill)
#[test]
fn j31_c_alone_in_its_cell_the_crash_ends() {
    let mut w = game_def(400, true);
    let (u, _) = shot_down(&mut w, false, 50);
    kill(&mut w, u);
    assert!(
        until(&mut w, 30, |w| w.obj(u).is_none_or(|_| !jump::crashing(w, u))),
        "the crash ends within a few ticks"
    );
}

// spec: S-0502 claim 18 (r24, Q-3042: reference: in an update that starts at speed 0 the neighbouring cell counts when a and S are above 0 and the maximum speed is at least min(a, S), so g is terrain + 42 with one other unit listed in its own cell; it settles at g − 5 − ⌈k⌉ = terrain + 32 and stays, with speed 0 at the end of each update it starts at or below g − 5); F3 (r24: N counts while v_c > 0 as F1 leaves it, after braking and before the hold)
#[test]
fn j31_e_with_its_maximum_speed_set_the_neighbour_counts() {
    let mut w = game_def(160, true);
    let u = put(&mut w, "JJDEF", Kind::Infantry, 0, 0, 0);
    let _other = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    go(&mut w, u, at(10, 0));
    assert!(until(&mut w, 60, |w| j(w, u).vm.gt(F64::ZERO)));
    let start = pos(&w, u).cell();
    kill(&mut w, u);
    let ground = w.surface_at(pos(&w, u).x, pos(&w, u).y);
    let mut settled = 0;
    for _ in 0..400 {
        step(&mut w);
        let o = w.obj(u).expect("still in play");
        assert_eq!(o.pos.cell(), start, "the run stays in its starting cell");
        if o.pos.z - ground == 32 {
            settled += 1;
            assert_eq!(j(&w, u).vc, F64::ZERO, "speed 0 once settled");
        } else {
            assert_eq!(settled, 0, "it stays at terrain + 32 once there");
        }
    }
    assert!(settled > 300, "settled at terrain + 32");
}

// spec: S-0502 Edge cases and F3 (r24: quirks.jumpjet_crash_lookahead off: an ordinary crashing jumpjet always uses the direct reference, so (a) reaches its crash end)
#[test]
fn j31_a_with_the_crash_lookahead_off_the_crash_ends() {
    let mut w = game_def(400, false);
    let (u, _) = shot_down(&mut w, true, 50);
    kill(&mut w, u);
    assert!(
        until(&mut w, 30, |w| w.obj(u).is_none_or(|_| !jump::crashing(w, u))),
        "the crash ends"
    );
}
