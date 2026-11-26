//! S-0213 claim 16e: a point delete cancels only a loop it leaves short
//! (issue 320). Synthetic data only.

use glasnost_map::CellXY;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::routes;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 40;
const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn game() -> World {
    let mut w = testkit::game_with(&testkit::ruleset(), SIZE, 7);
    while orders::blocked(&w) {
        testkit::run(&mut w, 1);
    }
    w
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn give(w: &mut World, house: HouseId, os: Vec<Order>) {
    for o in os {
        assert!(orders::issue(w, house, o));
    }
    testkit::run(w, 1);
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

fn route(w: &mut World, house: HouseId, id: ObjectId, cells: &[CellXY]) -> Vec<u32> {
    give(w, house, cells.iter().map(|&c| add(id, c)).collect());
    cells
        .iter()
        .map(|&c| routes::point_in(w, house, c).expect("a point in the cell"))
        .collect()
}

/// b1 (house A) with an uncommitted route of 3 points and a 2-point loop
/// closed by a join order at its second point (S-0213 claim 16b: closed
/// with no test when applied).
fn short_loop(w: &mut World) -> (ObjectId, Vec<u32>) {
    let b1 = testkit::spawn(w, Kind::Vehicle, "BUGGY", 0, at(0, 0), 0);
    let ps = route(w, A, b1, &[at(3, 3), at(5, 3), at(7, 3)]);
    let place = w.routes.points[&ps[1]].place;
    give(
        w,
        A,
        vec![Order::RouteJoin {
            actor: b1,
            point: ps[1],
            place,
        }],
    );
    let r = &w.routes.routes[&b1];
    assert_eq!((r.loop_from, r.loop_len), (Some(1), Some(2)), "setup");
    (b1, ps)
}

fn lp(w: &World, id: ObjectId) -> (Option<u32>, Option<u32>) {
    let r = &w.routes.routes[&id];
    (r.loop_from, r.loop_len)
}

fn steps(w: &World, id: ObjectId) -> Vec<u32> {
    w.routes.routes[&id].steps.iter().map(|s| s.point).collect()
}

// spec: S-0213 claim 16e (a loop is cancelled when a deleted point lay inside it and it is left with fewer than 3 points) and claim 16b (a press's join order closes a loop with no test, so it can hold fewer than 3 points)
#[test]
fn issue_320_an_unrelated_delete_leaves_a_short_loop() {
    // House A deletes a point of another route of its own.
    let mut w = game();
    let (b1, ps) = short_loop(&mut w);
    let b2 = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(1, 0), 0);
    let q = route(&mut w, A, b2, &[at(3, 8)])[0];
    give(&mut w, A, vec![Order::RouteDelete { point: q }]);
    assert_eq!(steps(&w, b1), ps, "b1's route holds the same points");
    assert_eq!(lp(&w, b1), (Some(1), Some(2)), "own house, other route");
    // House B deletes one of its own points.
    let mut w = game();
    let (b1, ps) = short_loop(&mut w);
    let e = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, at(-3, -3), 0);
    let q = route(&mut w, B, e, &[at(-6, -6)])[0];
    give(&mut w, B, vec![Order::RouteDelete { point: q }]);
    assert_eq!(steps(&w, b1), ps, "b1's route holds the same points");
    assert_eq!(lp(&w, b1), (Some(1), Some(2)), "other house");
    // Control: a delete inside the short loop cancels it.
    let mut w = game();
    let (b1, ps) = short_loop(&mut w);
    give(&mut w, A, vec![Order::RouteDelete { point: ps[2] }]);
    assert_eq!(lp(&w, b1), (None, None), "a point inside the loop");
}
