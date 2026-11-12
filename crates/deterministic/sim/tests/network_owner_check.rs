//! The owner check of a network game (S-0213 claim 15, Q-2972): every
//! machine checks an order's actor's owner immediately before applying
//! it, and refuses the order, with no effect at all, when the issuing
//! house does not own the actor then; the detonate order is excepted. A
//! skirmish game keeps the original's rule. Synthetic data only: the
//! testkit's sandbox game and its invented vehicle BUGGY.

use glasnost_map::CellXY;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 40;
const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn game(network: bool) -> World {
    let mut setup = testkit::setup(7);
    setup.skirmish.network = network;
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(
        &testkit::ruleset(),
        "synthetic-test",
    ));
    let mut w = glasnost_sim::load::new_game(
        rules,
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &setup,
    )
    .expect("synthetic test game loads");
    while orders::blocked(&w) {
        testkit::run(&mut w, 1);
    }
    w
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn mv(actor: ObjectId, c: CellXY) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

fn add(actor: ObjectId, c: CellXY) -> Order {
    Order::RoutePoint {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
        place: Some(Target::Cell(c)),
    }
}

fn steps(w: &World, id: ObjectId) -> usize {
    w.routes.routes.get(&id).map_or(0, |r| r.steps.len())
}

/// A BUGGY of house A, given `order` by A; its owner then changes to
/// `then` (and back to A when `back`) before the order boundary. Returns
/// the world after the tick and the buggy.
fn ordered_then_taken(
    network: bool,
    order: impl Fn(ObjectId) -> Order,
    back: bool,
) -> (World, ObjectId) {
    let mut w = game(network);
    let b = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(0, 0), 0);
    assert!(orders::issue(&mut w, A, order(b)));
    w.obj_mut(b).unwrap().owner = Some(B);
    if back {
        w.obj_mut(b).unwrap().owner = Some(A);
    }
    testkit::run(&mut w, 1);
    (w, b)
}

// spec: S-0213 claim 15 (a skirmish game: an actor whose owner changes between the issue and the application of an order still gets it)
#[test]
fn a_skirmish_order_still_applies_after_an_owner_change() {
    let goal = at(6, 0);
    let (w, b) = ordered_then_taken(false, |b| mv(b, goal), false);
    assert_eq!(w.obj(b).unwrap().dest, Some(Target::Cell(goal)));
}

// spec: S-0213 claim 15 (Network games, Q-2972: an order whose actor the issuing house does not own at the application step is refused)
#[test]
fn a_network_order_is_refused_after_an_owner_change() {
    let goal = at(6, 0);
    let (w, b) = ordered_then_taken(true, |b| mv(b, goal), false);
    assert_ne!(w.obj(b).unwrap().dest, Some(Target::Cell(goal)));
    let (w, b) = ordered_then_taken(
        true,
        |actor| Order::GuardSpot {
            actor,
            spot: Target::Cell(goal),
        },
        false,
    );
    assert_ne!(
        w.obj(b).unwrap().guard_spot,
        Some(Target::Cell(goal)),
        "claim 20: the guard-spot order is checked too"
    );
}

// spec: S-0213 claim 15 (Network games: an order is accepted when the issuing house owns the actor again at that step)
#[test]
fn a_network_order_applies_when_the_owner_is_back() {
    let goal = at(6, 0);
    let (w, b) = ordered_then_taken(true, |b| mv(b, goal), true);
    assert_eq!(w.obj(b).unwrap().dest, Some(Target::Cell(goal)));
}

// spec: S-0213 claim 15 (Network games: an order refused by the owner check has no effect at all and does not end the actor's planned route) and claim 17
#[test]
fn a_refused_network_order_keeps_the_route() {
    for (network, kept) in [(false, false), (true, true)] {
        let mut w = game(network);
        let b = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(0, 0), 0);
        for c in [at(3, 3), at(6, 3), at(6, 6)] {
            assert!(orders::issue(&mut w, A, add(b, c)));
        }
        testkit::run(&mut w, 1);
        assert!(orders::issue(&mut w, A, Order::RouteCommit));
        testkit::run(&mut w, 1);
        assert!(steps(&w, b) > 0);
        assert!(orders::issue(&mut w, A, Order::Stop { actor: b }));
        w.obj_mut(b).unwrap().owner = Some(B);
        testkit::run(&mut w, 1);
        assert_eq!(steps(&w, b) > 0, kept, "network {network}");
    }
}

// spec: S-0213 claim 15 (Network games: the check is made immediately before the order is applied, so an order whose actor was not the house's at its issue but is again at its application is accepted)
#[test]
fn a_network_order_is_judged_by_the_owner_at_its_application() {
    let mut w = game(true);
    let b = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, at(0, 0), 0);
    let goal = at(6, 0);
    assert!(
        orders::issue(&mut w, A, mv(b, goal)),
        "a network game makes the owner check at the application step"
    );
    w.obj_mut(b).unwrap().owner = Some(A);
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(b).unwrap().dest, Some(Target::Cell(goal)));
}
