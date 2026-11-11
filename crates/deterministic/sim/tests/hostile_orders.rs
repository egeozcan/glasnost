//! Orders from outside the simulation: what `orders::issue` takes from a
//! peer's frame, a replay or our AI (#155, #158, #219, #220). Every peer
//! issues the same orders in the same tick, so each refusal here is made
//! alike on all of them. Synthetic data only: the testkit's sandbox game and
//! its invented vehicle BUGGY.

use glasnost_map::CellXY;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 40;
const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn game() -> World {
    let mut w = testkit::game_with(&testkit::ruleset(), SIZE, 7);
    // spec: S-0213 claim 1 (orders are refused while the opening order block lasts)
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

fn join(actor: ObjectId, c: CellXY) -> Order {
    Order::RouteJoin {
        actor,
        point: 0,
        place: Target::Cell(c),
    }
}

fn steps(w: &World, id: ObjectId) -> usize {
    w.routes.routes.get(&id).map_or(0, |r| r.steps.len())
}

/// A game with one BUGGY of house A holding a committed route of three
/// points.
fn routed_buggy() -> (World, ObjectId) {
    let mut w = game();
    let b = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(0, 0), 0);
    for c in [at(3, 3), at(6, 3), at(6, 6)] {
        assert!(orders::issue(&mut w, A, add(b, c)));
    }
    testkit::run(&mut w, 1);
    assert!(orders::issue(&mut w, A, Order::RouteCommit));
    testkit::run(&mut w, 1);
    (w, b)
}

// --- #155: the route marker is the simulation's own ---

// spec: S-0213 claim 17 (any order that names an actor, apart from the route-mode orders, removes that actor's planned route; only orders a route itself issues do not)
#[test]
fn a_routed_order_from_outside_is_refused() {
    // The control: a plain stop order ends the route.
    let (mut w, b) = routed_buggy();
    assert!(steps(&w, b) > 0, "the buggy holds a route");
    assert!(orders::issue(&mut w, A, Order::Stop { actor: b }));
    testkit::run(&mut w, 1);
    assert_eq!(steps(&w, b), 0, "a plain stop ends the route");

    // The same order inside the simulation's route marker, from outside.
    let (mut w, b) = routed_buggy();
    let before = steps(&w, b);
    let taken = orders::issue(&mut w, A, Order::Routed(Box::new(Order::Stop { actor: b })));
    testkit::run(&mut w, 1);
    assert!(
        !taken,
        "issue took a route-marked order from outside; the route holds {} of {before} points",
        steps(&w, b)
    );
}

// --- #220: a house orders only its own objects ---

/// Every order kind that names an actor, for `actor`.
fn actor_orders(actor: ObjectId, c: CellXY) -> Vec<Order> {
    vec![
        mv(actor, c),
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Selling,
            attack_move: false,
            target: None,
            dest: None,
        },
        Order::Stop { actor },
        Order::Scatter { actor },
        Order::Deploy { actor },
        Order::GuardSpot {
            actor,
            spot: Target::Cell(c),
        },
        Order::Sell { actor },
        Order::Repair { actor },
        Order::PowerOff { actor },
        Order::PowerOn { actor },
        Order::Primary { actor },
        Order::Detonate { actor },
        add(actor, c),
        join(actor, c),
    ]
}

// spec: S-0213 claims 3 and 15 (a house is held to S-0214's input rules when it issues an order, which let it order only its own objects) and S-0214 Terms (an actor that is not an own object gets select or nothing: no order)
#[test]
fn an_order_for_another_houses_object_is_refused_at_issue() {
    let mut w = game();
    let theirs = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, at(0, 0), 0);
    let mine = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(4, 0), 0);
    let c = at(8, 4);
    let mut taken = Vec::new();
    for o in actor_orders(theirs, c) {
        if orders::issue(&mut w, A, o.clone()) {
            taken.push(o);
        }
    }
    assert!(
        taken.is_empty(),
        "house A's orders for house B's buggy were taken: {taken:?}"
    );
    // The control: the same orders for A's own buggy, and B's for its own.
    let mut refused = Vec::new();
    for (h, id) in [(A, mine), (B, theirs)] {
        for o in actor_orders(id, c) {
            if !orders::issue(&mut w, h, o.clone()) {
                refused.push(o);
            }
        }
    }
    assert!(
        refused.is_empty(),
        "orders for a house's own buggy were refused: {refused:?}"
    );
}

// spec: S-0213 claim 15 (an actor whose owner changes between the issue and the application of an order still gets it)
#[test]
fn an_order_taken_at_issue_still_applies_after_an_owner_change() {
    let mut w = game();
    let b = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(0, 0), 0);
    let goal = at(6, 0);
    assert!(orders::issue(&mut w, A, mv(b, goal)));
    // The owner changes before the order boundary (a take-over in the tick).
    w.obj_mut(b).unwrap().owner = Some(B);
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(b).unwrap().dest, Some(Target::Cell(goal)));
}

// --- #219: orders carry cells within a fixed window ---

/// The first cell past the window on each side, and far cells whose
/// centre in leptons does not fit an i32.
fn far_cells() -> Vec<CellXY> {
    vec![
        CellXY::new(0, i32::MIN),
        CellXY::new(10_000_000, 0),
        CellXY::new(8_388_608, 0),
        CellXY::new(i32::MAX, i32::MAX),
        CellXY::new(-513, 0),
        CellXY::new(0, 1024),
    ]
}

/// Every order kind that carries a cell, with `c`.
fn cell_orders(actor: ObjectId, c: CellXY) -> Vec<Order> {
    vec![
        mv(actor, c),
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Cell(c)),
            dest: None,
        },
        Order::GuardSpot {
            actor,
            spot: Target::Cell(c),
        },
        Order::Place {
            what: TypeRef {
                kind: Kind::Building,
                index: 0,
            },
            cell: Some(c),
        },
        Order::SellWall { cell: c },
        Order::FireSuperweapon { sw: 0, cell: c },
        add(actor, c),
        join(actor, c),
    ]
}

// nocite: our own input validation (#219): a cell far off every map is refused at issue, before any arithmetic on it
#[test]
fn an_order_with_a_cell_outside_the_window_is_refused_without_a_panic() {
    let mut w = game();
    let b = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(0, 0), 0);
    let mut taken = Vec::new();
    for c in far_cells() {
        for o in cell_orders(b, c) {
            if orders::issue(&mut w, A, o.clone()) {
                taken.push(o);
            }
        }
    }
    testkit::run(&mut w, 300);
    assert!(
        taken.is_empty(),
        "orders with far cells were taken: {taken:?}"
    );
}

// --- #158: a route's length has a bound ---

/// Joins alone double a route, four orders at a time; the bound stops it.
// nocite: our own input validation (#158): no route grows past the engine's step bound, whatever the press rules let through
#[test]
fn joins_do_not_double_a_route_without_bound() {
    let mut w = game();
    let x = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(0, 0), 0);
    let z = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(1, 0), 0);
    let x0 = at(0, 5);
    assert!(orders::issue(&mut w, A, add(x, x0)));
    testkit::run(&mut w, 1);
    // Two ticks of 8 rounds each, 32 orders a tick.
    let mut round = 0;
    for _ in 0..2 {
        for _ in 0..8 {
            let c = at(round - 8, 8);
            round += 1;
            for o in [Order::Stop { actor: z }, add(z, c), join(z, x0), join(x, c)] {
                assert!(orders::issue(&mut w, A, o));
            }
        }
        testkit::run(&mut w, 1);
    }
    let longest = w.routes.routes.values().map(|r| r.steps.len()).max();
    assert!(
        longest.is_some_and(|n| n <= 1024),
        "the longest route holds {longest:?} steps"
    );
}

/// Past 16 points a route takes point adds, but not without bound.
// nocite: our own input validation (#158)
#[test]
fn point_adds_past_16_points_stop_at_the_bound() {
    let mut w = game();
    let b = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(0, 0), 0);
    let a = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(1, 0), 0);
    for i in 0..16 {
        assert!(orders::issue(&mut w, A, add(b, at(i - 8, 3))));
    }
    assert!(orders::issue(&mut w, A, add(a, at(0, 5))));
    assert!(orders::issue(&mut w, A, join(a, at(-8, 3))));
    testkit::run(&mut w, 1);
    // spec: S-0213 claim 16b (a join's copy is added after the actor's own points, whatever their number, so its route can pass 16 points)
    assert_eq!(steps(&w, a), 17);
    for _ in 0..10 {
        for i in 0..128 {
            orders::issue(&mut w, A, add(a, at(i % 20 - 10, 7 + i / 20)));
        }
        testkit::run(&mut w, 1);
    }
    assert!(
        steps(&w, a) <= 1024,
        "the route holds {} steps",
        steps(&w, a)
    );
    assert!(
        w.routes.points.len() <= 1024 + 16,
        "the house holds {} points",
        w.routes.points.len()
    );
}
