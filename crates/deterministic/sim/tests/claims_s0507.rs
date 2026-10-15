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
    }
    let s = &mut r.sim;
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
    assert!(transport::riding(&w, b).is_none(), "position 1 leaves first");
    assert_eq!(transport::riding(&w, a), Some(t));
    while l.tick < w.tick {
        tick::step(&mut l);
    }
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
}
