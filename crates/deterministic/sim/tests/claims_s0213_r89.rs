//! Claim tests for S-0213 revision 89 (human orders: the place of an
//! attack-move route point, the point a delete selects after a route's last
//! point, a copied *capture* issued by a vehicle, a turreted building under
//! *move*, and an aircraft's issue at a flight's end) on synthetic data:
//! invented types, no game data.

mod common;
mod econ_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Locomotor, UnitType};
use glasnost_sim::air;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::routes::{self, RouteClick, RouteNote};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 48;
const A: HouseId = HouseId(0);

/// Attack-move entry (S-0214 claim 5): attack-move mode on.
const ENTRY: Modifiers = Modifiers {
    force_move: false,
    force_fire: false,
    force_select: false,
    attack_move_mode: true,
};

/// The testkit's combat ruleset (armed CART, unarmed BUGGY, the armed
/// turreted building TOWER) with
/// - POST: TOWER without the turret flag;
/// - ENG: an unarmed engineer (the testkit's WALKER with the engineer flag);
/// - LAB: a 2 × 2 building that engineers can capture;
/// - HELI: an unarmed flying aircraft with no dock that may land anywhere;
/// - the engine's default area-guard threshold, so that armed objects rest
///   in guard (S-0215 claim 15).
fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.iq_guard_area = None;
    let mut push = |reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push("buildings", "POST");
    push("buildings", "LAB");
    push("infantry", "ENG");
    push("aircraft", "HELI");
    let s = &mut r.sim;
    let mut post = s.buildings["TOWER"].clone();
    post.turret = None;
    s.buildings.insert("POST".into(), post);
    s.buildings.insert(
        "LAB".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([2, 2]),
            capturable: Some(true),
            ..Default::default()
        },
    );
    let mut eng = s.infantry["WALKER"].clone();
    eng.engineer = Some(true);
    eng.primary = None;
    s.infantry.insert("ENG".into(), eng);
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
fn game() -> World {
    let mut w = testkit::game_with(&ruleset(), SIZE, 7);
    while orders::blocked(&w) {
        testkit::run(&mut w, 1);
    }
    w
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
        assert!(orders::issue(w, A, o), "issued");
    }
    testkit::run(w, 1);
}

/// A point add of a move to `c`, at that cell.
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

fn route(w: &mut World, id: ObjectId, cells: &[CellXY]) -> Vec<u32> {
    give(w, cells.iter().map(|&c| add(id, c)).collect());
    cells.iter().map(|&c| point(w, c)).collect()
}

fn give_join(w: &mut World, actor: ObjectId, point: u32) {
    let place = w.routes.points[&point].place;
    give(
        w,
        vec![Order::RouteJoin {
            actor,
            point,
            place,
        }],
    );
}

fn current(w: &World, id: ObjectId) -> Option<Behaviour> {
    w.obj(id).unwrap().beh.current
}

fn queued(w: &World, id: ObjectId) -> Option<Behaviour> {
    w.obj(id).unwrap().beh.queued
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

fn selected(w: &World) -> Option<u32> {
    notes(w).into_iter().find_map(|n| match n {
        RouteNote::Selected(p) => Some(p),
        _ => None,
    })
}

/// The one point add a route-mode click gives.
fn one_point(r: RouteClick) -> Order {
    let RouteClick::Orders(os) = r else {
        panic!("a point add: {r:?}")
    };
    let [o] = os.as_slice() else {
        panic!("one order: {os:?}")
    };
    o.clone()
}

// ---------------------------------------------------------------------------
// The place of an attack-move click (claim 16b, Q-2230)

/// spec: S-0213 claim 16b (revision 89, Q-2230 question 1: an attack-move click has the place that the same click gives without attack-move entry; entry changes only the order's kind, not the target, destination or cell the click gave it, so an attack-move click on a cell puts the point on the clicked cell, not on the cell the nearby-cell search finds); Scenario O8 part q (6)
#[test]
fn claim_16b_an_attack_move_click_on_a_cell_has_the_clicked_cells_place() {
    use common::{Scene, WHEEL, cell, code_tile, unit_type};
    let mut s = Scene::new();
    let c = testkit::combat_ruleset();
    s.ruleset.sim.weapons.extend(c.sim.weapons.clone());
    s.ruleset.sim.projectiles.extend(c.sim.projectiles.clone());
    s.ruleset.sim.warheads.extend(c.sim.warheads.clone());
    s.vehicle(
        "CAR",
        UnitType {
            primary: Some("Gun".into()),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    // A cell enclosed by rock (no land row: blocked for every class), so that
    // S-0214 claim 11's nearby-cell search gives the destination.
    let enclosed = cell(6, 0);
    for dy in -1..=1 {
        for dx in -1..=1 {
            if (dx, dy) != (0, 0) {
                s.set_cell(
                    CellXY::new(enclosed.x + dx, enclosed.y + dy),
                    code_tile(7),
                    0,
                );
            }
        }
    }
    s.place_unit(0, "CAR", cell(0, 0), 64);
    let mut w = s.build();
    common::open_orders(&mut w);
    let car = common::object_in(&w, cell(0, 0)).expect("the car");
    let on = ClickOn::Cell(enclosed);
    assert!(
        targeting::shows_attack_move(&w, A, &[car], on, ENTRY),
        "the car is eligible for attack moves"
    );
    let plain = one_point(routes::click(&w, A, &[car], on, Modifiers::default()));
    let entry = one_point(routes::click(&w, A, &[car], on, ENTRY));
    let Order::RoutePoint {
        behaviour,
        attack_move,
        target,
        dest,
        place,
        ..
    } = entry.clone()
    else {
        panic!("a point add: {entry:?}")
    };
    assert_eq!(behaviour, Behaviour::Move);
    assert!(attack_move, "the entry makes it an attack move");
    assert_eq!(target, None);
    assert!(
        matches!(dest, Some(Target::Cell(d)) if d != enclosed),
        "the search moved the destination: {dest:?}"
    );
    assert_eq!(place, Some(Target::Cell(enclosed)), "the clicked cell");
    // The same click without entry: the same destination and place.
    let Order::RoutePoint {
        dest: plain_dest,
        place: plain_place,
        attack_move: plain_am,
        ..
    } = plain
    else {
        panic!("a point add")
    };
    assert!(!plain_am);
    assert_eq!((plain_dest, plain_place), (dest, place));
    // Applied, the marker stands on the clicked cell.
    assert!(orders::issue(&mut w, A, entry));
    common::run(&mut w, 1);
    let p = routes::point_in(&w, A, enclosed).expect("a marker on the clicked cell");
    assert_eq!(routes::point_cell(&w, &w.routes.points[&p]), enclosed);
    let s = &w.routes.routes[&car].steps[0];
    assert!(
        s.attack_move && s.dest == dest,
        "the recorded order is the attack move"
    );
}

/// spec: S-0213 claim 16b (revision 89, Q-2230 question 2: an attack-move click on an object puts the point at the target's place when its order names a target, and on the cell that the object stands on at the click when its order would be a move without entry)
#[test]
fn claim_16b_an_attack_move_click_on_an_object() {
    let mut w = game();
    let car = put(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    let e = put(&mut w, Kind::Vehicle, "BUGGY", 1, 6, 0);
    testkit::run(&mut w, 2);
    // An enemy vehicle: an attack, which names it as its target.
    let o = one_point(routes::click(&w, A, &[car], ClickOn::Object(e), ENTRY));
    assert!(
        matches!(
            o,
            Order::RoutePoint {
                behaviour: Behaviour::Attack,
                attack_move: true,
                target: Some(Target::Object(t)),
                place: Some(Target::Object(p)),
                ..
            } if t == e && p == e
        ),
        "{o:?}"
    );
    // An attack-move order on an object that would be a move without entry:
    // the cell the object stands on, whatever the destination.
    // The order's attack-move mark is not an input of the place.
    let d = at(9, 9);
    assert_eq!(
        routes::order_place(
            &w,
            ClickOn::Object(e),
            Behaviour::Move,
            None,
            Some(Target::Cell(d))
        ),
        Some(Target::Cell(at(6, 0)))
    );
}

// ---------------------------------------------------------------------------
// The point a delete selects (claim 16e, Q-2226)

/// A route whose loop ends early, on W, the longest holder of its last
/// point (claim 16b, *Who gets a press's orders*): W's P2, P5, P6, P1, P2,
/// P3, P4 with a loop of 3 points from its first step, built through V's
/// loop on P1 to P4, after which V leaves play. Returns (world, W, V's
/// world before it left, V, [P1..P6]).
fn early_loop() -> (World, ObjectId, World, ObjectId, Vec<u32>) {
    let mut w = game();
    let (v, wv) = (
        put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 0),
        put(&mut w, Kind::Vehicle, "BUGGY", 0, 1, 0),
    );
    let cs = [at(3, 3), at(6, 3), at(9, 3), at(9, 6), at(3, 9), at(6, 9)];
    give(
        &mut w,
        vec![
            add(v, cs[0]),
            add(v, cs[1]),
            add(wv, cs[1]),
            add(v, cs[2]),
            add(v, cs[3]),
            add(wv, cs[4]),
            add(wv, cs[5]),
        ],
    );
    let p: Vec<u32> = cs.iter().map(|&c| point(&w, c)).collect();
    give_join(&mut w, v, p[1]);
    give_join(&mut w, wv, p[0]);
    let r = &w.routes.routes[&wv];
    assert_eq!(
        steps(&w, wv),
        vec![p[1], p[4], p[5], p[0], p[1], p[2], p[3]]
    );
    assert_eq!((r.loop_from, r.loop_len), (Some(0), Some(3)));
    assert!(r.loop_ends_early());
    let with_v = w.clone();
    glasnost_sim::objects::remove(&mut w, v);
    // The record of a removed holder goes at the next order boundary.
    testkit::run(&mut w, 1);
    assert!(!w.routes.routes.contains_key(&v));
    assert_eq!(steps(&w, wv).len(), 7, "W's route is kept");
    (w, wv, with_v, v, p)
}

/// spec: S-0213 claim 16e, The delete key (revision 89, Q-2226 question 1: the point is found in the route of the deleted point's longest holder as it stands before the delete; when the deleted point is that route's last point, the count goes on round the loop's points only: with n the points from the loop's first point to the route's last, both included, and C the loop's number of points, the selected point is the one n mod C places after the loop's first point; on a loop that ends early it is a point of the loop, never one after it)
#[test]
fn claim_16e_a_delete_of_an_early_loops_last_point_selects_round_the_loop() {
    let (mut w, _, _, _, p) = early_loop();
    // n = 7 (P2 to P4), C = 3: 7 mod 3 = 1 place after the loop's first point P2.
    give(&mut w, vec![Order::RouteDelete { point: p[3] }]);
    assert_eq!(selected(&w), Some(p[4]), "P5, one place after P2");
}

/// spec: S-0213 claim 16e, The delete key (revision 89, Q-2226: on a loop that runs to the route's end n = C, so the selected point is the loop's first point; a route without a loop gives no point when its last point is deleted)
#[test]
fn claim_16e_a_delete_of_a_last_point_on_a_full_loop_and_without_a_loop() {
    // V's route P1, P2, P3, P4 with its loop from P2 to the end: V is P4's longest holder.
    let (_, _, mut w, v, p) = early_loop();
    let r = &w.routes.routes[&v];
    assert_eq!(steps(&w, v), vec![p[0], p[1], p[2], p[3]]);
    assert_eq!((r.loop_from, r.loop_len), (Some(1), Some(3)));
    give(&mut w, vec![Order::RouteDelete { point: p[3] }]);
    assert_eq!(selected(&w), Some(p[1]), "the loop's first point");
    // A route without a loop.
    let mut w = game();
    let x = put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 0);
    let ps = route(&mut w, x, &[at(3, 3), at(6, 3), at(9, 3)]);
    give(&mut w, vec![Order::RouteDelete { point: ps[2] }]);
    assert_eq!(selected(&w), None, "no point");
    assert_eq!(steps(&w, x), vec![ps[0], ps[1]]);
}

/// spec: S-0213 claim 16e, The delete key (revision 89, Q-2226 question 2: where the route holds the deleted point at two places, the point after its first place is taken; that first place is never the route's last point, so the count round the loop does not apply) and claim 16c (issuing a point of a loop moves it to the end of the route)
#[test]
fn claim_16e_a_delete_of_a_point_held_twice_takes_the_one_after_its_first_place() {
    let (mut w, wv, _, _, p) = early_loop();
    // The commit makes W issue P2, which moves to the route's end: P2 is now its last point and stands earlier too.
    give(&mut w, vec![Order::RouteCommit]);
    assert_eq!(
        steps(&w, wv),
        vec![p[4], p[5], p[0], p[1], p[2], p[3], p[1]]
    );
    give(&mut w, vec![Order::RouteDelete { point: p[1] }]);
    // The point after P2's first place (P3), not the count round the loop (7 mod 3 = 1: P6).
    assert_eq!(selected(&w), Some(p[2]));
}

// ---------------------------------------------------------------------------
// A copied capture issued by a vehicle (claim 16b, Joins, Q-2227 question 2)

/// Part ac's game up to the release: N's route F then X (capture), T
/// joined to the point at X's place, committed. Returns the world, N, T,
/// X, F, and X's health and owner at the release.
fn copied_capture() -> (
    World,
    ObjectId,
    ObjectId,
    ObjectId,
    CellXY,
    i32,
    Option<HouseId>,
) {
    let mut w = game();
    let n = put(&mut w, Kind::Infantry, "ENG", 0, 0, 0);
    let t = put(&mut w, Kind::Vehicle, "CART", 0, 0, 2);
    let x = put(&mut w, Kind::Building, "LAB", 1, 6, 0);
    let f = at(-14, 0);
    testkit::run(&mut w, 30);
    let hp = health(&w, x);
    let owner = w.obj(x).unwrap().owner;
    // T's own click action on X is attack.
    assert_eq!(
        targeting::action(&w, A, t, ClickOn::Object(x), Modifiers::default()),
        Action::Attack
    );
    // N's route: F, then X with the cursor showing capture.
    assert_eq!(
        targeting::action(&w, A, n, ClickOn::Object(x), Modifiers::default()),
        Action::Capture
    );
    let to_f = one_point(routes::click(
        &w,
        A,
        &[n],
        ClickOn::Cell(f),
        Modifiers::default(),
    ));
    give(&mut w, vec![to_f]);
    let to_x = one_point(routes::click(
        &w,
        A,
        &[n],
        ClickOn::Object(x),
        Modifiers::default(),
    ));
    give(&mut w, vec![to_x]);
    let px = routes::point_in(&w, A, cell(&w, x)).expect("a point at X");
    let rec = w.routes.routes[&n].steps[1].clone();
    assert_eq!(
        (rec.behaviour, rec.target, rec.dest),
        (Behaviour::Capture, None, Some(Target::Object(x)))
    );
    // T joins the point at X's place, and the release commits.
    give_join(&mut w, t, px);
    assert_eq!(steps(&w, t), vec![px]);
    give(&mut w, vec![Order::RouteCommit]);
    (w, n, t, x, f, hp, owner)
}

/// One tick of part ac's record: once T has *capture* current it keeps it,
/// with no target and X as its destination; T makes no shot and X keeps
/// its health and owner. Returns whether T has *capture* current.
fn copied_capture_tick(
    w: &World,
    (t, x, hp, owner): (ObjectId, ObjectId, i32, Option<HouseId>),
    capturing: bool,
) -> bool {
    let o = w.obj(t).unwrap();
    let capturing = capturing || o.beh.current == Some(Behaviour::Capture);
    if capturing {
        assert_eq!(o.beh.current, Some(Behaviour::Capture), "t={}", w.tick);
        assert_eq!(o.target, None, "no target: t={}", w.tick);
        assert_eq!(o.dest, Some(Target::Object(x)), "X its destination");
    }
    assert!(
        !w.effects
            .iter()
            .any(|e| matches!(e, Effect::Shot { firer, .. } if *firer == t)),
        "no shot of T's"
    );
    assert_eq!(health(w, x), hp, "X keeps its health");
    assert_eq!(w.obj(x).unwrap().owner, owner, "X keeps its owner");
    capturing
}

/// spec: S-0213 claim 16b, Joins (revision 89, Q-2227 question 2: a copied action that a click would not give the joining holder is applied as for any actor: a vehicle that issues an engineer's copied capture on an enemy building gets capture with the building as its destination and no target, and heads for it); Scenario O8 part ac, every tick from the release until T stands next to X
#[test]
fn claim_16b_a_vehicle_issues_a_copied_capture_and_heads_for_x() {
    let (mut w, n, t, x, f, hp, owner) = copied_capture();
    let mut capturing = false;
    let next_to_x = |w: &World| {
        let (c, b) = (cell(w, t), cell(w, x));
        (c.x - b.x).abs() <= 1 && (c.y - b.y).abs() <= 1
    };
    for _ in 0..2000 {
        if next_to_x(&w) || cell(&w, n) == f {
            break;
        }
        capturing = copied_capture_tick(&w, (t, x, hp, owner), capturing);
        testkit::run(&mut w, 1);
    }
    capturing = copied_capture_tick(&w, (t, x, hp, owner), capturing);
    assert!(capturing, "T has capture current");
    assert!(next_to_x(&w), "T heads for X and reaches it first");
    assert_ne!(cell(&w, n), f, "while N is still on its way to F");
    assert!(steps(&w, t).is_empty(), "a capture point is terminating");
}

/// spec: S-0213 claim 16b, Joins (revision 89, Q-2227 question 2: the vehicle does not attack the building, since the order names no target, and never captures it, since only an infantry unit makes a building entry, S-0509 claim 1); Scenario O8 part ac, every tick until N reaches F. Blocked on Q-2247: S-0207 claim 15 table T4 (repeated check) with S-0209 claim 4 row VP28 makes T, standing next to X with its next step in X's foundation, take X as its target by an override and attack it
#[test]
#[ignore = "Q-2247: S-0207 claim 15 table T4 makes T attack X as an enemy obstacle"]
fn claim_16b_a_vehicle_with_a_copied_capture_never_attacks_x() {
    let (mut w, n, t, x, f, hp, owner) = copied_capture();
    let mut capturing = false;
    for _ in 0..2000 {
        if cell(&w, n) == f {
            break;
        }
        capturing = copied_capture_tick(&w, (t, x, hp, owner), capturing);
        testkit::run(&mut w, 1);
    }
    assert_eq!(cell(&w, n), f, "N reaches F");
    assert!(capturing, "T has capture current");
}

// ---------------------------------------------------------------------------
// A turreted building under move (claim 16d, Q-2228)

/// spec: S-0213 claim 16d, Aircraft and buildings (revision 89, Q-2228: an attack order given outside route mode to a building under move ends its route and queues attack with the order's target; nothing sets a turreted building's ready mark while move is current, S-0215 claim 11, so it keeps move current and attack queued and makes no shot at that target); Scenario O8 part x (7), third game
#[test]
fn claim_16d_a_turreted_building_under_move_never_starts_a_later_attack() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 12);
    testkit::run(&mut w, 30);
    assert_eq!(current(&w, b), Some(Behaviour::Guard));
    let ps = route(&mut w, v, &[at(12, 12), at(16, 12)]);
    give_join(&mut w, b, ps[0]);
    give(&mut w, vec![Order::RouteCommit]);
    run_until(&mut w, 30, |w| current(w, b) == Some(Behaviour::Move));
    let e = put(&mut w, Kind::Vehicle, "BUGGY", 1, 3, 0);
    let hp = health(&w, e);
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
    assert!(steps(&w, b).is_empty(), "the order ends its route");
    for _ in 0..1000 {
        assert_eq!(current(&w, b), Some(Behaviour::Move), "t={}", w.tick);
        assert_eq!(queued(&w, b), Some(Behaviour::Attack), "t={}", w.tick);
        assert_eq!(health(&w, e), hp, "no shot: t={}", w.tick);
        testkit::run(&mut w, 1);
    }
}

// ---------------------------------------------------------------------------
// An aircraft's issue at a flight's end (claim 16d, Q-2229)

/// spec: S-0213 claim 16d, Aircraft and buildings (revision 89, Q-2229 question 2: an aircraft that issues a point at a flight's end begins no landing in that tick and stays in flight; the issued order applies at the end of that tick and gives it the next destination while it is neither landing nor taking off, so it flies on with no landing and no take-off) and question 3 (the issue is preceded by claim 16c's queue of wait, so no guard); Scenario O8 part x (6)'s per-tick record (no take-off sound is modelled for aircraft, so the take-off is read from the aircraft's take-off state)
#[test]
fn claim_16d_an_aircraft_flies_on_at_a_flights_end_tick_by_tick() {
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 2);
    let heli = put(&mut w, Kind::Aircraft, "HELI", 0, 0, 0);
    testkit::run(&mut w, 5);
    let (p1, p2) = (at(14, 0), at(14, 16));
    let ps = route(&mut w, v, &[p1, p2]);
    give_join(&mut w, heli, ps[0]);
    give(&mut w, vec![Order::RouteCommit]);
    let cruise = air::cruise_height(&w, heli);
    assert!(cruise > 0);
    run_until(&mut w, 600, |w| cell(w, heli) == p1);
    let mut ticks = 0;
    loop {
        let o = w.obj(heli).unwrap();
        let a = o.air.clone().unwrap_or_default();
        let on_way = o.dest == Some(Target::Cell(p2)) && o.pos.cell() != p1;
        if on_way {
            break;
        }
        assert!(!a.landing, "no landing at P1: t={}", w.tick);
        assert!(!a.taking_off, "no take-off at P1: t={}", w.tick);
        assert!(
            air::height(&w, heli) >= cruise,
            "no fall below cruise height: t={}",
            w.tick
        );
        assert!(
            matches!(o.dest, Some(Target::Cell(c)) if c == p1 || c == p2),
            "a destination in every tick: {:?}",
            o.dest
        );
        assert_ne!(o.beh.current, Some(Behaviour::Guard), "t={}", w.tick);
        assert_ne!(o.beh.queued, Some(Behaviour::Guard), "t={}", w.tick);
        ticks += 1;
        assert!(ticks < 300, "on its way to P2");
        testkit::run(&mut w, 1);
    }
    assert!(steps(&w, heli).is_empty(), "P2 issued");
    run_until(&mut w, 3000, |w| {
        cell(w, heli) == p2 && air::on_ground(w, heli)
    });
}

/// spec: S-0213 claim 16d, Aircraft and buildings (revision 89, Q-2229 question 1: the issue at a flight's end is S-0501 claim 6's going on to a further waypoint of a planned route and comes before that claim's pad-bound test, so a pad-bound aircraft away from its pad that has a point to issue issues it and does not fall back to rest) and Changes for other specs (S-0501 claim 6, planned-route exception); Scenario O8 part x (6), the pad-bound game
#[test]
fn claim_16d_a_pad_bound_aircraft_issues_before_the_pad_bound_test() {
    pad_bound_game_on_its_way_to_p2();
}

/// The pad-bound game of part x (6) up to the tick in which A, unarmed and
/// pad-bound with the pad type as its dock type, has left P1 for P2 with P2
/// issued. Returns the world, A, its pad and P2.
fn pad_bound_game_on_its_way_to_p2() -> (World, ObjectId, ObjectId, CellXY) {
    use econ_common::{building, game as econ_game, issue, ready, set_credits, t, wheel};
    let mut r = econ_common::ruleset();
    let pad = r.sim.buildings.get_mut("AIRPAD").unwrap();
    pad.helipad = Some(true);
    pad.econ.number_of_docks = Some(1);
    let plane = r.sim.aircraft.get_mut("PLANE").unwrap();
    plane.speed = Some(30);
    plane.rot = Some(12);
    plane.locomotor = Some(Locomotor::Fly);
    plane.econ.landable = Some(true);
    plane.econ.airport_bound = Some(true);
    plane.econ.dock = Some(vec!["AIRPAD".into()]);
    r.registries.get_mut("vehicles").unwrap().push("CAR".into());
    r.sim.vehicles.insert("CAR".into(), wheel(500));
    let mut w = econ_game(&r);
    let k = building(&mut w, "AIRPAD", 0, -10, -10);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let what = t(&w, Kind::Aircraft, "PLANE");
    assert!(issue(&mut w, 0, Order::Build { what }));
    let ti = what.index;
    let built = |w: &World| {
        w.objects
            .values()
            .find(|o| o.kind == Kind::Aircraft && o.type_index == ti && o.on_map)
            .map(|o| o.id)
    };
    for _ in 0..3000 {
        if built(&w).is_some() {
            break;
        }
        testkit::run(&mut w, 1);
    }
    let a = built(&w).expect("the aircraft is built");
    testkit::run(&mut w, 30);
    assert!(air::on_ground(&w, a), "at rest on its pad");
    assert_eq!(
        w.obj(a).unwrap().contact,
        Some(k),
        "in contact with its pad"
    );
    let car = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CAR",
        0,
        testkit::mid(econ_common::SIZE, 0, 6),
        0,
    );
    let (p1, p2) = (
        testkit::mid(econ_common::SIZE, 8, 0),
        testkit::mid(econ_common::SIZE, 8, 10),
    );
    let ps = route(&mut w, car, &[p1, p2]);
    give_join(&mut w, a, ps[0]);
    give(&mut w, vec![Order::RouteCommit]);
    // P1's recorded order is the car's: its destination is where A flies.
    run_until(&mut w, 100, |w| w.obj(a).unwrap().dest.is_some());
    let Some(Target::Cell(d1)) = w.obj(a).unwrap().dest else {
        panic!("a cell destination for P1")
    };
    run_until(&mut w, 1500, |w| cell(w, a) == d1);
    let pad_cell = w.obj(k).unwrap().pos.cell();
    let mut left_p1 = false;
    for _ in 0..1500 {
        let Some(o) = w.obj(a) else {
            panic!("it crashed at P1")
        };
        assert!(o.health > 0, "no crash: t={}", w.tick);
        if o.dest == Some(Target::Cell(p2)) && o.pos.cell() != d1 {
            left_p1 = true;
            break;
        }
        assert_ne!(
            o.dest,
            Some(Target::Object(k)),
            "no turn back towards the pad"
        );
        assert_ne!(o.pos.cell(), pad_cell);
        testkit::run(&mut w, 1);
    }
    assert!(left_p1, "it flies on to P2");
    assert!(steps(&w, a).is_empty(), "P2 issued");
    (w, a, k, p2)
}

/// spec: S-0213 claim 16d, Aircraft and buildings (when the aircraft has no point to issue, S-0501 claim 6's pad-bound test and landing follow unchanged) and Scenario O8 part x (6), the pad-bound game ("Record what A does after P2", no expectation given); S-0501 claim 6 (a pad-bound aircraft whose cell holds no building it is in contact with does not land but falls back to rest) and table T2 (in flight, unarmed, not in a team, no passengers: "in flight, otherwise", the F8 landing spot and *move*; the pad rows are for armed aircraft only); F8 (below 256 leptons from the chosen building, a free cell near its own cell). Q-2252 asks whether the original does this: the composed rules make A fall back to rest at every tick above its F8 cell near the pad, under *move*, and never dock or land
#[test]
fn claim_16d_after_p2_an_unarmed_pad_bound_aircraft_falls_back_above_its_f8_spot() {
    let (mut w, a, k, p2) = pad_bound_game_on_its_way_to_p2();
    run_until(&mut w, 1500, |w| cell(w, a) == p2);
    // From P2 it is sent to an F8 landing spot near its pad, not to the pad.
    run_until(
        &mut w,
        100,
        |w| matches!(w.obj(a).unwrap().dest, Some(Target::Cell(c)) if c != p2),
    );
    let Some(Target::Cell(spot)) = w.obj(a).unwrap().dest else {
        unreachable!()
    };
    let pad = w.obj(k).unwrap().pos.cell();
    assert!(
        (spot.x - pad.x).abs() <= 3 && (spot.y - pad.y).abs() <= 3,
        "a spot near the pad: {spot:?}, pad {pad:?}"
    );
    assert!(
        !w.cells
            .list(spot, glasnost_sim::world::Layer::Ground)
            .contains(&k),
        "not a cell of the pad"
    );
    run_until(&mut w, 1500, |w| cell(w, a) == spot);
    // Once it has slowed in that cell (claim 6's target fraction of 0), the landing trigger comes.
    run_until(&mut w, 200, |w| {
        w.obj(a).unwrap().beh.last_common_effects == Some(w.tick - 1)
    });
    for _ in 0..1300 {
        let o = w.obj(a).expect("no crash");
        assert!(o.health > 0);
        assert_eq!(current(&w, a), Some(Behaviour::Move));
        assert_eq!(o.dest, Some(Target::Cell(spot)), "the same F8 spot");
        assert_eq!(o.contact, None, "no pad contact");
        assert_eq!(o.pos.cell(), spot);
        assert!(!air::on_ground(&w, a), "it does not land");
        testkit::run(&mut w, 1);
        // S-0501 claim 6: it falls back to rest at each tick it stands in its destination cell.
        assert_eq!(
            w.obj(a).unwrap().beh.last_common_effects,
            Some(w.tick - 1),
            "a fall-back to rest in that tick"
        );
    }
}
