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
// spec: S-0507 claim 13 (landed, no destination: selection, then release of position 1 into the aircraft's own cell by S-0208 claim 3's choice)
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

// spec: S-0312 claim 12 (the dying vehicle is excluded from its passengers' escape entry tests; another stationary allied vehicle in the cell still counts, giving stationary friendly, so the passenger dies)
#[test]
fn escape_counts_other_vehicles_in_the_cell() {
    let r = ruleset();
    let (mut w, t, a, b) = loaded(&r);
    let c = w.obj(t).unwrap().pos.cell();
    let other = testkit::spawn(&mut w, Kind::Vehicle, "HAULER", 0, c, 0);
    glasnost_sim::combat::damage::apply(&mut w, t, hit(false));
    assert!(w.obj(t).is_none());
    assert!(w.obj(other).is_some(), "the second vehicle survives");
    assert!(w.obj(a).is_none() && w.obj(b).is_none(), "both died");
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

// spec: S-0312 claim 12 (a type that can crash, on the ground: its passengers die without attacker credit at removal) and S-0507 claim 16
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

// ---------------------------------------------------------------- revision 4

use glasnost_map::CellXY;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::world::{Pos, Target};

/// Forces *unload* on `t` (claim 8 holds whoever queued it), progress 0.
fn force_unload(w: &mut World, t: ObjectId) {
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(t)
        .unwrap()
        .beh
        .force(Behaviour::Unload, tick, stale);
    assert_eq!(w.obj(t).unwrap().beh.progress, 0);
}

// spec: S-0507 Terms (tunnel edge for unloading: map north or west; first marked and second not, or second marked and third not; every cell through the non-marked one exists; no south, east or diagonal test)
#[test]
fn tunnel_edge_predicate() {
    let c = CellXY::new(10, 10);
    let all = |_: CellXY| true;
    // The first neighbour north marked, the second not.
    assert!(transport::tunnel_edge(c, all, |x| x == CellXY::new(10, 9)));
    // The second neighbour west marked, the third not.
    assert!(transport::tunnel_edge(c, all, |x| x == CellXY::new(8, 10)));
    // The first and second north marked, the third not: the second case.
    let two = |x: CellXY| x == CellXY::new(10, 9) || x == CellXY::new(10, 8);
    assert!(transport::tunnel_edge(c, all, two));
    // A band north with the first, second and third marked: neither case.
    let band = |x: CellXY| x.x == 10 && (7..=9).contains(&x.y);
    assert!(!transport::tunnel_edge(c, all, band));
    // South, east and diagonal marks add nothing.
    for m in [CellXY::new(10, 11), CellXY::new(11, 10), CellXY::new(9, 9)] {
        assert!(!transport::tunnel_edge(c, all, |x| x == m), "{m:?}");
    }
    // The non-marked cell of the sequence must exist.
    assert!(!transport::tunnel_edge(
        c,
        |x| x != CellXY::new(10, 8),
        |x| x == CellXY::new(10, 9)
    ));
    assert!(!transport::tunnel_edge(
        c,
        |x| x != CellXY::new(7, 10),
        |x| x == CellXY::new(8, 10)
    ));
    // The cell itself marked is not an edge by this predicate.
    assert!(!transport::tunnel_edge(c, all, |x| x == c));
}

// spec: S-0507 Terms (tunnel land alone does not make a cell tunnel-marked; a map without tubes has none) and claim 7
#[test]
fn no_tube_no_tunnel_cell() {
    let r = ruleset();
    let w = game(&r);
    for dx in -2..=2 {
        for dy in -2..=2 {
            let c = testkit::mid(SIZE, dx, dy);
            assert!(!transport::tunnel_marked(&w, c));
            assert!(!transport::tunnel_cell(&w, c));
        }
    }
}

/// Places a new walker of house `house` at within-cell point (`u`, `v`)
/// of `c`.
fn walker_at(w: &mut World, house: u8, c: CellXY, u: i32, v: i32) -> ObjectId {
    let ti = w.rules().index_of(Kind::Infantry, "WALKER").unwrap();
    let id = glasnost_sim::objects::create(w, Kind::Infantry, ti, Some(HouseId(house)));
    let (x, y) = (256 * c.x + u, 256 * c.y + v);
    let z = w.surface_at(x, y);
    glasnost_sim::objects::place(w, id, Pos::new(x, y, z), 0);
    id
}

// spec: S-0507 claim 9 (the alliance test reads the ground-layer object nearest the cell's top corner by within-cell coordinates, not the infantry house; with none selected it passes)
#[test]
fn alliance_object_nearest_the_top_corner() {
    let r = ruleset();
    let mut w = game(&r);
    let n = testkit::mid(SIZE, 4, 4);
    assert_eq!(transport::alliance_object(&w, n), None);
    let friend = walker_at(&mut w, 0, n, 192, 64);
    assert_eq!(transport::alliance_object(&w, n), Some(friend));
    let foe = walker_at(&mut w, 1, n, 64, 64);
    // The top position is nearer than the right one (S-0208 table P).
    assert_eq!(transport::alliance_object(&w, n), Some(foe));
}

// spec: S-0507 claim 9 (right and left positions tie: the most recently added object is selected)
#[test]
fn alliance_object_tie_goes_to_the_latest() {
    let r = ruleset();
    let mut w = game(&r);
    let n = testkit::mid(SIZE, 4, 4);
    let _first = walker_at(&mut w, 0, n, 192, 64);
    let second = walker_at(&mut w, 1, n, 64, 192);
    assert_eq!(transport::alliance_object(&w, n), Some(second));
}

/// The standing point S-0208 table O's centre-class row gives for draw
/// `k` in an empty cell (table P: right 192,64; left 64,192; bottom 192,192).
fn centre_choice(k: i32) -> (i32, i32) {
    match k {
        0 | 1 => (192, 64),
        2 => (64, 192),
        _ => (192, 192),
    }
}

// spec: S-0507 claim 10 and S-0208 claim 3 (an infantry passenger: the serving neighbour's centre as desired point, one ranged draw over 0 … 3, right/left/bottom preferences; never centre or top)
#[test]
fn infantry_exit_takes_the_centre_class_choice() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", 0, 5, 5);
    put_in(&mut w, p, t);
    force_unload(&mut w, t);
    w.obj_mut(t).unwrap().beh.progress = transport::phase::LETTING_OUT;
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut rr = w.rng.clone();
    let k = rr.ranged(0, 3, half);
    transport::process_unload(&mut w, t);
    let po = w.obj(p).unwrap();
    assert!(po.on_map, "let out");
    assert_ne!(po.pos.cell(), w.obj(t).unwrap().pos.cell());
    assert_eq!(
        (po.pos.x.rem_euclid(256), po.pos.y.rem_euclid(256)),
        centre_choice(k),
        "draw {k}"
    );
}

// spec: S-0507 claim 10 and S-0212 claim 25 (a vehicle passenger: the nearby-cell search from the serving neighbour, at the selected cell's centre; its move destination stays the exit-round cell)
#[test]
fn vehicle_exit_uses_the_nearby_search() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let v = at(&mut w, Kind::Vehicle, "BUGGY", 0, 6, 6);
    put_in(&mut w, v, t);
    force_unload(&mut w, t);
    w.obj_mut(t).unwrap().beh.progress = transport::phase::LETTING_OUT;
    transport::process_unload(&mut w, t);
    let vo = w.obj(v).unwrap();
    assert!(vo.on_map, "let out");
    assert_eq!(
        (vo.pos.x.rem_euclid(256), vo.pos.y.rem_euclid(256)),
        (128, 128)
    );
    let c = w.obj(t).unwrap().pos.cell();
    let n = vo.pos.cell();
    let (dx, dy) = (n.x - c.x, n.y - c.y);
    assert!(dx.abs() <= 1 && dy.abs() <= 1 && (dx, dy) != (0, 0));
    assert_eq!(vo.beh.queued, Some(Behaviour::Move));
    assert_eq!(vo.dest, Some(Target::Cell(CellXY::new(n.x + dx, n.y + dy))));
}

/// A landed LIFTER carrying `n` walkers, in *unload*.
fn landed_lifter(r: &Ruleset, n: usize) -> (World, ObjectId, Vec<ObjectId>) {
    let mut w = game(r);
    let t = at(&mut w, Kind::Aircraft, "LIFTER", 0, 0, 0);
    testkit::run(&mut w, 5);
    assert!(glasnost_sim::air::on_ground(&w, t));
    let mut ps = Vec::new();
    for i in 0..n {
        let p = at(&mut w, Kind::Infantry, "WALKER", 0, 5, i as i32);
        put_in(&mut w, p, t);
        ps.push(p);
    }
    force_unload(&mut w, t);
    (w, t, ps)
}

// spec: S-0507 claim 13 (selection at height 0 with no destination: release at a later processing; release: position 1 at the aircraft's exact position by S-0208 claim 3's choice with the aircraft excluded, facing the first clear neighbour in preference 4, 5, 3, 7, 1, 0, 6, 2 times 8192, moving toward it; fall back to rest when empty)
#[test]
fn air_release_phases_and_placement() {
    let r = ruleset();
    let (mut w, t, ps) = landed_lifter(&r, 2);
    transport::process_air_unload(&mut w, t);
    assert_eq!(
        w.obj(t).unwrap().beh.progress,
        transport::air_phase::RELEASE
    );
    let c = w.obj(t).unwrap().pos.cell();
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut rr = w.rng.clone();
    let k = rr.ranged(0, 3, half);
    transport::process_air_unload(&mut w, t);
    let p = ps[1];
    let po = w.obj(p).unwrap();
    assert!(po.on_map);
    assert_eq!(po.pos.cell(), c, "in the aircraft's own cell");
    assert_eq!(
        (po.pos.x.rem_euclid(256), po.pos.y.rem_euclid(256)),
        centre_choice(k)
    );
    assert_eq!(
        i32::from(po.body.value(w.tick)),
        4 * 8192,
        "direction 4 is clear"
    );
    assert_eq!(po.beh.queued, Some(Behaviour::Move));
    assert_eq!(po.dest, Some(Target::Cell(CellXY::new(c.x, c.y + 1))));
    assert_eq!(transport::count(&w, t), 1);
    assert_eq!(transport::riding(&w, ps[0]), Some(t));
    transport::process_air_unload(&mut w, t);
    assert_eq!(transport::count(&w, t), 0);
}

// spec: S-0507 claim 13 (stopped check and reselect return 1 tick with no delay draw; stopped check goes to release once stopped; reselect returns to selection)
// spec: S-0215 claim 29 (aircraft passenger unloading: exactly 1 tick with no delay draw in those phases)
#[test]
fn air_stopped_check_and_reselect_take_one_tick() {
    let r = ruleset();
    let (mut w, t, _ps) = landed_lifter(&r, 1);
    for (from, to) in [
        (
            transport::air_phase::STOPPED_CHECK,
            transport::air_phase::RELEASE,
        ),
        (
            transport::air_phase::RESELECT,
            transport::air_phase::SELECTION,
        ),
    ] {
        w.obj_mut(t).unwrap().beh.progress = from;
        let before = w.rng.clone();
        assert_eq!(transport::process_air_unload(&mut w, t), 1);
        assert_eq!(w.rng, before, "no delay draw");
        assert_eq!(w.obj(t).unwrap().beh.progress, to);
    }
}

/// A LIFTER with one walker aboard, airborne on its way to a far cell, in
/// *unload*.
fn flying_lifter(r: &Ruleset) -> (World, ObjectId) {
    let mut w = game(r);
    let t = at(&mut w, Kind::Aircraft, "LIFTER", 0, 0, 0);
    let a = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 0);
    testkit::run(&mut w, 5);
    put_in(&mut w, a, t);
    orders::issue(
        &mut w,
        ALPHA,
        orders::Order::Behaviour {
            actor: t,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(testkit::mid(SIZE, 8, 8))),
        },
    );
    assert!(
        run_until(&mut w, 400, |w| glasnost_sim::air::height(w, t) > 0),
        "airborne"
    );
    force_unload(&mut w, t);
    (w, t)
}

// spec: S-0507 claim 13 and Edge cases (quirks.air_unload_reselects_clear_destination: a destination passing F7 is replaced through the replacement search around map waypoint 700; selection stays)
#[test]
fn air_selection_replaces_a_clear_destination() {
    let r = ruleset();
    let (mut w, t) = flying_lifter(&r);
    let d = w.obj(t).unwrap().dest.unwrap();
    assert!(glasnost_sim::air::landing_test(&w, t, d));
    let wp = testkit::mid(SIZE, -5, 4);
    w.triggers.waypoints.insert(700, (wp.x as i16, wp.y as i16));
    transport::process_air_unload(&mut w, t);
    let o = w.obj(t).unwrap();
    assert_eq!(o.beh.progress, transport::air_phase::SELECTION);
    // Radius 0 is the waypoint cell itself, open and unreserved.
    assert_eq!(o.dest, Some(Target::Cell(wp)));
}

// spec: S-0507 Edge cases (switch off: a passing destination takes the height-based approach or reselect transition and is kept)
#[test]
fn air_selection_quirk_off_keeps_a_clear_destination() {
    let mut r = ruleset();
    r.globals.quirks.air_unload_reselects_clear_destination = false;
    let (mut w, t) = flying_lifter(&r);
    let d = w.obj(t).unwrap().dest;
    let h = glasnost_sim::air::height(&w, t);
    let cruise = glasnost_sim::air::cruise_height(&w, t);
    transport::process_air_unload(&mut w, t);
    let o = w.obj(t).unwrap();
    assert_eq!(o.dest, d, "kept");
    let want = if h == cruise {
        transport::air_phase::APPROACH
    } else {
        transport::air_phase::RESELECT
    };
    assert_eq!(o.beh.progress, want);
}

// spec: S-0507 claim 13 (airborne with no destination, not a dropship: stopped check, no landing-spot search)
#[test]
fn air_selection_without_destination_checks_stopped() {
    let r = ruleset();
    let (mut w, t) = flying_lifter(&r);
    glasnost_sim::econ::assign_destination(&mut w, t, None);
    transport::process_air_unload(&mut w, t);
    let o = w.obj(t).unwrap();
    assert_eq!(o.beh.progress, transport::air_phase::STOPPED_CHECK);
    assert_eq!(o.dest, None);
}

/// Releases position 1 of landed LIFTER `t` raised `up` leptons above the
/// terrain, returning the aircraft's position.
fn release_at(w: &mut World, t: ObjectId, up: i32) -> Pos {
    w.obj_mut(t).unwrap().pos.z += up;
    w.obj_mut(t).unwrap().beh.progress = transport::air_phase::RELEASE;
    let pos = w.obj(t).unwrap().pos;
    transport::process_air_unload(w, t);
    pos
}

// spec: S-0507 claim 13 (release placement by passenger kind and height: infantry above the surface and every vehicle at the aircraft's exact horizontal position and height, with no standing-position choice and no nearby search; both kinds face and move by the first clear neighbour)
#[test]
fn air_release_by_kind_and_height() {
    let r = ruleset();
    // Airborne infantry: the exact position and height; the standing-position
    // choice from the cell centre would never give the centre.
    let (mut w, t, ps) = landed_lifter(&r, 1);
    let pos = release_at(&mut w, t, 300);
    assert_eq!((pos.x.rem_euclid(256), pos.y.rem_euclid(256)), (128, 128));
    let po = w.obj(ps[0]).unwrap();
    assert!(po.on_map);
    assert_eq!(
        po.pos, pos,
        "no standing-position choice, no projection to the surface"
    );
    assert_eq!(i32::from(po.body.value(w.tick)), 4 * 8192);
    // A landed vehicle: the exact horizontal position, no choice or search.
    let (mut w, t, _) = landed_lifter(&r, 0);
    let v = at(&mut w, Kind::Vehicle, "BUGGY", 0, 6, 6);
    put_in(&mut w, v, t);
    let pos = release_at(&mut w, t, 0);
    let vo = w.obj(v).unwrap();
    assert!(vo.on_map);
    assert_eq!(vo.pos, pos);
    let c = pos.cell();
    assert_eq!(
        i32::from(vo.body.value(w.tick)),
        4 * 8192,
        "same preference as infantry"
    );
    assert_eq!(vo.dest, Some(Target::Cell(CellXY::new(c.x, c.y + 1))));
    // An airborne vehicle keeps the aircraft's height.
    let (mut w, t, _) = landed_lifter(&r, 0);
    let v = at(&mut w, Kind::Vehicle, "BUGGY", 0, 6, 6);
    put_in(&mut w, v, t);
    let pos = release_at(&mut w, t, 300);
    assert_eq!(w.obj(v).unwrap().pos, pos);
}

// spec: S-0507 claim 13 (the ordinary placement entry test must also find the aircraft's own cell clear; a refused own cell fails the placement even with a clear neighbour: the unit stays off the map, out of the passenger order)
#[test]
fn air_release_needs_a_clear_own_cell() {
    let r = ruleset();
    let (mut w, t, _) = landed_lifter(&r, 0);
    let v = at(&mut w, Kind::Vehicle, "BUGGY", 0, 6, 6);
    put_in(&mut w, v, t);
    let c = w.obj(t).unwrap().pos.cell();
    let _blocker = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, c, 0);
    release_at(&mut w, t, 0);
    assert!(!w.obj(v).is_some_and(|o| o.on_map), "not placed");
    assert_eq!(transport::count(&w, t), 0);
}

// spec: S-0507 claim 13 and Import mapping (an undefined waypoint 700 supplies cell (0, 0); the search still runs with one draw per radius 0 to 31; failure keeps the centre (0, 0), not the previous destination)
#[test]
fn air_selection_without_waypoint_700_searches_around_the_origin() {
    let r = ruleset();
    let (mut w, t) = flying_lifter(&r);
    assert!(!w.triggers.waypoints.contains_key(&700));
    let mut w2 = w.clone();
    let want = glasnost_sim::air::replacement_search(&mut w2, t, CellXY::new(0, 0));
    let before = w.obj(t).unwrap().dest;
    transport::process_air_unload(&mut w, t);
    let o = w.obj(t).unwrap();
    assert_ne!(o.dest, before, "not the previous destination");
    assert_eq!(
        o.dest,
        Some(Target::Cell(want.unwrap_or(CellXY::new(0, 0)))),
        "the search around (0, 0); its centre when it fails"
    );
}

// spec: S-0507 claim 16 (removal cleanup: passengers still carried are killed, nested passengers too, with no attacker credit and no death effects or draws)
// spec: S-0408 claim 11 (a sold vehicle's remaining passengers die by that cleanup)
#[test]
fn removal_kills_remaining_passengers() {
    let mut r = ruleset();
    let mut mini = r.sim.vehicles["HAULER"].clone();
    mini.passengers = Some(1);
    mini.size = dec("1");
    r.sim.vehicles.insert("MINI".into(), mini);
    if let Some(v) = r.registries.get_mut("vehicles") {
        v.push("MINI".into());
    }
    let mut w = game(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let m = at(&mut w, Kind::Vehicle, "MINI", 0, 3, 0);
    let a = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 3);
    let b = at(&mut w, Kind::Infantry, "WALKER", 0, -3, 3);
    put_in(&mut w, a, m);
    put_in(&mut w, m, t);
    put_in(&mut w, b, t);
    let rng = w.rng.clone();
    w.effects.clear();
    glasnost_sim::objects::remove(&mut w, t);
    assert!(w.obj(t).is_none());
    for p in [m, a, b] {
        assert!(w.obj(p).is_none(), "{p:?} died");
    }
    assert_eq!(w.rng, rng, "no draws");
    // Only the kill notices: no animation or other death effect.
    assert!(
        w.effects
            .iter()
            .all(|e| matches!(e, glasnost_sim::combat::effects::Effect::Killed { .. })),
        "no death effects: {:?}",
        w.effects
    );
}

// spec: S-0507 claim 16 and S-0312 claim 15 (an aircraft killed on the ground leaves play at once; its passengers die without attacker credit)
#[test]
fn aircraft_killed_on_the_ground_loses_its_passengers() {
    let r = ruleset();
    let mut w = game(&r);
    let t = at(&mut w, Kind::Aircraft, "LIFTER", 0, 0, 0);
    testkit::run(&mut w, 5);
    assert!(glasnost_sim::air::on_ground(&w, t));
    let a = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 0);
    put_in(&mut w, a, t);
    glasnost_sim::combat::damage::apply(&mut w, t, hit(false));
    assert!(w.obj(t).is_none());
    assert!(w.obj(a).is_none());
}

// --- Notifications (claim 6; S-0524 claim 14) ------------------------------------

/// A game on a map with the entry-notification tags of `testkit::entry_triggers`.
fn tagged(r: &Ruleset) -> World {
    testkit::game_with_triggers(r, SIZE, 7, testkit::entry_triggers())
}

fn tag(w: &mut World, id: ObjectId, name: &str) {
    glasnost_sim::triggers::object_tag(w, id, Some(name.as_bytes()));
}

/// Variable `n` of the entry triggers.
fn var(w: &World, n: usize) -> bool {
    w.triggers.variables[n]
}

// spec: S-0507 claim 6 and S-0524 claim 14 (an infantry unit's qualifying arrival gives one entered-by raise on the transport's tag, none on receiving the order; no own-tag destruction raise for the passenger; unloading adds none and a fresh arrival notifies again)
#[test]
fn claim_6_infantry_boarding_raises_the_transport_tag_once() {
    let r = ruleset();
    for (kind, name) in [(Kind::Vehicle, "HAULER"), (Kind::Aircraft, "LIFTER")] {
        let mut w = tagged(&r);
        let t = at(&mut w, kind, name, 0, 0, 0);
        let p = at(&mut w, Kind::Infantry, "WALKER", 0, 3, 0);
        tag(&mut w, t, "ENT");
        tag(&mut w, p, "DIE");
        testkit::run(&mut w, 5);
        let f0 = w.triggers.fired;
        click(&mut w, &[p], ClickOn::Object(t));
        assert_eq!(w.triggers.fired, f0, "{name}: no raise on receiving the order");
        assert!(run_until(&mut w, 900, |w| transport::riding(w, p) == Some(t)), "{name}: boarded");
        assert_eq!(w.triggers.fired - f0, 1, "{name}: one entry raise");
        assert!(var(&w, 0) && !var(&w, 1), "{name}: the passenger gets no destruction raise");
        // Unloading adds no entered-by raise; boarding again notifies again.
        let f1 = w.triggers.fired;
        click(&mut w, &[t], ClickOn::Object(t));
        assert!(run_until(&mut w, 900, |w| transport::riding(w, p).is_none()), "{name}: let out");
        assert_eq!(w.triggers.fired, f1, "{name}: unloading raises nothing");
        click(&mut w, &[p], ClickOn::Object(t));
        assert!(run_until(&mut w, 900, |w| transport::riding(w, p) == Some(t)), "{name}: boarded again");
        assert_eq!(w.triggers.fired - f1, 1, "{name}: a fresh arrival notifies again");
    }
}

// spec: S-0507 claim 6 and S-0524 claim 14 (a vehicle's boarding gives no entered-by raise on the transport's tag and no destruction raise for the vehicle)
#[test]
fn claim_6_vehicle_boarding_raises_nothing() {
    let r = ruleset();
    let mut w = tagged(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    let v = at(&mut w, Kind::Vehicle, "BUGGY", 0, 4, 0);
    tag(&mut w, t, "ENT");
    tag(&mut w, v, "DIE");
    testkit::run(&mut w, 5);
    let f0 = w.triggers.fired;
    click(&mut w, &[v], ClickOn::Object(t));
    assert!(run_until(&mut w, 900, |w| transport::riding(w, v) == Some(t)), "boarded");
    assert_eq!(w.triggers.fired, f0);
    assert!(!var(&w, 0) && !var(&w, 1));
}

// spec: S-0507 claim 6 and S-0524 claim 14 (the arrival raise precedes the final acceptance test: a boarding that the transport then refuses has already notified)
#[test]
fn claim_6_refused_boarding_has_already_notified() {
    let r = ruleset();
    let mut w = tagged(&r);
    let t = at(&mut w, Kind::Vehicle, "HAULER", 0, 0, 0);
    // Four passengers fill the transport, so the final acceptance test refuses.
    for i in 0..4 {
        let q = at(&mut w, Kind::Infantry, "WALKER", 0, 6, i);
        put_in(&mut w, q, t);
    }
    // An arriving infantry unit in the transport's cell with the transport as its destination.
    let p = at(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    tag(&mut w, t, "ENT");
    testkit::run(&mut w, 5);
    w.obj_mut(p).unwrap().dest = Some(glasnost_sim::world::Target::Object(t));
    assert_eq!(transport::answer(&w, t, p), transport::Answer::Refuses);
    let f0 = w.triggers.fired;
    assert!(!transport::arrival(&mut w, p), "turned away");
    assert_eq!(w.triggers.fired - f0, 1, "the arrival raise came first");
    assert_eq!(transport::riding(&w, p), None);
    assert_ne!(w.obj(p).unwrap().dest, Some(glasnost_sim::world::Target::Object(t)), "it gave up its destination");
}
