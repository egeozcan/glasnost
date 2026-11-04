//! Claim tests for S-0213 revisions 84 and 87 (human orders: route points' recorded
//! orders and places, attack moves in a loop, a loop that ends early, and
//! aircraft and building holders) on synthetic data: invented types, no
//! game data.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::air;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::routes::{self, RouteClick, RouteNote};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::absorb;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 48;
const A: HouseId = HouseId(0);

/// The testkit's combat ruleset (armed CART, unarmed BUGGY, the armed
/// building TOWER) with
/// - SAMS: TOWER with the SAM flag;
/// - POST: TOWER without the turret flag, so that its idle body range sets
///   its ready mark (S-0215 claim 11) under a behaviour it does not process;
/// - FAC: a 2 × 2 vehicle factory;
/// - DEPOT: a 3 × 3 repair building whose cells let vehicles through;
/// - TRUCK: an unarmed wheeled vehicle;
/// - HELI: an unarmed flying aircraft with no dock that may land anywhere;
/// - BIO: a 2 × 2 infantry absorber for three;
/// - the engine's default area-guard threshold, so that armed objects rest
///   in guard (S-0215 claim 15).
fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.iq_guard_area = None;
    let mut push = |reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push("buildings", "SAMS");
    push("buildings", "POST");
    push("buildings", "FAC");
    push("buildings", "DEPOT");
    push("vehicles", "TRUCK");
    push("aircraft", "HELI");
    push("buildings", "BIO");
    let s = &mut r.sim;
    let mut sam = s.buildings["TOWER"].clone();
    sam.sam = Some(true);
    s.buildings.insert("SAMS".into(), sam);
    let mut post = s.buildings["TOWER"].clone();
    post.turret = None;
    s.buildings.insert("POST".into(), post);
    s.buildings.insert(
        "FAC".into(),
        BuildingType {
            strength: Some(500),
            foundation: Some([2, 2]),
            factory: Some("vehicle".into()),
            ..Default::default()
        },
    );
    s.buildings.insert(
        "DEPOT".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([3, 3]),
            unit_repair: Some(true),
            number_impassable_rows: Some(0),
            ..Default::default()
        },
    );
    s.buildings.insert(
        "BIO".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([2, 2]),
            infantry_absorb: Some(true),
            passengers: Some(3),
            size_limit: Some(Decimal("1".into())),
            ..Default::default()
        },
    );
    s.vehicles.insert(
        "TRUCK".into(),
        UnitType {
            strength: Some(600),
            speed: Some(8),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            ..Default::default()
        },
    );
    let mut heli = UnitType {
        strength: Some(200),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        ..Default::default()
    };
    heli.econ.landable = Some(true);
    s.aircraft.insert("HELI".into(), heli);
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

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
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

/// A point add of `behaviour` to the cell `c`, at that cell.
fn add_as(actor: ObjectId, behaviour: Behaviour, c: CellXY) -> Order {
    Order::RoutePoint {
        actor,
        behaviour,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
        place: Some(Target::Cell(c)),
    }
}

/// A point add of a move to `c`.
fn add(actor: ObjectId, c: CellXY) -> Order {
    add_as(actor, Behaviour::Move, c)
}

/// A point add of an attack on object `t`.
fn add_attack(actor: ObjectId, t: ObjectId) -> Order {
    Order::RoutePoint {
        actor,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(t)),
        dest: None,
        place: Some(Target::Object(t)),
    }
}

fn steps(w: &World, id: ObjectId) -> Vec<u32> {
    w.routes
        .routes
        .get(&id)
        .map(|r| r.steps.iter().map(|s| s.point).collect())
        .unwrap_or_default()
}

/// The attack-move marks of `id`'s steps, in order.
fn marks(w: &World, id: ObjectId) -> Vec<bool> {
    w.routes.routes[&id]
        .steps
        .iter()
        .map(|s| s.attack_move)
        .collect()
}

fn point(w: &World, c: CellXY) -> u32 {
    routes::point_in(w, A, c).expect("a point in the cell")
}

fn route(w: &mut World, id: ObjectId, cells: &[CellXY]) -> Vec<u32> {
    give(w, cells.iter().map(|&c| add(id, c)).collect());
    cells.iter().map(|&c| point(w, c)).collect()
}

fn join(w: &World, actor: ObjectId, point: u32) -> Order {
    let place = w.routes.points[&point].place;
    Order::RouteJoin {
        actor,
        point,
        place,
    }
}

fn give_join(w: &mut World, actor: ObjectId, point: u32) {
    let o = join(w, actor, point);
    give(w, vec![o]);
}

fn current(w: &World, id: ObjectId) -> Option<Behaviour> {
    w.obj(id).unwrap().beh.current
}

fn cell(w: &World, id: ObjectId) -> CellXY {
    w.obj(id).unwrap().pos.cell()
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).unwrap().health
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

fn notes(w: &World) -> Vec<RouteNote> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Route { house, note } if *house == A => Some(*note),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The order's place and the recorded order (claim 16b)

/// spec: S-0213 claim 16b (revision 84: the point stands at the order's place: the target's; a move order names no target and its place is the clicked cell's, or for a click on an object that of the cell the object stands on at the click, whatever the move's destination; any other order that names no target has its destination's place; an order that names neither has no place)
#[test]
fn claim_16b_the_orders_place() {
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 0);
    let x = put(&mut w, Kind::Vehicle, "BUGGY", 0, 3, 0);
    let (c, d) = (at(5, 5), at(6, 5));
    let place = |on, b, _attack_move: bool, t, de| routes::order_place(&w, on, b, t, de);
    // A move: the clicked cell, not a destination the nearby-cell search found.
    assert_eq!(
        place(
            ClickOn::Cell(c),
            Behaviour::Move,
            false,
            None,
            Some(Target::Cell(d))
        ),
        Some(Target::Cell(c))
    );
    // A move on an object: the cell the object stands on at the click.
    assert_eq!(
        place(
            ClickOn::Object(x),
            Behaviour::Move,
            false,
            None,
            Some(Target::Cell(d))
        ),
        Some(Target::Cell(at(3, 0)))
    );
    // A patrol: its destination's place.
    assert_eq!(
        place(
            ClickOn::Cell(c),
            Behaviour::Patrol,
            false,
            None,
            Some(Target::Cell(d))
        ),
        Some(Target::Cell(d))
    );
    // An order with a target: the target's place.
    assert_eq!(
        place(
            ClickOn::Object(x),
            Behaviour::Attack,
            false,
            Some(Target::Object(x)),
            None
        ),
        Some(Target::Object(x))
    );
    // Neither a target nor a destination: no place.
    assert_eq!(
        place(ClickOn::Object(v), Behaviour::Unload, false, None, None),
        None
    );
    // An ordinary click's point add carries the place.
    let RouteClick::Orders(os) = routes::click(&w, A, &[v], ClickOn::Cell(c), Modifiers::default())
    else {
        panic!("a point add")
    };
    assert!(matches!(
        os.as_slice(),
        [Order::RoutePoint { place: Some(Target::Cell(p)), .. }] if *p == c
    ));
}

/// spec: S-0213 claim 16b (revision 84: an order that names neither a target nor a destination has no place: when it is applied it adds no point, with no message and no sound, and has no other effect) and Q-2192 question 3 (an aircraft's deploy, the behaviour unload, adds no point and the aircraft does not unload)
#[test]
fn claim_16b_an_order_with_no_place_adds_nothing() {
    let mut w = game();
    let heli = put(&mut w, Kind::Aircraft, "HELI", 0, 0, 0);
    testkit::run(&mut w, 2);
    let before = current(&w, heli);
    give(
        &mut w,
        vec![Order::RoutePoint {
            actor: heli,
            behaviour: Behaviour::Unload,
            attack_move: false,
            target: None,
            dest: None,
            place: None,
        }],
    );
    assert!(steps(&w, heli).is_empty());
    assert!(w.routes.points.is_empty());
    assert!(notes(&w).is_empty(), "no message");
    assert_eq!(routes::count(&w, A), 0);
    testkit::run(&mut w, 10);
    assert_eq!(current(&w, heli), before, "no other effect");
}

/// spec: S-0213 claim 16b (revision 84: the order a holder issues for a point is the one recorded for it at the add; it is not classified anew when it is issued: a full-health vehicle's point on an own repair building that accepts it issues enter, S-0214 table V row n3) and Changes for other specs (S-0215 claim 13: the route row's service guards and cell fallback do not apply to these points); Scenario O8 part ab
#[test]
fn claim_16b_a_point_issues_its_recorded_order() {
    for damaged in [false, true] {
        let mut w = game();
        let depot = put(&mut w, Kind::Building, "DEPOT", 0, 8, 0);
        let v = put(&mut w, Kind::Vehicle, "TRUCK", 0, 0, 0);
        if damaged {
            w.obj_mut(v).unwrap().health = 300;
        }
        let full = health(&w, v);
        let p1 = at(3, 3);
        let RouteClick::Orders(os) =
            routes::click(&w, A, &[v], ClickOn::Object(depot), Modifiers::default())
        else {
            panic!("a point add on the depot")
        };
        let [
            Order::RoutePoint {
                behaviour: Behaviour::Enter,
                ..
            },
        ] = os.as_slice()
        else {
            panic!("an enter point: {os:?}")
        };
        give(&mut w, vec![add(v, p1)]);
        give(&mut w, os);
        assert_eq!(steps(&w, v).len(), 2);
        give(&mut w, vec![Order::RouteCommit]);
        run_until(&mut w, 1200, |w| steps(w, v).is_empty());
        run_until(&mut w, 30, |w| current(w, v) == Some(Behaviour::Enter));
        assert_eq!(
            w.obj(v).unwrap().dest,
            Some(Target::Object(depot)),
            "damaged {damaged}: enter with the depot as destination"
        );
        assert!(health(&w, v) >= full);
    }
}

// ---------------------------------------------------------------------------
// Attack moves in a loop (claim 16c)

/// spec: S-0213 claim 16c, Attack moves in a loop (revision 84: when a loop-close order is applied and a holder of the loop's first point is of a type eligible for attack moves, every point from the loop's first point to the route's end whose order is a move becomes an attack-move order with the same destination; points before the loop's first point keep their orders; a holder of a type that is not eligible moves as before); Scenario O8 part aa
#[test]
fn claim_16c_a_loop_close_makes_attack_moves() {
    for armed in [true, false] {
        let mut w = game();
        let v = put(
            &mut w,
            Kind::Vehicle,
            if armed { "CART" } else { "BUGGY" },
            0,
            0,
            0,
        );
        let ps = route(&mut w, v, &[at(2, 6), at(4, 6), at(6, 6), at(8, 6)]);
        assert_eq!(marks(&w, v), vec![false; 4]);
        give_join(&mut w, v, ps[1]);
        assert_eq!(w.routes.routes[&v].loop_from, Some(1));
        let want = if armed {
            vec![false, true, true, true]
        } else {
            vec![false; 4]
        };
        assert_eq!(marks(&w, v), want, "armed {armed}");
        if armed {
            // The changed order keeps the destination; claim 6 translates it at the issue.
            let r = &w.routes.routes[&v];
            assert!(r.steps.iter().all(|s| s.behaviour == Behaviour::Move));
            assert_eq!(r.steps[1].dest, Some(Target::Cell(at(4, 6))));
            give(&mut w, vec![Order::RouteCommit]);
            run_until(&mut w, 1200, |w| {
                w.obj(v).unwrap().dest == Some(Target::Cell(at(4, 6)))
            });
            assert!(
                w.obj(v).unwrap().attack_move.is_some(),
                "an attack move under claim 6's translation"
            );
        }
    }
}

/// spec: S-0213 claim 16c, Attack moves in a loop (the change is made at every loop close, a second loop closed in the same tick included; the original's disagreement between clients there is not copied) and claim 1 (all orders issued in the same tick are applied together) (M8 seam sweep 5a)
#[test]
fn claim_16c_a_second_loop_closed_in_the_same_tick_changes_too() {
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    let u = put(&mut w, Kind::Vehicle, "CART", 0, 1, 0);
    let vs = route(&mut w, v, &[at(2, 6), at(4, 6), at(6, 6)]);
    let us = route(&mut w, u, &[at(2, 9), at(4, 9), at(6, 9)]);
    // Both loop closes in one tick.
    let (jv, ju) = (join(&w, v, vs[0]), join(&w, u, us[1]));
    give(&mut w, vec![jv, ju]);
    assert_eq!(w.routes.routes[&v].loop_from, Some(0));
    assert_eq!(w.routes.routes[&u].loop_from, Some(1));
    assert_eq!(marks(&w, v), vec![true; 3], "the first loop");
    assert_eq!(marks(&w, u), vec![false, true, true], "the second loop");
}

/// spec: S-0213 claim 16c, Attack moves in a loop (revision 84: the test is made per holder: at a point that holds different actions for different holders, each holder whose action there is a move gets the attack-move order and each whose action is not a move keeps it; eligibility is read from the holders of the loop's first point only; the change is made for every holder of each point)
#[test]
fn claim_16c_attack_moves_per_holder() {
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    let u = put(&mut w, Kind::Vehicle, "BUGGY", 0, 1, 0);
    let (p1, c, p3) = (at(2, 6), at(5, 6), at(8, 6));
    // One tick: V's P1, V's patrol to C, U's move to C (a shared point), V's P3.
    give(
        &mut w,
        vec![
            add(v, p1),
            add_as(v, Behaviour::Patrol, c),
            add(u, c),
            add(v, p3),
        ],
    );
    let pc = point(&w, c);
    assert_eq!(steps(&w, u), vec![pc]);
    assert_eq!(steps(&w, v)[1], pc);
    let q1 = point(&w, p1);
    give_join(&mut w, v, q1);
    // V's patrol at C is kept; its moves become attack moves; U's move at C changes too, though U is not eligible.
    assert_eq!(marks(&w, v), vec![true, false, true]);
    assert_eq!(marks(&w, u), vec![true]);
    // An ineligible type at the first point decides nothing alone: a loop closed by U changes nothing.
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    let u = put(&mut w, Kind::Vehicle, "BUGGY", 0, 1, 0);
    let us = route(&mut w, u, &[at(2, 9), at(4, 9), at(6, 9)]);
    give(&mut w, vec![add(v, at(9, 9))]);
    give_join(&mut w, u, us[0]);
    assert_eq!(w.routes.routes[&u].loop_from, Some(0));
    assert_eq!(marks(&w, u), vec![false; 3]);
    assert_eq!(marks(&w, v), vec![false]);
}

/// spec: S-0213 claim 16c, Attack moves in a loop (revision 84: a building holder of such a point gets claim 6's building result, attack move itself, which S-0215 claim 5 processes as sleep: the building does nothing there and issues no further point) and claim 6 (a building's attack-move order is the behaviour attack move)
#[test]
fn claim_16c_a_building_holder_of_a_loops_move_point() {
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    let b = put(&mut w, Kind::Building, "TOWER", 0, -10, -10);
    let ps = route(&mut w, v, &[at(2, 6), at(4, 6), at(6, 6)]);
    give_join(&mut w, b, ps[0]);
    assert_eq!(steps(&w, b), ps);
    give_join(&mut w, v, ps[0]);
    assert_eq!(marks(&w, b), vec![true; 3], "the building's points change");
    give(&mut w, vec![Order::RouteCommit]);
    run_until(&mut w, 30, |w| current(w, b) == Some(Behaviour::AttackMove));
    let left = steps(&w, b);
    assert_eq!(left, ps[1..].to_vec());
    let e = put(&mut w, Kind::Vehicle, "BUGGY", 1, -8, -10);
    let hp = health(&w, e);
    testkit::run(&mut w, 300);
    assert_eq!(current(&w, b), Some(Behaviour::AttackMove));
    assert_eq!(steps(&w, b), left, "no further point");
    assert_eq!(health(&w, e), hp, "no shot");
}

// ---------------------------------------------------------------------------
// A loop that ends early (claims 16b, 16c and 16e)

/// A route whose loop ends early (claim 16b, *Who gets a press's orders*,
/// under `quirks.press_copy_ignores_route_state`): W's P2, P5, P1, P2, P3,
/// P4 with a loop of 3 from its first step. Returns (world, W, [P1..P5]).
fn early_loop() -> (World, ObjectId, Vec<u32>) {
    let mut w = game();
    let (v, wv) = (
        put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 0),
        put(&mut w, Kind::Vehicle, "BUGGY", 0, 1, 0),
    );
    let cs = [at(3, 3), at(6, 3), at(9, 3), at(9, 6), at(3, 9)];
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
    give_join(&mut w, v, p[1]);
    give_join(&mut w, wv, p[0]);
    let r = &w.routes.routes[&wv];
    assert_eq!(steps(&w, wv), vec![p[1], p[4], p[0], p[1], p[2], p[3]]);
    assert_eq!((r.loop_from, r.loop_len), (Some(0), Some(3)));
    assert!(r.loop_ends_early());
    (w, wv, p)
}

/// spec: S-0213 claim 16e (revision 84: on a route whose loop ends early, a point after the loop counts as one inside it: deleting any point at or after the loop's first point lowers the loop's number of points by 1, and the loop is cancelled once that number is below 3)
#[test]
fn claim_16e_a_delete_on_a_loop_that_ends_early() {
    let (mut w, wv, p) = early_loop();
    // P4 stands after the loop: the loop's number falls to 2 and the loop is cancelled.
    give(&mut w, vec![Order::RouteDelete { point: p[3] }]);
    let r = &w.routes.routes[&wv];
    assert_eq!(steps(&w, wv), vec![p[1], p[4], p[0], p[1], p[2]]);
    assert_eq!((r.loop_from, r.loop_len), (None, None), "cancelled below 3");
}

/// spec: S-0213 claim 16b, Joins (revision 84: the copy's loop is given as many points as the source route's loop has, also when that loop ends early; when the source loop ends early the copy holds the same points as otherwise, so its loop also ends early) and claim 16c, A loop that ends early (the holder goes round every point from the loop's first point to the route's last)
#[test]
fn claim_16b_a_join_copies_a_loop_that_ends_early() {
    let (mut w, _wv, p) = early_loop();
    let x = put(&mut w, Kind::Vehicle, "BUGGY", 0, 2, 0);
    // X joins P5, which W alone holds: W's route from P5, then W's points from its loop's first up to the one before P5.
    give_join(&mut w, x, p[4]);
    let r = &w.routes.routes[&x];
    assert_eq!(steps(&w, x), vec![p[4], p[0], p[1], p[2], p[3], p[1]]);
    assert_eq!((r.loop_from, r.loop_len), (Some(0), Some(3)));
    assert!(r.loop_ends_early());
    // X goes round every point of its route: after its sixth issue its route is as it was.
    give(&mut w, vec![Order::RouteCommit]);
    let before = steps(&w, x);
    assert_eq!(before, vec![p[0], p[1], p[2], p[3], p[1], p[4]]);
}

// ---------------------------------------------------------------------------
// Aircraft and buildings (claim 16d)

/// spec: S-0213 claim 16d, Aircraft and buildings (revision 84, Q-2192: an aircraft that holds route points is a route holder; a commit after its join starts it at rest on the ground; it issues its next point when a flight ends, at the moment it would begin to land, so it flies on to P2 without landing at P1); Scenario O8 part x (6)
#[test]
fn claim_16d_an_aircraft_issues_its_next_point_at_a_flights_end() {
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 2);
    let heli = put(&mut w, Kind::Aircraft, "HELI", 0, 0, 0);
    testkit::run(&mut w, 5);
    assert!(air::on_ground(&w, heli));
    let (p1, p2) = (at(10, 0), at(10, 14));
    let ps = route(&mut w, v, &[p1, p2]);
    give_join(&mut w, heli, ps[0]);
    assert_eq!(steps(&w, heli), ps);
    assert_eq!(routes::count(&w, A), 1, "the join raises no count");
    give(&mut w, vec![Order::RouteCommit]);
    assert_eq!(steps(&w, heli), vec![ps[1]], "the commit starts it");
    run_until(&mut w, 300, |w| air::height(w, heli) > 100);
    let mut over_p1 = false;
    for _ in 0..3000 {
        if cell(&w, heli) == p2 && air::on_ground(&w, heli) {
            break;
        }
        if cell(&w, heli) == p1 {
            over_p1 = true;
            assert!(!air::on_ground(&w, heli), "no landing at P1");
        }
        testkit::run(&mut w, 1);
    }
    assert!(over_p1, "flies over P1");
    assert_eq!(cell(&w, heli), p2);
    assert!(air::on_ground(&w, heli), "lands at P2");
    assert!(steps(&w, heli).is_empty());
}

/// spec: S-0213 claim 16d, Aircraft and buildings (revision 84, Q-2192: an aircraft issues no point when it falls back to rest) and Changes for other specs (S-0215 claim 13's route row: infantry units and vehicles only)
#[test]
fn claim_16d_an_aircraft_issues_no_point_at_a_fall_back() {
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 2);
    let heli = put(&mut w, Kind::Aircraft, "HELI", 0, 0, 0);
    testkit::run(&mut w, 5);
    let ps = route(&mut w, v, &[at(10, 0), at(10, 14)]);
    give_join(&mut w, heli, ps[0]);
    give_join(&mut w, v, ps[0]);
    for s in &mut w.routes.routes.get_mut(&heli).unwrap().steps {
        s.committed = true;
    }
    for s in &mut w.routes.routes.get_mut(&v).unwrap().steps {
        s.committed = true;
    }
    w.effects.clear();
    routes::fall_back(&mut w, heli);
    assert_eq!(steps(&w, heli), ps, "the aircraft issues nothing");
    routes::fall_back(&mut w, v);
    assert_eq!(steps(&w, v), vec![ps[1], ps[0]], "the vehicle issues P1");
}

/// spec: S-0213 claim 16d, Aircraft and buildings (revision 84, Q-2192: a building with no route record joins a point and gets a route; a commit starts it from guard and it issues its first point; under move a building does nothing: it neither moves nor looks for targets nor fires, and issues no further point; a later commit does not start it again) and claim 17 (an order outside route mode ends its route); Scenario O8 part x (7)
#[test]
fn claim_16d_a_building_under_move_does_nothing() {
    // spec: S-0213 claim 16d (revision 89, Q-2228: POST has no turret, so its body range sets its ready mark and it starts a later attack and fires; a turreted building never does, claims_s0213_r89.rs) and Scenario O8 part x (7), first game
    let mut w = game();
    let b = put(&mut w, Kind::Building, "POST", 0, 0, 0);
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 12);
    testkit::run(&mut w, 30);
    assert_eq!(current(&w, b), Some(Behaviour::Guard));
    let ps = route(&mut w, v, &[at(12, 12), at(16, 12)]);
    give_join(&mut w, b, ps[0]);
    assert_eq!(steps(&w, b), ps);
    give(&mut w, vec![Order::RouteCommit]);
    assert_eq!(steps(&w, b), vec![ps[1]], "the first point issued");
    run_until(&mut w, 30, |w| current(w, b) == Some(Behaviour::Move));
    let e = put(&mut w, Kind::Vehicle, "BUGGY", 1, 3, 0);
    let hp = health(&w, e);
    testkit::run(&mut w, 300);
    assert_eq!(health(&w, e), hp, "no shot under move");
    assert_eq!(current(&w, b), Some(Behaviour::Move));
    assert_eq!(steps(&w, b), vec![ps[1]], "no further point");
    give(&mut w, vec![Order::RouteCommit]);
    testkit::run(&mut w, 30);
    assert_eq!(
        steps(&w, b),
        vec![ps[1]],
        "a later commit does not start it"
    );
    assert_eq!(current(&w, b), Some(Behaviour::Move));
    // An attack order outside route mode ends the route, and the building fires.
    give(
        &mut w,
        vec![Order::Behaviour {
            actor: b,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(e)),
            dest: None,
        }],
    );
    assert!(steps(&w, b).is_empty());
    run_until(&mut w, 1500, |w| health(w, e) < hp);
}

/// spec: S-0213 claim 16d, Aircraft and buildings (revision 84, Q-2192: a building without the SAM flag issues its next point only when its attack behaviour has no target, so after an attack point it goes on when that target is gone; it first issues the point with claim 16c's queue of wait, and when wait is then current it queues no guard; with no point to issue the guard of S-0215 claim 32 follows) and Changes for other specs (S-0215 claim 32)
#[test]
fn claim_16d_a_building_issues_its_next_point_when_its_target_is_gone() {
    for more in [true, false] {
        let mut w = game();
        let b = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
        // A force-fire attack on an own vehicle in range: guard does not take it on its own.
        let e = put(&mut w, Kind::Vehicle, "TRUCK", 0, 4, 0);
        testkit::run(&mut w, 30);
        let mut os = vec![add_attack(b, e)];
        if more {
            os.push(add(b, at(-12, -12)));
        }
        give(&mut w, os);
        give(&mut w, vec![Order::RouteCommit]);
        run_until(&mut w, 1000, |w| current(w, b) == Some(Behaviour::Attack));
        assert_eq!(w.obj(b).unwrap().target, Some(Target::Object(e)));
        let hp = health(&w, e);
        run_until(&mut w, 600, |w| health(w, e) < hp);
        assert_eq!(current(&w, b), Some(Behaviour::Attack), "more {more}");
        glasnost_sim::objects::remove(&mut w, e);
        let mut waited = false;
        for _ in 0..60 {
            let o = w.obj(b).unwrap();
            if o.beh.current == Some(Behaviour::Wait) {
                waited = true;
                assert_ne!(o.beh.queued, Some(Behaviour::Guard), "no guard queued");
            }
            if !more && o.beh.current == Some(Behaviour::Guard) {
                break;
            }
            if more && o.beh.current == Some(Behaviour::Move) {
                break;
            }
            testkit::run(&mut w, 1);
        }
        if more {
            assert!(waited, "wait current before the issued move");
            assert_eq!(current(&w, b), Some(Behaviour::Move));
        } else {
            assert!(!waited);
            assert_eq!(current(&w, b), Some(Behaviour::Guard));
        }
        assert!(steps(&w, b).is_empty());
    }
}

/// spec: S-0213 claim 16d, Aircraft and buildings (revision 84, Q-2192: a building whose type has the SAM flag issues no later point: its attack follows S-0215 claim 32's SAM rows unchanged, so without a suitable target it drops the target and queues guard; it keeps its remaining points, and a later commit does not start it again)
#[test]
fn claim_16d_a_sam_building_issues_no_later_point() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "SAMS", 0, 0, 0);
    let e = put(&mut w, Kind::Vehicle, "BUGGY", 1, 14, 0);
    testkit::run(&mut w, 30);
    let p2 = at(-12, -12);
    give(&mut w, vec![add_attack(b, e), add(b, p2)]);
    give(&mut w, vec![Order::RouteCommit]);
    let left = steps(&w, b);
    assert_eq!(left.len(), 1, "the first point issued");
    testkit::run(&mut w, 120);
    assert_eq!(current(&w, b), Some(Behaviour::Guard));
    assert_eq!(w.obj(b).unwrap().target, None, "a ground target dropped");
    assert_eq!(steps(&w, b), left, "no later point");
    give(&mut w, vec![Order::RouteCommit]);
    testkit::run(&mut w, 60);
    assert_eq!(steps(&w, b), left);
}

/// spec: S-0213 claim 16b (revision 84, Q-2192 question 3: of a building's actions, the attack action gives a behaviour order and so a point add) and claim 16d (only claim 16b's four refusals apply to its point adds)
#[test]
fn claim_16b_a_buildings_attack_click_adds_a_point() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    let e = put(&mut w, Kind::Vehicle, "BUGGY", 1, 5, 0);
    testkit::run(&mut w, 2);
    let got = routes::click(&w, A, &[b], ClickOn::Object(e), Modifiers::default());
    let RouteClick::Orders(os) = got else {
        panic!("a point add: {got:?}")
    };
    assert!(matches!(
        os.as_slice(),
        [Order::RoutePoint { behaviour: Behaviour::Attack, place: Some(Target::Object(t)), .. }] if *t == e
    ));
    give(&mut w, os);
    assert_eq!(steps(&w, b).len(), 1);
    assert_eq!(notes(&w), vec![RouteNote::Added { info: None }]);
    assert_eq!(
        routes::count(&w, A),
        1,
        "a first point add raises the count"
    );
}

/// spec: S-0213 claim 16d (an aircraft or building in the route-holder limit: a first point add raises the count, and at the release it counts as a new object) and claim 16 with Formulas (refused when the count plus the selected objects whose routes hold no point would be more than 128) (M8 seam sweep 5a)
#[test]
fn claim_16d_a_building_and_an_aircraft_count_against_the_holder_limit() {
    use glasnost_sim::routes::HOLDER_LIMIT;
    for (kind, name) in [(Kind::Building, "TOWER"), (Kind::Aircraft, "HELI")] {
        let mut w = game();
        let b = put(&mut w, kind, name, 0, 0, 0);
        let e = put(&mut w, Kind::Vehicle, "BUGGY", 1, 5, 0);
        testkit::run(&mut w, 2);
        let on = if kind == Kind::Building {
            ClickOn::Object(e)
        } else {
            ClickOn::Cell(at(4, 4))
        };
        // At the limit: refused, as a new object.
        w.routes.counts.insert(A, HOLDER_LIMIT);
        assert_eq!(
            routes::click(&w, A, &[b], on, Modifiers::default()),
            RouteClick::Refused("MSG:PlannerMaximum"),
            "{name} at the limit"
        );
        // One below: the point add is given and raises the count to the limit.
        w.routes.counts.insert(A, HOLDER_LIMIT - 1);
        let RouteClick::Orders(os) = routes::click(&w, A, &[b], on, Modifiers::default()) else {
            panic!("{name}: a point add")
        };
        give(&mut w, os);
        assert_eq!(steps(&w, b).len(), 1, "{name}");
        assert_eq!(routes::count(&w, A), HOLDER_LIMIT, "{name}: the count rose");
        // Now holding a point, it is no new object: a second add passes.
        let on2 = if kind == Kind::Building {
            on
        } else {
            ClickOn::Cell(at(6, 4))
        };
        assert!(
            matches!(
                routes::click(&w, A, &[b], on2, Modifiers::default()),
                RouteClick::Orders(_)
            ),
            "{name}: no new object"
        );
    }
}

/// spec: S-0213 claim 16b (revision 84, Q-2192 question 3: a click on a cell that sets a factory's rally point sends that order directly: it is not a behaviour order, adds no point and takes effect when applied; it meets claim 16d's release-time tests and is sent at the release); Scenario O8 part h's third game
#[test]
fn claim_16b_a_rally_point_click_is_sent_directly() {
    let mut w = game();
    let f = put(&mut w, Kind::Building, "FAC", 0, 0, 0);
    testkit::run(&mut w, 2);
    let c = at(6, 6);
    let got = routes::click(&w, A, &[f], ClickOn::Cell(c), Modifiers::default());
    let RouteClick::Orders(os) = got else {
        panic!("the rally order: {got:?}")
    };
    assert!(matches!(os.as_slice(), [Order::GuardSpot { actor, .. }] if *actor == f));
    give(&mut w, os);
    assert!(
        w.obj(f).unwrap().guard_spot.is_some(),
        "the rally point set"
    );
    assert!(w.routes.points.is_empty(), "no point");
    assert!(steps(&w, f).is_empty());
    assert!(notes(&w).is_empty());
    // The release-time holder-limit test: the factory holds no point, so it counts as new.
    w.routes.counts.insert(A, routes::HOLDER_LIMIT);
    assert_eq!(
        routes::click(&w, A, &[f], ClickOn::Cell(c), Modifiers::default()),
        RouteClick::Refused("MSG:PlannerMaximum")
    );
}

/// spec: S-0213 claim 16b (revision 87, Q-2192 question 3: a building's *move* on an object sends the guard-spot order of S-0214 table M part 3 directly, with no point, chosen around the object's cell for a factory of vehicles, and it then acts as claim 20 says; a building's deploy action is not refused in route mode, and its primary order, when the building is factory-marked, is sent directly with no point; each meets claim 16d's release-time tests)
#[test]
fn claim_16b_a_buildings_move_on_an_object_and_primary_are_sent_directly() {
    let mut w = game();
    let f1 = put(&mut w, Kind::Building, "FAC", 0, 0, 0);
    let f2 = put(&mut w, Kind::Building, "FAC", 0, 6, 0);
    let v = put(&mut w, Kind::Vehicle, "TRUCK", 0, 0, 8);
    testkit::run(&mut w, 2);
    // A move on an own object (force-move): the guard spot, a cell around the object's cell.
    let fm = Modifiers {
        force_move: true,
        ..Modifiers::default()
    };
    let got = routes::click(&w, A, &[f1], ClickOn::Object(v), fm);
    let RouteClick::Orders(os) = got else {
        panic!("the guard-spot order: {got:?}")
    };
    assert!(matches!(
        os.as_slice(),
        [Order::GuardSpot { actor, spot: Target::Cell(_) }] if *actor == f1
    ));
    give(&mut w, os);
    assert!(w.obj(f1).unwrap().guard_spot.is_some(), "claim 20: set");
    assert!(w.routes.points.is_empty(), "no point");
    assert!(steps(&w, f1).is_empty());
    // The deploy action on a factory that is not the primary one: the primary order.
    let other = if w.obj(f1).unwrap().econ.building.primary {
        f2
    } else {
        f1
    };
    assert!(!w.obj(other).unwrap().econ.building.primary);
    let got = routes::click(
        &w,
        A,
        &[other],
        ClickOn::Object(other),
        Modifiers::default(),
    );
    let RouteClick::Orders(os) = got else {
        panic!("the primary order, not refused: {got:?}")
    };
    assert!(matches!(os.as_slice(), [Order::Primary { actor }] if *actor == other));
    give(&mut w, os);
    assert!(w.obj(other).unwrap().econ.building.primary, "made primary");
    assert!(w.routes.points.is_empty(), "no point");
    assert!(notes(&w).is_empty());
    // The release-time holder-limit test applies to both.
    w.routes.counts.insert(A, routes::HOLDER_LIMIT);
    assert_eq!(
        routes::click(&w, A, &[f1], ClickOn::Object(v), fm),
        RouteClick::Refused("MSG:PlannerMaximum")
    );
}

/// spec: S-0213 claim 16b (revision 87: a building's deploy action is not refused in route mode; a building is given its deploy action only by a click on the building itself; each behaviour order *unload* that the action gives names the building itself as its target, so it has the building's own place and is a point add there; its points are issued as claim 16d says) and Changes for other specs (S-0214 table M part 3, deploy row)
#[test]
fn claim_16b_a_buildings_unload_is_a_point_at_its_own_place() {
    let mut w = game();
    let bio = put(&mut w, Kind::Building, "BIO", 0, 0, 0);
    let u = put(&mut w, Kind::Infantry, "WALKER", 0, 4, 0);
    testkit::run(&mut w, 2);
    // The ordinary click that sends the walker in.
    let os = targeting::click(&w, A, &[u], ClickOn::Object(bio), Modifiers::default()).orders;
    give(&mut w, os);
    run_until(&mut w, 900, |w| absorb::absorbed_in(w, u) == Some(bio));
    // Its deploy action is not refused in route mode: the cursor keeps deploy.
    let shown = targeting::cursor(&w, A, &[bio], ClickOn::Object(bio), Modifiers::default());
    assert_eq!(shown, Action::Deploy);
    assert_eq!(
        targeting::in_route_mode(&w, &[bio], ClickOn::Object(bio), shown),
        Action::Deploy
    );
    let got = routes::click(&w, A, &[bio], ClickOn::Object(bio), Modifiers::default());
    let RouteClick::Orders(os) = got else {
        panic!("a point add: {got:?}")
    };
    let me = Some(Target::Object(bio));
    assert!(matches!(
        os.as_slice(),
        [Order::RoutePoint { actor, behaviour: Behaviour::Unload, target, dest: None, place, .. }]
            if *actor == bio && *target == me && *place == me
    ));
    give(&mut w, os);
    let ps = steps(&w, bio);
    assert_eq!(ps.len(), 1, "a point at the building's own place");
    assert_eq!(absorb::count(&w, bio), 1, "not unloaded at the add");
    // The commit starts it from guard: it issues the unload, which lets the passenger out.
    give(&mut w, vec![Order::RouteCommit]);
    assert!(steps(&w, bio).is_empty(), "the point issued");
    run_until(&mut w, 900, |w| absorb::count(w, bio) == 0);
}

/// spec: S-0213 claim 16d, When the tests are made (revision 84, Q-2193: the click areas here are those of every point of every house, drawn for the player or not: a press inside the click area of a point the player cannot see is not tested for kind) and Edge cases (quirks.hidden_point_refuses_click off: such a point plays no part in a press, which counts as one outside every click area and meets the kind test); Scenario O8 part h's third game
#[test]
fn claim_16d_a_hidden_points_click_area_and_the_kind_test_both_ways() {
    for hidden in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.hidden_point_refuses_click = hidden;
        let mut w = game_from(&r);
        let f = put(&mut w, Kind::Building, "FAC", 0, 0, 0);
        let enemy = put(&mut w, Kind::Vehicle, "BUGGY", 1, 0, 8);
        assert!(orders::issue(&mut w, HouseId(1), add(enemy, at(8, 8))));
        testkit::run(&mut w, 1);
        let theirs = routes::point_in(&w, HouseId(1), at(8, 8)).expect("Beta's point");
        assert!(!routes::drawn_for(&w, A, &w.routes.points[&theirs]));
        let want = (!hidden).then_some("MSG:PlanningModeNoBuildings");
        assert_eq!(
            routes::press_kind_refusal(&w, A, &[f], &[theirs]),
            want,
            "hidden {hidden}"
        );
        // With no point under it the press meets the kind test either way.
        assert_eq!(
            routes::press_kind_refusal(&w, A, &[f], &[]),
            Some("MSG:PlanningModeNoBuildings")
        );
    }
}
