//! Marks that an order clears (#195, #196): the opportunity mark of S-0309
//! claim 24 and the charging mark of S-0518 claim 15. Synthetic data only.

#[path = "linked_common/mod.rs"]
mod linked_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 40;
const A: HouseId = HouseId(0);

fn combat_rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    // Armed units rest in guard, not area guard.
    r.sim.general.iq_guard_area = None;
    r
}

fn ready(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 7);
    while orders::blocked(&w) {
        testkit::run(&mut w, 1);
    }
    w
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn attack(actor: ObjectId, t: ObjectId) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(t)),
        dest: None,
    }
}

/// Issues the order and runs the tick that applies it.
fn give(w: &mut World, o: Order) {
    assert!(orders::issue(w, A, o));
    testkit::run(w, 1);
}

fn run_until(w: &mut World, limit: i32, done: impl Fn(&World) -> bool) {
    for _ in 0..limit {
        if done(w) {
            return;
        }
        testkit::run(w, 1);
    }
    assert!(done(w), "not done after {limit} ticks");
}

/// A CART of house 0 that holds an opportunity target of its own per-tick
/// search, and the world.
fn holding_an_opportunity_target(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = ready(r);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, at(0, 0), 0);
    let near = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, at(3, 0), 0);
    w.obj_mut(near).unwrap().health = 100_000;
    run_until(&mut w, 200, |w| {
        let o = w.obj(v).unwrap();
        o.target == Some(Target::Object(near)) && o.combat.opportunity
    });
    (w, v, near)
}

// spec: S-0309 claim 24 (every setting of a target leaves a target that is not an opportunity target, the per-tick and teleport searches excepted) and S-0213 table T3 (f) (the order's target)
#[test]
fn an_ordered_target_is_not_an_opportunity_target() {
    let (mut w, v, _) = holding_an_opportunity_target(&combat_rules());
    let far = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, at(14, 0), 0);
    give(&mut w, attack(v, far));
    let o = w.obj(v).unwrap();
    assert_eq!(o.target, Some(Target::Object(far)), "the order's target");
    assert!(
        !o.combat.opportunity,
        "the order's target is marked as an opportunity target"
    );
}

// spec: S-0309 claim 24 (a setting to no target leaves no opportunity target) and S-0213 claim 8 (stop: the target becomes nothing)
#[test]
fn a_stop_order_clears_the_opportunity_mark() {
    let (mut w, v, _) = holding_an_opportunity_target(&combat_rules());
    give(&mut w, Order::Stop { actor: v });
    let o = w.obj(v).unwrap();
    assert!(
        !o.combat.opportunity,
        "the stop order left the mark set: target {:?}",
        o.target
    );
}

/// An omni-crusher of a player house that holds a target of its own
/// per-tick search is ordered to attack a crushable enemy out of weapon
/// range. Returns whether its destination became the target object within
/// 60 ticks (the crushing approach).
fn drives_at_it() -> bool {
    let mut r = combat_rules();
    r.sim.vehicles.get_mut("CRUSHER").unwrap().omni_crusher = Some(true);
    let mut w = ready(&r);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, at(0, 0), 0);
    let near = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, at(3, 0), 0);
    let victim = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, at(0, 9), 0);
    w.obj_mut(near).unwrap().health = 100_000;
    run_until(&mut w, 200, |w| {
        let o = w.obj(v).unwrap();
        o.target == Some(Target::Object(near)) && o.combat.opportunity
    });
    give(&mut w, attack(v, victim));
    assert_eq!(w.obj(v).unwrap().target, Some(Target::Object(victim)));
    for _ in 0..60 {
        testkit::run(&mut w, 1);
        let Some(o) = w.obj(v) else { return false };
        if o.target != Some(Target::Object(victim)) {
            return false;
        }
        if o.dest == Some(Target::Object(victim)) {
            return true;
        }
    }
    false
}

// spec: S-0210 claim 11 (omni-crusher variant: drives at a target it can crush, at any distance; not at a player house's opportunity target) and S-0309 claim 24 (an order's target is not an opportunity target)
#[test]
fn an_omni_crusher_drives_at_an_ordered_target() {
    assert!(
        drives_at_it(),
        "the ordered target is taken for an opportunity target: no crushing approach"
    );
}

/// A charger of house 0 that started charging its own coil by itself, then
/// was given `order`; a near enemy appears afterwards. Returns (the mark
/// after the order applied, the target 60 ticks later, far enemy, near
/// enemy).
fn charger_given(order: impl Fn(ObjectId, ObjectId) -> Order) -> (bool, Option<Target>, ObjectId, ObjectId) {
    use linked_common::{at as lat, spawn, world};
    let mut w: World = world(|_| {});
    let coil = spawn(&mut w, Kind::Building, "COIL", 0, lat(1, 0));
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(coil).unwrap().beh.force(Behaviour::Sleep, tick, stale);
    let u = spawn(&mut w, Kind::Infantry, "CHARGER", 0, lat(0, 0));
    // The unit starts charging by itself in guard (S-0518 claim 14).
    let mut n = 0;
    while !(w.obj(u).unwrap().combat.charging && w.tick > w.orders_blocked_through) {
        testkit::run(&mut w, 1);
        n += 1;
        assert!(n < 400, "the unit never started charging");
    }
    assert_eq!(w.obj(u).unwrap().target, Some(Target::Object(coil)));
    let far = spawn(&mut w, Kind::Vehicle, "VICT", 1, lat(0, 15));
    assert!(orders::issue(&mut w, HouseId(0), order(u, far)));
    testkit::run(&mut w, 1);
    let mark = w.obj(u).unwrap().combat.charging;
    let near = spawn(&mut w, Kind::Vehicle, "VICT", 1, lat(-3, 0));
    testkit::run(&mut w, 60);
    (mark, w.obj(u).unwrap().target, far, near)
}

// spec: S-0518 claim 15 (the mark also clears whenever the unit is given a target by an order) and S-0213 table T3 (f)
#[test]
fn an_attack_order_clears_the_charging_mark() {
    let (mark, later, far, near) = charger_given(attack);
    assert!(!mark, "the charging mark survived the attack order");
    assert_ne!(
        later,
        Some(Target::Object(near)),
        "the break-off search replaced the ordered target"
    );
    assert_eq!(later, Some(Target::Object(far)));
}

// spec: S-0518 claim 15 (the mark clears at any other ordinary target change) and S-0213 claim 8 (stop: the target becomes nothing)
#[test]
fn a_stop_order_clears_the_charging_mark() {
    let (mark, _, _, _) = charger_given(|actor, _| Order::Stop { actor });
    assert!(!mark, "the charging mark survived the stop order");
}
