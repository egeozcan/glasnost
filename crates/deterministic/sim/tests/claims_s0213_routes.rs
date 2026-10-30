//! Claim tests for S-0213's route mode (claims 16 to 16f and 17) and
//! S-0215 claim 13's route row, on synthetic data: the testkit's sandbox
//! game (houses Alpha 0 and Beta 1, not allied) with its invented vehicle
//! BUGGY and an invented building HUT. No stock values.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::routes::{self, HOLDER_LIMIT, RouteClick, RouteNote};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::save;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 40;
const A: HouseId = HouseId(0);

/// The testkit's ruleset with HUT, a 1 × 1 building.
fn ruleset() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries.insert("buildings".into(), vec!["HUT".into()]);
    r.sim.buildings.insert(
        "HUT".into(),
        BuildingType {
            strength: Some(500),
            foundation: Some([1, 1]),
            ..Default::default()
        },
    );
    r
}

/// A sandbox game past the opening order block (S-0201 claim 14).
fn game_from(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 7);
    while orders::blocked(&w) {
        testkit::run(&mut w, 1);
    }
    w
}

fn game() -> World {
    game_from(&ruleset())
}

/// [`game`] with the route quirks set.
fn game_quirks(spent_blocks: bool, drift: bool) -> World {
    let mut r = ruleset();
    r.globals.quirks.spent_route_blocks_join = spent_blocks;
    r.globals.quirks.route_holder_count_drift = drift;
    game_from(&r)
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// An Alpha BUGGY at `at(dx, dy)`, resting in guard.
fn buggy(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, "BUGGY", 0, at(dx, dy), 0)
}

/// Issues `os` in Alpha's name and runs the tick that applies them; the
/// effects are cleared first.
fn give(w: &mut World, os: Vec<Order>) {
    w.effects.clear();
    for o in os {
        assert!(orders::issue(w, A, o));
    }
    testkit::run(w, 1);
}

/// A point add of a move to `c`.
fn add(actor: ObjectId, c: CellXY) -> Order {
    Order::RoutePoint {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

/// The route notes since the last [`give`].
fn notes(w: &World) -> Vec<RouteNote> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Route { house, note } if *house == A => Some(*note),
            _ => None,
        })
        .collect()
}

fn steps(w: &World, id: ObjectId) -> Vec<u32> {
    w.routes
        .routes
        .get(&id)
        .map(|r| r.steps.iter().map(|s| s.point).collect())
        .unwrap_or_default()
}

fn point(w: &World, c: CellXY) -> u32 {
    routes::point_in(w, A, c).expect("a point in the cell")
}

fn cell(w: &World, id: ObjectId) -> CellXY {
    w.obj(id).unwrap().pos.cell()
}

/// A route of move points through `cells` for `id`, one add per cell.
fn route(w: &mut World, id: ObjectId, cells: &[CellXY]) -> Vec<u32> {
    give(w, cells.iter().map(|&c| add(id, c)).collect());
    cells.iter().map(|&c| point(w, c)).collect()
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

fn click(w: &World, sel: &[ObjectId], c: CellXY) -> RouteClick {
    routes::click(w, A, sel, ClickOn::Cell(c), c, Modifiers::default())
}

/// spec: S-0213 claim 16a (each object its own route; one click with several objects adds the same point, at the order's place, to each route; a later add at that place reuses the point) and claim 16b (an accepted point is announced; no information message for an ordinary point) and Terms (a first point raises the count)
#[test]
fn claim_16a_point_adds_share_points_and_build_one_route_each() {
    let mut w = game();
    let (b1, b2) = (buggy(&mut w, 0, 0), buggy(&mut w, 1, 0));
    let (c1, c2) = (at(5, 0), at(5, 5));
    give(&mut w, vec![add(b1, c1), add(b2, c1)]);
    assert_eq!(w.routes.points.len(), 1);
    let p1 = point(&w, c1);
    assert_eq!(steps(&w, b1), vec![p1]);
    assert_eq!(steps(&w, b2), vec![p1]);
    assert_eq!(routes::count(&w, A), 2);
    assert_eq!(notes(&w), vec![RouteNote::Added { info: None }; 2]);
    give(&mut w, vec![add(b1, c2)]);
    let p2 = point(&w, c2);
    assert_eq!(steps(&w, b1), vec![p1, p2]);
    assert_eq!(steps(&w, b2), vec![p1]);
    assert_eq!(routes::count(&w, A), 2, "a second point raises nothing");
    give(&mut w, vec![add(b2, c2)]);
    assert_eq!(steps(&w, b2), vec![p1, p2], "the point at c2 is reused");
    assert_eq!(w.routes.points.len(), 2);
    // spec: S-0213 claim 16c (points added since the last commit are not followed)
    assert!(w.routes.points.values().all(|p| !p.committed));
    testkit::run(&mut w, 40);
    assert_eq!(cell(&w, b1), at(0, 0), "uncommitted: no move");
    // The route state is saved and hashed (CLAUDE.md determinism rules).
    let bytes = save::save(&w);
    let l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    assert_eq!(l.routes, w.routes);
}

/// spec: S-0213 claim 16c (a commit commits every point; a holder in guard starts at once and issues its next point, which takes effect at the next order boundary; issuing takes the point off the front, so the route is used up at its last point; the holder then keeps its last action), claim 16f (used up: the count is lowered then, and points no route holds are deleted) and S-0215 claim 13 (the route row issues the next point at the fall-back to rest)
#[test]
fn claim_16c_commit_starts_a_guarding_holder_and_the_route_is_used_up() {
    let mut w = game();
    let b1 = buggy(&mut w, 0, 0);
    let (c1, c2) = (at(4, 0), at(4, 4));
    let [_, p2] = route(&mut w, b1, &[c1, c2])[..] else {
        unreachable!()
    };
    assert_eq!(w.obj(b1).unwrap().beh.current, Some(Behaviour::Guard));
    give(&mut w, vec![Order::RouteCommit]);
    assert!(w.routes.points.values().all(|p| p.committed));
    assert_eq!(steps(&w, b1), vec![p2], "the first point is issued");
    assert_eq!(routes::count(&w, A), 1);
    run_until(&mut w, 400, |w| steps(w, b1).is_empty());
    assert_eq!(routes::count(&w, A), 0, "used up");
    assert!(w.routes.points.is_empty(), "no route holds a point");
    assert!(w.routes.routes.contains_key(&b1), "the record stays");
    run_until(&mut w, 400, |w| cell(w, b1) == c2);
    testkit::run(&mut w, 60);
    assert_eq!(cell(&w, b1), c2, "the route does not start again");
}

/// spec: S-0213 claim 16c (any other holder, one still moving under an earlier order, is not interrupted: it issues its next point at its next fall-back to rest) and claim 16e (a route can be extended while followed)
#[test]
fn claim_16c_a_moving_holder_is_not_interrupted() {
    let mut w = game();
    let b1 = buggy(&mut w, 0, 0);
    let (far, c1) = (at(8, 0), at(0, 6));
    give(
        &mut w,
        vec![Order::Behaviour {
            actor: b1,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(far)),
        }],
    );
    testkit::run(&mut w, 2);
    let p1 = route(&mut w, b1, &[c1])[0];
    give(&mut w, vec![Order::RouteCommit]);
    assert_eq!(steps(&w, b1), vec![p1], "still moving: not issued");
    run_until(&mut w, 400, |w| cell(w, b1) == far);
    run_until(&mut w, 400, |w| cell(w, b1) == c1);
}

/// spec: S-0213 claim 16c (a click on a point already in the route closes a loop from it to the end, announced by an information message; the loop's points are issued in turn, the first again after the last; points before the loop are issued once) and claim 17 (an order outside route mode ends the route; claim 16f: the count is lowered and the points no route holds are deleted)
#[test]
fn claim_16c_a_loop_cycles_until_an_order_ends_it() {
    let mut w = game();
    let b1 = buggy(&mut w, 0, 0);
    let (c0, c1, c2, c3) = (at(0, 3), at(4, 3), at(4, 7), at(0, 7));
    let ps = route(&mut w, b1, &[c0, c1, c2, c3]);
    give(
        &mut w,
        vec![Order::RouteJoin {
            actor: b1,
            point: ps[1],
        }],
    );
    assert_eq!(notes(&w), vec![RouteNote::Looped]);
    assert_eq!(w.routes.routes[&b1].loop_from, Some(1));
    give(&mut w, vec![Order::RouteCommit]);
    let mut seen = Vec::new();
    for _ in 0..1600 {
        testkit::run(&mut w, 1);
        let c = cell(&w, b1);
        if [c0, c1, c2, c3].contains(&c) && seen.last() != Some(&c) {
            seen.push(c);
        }
        if seen.len() >= 6 {
            break;
        }
    }
    assert_eq!(seen, vec![c0, c1, c2, c3, c1, c2]);
    assert_eq!(
        steps(&w, b1),
        ps[1..].to_vec(),
        "the point before the loop is gone"
    );
    assert_eq!(routes::count(&w, A), 1);
    // spec: S-0213 claim 17 (stop names the actor: it ends the route at its application)
    give(&mut w, vec![Order::Stop { actor: b1 }]);
    assert!(steps(&w, b1).is_empty());
    assert_eq!(routes::count(&w, A), 0);
    assert!(w.routes.points.is_empty());
}

/// spec: S-0213 claim 17 (a behaviour order ends the route when applied, whether or not it then passes its checks; route orders and orders a route issues do not)
#[test]
fn claim_17_an_order_ends_the_route_even_when_it_fails() {
    let mut w = game();
    let b1 = buggy(&mut w, 0, 0);
    let hut = testkit::spawn(&mut w, Kind::Building, "HUT", 1, at(-6, 0), 0);
    route(&mut w, b1, &[at(3, 3), at(6, 3)]);
    give(&mut w, vec![Order::RouteCommit]);
    assert_eq!(steps(&w, b1).len(), 1, "a routed order did not end it");
    // An attack order on a target removed before its tick's boundary.
    assert!(orders::issue(
        &mut w,
        A,
        Order::Behaviour {
            actor: b1,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(hut)),
            dest: None,
        }
    ));
    glasnost_sim::objects::remove(&mut w, hut);
    testkit::run(&mut w, 1);
    assert!(steps(&w, b1).is_empty());
    assert_eq!(routes::count(&w, A), 0);
}

/// spec: S-0213 claim 16b (a point add is refused when applied after a continual or terminating last point, in a loop, or at 16 points, each with its message; an accepted continual, terminating or 16th point shows that case's message as information)
#[test]
fn claim_16b_refusals_when_a_point_add_is_applied() {
    let mut w = game();
    let (b1, b2, b3, b4) = (
        buggy(&mut w, 0, 0),
        buggy(&mut w, 1, 0),
        buggy(&mut w, 2, 0),
        buggy(&mut w, 3, 0),
    );
    let hut = testkit::spawn(&mut w, Kind::Building, "HUT", 0, at(-6, -6), 0);
    // Continual: area guard on a cell.
    give(
        &mut w,
        vec![Order::RoutePoint {
            actor: b1,
            behaviour: Behaviour::AreaGuard,
            attack_move: false,
            target: Some(Target::Cell(at(4, 4))),
            dest: None,
        }],
    );
    let info = |m| RouteNote::Added { info: Some(m) };
    assert_eq!(notes(&w), vec![info("MSG:PostContinualCommand")]);
    give(&mut w, vec![add(b1, at(6, 6))]);
    assert_eq!(
        notes(&w),
        vec![RouteNote::Refused("MSG:PostContinualCommand")]
    );
    assert_eq!(steps(&w, b1).len(), 1);
    // Terminating: enter.
    give(
        &mut w,
        vec![Order::RoutePoint {
            actor: b2,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: Some(Target::Object(hut)),
            dest: None,
        }],
    );
    assert_eq!(notes(&w), vec![info("MSG:PostTerminatingCommand")]);
    give(&mut w, vec![add(b2, at(6, 6))]);
    assert_eq!(
        notes(&w),
        vec![RouteNote::Refused("MSG:PostTerminatingCommand")]
    );
    // A loop.
    let ps = route(&mut w, b3, &[at(2, 8), at(4, 8), at(6, 8)]);
    give(
        &mut w,
        vec![Order::RouteJoin {
            actor: b3,
            point: ps[0],
        }],
    );
    give(&mut w, vec![add(b3, at(8, 8))]);
    assert_eq!(notes(&w), vec![RouteNote::Refused("MSG:PostLoop")]);
    assert_eq!(steps(&w, b3).len(), 3);
    // 16 points.
    let cells: Vec<CellXY> = (0..16).map(|i| at(i - 8, 12)).collect();
    route(&mut w, b4, &cells);
    assert_eq!(steps(&w, b4).len(), routes::MAX_POINTS);
    assert_eq!(notes(&w).last(), Some(&info("MSG:NodeMaximum")));
    give(&mut w, vec![add(b4, at(9, 12))]);
    assert_eq!(notes(&w), vec![RouteNote::Refused("MSG:NodeMaximum")]);
    assert_eq!(steps(&w, b4).len(), routes::MAX_POINTS);
}

/// spec: S-0213 claim 16b (clicks on existing points: refused with the invalid-point message when a selected object with a route record lacks the point, has it last or one before last, or has a loop; otherwise joins or loop closes), claim 16d (kinds, the same-route-state rule before the holder limit) and claim 16 (the holder limit: count plus new objects more than 128) and claim 16e (no selection: a click on a point selects it)
#[test]
fn claim_16b_and_16d_click_tests() {
    let mut w = game();
    let (b1, b2) = (buggy(&mut w, 0, 0), buggy(&mut w, 1, 0));
    let hut = testkit::spawn(&mut w, Kind::Building, "HUT", 0, at(-6, -6), 0);
    let cs = [at(3, 3), at(5, 3), at(7, 3)];
    let ps = route(&mut w, b1, &cs);
    let refused = RouteClick::Refused("MSG:PlanningModeInvalidNodeX");
    assert_eq!(click(&w, &[b1], cs[2]), refused, "the last point");
    assert_eq!(click(&w, &[b1], cs[1]), refused, "the one before");
    assert_eq!(
        click(&w, &[b1], cs[0]),
        RouteClick::Orders(vec![Order::RouteJoin {
            actor: b1,
            point: ps[0]
        }])
    );
    assert_eq!(
        click(&w, &[b2], cs[2]),
        RouteClick::Orders(vec![Order::RouteJoin {
            actor: b2,
            point: ps[2]
        }]),
        "no route record: a join"
    );
    assert_eq!(
        click(&w, &[b1, b2], at(9, 9)),
        RouteClick::Refused("MSG:PlanningModeHeteroSel")
    );
    assert_eq!(
        click(&w, &[b2, hut], at(9, 9)),
        RouteClick::Refused("MSG:PlanningModeNoBuildings")
    );
    // spec: S-0213 claim 16e (with nothing selected a click on a point selects it; elsewhere it is an ordinary click)
    assert_eq!(click(&w, &[], cs[1]), RouteClick::SelectPoint(ps[1]));
    assert_eq!(click(&w, &[], at(9, 9)), RouteClick::Ordinary);
    // A plain point add.
    assert!(matches!(
        click(&w, &[b2], at(9, 9)),
        RouteClick::Orders(o) if matches!(o.as_slice(), [Order::RoutePoint { actor, behaviour: Behaviour::Move, .. }] if *actor == b2)
    ));
    // The holder limit: the count plus the selected objects holding no point.
    w.routes.counts.insert(A, HOLDER_LIMIT);
    assert_eq!(
        click(&w, &[b2], at(9, 9)),
        RouteClick::Refused("MSG:PlannerMaximum")
    );
    assert!(
        matches!(click(&w, &[b1], at(9, 9)), RouteClick::Orders(_)),
        "no new object"
    );
    // spec: S-0213 claim 16d (both refusals: only the incompatible-selection message)
    assert_eq!(
        click(&w, &[b1, b2], at(9, 9)),
        RouteClick::Refused("MSG:PlanningModeHeteroSel")
    );
}

/// spec: S-0213 claim 16b (a join gives the empty route a copy of a holder's route from the joined point to its end, with that route's loop) and claim 16f with Edge cases (quirks.route_holder_count_drift: on, a join that begins a route does not raise the count; off, the count is the number of holders)
#[test]
fn claim_16b_a_join_copies_the_rest_of_a_route() {
    for drift in [true, false] {
        let mut w = game_quirks(true, drift);
        let (b1, b2) = (buggy(&mut w, 0, 0), buggy(&mut w, 1, 0));
        let ps = route(&mut w, b1, &[at(3, 3), at(5, 3), at(7, 3)]);
        give(
            &mut w,
            vec![Order::RouteJoin {
                actor: b1,
                point: ps[0],
            }],
        );
        give(
            &mut w,
            vec![Order::RouteJoin {
                actor: b2,
                point: ps[1],
            }],
        );
        assert_eq!(steps(&w, b2), ps[1..].to_vec());
        // design (Q-2137): a loop whose first point lies before the joined point loops the whole copy
        assert_eq!(w.routes.routes[&b2].loop_from, Some(0));
        assert_eq!(
            routes::count(&w, A),
            if drift { 1 } else { 2 },
            "drift {drift}"
        );
    }
}

/// spec: S-0213 claim 16e (a point delete removes the point from every route; a loop left with fewer than 3 points is cancelled) and claim 16f with Edge cases (quirks.route_holder_count_drift: on, deleting the last remaining point ends holding without lowering the count; off, the count is the holders')
#[test]
fn claim_16e_a_point_delete_and_the_count() {
    for drift in [true, false] {
        let mut w = game_quirks(true, drift);
        let (b1, b2) = (buggy(&mut w, 0, 0), buggy(&mut w, 1, 0));
        let ps = route(&mut w, b1, &[at(3, 3), at(5, 3), at(7, 3), at(9, 3)]);
        give(
            &mut w,
            vec![Order::RouteJoin {
                actor: b1,
                point: ps[0],
            }],
        );
        give(&mut w, vec![Order::RouteDelete { point: ps[1] }]);
        assert_eq!(steps(&w, b1), vec![ps[0], ps[2], ps[3]]);
        assert_eq!(w.routes.routes[&b1].loop_from, Some(0), "3 points: kept");
        give(&mut w, vec![Order::RouteDelete { point: ps[2] }]);
        assert_eq!(steps(&w, b1), vec![ps[0], ps[3]]);
        assert_eq!(w.routes.routes[&b1].loop_from, None, "cancelled below 3");
        assert!(!w.routes.points.contains_key(&ps[1]));
        let p = route(&mut w, b2, &[at(3, 8)])[0];
        assert_eq!(routes::count(&w, A), 2);
        give(&mut w, vec![Order::RouteDelete { point: p }]);
        assert!(steps(&w, b2).is_empty());
        assert_eq!(
            routes::count(&w, A),
            if drift { 2 } else { 1 },
            "drift {drift}"
        );
        // spec: S-0213 claim 16f (a later order does not lower it: the route holds no point)
        give(&mut w, vec![Order::Stop { actor: b2 }]);
        assert_eq!(routes::count(&w, A), if drift { 2 } else { 1 });
    }
}

/// spec: S-0213 claim 16f (a destroyed holder removed from play stops holding, lowering the count, and leaves its points; a point no route holds is deleted)
#[test]
fn claim_16f_a_removed_holder_leaves_its_points() {
    let mut w = game();
    let (b1, b2) = (buggy(&mut w, 0, 0), buggy(&mut w, 1, 0));
    let (c1, c2) = (at(3, 3), at(5, 5));
    route(&mut w, b1, &[c1, c2]);
    route(&mut w, b2, &[c2]);
    assert_eq!(routes::count(&w, A), 2);
    glasnost_sim::objects::remove(&mut w, b1);
    testkit::run(&mut w, 1);
    assert!(!w.routes.routes.contains_key(&b1));
    assert_eq!(routes::point_in(&w, A, c1), None, "deleted");
    assert!(routes::point_in(&w, A, c2).is_some(), "still held");
    assert_eq!(routes::count(&w, A), 1);
}

/// spec: S-0213 claim 16b and Edge cases (quirks.spent_route_blocks_join: on, an object keeps its route record once its route is empty and fails the point-click test; off, an empty route counts as no record, so it can join again)
#[test]
fn claim_16b_spent_route_blocks_join_both_ways() {
    for blocks in [true, false] {
        let mut w = game_quirks(blocks, true);
        let (b1, b2) = (buggy(&mut w, 0, 0), buggy(&mut w, 1, 0));
        let p = route(&mut w, b1, &[at(3, 3)])[0];
        give(&mut w, vec![Order::RouteDelete { point: p }]);
        assert!(steps(&w, b1).is_empty());
        let cs = [at(5, 5), at(7, 5), at(9, 5)];
        let ps = route(&mut w, b2, &cs);
        let got = click(&w, &[b1], cs[0]);
        if blocks {
            assert_eq!(got, RouteClick::Refused("MSG:PlanningModeInvalidNodeX"));
        } else {
            assert_eq!(
                got,
                RouteClick::Orders(vec![Order::RouteJoin {
                    actor: b1,
                    point: ps[0]
                }])
            );
        }
    }
}

/// spec: S-0213 claim 16b (the deploy action is refused in route mode: a route-mode click that would order a deploy gives no order and adds no point) and S-0214 table VD row VD1 (route mode held: no-deploy)
#[test]
fn claim_16b_a_deploy_click_is_refused() {
    let mut r = ruleset();
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("DEPCAR".into());
    let mut car = r.sim.vehicles["BUGGY"].clone();
    car.is_simple_deployer = Some(true);
    r.sim.vehicles.insert("DEPCAR".into(), car);
    let mut w = game_from(&r);
    let m = testkit::spawn(&mut w, Kind::Vehicle, "DEPCAR", 0, at(0, 0), 0);
    // Outside route mode, the click on itself deploys (S-0214 table VD row VD3).
    let on = ClickOn::Object(m);
    let plain = targeting::click(&w, A, &[m], on, Modifiers::default());
    assert_eq!(plain.orders.len(), 1, "a deploy order outside route mode");
    let shown = targeting::cursor(&w, A, &[m], on, Modifiers::default());
    assert_eq!(shown, Action::Deploy);
    assert_eq!(targeting::in_route_mode(shown), Action::NoDeploy);
    assert_eq!(
        routes::click(
            &w,
            A,
            &[m],
            ClickOn::Object(m),
            at(0, 0),
            Modifiers::default()
        ),
        RouteClick::Ignored
    );
    assert!(steps(&w, m).is_empty());
}
