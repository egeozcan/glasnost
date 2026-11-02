//! The own AI and a guard unit's stale target (`scripted::idle`, our own
//! design on S-0215 claim 24): a unit resting in guard keeps a target it
//! cannot shoot at and stands still, so the script treats it as free for an
//! order. Whether the target is stale is asked whatever the moment: the
//! firing check without the reasons of the moment (S-0214 claim 6 (e),
//! S-0301 claim 9 ranks 5 to 8), and the range test on its own (S-0307), so
//! a rearming unit does not hide a target its weapon may not hurt, or one
//! out of its range.

use glasnost_ai::scripted;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 40;

/// A guard tank of house 0 and three units of house 1: a cart its gun's
/// warhead does nothing to (versus 0 for class light) and a heavy tank,
/// both 3 cells off (within the gun's range of 5.5), and a heavy tank 12
/// cells off (out of range).
fn scene() -> (World, ObjectId, ObjectId, ObjectId, ObjectId) {
    let mut r = testkit::combat_ruleset();
    // The gun's warhead does nothing to class light (index 3, the cart's).
    r.sim
        .warheads
        .get_mut("HE")
        .unwrap()
        .verses
        .as_mut()
        .unwrap()[3] = glasnost_rules::sim::Decimal("0".into());
    let mut w = testkit::game_with(&r, SIZE, 7);
    let tank = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, mid(SIZE, 0, 0), 0);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 3, 0), 0);
    let near = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, mid(SIZE, 0, 3), 0);
    let far = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, mid(SIZE, 0, 12), 0);
    (w, tank, cart, near, far)
}

/// The tank rests in guard with `target`, no destination, and its weapon
/// rearming (`rearming`) or ready.
fn guard(w: &mut World, tank: ObjectId, target: ObjectId, rearming: bool) {
    let tick = w.tick;
    let o = w.obj_mut(tank).unwrap();
    o.beh.current = Some(Behaviour::Guard);
    o.dest = None;
    o.target = Some(Target::Object(target));
    o.combat.rearm_until = if rearming { tick + 50 } else { tick };
}

// design: a guard unit with a stale target is free for an order (S-0215 claim 24, guard never pursues); spec: S-0214 claim 6 (e) and S-0301 claim 9 (rearming, rank 5, does not hide the rank-9 illegal answer) and S-0304 claim 1 (versus 0: illegal)
#[test]
fn a_rearming_guard_unit_with_a_target_it_may_not_hurt_is_free() {
    let (mut w, tank, cart, _, _) = scene();
    guard(&mut w, tank, cart, false);
    assert!(
        scripted::idle(&w, tank),
        "ready: the illegal target is stale"
    );
    guard(&mut w, tank, cart, true);
    assert!(
        scripted::idle(&w, tank),
        "rearming: the illegal target is still stale"
    );
}

// design: a guard unit with a stale target is free for an order (S-0215 claim 24); spec: S-0214 claim 6 (e) and S-0301 claim 9 (rearming does not hide the range answer) and S-0307 claims 1 to 11 (the range test, asked on its own)
#[test]
fn a_rearming_guard_unit_with_a_target_out_of_range_is_free() {
    let (mut w, tank, _, _, far) = scene();
    guard(&mut w, tank, far, false);
    assert!(scripted::idle(&w, tank), "ready: out of range is stale");
    guard(&mut w, tank, far, true);
    assert!(
        scripted::idle(&w, tank),
        "rearming: out of range is still stale"
    );
}

// design: a guard unit whose target is lawful and in range is busy with it, rearming or not (S-0215 claim 24: the firing rules fire at a target in range); spec: S-0307 claims 1 to 11
#[test]
fn a_rearming_guard_unit_with_a_lawful_target_in_range_is_busy() {
    let (mut w, tank, _, near, _) = scene();
    guard(&mut w, tank, near, false);
    assert!(!scripted::idle(&w, tank), "ready: the target is kept");
    guard(&mut w, tank, near, true);
    assert!(!scripted::idle(&w, tank), "rearming: the target is kept");
}
