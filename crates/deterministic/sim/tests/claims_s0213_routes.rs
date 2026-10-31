//! Claim tests for S-0213's route mode (claims 16 to 16f and 17) and
//! S-0215 claim 13's route row, on synthetic data: the testkit's sandbox
//! game (houses Alpha 0 and Beta 1, not allied) with its invented vehicle
//! BUGGY and an invented building HUT. No stock values.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::routes::{self, HOLDER_LIMIT, Press, RouteClick, RouteNote};
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

/// An ordinary route-mode click's release on cell `c` (S-0213 claims 16b
/// and 16d): one outside every click area.
fn click(w: &World, sel: &[ObjectId], c: CellXY) -> RouteClick {
    routes::click(w, A, sel, ClickOn::Cell(c), Modifiers::default())
}

/// A route-mode press by Alpha with `sel` with the cursor in the click
/// areas of `under` (S-0213 claim 16b, *Click areas*).
fn press(w: &World, sel: &[ObjectId], under: &[u32]) -> Press {
    routes::press(w, A, sel, under)
}

/// Whether every step of `id`'s route is committed (S-0213 claim 16c:
/// being committed belongs to each route).
fn committed(w: &World, id: ObjectId) -> bool {
    w.routes.routes[&id].steps.iter().all(|s| s.committed)
}

/// A plain move order to `c`, outside route mode.
fn move_to(actor: ObjectId, c: CellXY) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

/// A press's join order for `point`, named by its place (S-0213 claim
/// 16b, *Click areas*).
fn join(w: &World, actor: ObjectId, point: u32) -> Order {
    let place = w.routes.points[&point].place;
    Order::RouteJoin {
        actor,
        point,
        place,
    }
}

/// Gives one join order (see [`join`]).
fn give_join(w: &mut World, actor: ObjectId, point: u32) {
    let o = join(w, actor, point);
    give(w, vec![o]);
}

fn current(w: &World, id: ObjectId) -> Option<Behaviour> {
    w.obj(id).unwrap().beh.current
}

/// spec: S-0213 claim 16a (r60: each object its own route; one click with several objects adds the same point, at the order's place, to each route, the point adds of one tick sharing it; a point add applied in a later tick makes a new point, even at an existing point's place) and claim 16b (an accepted point is announced; no information message for an ordinary point) and Terms (a first point raises the count)
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
    let s2 = steps(&w, b2);
    assert_eq!(s2[0], p1);
    assert_ne!(s2[1], p2, "a later tick: a new point at c2");
    assert_eq!(w.routes.points.len(), 3, "two points stand at c2");
    assert_eq!(routes::point_cell(&w, &w.routes.points[&s2[1]]), c2);
    // spec: S-0213 claim 16c (points added since the last commit are not followed)
    assert!(!committed(&w, b1) && !committed(&w, b2));
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
    assert!(committed(&w, b1));
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
    give_join(&mut w, b1, ps[1]);
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
    // spec: S-0213 claim 16c (r60: issuing a loop point moves it to the end, so the route holds the loop's points in the order they will next be issued) and claim 16f (the point issued before the loop, which no other route holds, is deleted)
    let mut held = steps(&w, b1);
    let next = held[0];
    held.sort_unstable();
    assert_eq!(held, ps[1..].to_vec(), "the point before the loop is gone");
    assert!(!w.routes.points.contains_key(&ps[0]));
    let order: Vec<u32> = (0..3)
        .map(|k| ps[1 + (ps[1..].iter().position(|&p| p == next).unwrap() + k) % 3])
        .collect();
    assert_eq!(
        steps(&w, b1),
        order,
        "the next point first, then the loop's order"
    );
    assert_eq!(w.routes.routes[&b1].loop_from, Some(0));
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
    give_join(&mut w, b3, ps[0]);
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

/// spec: S-0213 claim 16b, Click areas (a press inside a point's click area is tested by the point-click test: refused with the invalid-point message when a selected object with a route record lacks the point, has it last or one before last, or has a loop; otherwise the press takes it and sends joins named by the point's place), claim 16d (the kind test at a press outside every click area; the same-route-state test before the holder limit at the release) and claim 16 (the holder limit: count plus new objects more than 128) and claim 16e (no selection: a press takes the point to select it)
#[test]
fn claim_16b_and_16d_click_tests() {
    let mut w = game();
    let (b1, b2) = (buggy(&mut w, 0, 0), buggy(&mut w, 1, 0));
    let hut = testkit::spawn(&mut w, Kind::Building, "HUT", 0, at(-6, -6), 0);
    let cs = [at(3, 3), at(5, 3), at(7, 3)];
    let ps = route(&mut w, b1, &cs);
    assert_eq!(press(&w, &[b1], &[ps[2]]), Press::Refused, "the last point");
    assert_eq!(press(&w, &[b1], &[ps[1]]), Press::Refused, "the one before");
    assert_eq!(press(&w, &[b1], &[ps[0]]), Press::Take(ps[0]));
    assert_eq!(
        routes::press_orders(&w, A, &[b1], ps[0]),
        vec![join(&w, b1, ps[0])]
    );
    assert_eq!(
        press(&w, &[b2], &[ps[2]]),
        Press::Take(ps[2]),
        "no route record: a join"
    );
    // spec: S-0213 claim 16d (a press that takes a point meets only the point-click test: a selection of mixed route states is not refused for it)
    assert_eq!(press(&w, &[b1, b2], &[ps[0]]), Press::Take(ps[0]));
    // spec: S-0213 claim 16b, Click areas (with no click area under the cursor the press is an ordinary route-mode click)
    assert_eq!(press(&w, &[b1], &[]), Press::Ordinary);
    assert_eq!(
        click(&w, &[b1, b2], at(9, 9)),
        RouteClick::Refused("MSG:PlanningModeHeteroSel")
    );
    // spec: S-0213 claim 16d, When the tests are made (the kind test is made at the press outside every click area, with the message of the first failing object in selection order; none with nothing selected)
    assert_eq!(
        routes::press_kind_refusal(&w, A, &[b2, hut], &[]),
        Some("MSG:PlanningModeNoBuildings")
    );
    assert_eq!(routes::press_kind_refusal(&w, A, &[b2], &[]), None);
    assert_eq!(routes::press_kind_refusal(&w, A, &[], &[]), None);
    // spec: S-0213 claim 16d (a press inside a click area is not tested for kind)
    assert_eq!(routes::press_kind_refusal(&w, A, &[hut], &[ps[0]]), None);
    // spec: S-0213 claim 16e (with nothing selected a press takes the point under it, to select it; elsewhere it is an ordinary click)
    assert_eq!(press(&w, &[], &[ps[1]]), Press::Take(ps[1]));
    assert_eq!(press(&w, &[], &[]), Press::Ordinary);
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
    // spec: S-0213 claim 16d (a press that takes a point is not tested for the holder limit)
    assert_eq!(press(&w, &[b2], &[ps[2]]), Press::Take(ps[2]));
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
        give_join(&mut w, b1, ps[0]);
        give_join(&mut w, b2, ps[1]);
        // spec: S-0213 claim 16b (r60, Q-2137: a loop whose first point comes before the joined point: the copy goes on with the loop's points up to the one before the joined point, and its loop is the whole copy, beginning at the joined point)
        assert_eq!(steps(&w, b2), vec![ps[1], ps[2], ps[0]]);
        assert_eq!(w.routes.routes[&b2].loop_from, Some(0));
        // spec: S-0213 claim 16b (r60: the copied points are uncommitted until the next commit)
        assert!(!committed(&w, b2));
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
        give_join(&mut w, b1, ps[0]);
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
        let got = press(&w, &[b1], &[ps[0]]);
        if blocks {
            assert_eq!(got, Press::Refused);
        } else {
            assert_eq!(got, Press::Take(ps[0]));
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
        routes::click(&w, A, &[m], ClickOn::Object(m), Modifiers::default()),
        RouteClick::Ignored
    );
    assert!(steps(&w, m).is_empty());
}

/// spec: S-0213 claim 16b (r60, Q-2137: a join copies the route of the point's longest holder, the one whose route has held the point for the longest time, not the one with the lowest object number; the copy runs from the joined point to that route's end); Scenario O8 part r (1)
#[test]
fn claim_16b_a_join_copies_the_longest_holder() {
    let mut w = game();
    // B is created before A: the lower object number.
    let b = buggy(&mut w, 1, 0);
    let a = buggy(&mut w, 0, 0);
    let c = buggy(&mut w, 2, 0);
    let (p1, p2, p3, p4) = (at(3, 3), at(5, 3), at(7, 3), at(9, 3));
    let ps = route(&mut w, a, &[p1, p2, p3]);
    give_join(&mut w, b, ps[1]);
    assert_eq!(steps(&w, b), vec![ps[1], ps[2]]);
    give(&mut w, vec![add(b, p4)]);
    let q4 = point(&w, p4);
    assert_eq!(steps(&w, b), vec![ps[1], ps[2], q4]);
    give_join(&mut w, c, ps[1]);
    assert_eq!(steps(&w, c), vec![ps[1], ps[2]], "A's route, not B's");
}

/// spec: S-0213 claim 16b (r60: of holders given the point in the same tick, the one whose order was applied first counts as the longest holder: for the orders of one click, the first in selection order); Scenario O8 part r (5)
#[test]
fn claim_16b_a_join_tie_goes_to_the_first_applied() {
    for j_first in [true, false] {
        let mut w = game();
        let (j, k, l) = (
            buggy(&mut w, 0, 0),
            buggy(&mut w, 1, 0),
            buggy(&mut w, 2, 0),
        );
        let (s1, s2, s3) = (at(3, 3), at(5, 5), at(7, 7));
        let first = if j_first {
            vec![add(j, s1), add(k, s1)]
        } else {
            vec![add(k, s1), add(j, s1)]
        };
        give(&mut w, first);
        let p1 = point(&w, s1);
        assert_eq!(w.routes.points.len(), 1, "one tick: one shared point");
        give(&mut w, vec![add(j, s2)]);
        give(&mut w, vec![add(k, s3)]);
        let (q2, q3) = (point(&w, s2), point(&w, s3));
        give_join(&mut w, l, p1);
        let want = if j_first { vec![p1, q2] } else { vec![p1, q3] };
        assert_eq!(steps(&w, l), want, "j first: {j_first}");
    }
}

/// spec: S-0213 claim 16b (r60: a loop whose first point is the joined point or a later one is copied as it is, from the same first point; one whose first point comes before the joined point makes the whole copy a loop entered at the joined point, whatever point its holder is going to) and claim 16f (a point issued before a loop's first point, which no route holds any more, is deleted at once); Scenario O8 part r (2) to (4)
#[test]
fn claim_16b_joins_copy_loops() {
    let mut w = game();
    let (d, e, f, g, h) = (
        buggy(&mut w, 0, 0),
        buggy(&mut w, 1, 0),
        buggy(&mut w, 2, 0),
        buggy(&mut w, 3, 0),
        buggy(&mut w, 4, 0),
    );
    // (3) F's route R1 to R4, a loop from R2; G joins R1.
    let rs = route(&mut w, f, &[at(-9, 9), at(-7, 9), at(-5, 9), at(-3, 9)]);
    give_join(&mut w, f, rs[1]);
    give_join(&mut w, g, rs[0]);
    assert_eq!(steps(&w, g), rs);
    assert_eq!(w.routes.routes[&g].loop_from, Some(1), "the same loop");
    // (2) D at rest: Q1 to Q4 with a loop from Q2, committed; D starts at once and issues Q1.
    let (q1, q2, q3, q4) = (at(-2, 4), at(3, 4), at(3, 8), at(-2, 8));
    let qs = route(&mut w, d, &[q1, q2, q3, q4]);
    give_join(&mut w, d, qs[1]);
    give(&mut w, vec![Order::RouteCommit]);
    assert_eq!(steps(&w, d), qs[1..].to_vec());
    assert_eq!(routes::point_in(&w, A, q1), None, "Q1 is deleted");
    assert!(routes::point_in(&w, A, q2).is_some());
    give_join(&mut w, e, qs[2]);
    assert_eq!(steps(&w, e), vec![qs[2], qs[3], qs[1]]);
    assert_eq!(w.routes.routes[&e].loop_from, Some(0));
    // spec: S-0213 claim 16b (the copied points are uncommitted until the next commit)
    assert!(
        w.routes.routes[&e].steps.iter().all(|s| !s.committed),
        "a copy of D's committed loop is uncommitted"
    );
    let e_at = cell(&w, e);
    testkit::run(&mut w, 40);
    assert_eq!(cell(&w, e), e_at, "E waits for a commit");
    give(&mut w, vec![Order::RouteCommit]);
    assert!(committed(&w, e));
    // (4) After D has gone round its loop once, H joins Q4.
    run_until(&mut w, 3000, |w| cell(w, d) == q4);
    run_until(&mut w, 3000, |w| cell(w, d) == q2);
    let at_d = steps(&w, d);
    give_join(&mut w, h, qs[3]);
    let i = at_d.iter().position(|&p| p == qs[3]).unwrap();
    let want: Vec<u32> = at_d[i..].iter().chain(&at_d[..i]).copied().collect();
    assert_eq!(steps(&w, h), want);
    assert_eq!(steps(&w, h)[0], qs[3]);
    assert_eq!(w.routes.routes[&h].loop_from, Some(0), "the whole loop");
}

/// spec: S-0213 claims 16a and 16c (r60: being committed belongs to each route; a point add applied after a commit at a committed point's place adds a new, uncommitted point, and the actor waits there for the next commit; the routes holding the committed point are not changed); Scenario O8 part s
#[test]
fn claim_16c_a_point_add_at_a_committed_points_place_waits() {
    let mut w = game();
    let (v, wv) = (buggy(&mut w, 0, 0), buggy(&mut w, 0, 2));
    let (far, p1, y, q1) = (at(-15, -15), at(10, 0), at(10, 6), at(4, 2));
    give(&mut w, vec![move_to(v, far)]);
    testkit::run(&mut w, 2);
    let vs = route(&mut w, v, &[p1, y]);
    give(&mut w, vec![Order::RouteCommit]);
    assert!(committed(&w, v));
    // W at rest heads for Q1.
    route(&mut w, wv, &[q1]);
    give(&mut w, vec![Order::RouteCommit]);
    testkit::run(&mut w, 2);
    give(&mut w, vec![add(wv, y)]);
    let ws = steps(&w, wv);
    assert_eq!(ws.len(), 1);
    assert_ne!(ws[0], vs[1], "not V's committed point");
    assert!(!committed(&w, wv), "uncommitted");
    assert!(committed(&w, v), "V's route unchanged");
    assert_eq!(steps(&w, v), vs);
    run_until(&mut w, 600, |w| cell(w, wv) == q1);
    testkit::run(&mut w, 120);
    assert_eq!(cell(&w, wv), q1, "waits at Q1 for a commit");
    give(&mut w, vec![Order::RouteCommit]);
    run_until(&mut w, 600, |w| cell(w, wv) == y);
}

/// spec: S-0213 claim 16f (r60, Q-2136: a route point that no route holds any more is deleted at once, whichever way its last holder left it: an order that ends the route, or issuing it from a route without a loop); Scenario O8 part u
#[test]
fn claim_16f_points_no_route_holds_are_deleted() {
    let mut w = game();
    let (v, wv) = (buggy(&mut w, 0, 0), buggy(&mut w, 1, 0));
    let (far, p1, p2, p3) = (at(-15, 15), at(4, -6), at(8, -6), at(12, -6));
    give(&mut w, vec![move_to(v, far)]);
    testkit::run(&mut w, 2);
    let ps = route(&mut w, v, &[p1, p2, p3]);
    give(&mut w, vec![Order::RouteCommit]);
    give_join(&mut w, wv, ps[1]);
    give(&mut w, vec![Order::RouteCommit]);
    assert_eq!(steps(&w, wv), vec![ps[2]], "W starts at once and issues P2");
    // (1) every marker still stands: V holds them.
    for c in [p1, p2, p3] {
        assert!(routes::point_in(&w, A, c).is_some());
    }
    // (2) V's route ends by a plain move order.
    give(&mut w, vec![move_to(v, far)]);
    assert_eq!(routes::point_in(&w, A, p1), None);
    assert_eq!(routes::point_in(&w, A, p2), None);
    assert!(routes::point_in(&w, A, p3).is_some(), "W still holds P3");
    assert_eq!(
        press(&w, &[], &[ps[1]]),
        Press::Ordinary,
        "nothing to select"
    );
    // (3) W issues P3 at P2: no route holds it.
    run_until(&mut w, 600, |w| steps(w, wv).is_empty());
    assert_eq!(routes::point_in(&w, A, p3), None);
}

/// The testkit's ruleset with HUT and the engine's default area-guard
/// threshold, so that armed vehicles rest in guard (S-0215 claim 15).
fn guard_ruleset() -> Ruleset {
    let mut r = ruleset();
    r.sim.general.iq_guard_area = None;
    r
}

/// spec: S-0213 claim 16c (r60: giving up at the commit is a queue of wait with an immediate start, S-0215 claim 7: a ready holder has wait current from the commit until the issued order's behaviour becomes current; each issue at a fall-back to rest is preceded by the same queue of wait); Scenario O8 part t (1), with a vehicle and an infantry unit
#[test]
fn claim_16c_a_holder_waits_at_the_commit_and_before_each_issue() {
    for infantry in [false, true] {
        let mut w = game_from(&guard_ruleset());
        let u = if infantry {
            testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(0, 0), 0)
        } else {
            buggy(&mut w, 0, 0)
        };
        testkit::run(&mut w, 30);
        assert_eq!(
            current(&w, u),
            Some(Behaviour::Guard),
            "infantry {infantry}"
        );
        let (p1, p2) = (at(4, 0), at(4, 4));
        route(&mut w, u, &[p1, p2]);
        give(&mut w, vec![Order::RouteCommit]);
        assert_eq!(
            current(&w, u),
            Some(Behaviour::Wait),
            "wait current at the commit"
        );
        run_until(&mut w, 60, |w| current(w, u) == Some(Behaviour::Move));
        run_until(&mut w, 1200, |w| cell(w, u) == p1);
        let mut waited = false;
        for _ in 0..600 {
            let heading = w.obj(u).unwrap().dest == Some(Target::Cell(p2));
            if heading && current(&w, u) == Some(Behaviour::Move) {
                break;
            }
            waited |= current(&w, u) == Some(Behaviour::Wait);
            testkit::run(&mut w, 1);
        }
        assert!(
            waited,
            "a tick of wait at P1 before P2 (infantry {infantry})"
        );
        run_until(&mut w, 1200, |w| cell(w, u) == p2);
    }
}

/// spec: S-0213 claim 16c (r60: the commit applies at its tick's order boundary, so the holder keeps its target in the commit's tick; wait is first current at S-0215 claim 33's drop rule in the next tick, which drops an opportunity target, and the holder makes no shot at it until the issued order's behaviour is current); Scenario O8 part t (2)
#[test]
fn claim_16c_the_wait_drops_an_opportunity_target() {
    // The testkit's combat ruleset (armed CART), resting in guard.
    let mut r = testkit::combat_ruleset();
    r.sim.general.iq_guard_area = None;
    let mut w = game_from(&r);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, at(0, 0), 0);
    let e = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, at(3, 0), 0);
    run_until(&mut w, 120, |w| {
        let o = w.obj(v).unwrap();
        o.target == Some(Target::Object(e)) && o.combat.opportunity
    });
    assert_eq!(current(&w, v), Some(Behaviour::Guard));
    route(&mut w, v, &[at(-6, 0), at(-6, 6)]);
    assert_eq!(w.obj(v).unwrap().target, Some(Target::Object(e)));
    give(&mut w, vec![Order::RouteCommit]);
    assert_eq!(
        w.obj(v).unwrap().target,
        Some(Target::Object(e)),
        "kept in the commit's tick"
    );
    let health = w.obj(e).unwrap().health;
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(v).unwrap().target, None, "dropped in the next tick");
    while current(&w, v) != Some(Behaviour::Move) {
        assert_eq!(w.obj(e).unwrap().health, health, "no shot under wait");
        assert_eq!(current(&w, v), Some(Behaviour::Wait));
        testkit::run(&mut w, 1);
    }
}

/// spec: S-0213 claim 16b, Two points at one place (r74, Q-2166: the earliest-made point is the candidate whatever the selected routes hold; a selection whose routes hold only a later-made one is refused; with nothing selected the press takes the earliest-made) and Edge cases (quirks.stacked_point_takes_earliest off: with objects selected, the earliest-made point that a selected route holds, else the earliest-made); Scenario O8 part w (6)
#[test]
fn claim_16b_two_points_at_one_place_both_ways() {
    for earliest in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.stacked_point_takes_earliest = earliest;
        let mut w = game_from(&r);
        let (v, wv) = (buggy(&mut w, 0, 0), buggy(&mut w, 1, 0));
        let y = at(6, 6);
        let vs = route(&mut w, v, &[y, at(8, 6), at(10, 6)]);
        route(&mut w, wv, &[y, at(6, 8), at(6, 10)]);
        let ws = steps(&w, wv);
        assert!(vs[0] < ws[0], "V's point is the earlier made");
        let under = [ws[0], vs[0]];
        assert_eq!(press(&w, &[v], &under), Press::Take(vs[0]), "V's own point");
        let want = if earliest {
            Press::Refused
        } else {
            Press::Take(ws[0])
        };
        assert_eq!(press(&w, &[wv], &under), want, "earliest {earliest}");
        assert_eq!(
            press(&w, &[], &under),
            Press::Take(vs[0]),
            "nothing selected"
        );
    }
}

/// spec: S-0213 claim 16b, Click areas (the orders a press sends name the point by its place; when applied they act on the house's earliest-made point at that place, and do nothing when no point of the house is there) and Edge cases (quirks.stacked_point_takes_earliest off: they act on the point they name, and do nothing when it has gone)
#[test]
fn claim_16b_a_press_order_acts_on_the_earliest_point_at_its_place() {
    for earliest in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.stacked_point_takes_earliest = earliest;
        let mut w = game_from(&r);
        let (v, wv, x) = (
            buggy(&mut w, 0, 0),
            buggy(&mut w, 1, 0),
            buggy(&mut w, 2, 0),
        );
        let y = at(6, 6);
        let vs = route(&mut w, v, &[y, at(8, 6), at(10, 6)]);
        let ws = route(&mut w, wv, &[y, at(6, 8), at(6, 10)]);
        let ws0 = steps(&w, wv)[0];
        assert_ne!(ws[0], ws0);
        // An order naming W's later-made point at Y: with the quirk on it acts on V's.
        give_join(&mut w, x, ws0);
        let want = if earliest {
            vec![vs[0], vs[1], vs[2]]
        } else {
            steps(&w, wv)
        };
        assert_eq!(steps(&w, x), want, "earliest {earliest}");
        // Once no point of the house stands at the place, the order does nothing.
        let o = join(&w, x, ws0);
        give(
            &mut w,
            vec![Order::Stop { actor: v }, Order::Stop { actor: wv }],
        );
        give(&mut w, vec![Order::Stop { actor: x }]);
        give(&mut w, vec![o]);
        assert!(steps(&w, x).is_empty());
    }
}

/// [`game`] with one of intake 56's route quirks set (S-0213 Edge cases).
fn game_with_quirk(set: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) -> World {
    let mut r = ruleset();
    set(&mut r.globals.quirks);
    game_from(&r)
}

/// spec: S-0213 claim 16b, Click areas (a square of 16 by 16 screen pixels around the screen position of the point's place: from 8 left of and above it to 7 right of and below it); Scenario O8 part w (1) and (2)
#[test]
fn claim_16b_the_click_area_is_a_16_pixel_square() {
    // (1) joins: 7 right and 7 below, 8 left and 8 above, 8 left level, 8 straight above.
    for (dx, dy) in [(7, 7), (-8, -8), (-8, 0), (0, -8), (0, 0)] {
        assert!(routes::in_click_area(dx, dy), "({dx}, {dy}) inside");
    }
    // (2) point adds: 8 right, 8 below, 9 left level, 9 straight above.
    for (dx, dy) in [(8, 0), (0, 8), (-9, 0), (0, -9)] {
        assert!(!routes::in_click_area(dx, dy), "({dx}, {dy}) outside");
    }
}

/// spec: S-0213 claim 16b, Click areas (a press that takes no highlighted point is tested against the earliest-made point of any house, drawn or not, whose click area holds the cursor) and Edge cases (quirks.hidden_point_refuses_click off: only points drawn for the player are tested, so an unseen point never refuses a press)
#[test]
fn claim_16b_a_hidden_point_refuses_a_press_both_ways() {
    for hidden in [true, false] {
        let mut w = game_with_quirk(|q| q.hidden_point_refuses_click = hidden);
        let b1 = buggy(&mut w, 0, 0);
        route(&mut w, b1, &[at(3, 3), at(5, 3)]);
        // Beta, not allied with Alpha, makes a point at another place.
        let enemy = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, at(0, 6), 0);
        assert!(orders::issue(&mut w, HouseId(1), add(enemy, at(8, 8))));
        testkit::run(&mut w, 1);
        let theirs = routes::point_in(&w, HouseId(1), at(8, 8)).expect("Beta's point");
        assert!(!routes::drawn_for(&w, A, &w.routes.points[&theirs]));
        let want = if hidden {
            Press::Refused
        } else {
            Press::Ordinary
        };
        assert_eq!(press(&w, &[b1], &[theirs]), want, "hidden {hidden}");
        // A selection with no route record accepts every point: an ordinary click.
        let b2 = buggy(&mut w, 1, 0);
        assert_eq!(press(&w, &[b2], &[theirs]), Press::Ordinary);
        // spec: S-0213 claim 16b, Click areas (a point of another house is never the candidate a press takes)
        assert_eq!(press(&w, &[], &[theirs]), Press::Ordinary);
    }
}

/// spec: S-0213 claim 16d, When the tests are made (a press inside a click area is not tested for kind, so a selected building is not refused there) and Edge cases (quirks.click_area_skips_kind_test off: every route-mode battlefield press with objects selected meets the kind test first); Scenario O8 part x (5)
#[test]
fn claim_16d_a_click_area_skips_the_kind_test_both_ways() {
    for skips in [true, false] {
        let mut w = game_with_quirk(|q| q.click_area_skips_kind_test = skips);
        let b1 = buggy(&mut w, 0, 0);
        let hut = testkit::spawn(&mut w, Kind::Building, "HUT", 0, at(-6, -6), 0);
        let ps = route(&mut w, b1, &[at(3, 3), at(5, 3), at(7, 3)]);
        let got = routes::press_kind_refusal(&w, A, &[hut], &[ps[0]]);
        let want = (!skips).then_some("MSG:PlanningModeNoBuildings");
        assert_eq!(got, want, "skips {skips}");
        // Outside every click area the kind test is made either way.
        assert_eq!(
            routes::press_kind_refusal(&w, A, &[hut], &[]),
            Some("MSG:PlanningModeNoBuildings")
        );
        if skips {
            // spec: S-0213 claim 16d (a building with no route record then joins the point and gets a route, low); design (Q-2192): what it does with it is not studied
            assert_eq!(press(&w, &[hut], &[ps[0]]), Press::Take(ps[0]));
            let os = routes::press_orders(&w, A, &[hut], ps[0]);
            give(&mut w, os);
            assert_eq!(steps(&w, hut), ps);
        }
    }
}

/// spec: S-0213 claim 16b, Who gets a press's orders (the selection in selection order; a selected object with no route record or an empty route listed itself; for one whose route holds points, every object whose route holds that route's last point, in the order they were given it, selected or not, each once) and Joins; Scenario O8 part w (8)
#[test]
fn claim_16b_a_press_orders_the_holders_of_the_last_point() {
    let mut w = game();
    let (v, wv, x, u) = (
        buggy(&mut w, 0, 0),
        buggy(&mut w, 1, 0),
        buggy(&mut w, 2, 0),
        buggy(&mut w, 3, 0),
    );
    let ps = route(&mut w, v, &[at(3, 3), at(6, 3), at(9, 3)]);
    // W joins P2: its route is P2 and P3; X joins P1: P1, P2 and P3.
    give_join(&mut w, wv, ps[1]);
    give_join(&mut w, x, ps[0]);
    assert_eq!(steps(&w, wv), ps[1..].to_vec());
    assert_eq!(steps(&w, x), ps);
    assert_eq!(routes::press_list(&w, A, &[v]), vec![v, wv, x]);
    assert_eq!(routes::press_list(&w, A, &[u, v]), vec![u, v, wv, x]);
    assert_eq!(
        routes::press_list(&w, A, &[x, v]),
        vec![v, wv, x],
        "each once"
    );
    // V alone presses P1's marker.
    assert_eq!(press(&w, &[v], &[ps[0]]), Press::Take(ps[0]));
    let os = routes::press_orders(&w, A, &[v], ps[0]);
    assert_eq!(
        os,
        vec![join(&w, v, ps[0]), join(&w, wv, ps[0]), join(&w, x, ps[0])]
    );
    give(&mut w, os);
    // V and X close a loop; W, which lacks P1, joins after its own points.
    assert_eq!(w.routes.routes[&v].loop_from, Some(0));
    assert_eq!(w.routes.routes[&x].loop_from, Some(0));
    assert_eq!(steps(&w, wv), vec![ps[1], ps[2], ps[0], ps[1], ps[2]]);
    assert_eq!(w.routes.routes[&wv].loop_from, Some(2));
    assert_eq!(w.routes.routes[&wv].loop_to, None);
    // spec: S-0213 claim 16h (selecting V while the key is held also selects the other holders of its last point, in the order they were given it)
    assert_eq!(routes::also_selected(&w, v), vec![wv, x]);
}

/// spec: S-0213 claim 16b, Who gets a press's orders (the copy an object that is not selected is given goes after its own points whatever their number, so its route can pass 16 points; its loop begins at the first point of its route that is the copied loop's first point, and when that is one of its own points the loop has as many points as the copied loop, ending before the route's last point) and Constants (a point add is refused for length only at exactly 16) and Edge cases (quirks.press_copy_ignores_route_state off: the join does nothing past 16 points and the copy's loop begins in the copy)
#[test]
fn claim_16b_a_press_copy_ignores_route_state_both_ways() {
    for ignores in [true, false] {
        let mut w = game_with_quirk(|q| q.press_copy_ignores_route_state = ignores);
        let (v, wv) = (buggy(&mut w, 0, 0), buggy(&mut w, 1, 0));
        let cs = [at(3, 3), at(6, 3), at(9, 3), at(9, 6), at(3, 9)];
        // One tick: V's P1 to P4, and W's P2 (shared) and P5.
        give(
            &mut w,
            vec![
                add(v, cs[0]),
                add(v, cs[1]),
                add(wv, cs[1]),
                add(v, cs[2]),
                add(v, cs[3]),
                add(wv, cs[4]),
            ],
        );
        let p: Vec<u32> = cs.iter().map(|&c| point(&w, c)).collect();
        assert_eq!(steps(&w, wv), vec![p[1], p[4]]);
        // V closes a loop at P2: P2, P3 and P4.
        give_join(&mut w, v, p[1]);
        assert_eq!(w.routes.routes[&v].loop_from, Some(1));
        // W, not holding P1, is given V's route from P1.
        give_join(&mut w, wv, p[0]);
        let r = &w.routes.routes[&wv];
        assert_eq!(steps(&w, wv), vec![p[1], p[4], p[0], p[1], p[2], p[3]]);
        if ignores {
            assert_eq!((r.loop_from, r.loop_to), (Some(0), Some(3)), "P2, P5, P1");
        } else {
            assert_eq!((r.loop_from, r.loop_to), (Some(3), None), "in the copy");
        }
        // design (Q-2191): a loop that ends before the route's last point is gone round as any loop: the issued point moves to the loop's end and the points after it are never issued
        give(&mut w, vec![Order::RouteCommit]);
        let r = &w.routes.routes[&wv];
        if ignores {
            assert_eq!(steps(&w, wv), vec![p[4], p[0], p[1], p[1], p[2], p[3]]);
            assert_eq!((r.loop_from, r.loop_to), (Some(0), Some(3)));
        } else {
            assert_eq!(steps(&w, wv), vec![p[4], p[0], p[1], p[2], p[3]]);
            assert_eq!((r.loop_from, r.loop_to), (Some(2), None));
        }
    }
    // Past 16 points.
    for ignores in [true, false] {
        let mut w = game_with_quirk(|q| q.press_copy_ignores_route_state = ignores);
        let (v, wv) = (buggy(&mut w, 0, 0), buggy(&mut w, 1, 0));
        let ps = route(&mut w, v, &[at(3, 14), at(6, 14), at(9, 14)]);
        let cells: Vec<CellXY> = (0..15).map(|i| at(i - 8, 12)).collect();
        route(&mut w, wv, &cells);
        give_join(&mut w, wv, ps[0]);
        let n = steps(&w, wv).len();
        assert_eq!(n, if ignores { 18 } else { 15 }, "ignores {ignores}");
        if ignores {
            // spec: S-0213 Constants (a route that holds more than 16 points is not refused a point add for its length)
            give(&mut w, vec![add(wv, at(12, 14))]);
            assert_eq!(notes(&w), vec![RouteNote::Added { info: None }]);
            assert_eq!(steps(&w, wv).len(), 19);
        }
    }
}

/// spec: S-0213 claim 16e, The delete key (when a point-delete order of the local player's house is applied, the point after the deleted one in the route of its longest holder, going round a loop, becomes the selected point, if there is one); Scenario O8 part z (1)
#[test]
fn claim_16e_a_point_delete_selects_the_next_point() {
    let mut w = game();
    let v = buggy(&mut w, 0, 0);
    let ps = route(
        &mut w,
        v,
        &[at(3, 3), at(5, 3), at(7, 3), at(9, 3), at(9, 6)],
    );
    give(&mut w, vec![Order::RouteDelete { point: ps[1] }]);
    assert_eq!(notes(&w), vec![RouteNote::Selected(ps[2])]);
    give_join(&mut w, v, ps[2]);
    // A loop of P3, P4 and P5: after P5 comes P3.
    give(&mut w, vec![Order::RouteDelete { point: ps[4] }]);
    assert_eq!(notes(&w), vec![RouteNote::Selected(ps[2])]);
    assert_eq!(w.routes.routes[&v].loop_from, None, "cancelled below 3");
    // The route's last point, with no loop: no next point.
    give(&mut w, vec![Order::RouteDelete { point: ps[3] }]);
    assert!(notes(&w).is_empty());
    assert_eq!(steps(&w, v), vec![ps[0], ps[2]]);
}
