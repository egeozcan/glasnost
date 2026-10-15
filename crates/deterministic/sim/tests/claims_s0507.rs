//! Claim tests of S-0507 (transports) on an empty synthetic map with
//! invented types (every name and number here is made up; no game data).

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::transport;
use glasnost_sim::world::{HouseId, ObjectId, World};
use glasnost_sim::{orders, save, testkit, tick};

const ALPHA: HouseId = HouseId(0);
const SIZE: i32 = 24;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

/// The combat ruleset plus HAULER, a wheeled vehicle transport for four
/// (size limit 2), and HEAVY, a walker of size 3.
fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    if let Some(v) = r.registries.get_mut("vehicles") {
        v.push("HAULER".into());
    }
    if let Some(v) = r.registries.get_mut("infantry") {
        v.push("HEAVY".into());
        v.push("HALF".into());
    }
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("LIFTER".into());
    let s = &mut r.sim;
    let mut lifter = UnitType {
        strength: Some(300),
        speed: Some(20),
        rot: Some(10),
        locomotor: Some(Locomotor::Fly),
        passengers: Some(2),
        ..Default::default()
    };
    lifter.econ.landable = Some(true);
    s.aircraft.insert("LIFTER".into(), lifter);
    s.vehicles.insert(
        "HAULER".into(),
        UnitType {
            strength: Some(400),
            speed: Some(6),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            passengers: Some(4),
            size_limit: dec("2"),
            ..Default::default()
        },
    );
    let mut heavy = s.infantry["WALKER"].clone();
    heavy.size = dec("3");
    s.infantry.insert("HEAVY".into(), heavy);
    let mut half = s.infantry["WALKER"].clone();
    half.size = dec("1.5");
    s.infantry.insert("HALF".into(), half);
    r
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 7)
}

fn at(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

/// Clicks `on` with `sel` selected and issues the click's orders.
fn click(w: &mut World, sel: &[ObjectId], on: ClickOn) -> targeting::ClickResult {
    let r = targeting::click(w, ALPHA, sel, on, Modifiers::default());
    for o in r.orders.clone() {
        orders::issue(w, ALPHA, o);
    }
    r
}

fn run_until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        tick::step(w);
    }
    done(w)
}

fn cursor(w: &World, sel: &[ObjectId], on: ObjectId) -> Action {
    targeting::cursor(w, ALPHA, sel, ClickOn::Object(on), Modifiers::default())
}

// spec: S-0507 claim 6 (infantry clicked onto a friendly transport walks to it and boards)
// spec: S-0214 table I row x3 (docking answer accepts: enter)
// spec: S-0507 claims 8 to 10 (the deploy order unloads it next to the transport)
#[test]
fn click_board_and_deploy_unload() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 0);
    testkit::run(&mut w, 5);
    assert_eq!(cursor(&w, &[p], t), Action::Enter);
    let c = click(&mut w, &[p], ClickOn::Object(t));
    assert_eq!(c.orders.len(), 1);
    assert!(
        run_until(&mut w, 600, |w| transport::riding(w, p) == Some(t)),
        "boarded"
    );
    assert_eq!(transport::passengers(&w, t), vec![p]);
    assert!(!w.obj(p).unwrap().on_map);
    assert_eq!(cursor(&w, &[t], t), Action::Deploy);
    click(&mut w, &[t], ClickOn::Object(t));
    assert!(
        run_until(&mut w, 600, |w| transport::riding(w, p).is_none()),
        "let out"
    );
    let po = w.obj(p).unwrap();
    assert!(po.on_map);
    assert_ne!(po.pos.cell(), w.obj(t).unwrap().pos.cell());
    assert_eq!(transport::count(&w, t), 0);
}

// spec: S-0507 claim 3 (boarding goes to position 1; leaving is from position 1)
// spec: S-0507 claims 6 and 10
#[test]
fn last_in_first_out_and_save_round_trip() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let a = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 0);
    let b = at(&mut w, Kind::Infantry, "WALKER", 0, -3, 0);
    testkit::run(&mut w, 5);
    click(&mut w, &[a], ClickOn::Object(t));
    assert!(run_until(&mut w, 600, |w| transport::riding(w, a) == Some(t)));
    click(&mut w, &[b], ClickOn::Object(t));
    assert!(run_until(&mut w, 600, |w| transport::riding(w, b) == Some(t)));
    assert_eq!(transport::passengers(&w, t), vec![b, a]);
    // A save with passengers aboard loads to the same state hash and plays on alike.
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    assert_eq!(transport::passengers(&l, t), vec![b, a]);
    click(&mut w, &[t], ClickOn::Object(t));
    click(&mut l, &[t], ClickOn::Object(t));
    assert!(run_until(&mut w, 600, |w| transport::count(w, t) < 2));
    assert!(
        transport::riding(&w, b).is_none(),
        "position 1 leaves first"
    );
    assert_eq!(transport::riding(&w, a), Some(t));
    while l.tick < w.tick {
        tick::step(&mut l);
    }
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
}

/// Boards `p` into `t` at once (claim 6), for tests of what follows.
fn put_in(w: &mut World, p: ObjectId, t: ObjectId) {
    transport::board(w, p, t);
}

// spec: S-0507 claim 6 (a vehicle boards while in enter with a transport contact)
// spec: S-0214 table V row q2 (docking answer accepts: enter)
#[test]
fn vehicle_boards_vehicle_transport() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let v = at(&mut w, Kind::Vehicle, "BUGGY", 0, 4, 0);
    testkit::run(&mut w, 5);
    assert_eq!(cursor(&w, &[v], t), Action::Enter);
    click(&mut w, &[v], ClickOn::Object(t));
    assert!(
        run_until(&mut w, 900, |w| transport::riding(w, v) == Some(t)),
        "the vehicle boarded"
    );
}

// spec: S-0507 claim 1 and F1 (capacity by size, the sum truncated after each addition)
// spec: S-0507 Edge cases (quirks.transport_load_truncated)
#[test]
fn capacity_by_size_truncated() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let mut halves = Vec::new();
    for i in 0..4 {
        halves.push(at(&mut w, Kind::Infantry, "HALF", 0, 3, i - 2));
    }
    let heavy = at(&mut w, Kind::Infantry, "HEAVY", 0, -3, 0);
    // spec: S-0507 claim 1 (s ≤ S_max: a size above the size limit is refused)
    assert_eq!(transport::answer(&w, t, heavy), transport::Answer::Refuses);
    assert_eq!(cursor(&w, &[heavy], t), Action::NoEnter);
    // 1.5 gives 1, 1 + 1.5 gives 2, 2 + 1.5 gives 3; 3 + 1.5 = 4.5 > 4.
    for &h in &halves[..3] {
        assert_eq!(transport::answer(&w, t, h), transport::Answer::Accepts);
        put_in(&mut w, h, t);
    }
    assert_eq!(
        transport::load(&w, t),
        glasnost_num::F64::from_i64(3, w.rules().mode)
    );
    assert_eq!(
        transport::answer(&w, t, halves[3]),
        transport::Answer::Refuses
    );
}

// spec: S-0507 Edge cases (switch off: the double-precision sum, 1.5 + 1.5 + 1.5 = 4.5 > 4 refuses the third)
#[test]
fn capacity_by_size_untruncated() {
    let mut r = ruleset();
    r.globals.quirks.transport_load_truncated = false;
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let a = at(&mut w, Kind::Infantry, "HALF", 0, 3, 0);
    let b = at(&mut w, Kind::Infantry, "HALF", 0, 3, 1);
    let c = at(&mut w, Kind::Infantry, "HALF", 0, 3, 2);
    put_in(&mut w, a, t);
    put_in(&mut w, b, t);
    assert_eq!(transport::answer(&w, t, c), transport::Answer::Refuses);
}

// spec: S-0507 claim 4 (a candidate not allied with the transport's owner: neither)
#[test]
fn enemy_gets_neither() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let e = at(&mut w, Kind::Infantry, "WALKER", 1, 3, 0);
    assert_eq!(transport::answer(&w, t, e), transport::Answer::Neither);
}

// spec: S-0507 F2 (worked example: f = 0 scores 0, −32, −64, −96, 28, 96, 64, 32; d = 5)
// spec: S-0507 F3 (q for f = 0 is 4)
#[test]
fn unloading_cell_worked_example() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    assert_eq!(w.obj(t).unwrap().body.value(w.tick), 0);
    assert_eq!(transport::rear_256(0), 128);
    assert_eq!(transport::rear_direction(0), 4);
    assert_eq!(transport::choose_direction(&w, t), Some(5));
}

// spec: S-0507 claim 2 (an air transport counts heads)
// spec: S-0507 claim 13 (landed: release position 1 at its own cell)
// spec: S-0214 table AC row AC7 (deploy if the aircraft carries passengers)
#[test]
fn air_transport_counts_heads_and_releases_landed() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Aircraft, "LIFTER", 0, 0, 0);
    testkit::run(&mut w, 5);
    assert!(glasnost_sim::air::on_ground(&w, t));
    let a = at(&mut w, Kind::Infantry, "HEAVY", 0, 3, 0);
    let b = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 1);
    let c = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 2);
    assert_eq!(transport::answer(&w, t, a), transport::Answer::Accepts);
    assert_eq!(cursor(&w, &[t], t), Action::Nothing);
    click(&mut w, &[a], ClickOn::Object(t));
    assert!(
        run_until(&mut w, 900, |w| transport::riding(w, a) == Some(t)),
        "boarded"
    );
    put_in(&mut w, b, t);
    assert_eq!(transport::answer(&w, t, c), transport::Answer::Refuses);
    assert_eq!(cursor(&w, &[t], t), Action::Deploy);
    click(&mut w, &[t], ClickOn::Object(t));
    assert!(
        run_until(&mut w, 900, |w| transport::count(w, t) < 2),
        "released one"
    );
    assert!(transport::riding(&w, b).is_none(), "position 1 first");
    assert_eq!(w.obj(b).unwrap().pos.cell(), w.obj(t).unwrap().pos.cell());
    assert!(
        run_until(&mut w, 900, |w| transport::count(w, t) == 0),
        "released all"
    );
}

/// A lethal hit with the combat ruleset's first warhead and no attacker.
fn hit(ignore_defences: bool) -> glasnost_sim::combat::damage::Hit {
    glasnost_sim::combat::damage::Hit {
        amount: 100_000,
        warhead: Some(0),
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences,
        no_escape: false,
    }
}

fn loaded(r: &Ruleset) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = game(r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let a = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 0);
    let b = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 1);
    put_in(&mut w, a, t);
    put_in(&mut w, b, t);
    (w, t, a, b)
}

// spec: S-0312 claim 12 (208 leptons or less, cannot crash, no bomb mark: each passenger gets out at the vehicle's position, with a forced scatter request)
#[test]
fn passengers_get_out_of_a_destroyed_vehicle() {
    let r = ruleset();
    let (mut w, t, a, b) = loaded(&r);
    let c = w.obj(t).unwrap().pos.cell();
    glasnost_sim::combat::damage::apply(&mut w, t, hit(false));
    assert!(w.obj(t).is_none(), "the transport left play");
    for p in [a, b] {
        let o = w.obj(p).expect("the passenger lives");
        assert!(o.on_map);
        assert_eq!(o.pos.cell(), c);
        assert!(transport::riding(&w, p).is_none());
    }
}

// spec: S-0312 claim 12 (a passenger dies instead when the killing damage ignores defences)
#[test]
fn passengers_die_when_the_hit_ignores_defences() {
    let r = ruleset();
    let (mut w, t, a, b) = loaded(&r);
    glasnost_sim::combat::damage::apply(&mut w, t, hit(true));
    assert!(w.obj(t).is_none());
    assert!(w.obj(a).is_none() && w.obj(b).is_none());
}

// spec: S-0312 claim 5 (an exploding object's passengers die with it)
#[test]
fn exploding_transport_kills_its_passengers() {
    let mut r = ruleset();
    if let Some(v) = r.sim.vehicles.get_mut("HAULER") {
        v.combat.explodes = Some(true);
    }
    let (mut w, t, a, b) = loaded(&r);
    glasnost_sim::combat::damage::apply(&mut w, t, hit(false));
    assert!(w.obj(t).is_none());
    assert!(w.obj(a).is_none() && w.obj(b).is_none());
}

// spec: S-0312 claim 12 (a type that can crash, on the ground: its passengers die when it leaves play)
#[test]
fn crashable_vehicle_passengers_die_on_the_ground() {
    let mut r = ruleset();
    if let Some(v) = r.sim.vehicles.get_mut("HAULER") {
        v.crashable = Some(true);
    }
    let (mut w, t, a, b) = loaded(&r);
    glasnost_sim::combat::damage::apply(&mut w, t, hit(false));
    assert!(w.obj(t).is_none());
    assert!(w.obj(a).is_none() && w.obj(b).is_none());
}

// spec: S-0312 claim 16 (an aircraft that starts to crash loses all its passengers)
#[test]
fn crashing_aircraft_loses_its_passengers() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Aircraft, "LIFTER", 0, 0, 0);
    let a = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 0);
    testkit::run(&mut w, 5);
    put_in(&mut w, a, t);
    orders::issue(
        &mut w,
        ALPHA,
        orders::Order::Behaviour {
            actor: t,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Cell(testkit::mid(SIZE, 8, 8))),
        },
    );
    assert!(
        run_until(&mut w, 400, |w| glasnost_sim::air::height(w, t) > 0),
        "airborne"
    );
    glasnost_sim::combat::damage::apply(&mut w, t, hit(false));
    assert!(glasnost_sim::air::crashing(&w, t));
    assert!(w.obj(a).is_none());
    assert_eq!(transport::count(&w, t), 0);
}

/// The ruleset with OPEN, an open-topped HAULER.
fn open_ruleset() -> Ruleset {
    let mut r = ruleset();
    if let Some(v) = r.registries.get_mut("vehicles") {
        v.push("OPEN".into());
    }
    let mut open = r.sim.vehicles["HAULER"].clone();
    open.open_topped = Some(true);
    r.sim.vehicles.insert("OPEN".into(), open);
    r
}

// spec: S-0507 claim 14 (an open-topped transport's passengers are in play and take turns, S-0203 claim 3)
// spec: S-0507 claim 15 (whenever the transport moves, its passengers' positions become its position)
// spec: S-0213 table T3 (f) (an open-topped transport gives its target to its passengers)
#[test]
fn open_topped_passengers_ride_along_and_take_the_target() {
    let r = open_ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "OPEN", 0, 0, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 0);
    let e = at(&mut w, Kind::Vehicle, "BUGGY", 1, 9, 9);
    put_in(&mut w, p, t);
    assert!(
        w.update.contains(p),
        "an open-topped passenger stays in the update order"
    );
    testkit::run(&mut w, 5);
    orders::issue(
        &mut w,
        ALPHA,
        orders::Order::Behaviour {
            actor: t,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Cell(testkit::mid(SIZE, -5, 0))),
        },
    );
    testkit::run(&mut w, 60);
    let tp = w.obj(t).unwrap().pos;
    assert_ne!(tp.cell(), testkit::mid(SIZE, 0, 0), "the transport moved");
    assert_eq!(w.obj(p).unwrap().pos, tp);
    orders::issue(
        &mut w,
        ALPHA,
        orders::Order::Behaviour {
            actor: t,
            behaviour: glasnost_sim::rules::Behaviour::Attack,
            attack_move: false,
            target: Some(glasnost_sim::world::Target::Object(e)),
            dest: None,
        },
    );
    testkit::run(&mut w, 1);
    assert_eq!(
        w.obj(p).unwrap().target,
        Some(glasnost_sim::world::Target::Object(e))
    );
    // spec: S-0213 claim 8 (stop: an open-topped transport's passengers lose their targets too)
    orders::issue(&mut w, ALPHA, orders::Order::Stop { actor: t });
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(p).unwrap().target, None);
}

// spec: S-0507 claim 6 and S-0203 claim 3 (a passenger of a closed transport is out of play)
#[test]
fn closed_transport_passenger_leaves_the_update_order() {
    let r = ruleset();
    let (w, _t, a, _b) = loaded(&r);
    assert!(!w.update.contains(a));
}

// spec: S-0507 claim 14 (open-topped passengers choose and fire weapons by the combat specs)
#[test]
fn open_topped_passenger_fires() {
    let r = open_ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "OPEN", 0, 0, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", 0, 0, 3);
    let e = at(&mut w, Kind::Vehicle, "BUGGY", 1, 3, 0);
    put_in(&mut w, p, t);
    let full = w.obj(e).unwrap().health;
    testkit::run(&mut w, 300);
    assert!(
        w.obj(e).is_none_or(|o| o.health < full),
        "the passenger's fire hurt the enemy"
    );
    assert_eq!(transport::riding(&w, p), Some(t));
}

fn move_to(w: &mut World, actor: ObjectId, dx: i32, dy: i32) {
    assert!(orders::issue(
        w,
        ALPHA,
        orders::Order::Behaviour {
            actor,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Cell(testkit::mid(
                SIZE, dx, dy
            ))),
        }
    ));
}

// spec: S-0507 claims 5 and 6 (a transport whose locomotor is moving refuses; the passenger arriving is turned away: no boarding, destination cleared)
#[test]
fn moving_transport_turns_the_passenger_away() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, -8, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", 0, -7, 1);
    testkit::run(&mut w, 5);
    move_to(&mut w, t, 9, 0);
    tick::step(&mut w);
    click(&mut w, &[p], ClickOn::Object(t));
    let moving = |w: &World| w.obj(t).unwrap().dest.is_some();
    assert!(
        run_until(&mut w, 900, |w| !moving(w)
            || transport::riding(w, p).is_some()
            || w.obj(p).unwrap().dest.is_none()),
        "the transport stops or the passenger gives up"
    );
    assert_eq!(transport::riding(&w, p), None, "not boarded while moving");
    assert!(
        w.obj(p).unwrap().dest.is_none(),
        "the turned-away passenger has no destination"
    );
}

/// HAULER with a turret count of 1 (claim 11), as KEEPER.
fn keeper_ruleset() -> Ruleset {
    let mut r = ruleset();
    let mut k = r.sim.vehicles["HAULER"].clone();
    k.turret_count = Some(1);
    r.sim.vehicles.insert("KEEPER".into(), k);
    if let Some(v) = r.registries.get_mut("vehicles") {
        v.push("KEEPER".into());
    }
    r
}

// spec: S-0507 claim 11 (a turret count above 0 keeps one passenger: letting out stops at it and the unload ends)
// spec: S-0507 table U (letting out: no more than kept, finished; finished: unload ends with guard)
#[test]
fn turret_count_keeps_one_passenger() {
    let r = keeper_ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "KEEPER", 0, 0, 0);
    let a = at(&mut w, Kind::Infantry, "WALKER", 0, 0, 4);
    let b = at(&mut w, Kind::Infantry, "WALKER", 0, 0, -4);
    put_in(&mut w, a, t);
    put_in(&mut w, b, t);
    testkit::run(&mut w, 5);
    assert_eq!(cursor(&w, &[t], t), Action::Deploy);
    click(&mut w, &[t], ClickOn::Object(t));
    assert!(
        run_until(&mut w, 900, |w| transport::count(w, t) < 2),
        "one is let out"
    );
    assert!(
        run_until(&mut w, 300, |w| w.obj(t).unwrap().beh.current
            != Some(glasnost_sim::rules::Behaviour::Unload)),
        "the unload ends"
    );
    testkit::run(&mut w, 60);
    assert_eq!(transport::count(&w, t), 1, "one passenger is kept");
    assert_eq!(transport::riding(&w, b), None, "position 1 got out");
    assert_eq!(transport::riding(&w, a), Some(t));
}

// spec: S-0507 claim 8 (settling: a moving transport waits; it unloads once stopped)
#[test]
fn deploy_while_moving_unloads_after_stopping() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, -6, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", 0, 0, 5);
    put_in(&mut w, p, t);
    testkit::run(&mut w, 5);
    move_to(&mut w, t, 6, 0);
    testkit::run(&mut w, 10);
    assert!(w.obj(t).unwrap().dest.is_some(), "still on its way");
    assert!(orders::issue(
        &mut w,
        ALPHA,
        orders::Order::Behaviour {
            actor: t,
            behaviour: glasnost_sim::rules::Behaviour::Unload,
            attack_move: false,
            target: None,
            dest: None,
        }
    ));
    assert!(
        run_until(&mut w, 900, |w| transport::riding(w, p).is_none()),
        "let out once stopped"
    );
    assert!(w.obj(p).unwrap().on_map);
}

// spec: S-0507 Edge cases (quirks.unload_facing_gap_no_wrap off: the signed facing difference; f = 0 gives 96 to both d = 3 and d = 5, the smaller d wins)
#[test]
fn unloading_cell_with_the_gap_wrapped() {
    let mut r = ruleset();
    r.globals.quirks.unload_facing_gap_no_wrap = false;
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    assert_eq!(transport::choose_direction(&w, t), Some(3));
}

/// A transport facing 0 (rear direction 4, one cell south) with a
/// passenger aboard, every other neighbour and the cell beyond the rear
/// neighbour held by vehicles, and an unload order given.
fn boxed_in(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let blockers = [
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
        (0, 2),
    ];
    for (dx, dy) in blockers {
        at(&mut w, Kind::Vehicle, "HAULER", 0, dx, dy);
    }
    let p = at(&mut w, Kind::Infantry, "WALKER", 0, 5, 5);
    put_in(&mut w, p, t);
    testkit::run(&mut w, 5);
    assert!(orders::issue(
        &mut w,
        ALPHA,
        orders::Order::Behaviour {
            actor: t,
            behaviour: glasnost_sim::rules::Behaviour::Unload,
            attack_move: false,
            target: None,
            dest: None,
        }
    ));
    (w, t, p)
}

// spec: S-0507 claim 10 and Edge cases (quirks.unload_second_round_skips_rear: the only free neighbour is the rear, so nobody gets out)
#[test]
fn second_round_skips_the_rear() {
    let r = ruleset();
    let (mut w, t, p) = boxed_in(&r);
    testkit::run(&mut w, 300);
    assert_eq!(transport::riding(&w, p), Some(t));
}

// spec: S-0507 Edge cases (switch off: the second round tries the rear direction first)
#[test]
fn second_round_tries_the_rear_with_the_quirk_off() {
    let mut r = ruleset();
    r.globals.quirks.unload_second_round_skips_rear = false;
    let (mut w, _t, p) = boxed_in(&r);
    assert!(
        run_until(&mut w, 300, |w| transport::riding(w, p).is_none()),
        "let out behind"
    );
    assert_eq!(w.obj(p).unwrap().pos.cell(), testkit::mid(SIZE, 0, 1));
}
