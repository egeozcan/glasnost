//! Claim tests of S-0507 (transports) on a synthetic map with a lake:
//! hovering and ship transports (S-0503, S-0504) and hovering passengers.
//! Every name and number here is made up; no game data.

mod common;

use common::{Scene, cell, code_tile};
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType};
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::transport;
use glasnost_sim::world::{HouseId, ObjectId, World};
use glasnost_sim::{orders, testkit, tick};

const ALPHA: HouseId = HouseId(0);
/// The terrain code of a water tile in the scene's catalogue (S-0211 Terms).
const WATER_CODE: u8 = 9;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn land_row(ground: &str, float: &str) -> LandRow {
    LandRow {
        foot: dec(ground),
        track: dec(ground),
        wheel: dec(ground),
        hover: dec("1"),
        float: dec(float),
        amphibious: dec("1"),
        float_beach: dec(float),
        buildable: None,
    }
}

fn hover_type() -> UnitType {
    UnitType {
        strength: Some(100),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Hover),
        locomotor: Some(Locomotor::Hover),
        movement_zone: Some(MovementZone::Amphibious),
        ..Default::default()
    }
}

/// The combat ruleset with water rows and hover fields, and
/// - HOVERT: an amphibious hovering transport for four;
/// - FERRY: a ship transport for four;
/// - HOVER: a hovering vehicle;
/// - HAULER: a wheeled transport for four.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.land_types.insert(LandType::Clear, land_row("1", "0"));
    s.land_types.insert(LandType::Road, land_row("1", "0"));
    s.land_types.insert(LandType::Water, land_row("0", "1"));
    s.general.wake = Some("Ripple".into());
    s.general.hover_height = Some(100);
    s.general.hover_bob = dec("0.02");
    s.general.hover_boost = dec("1.5");
    s.general.hover_acceleration = dec("0.02");
    s.general.hover_brake = dec("0.01");
    s.general.hover_dampen = dec("0.75");
    let hovert = UnitType {
        passengers: Some(4),
        size_limit: dec("2"),
        ..hover_type()
    };
    let ferry = UnitType {
        strength: Some(300),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Float),
        locomotor: Some(Locomotor::Ship),
        movement_zone: Some(MovementZone::Water),
        passengers: Some(4),
        size_limit: dec("2"),
        ..Default::default()
    };
    let hauler = UnitType {
        strength: Some(400),
        speed: Some(6),
        rot: Some(8),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        passengers: Some(4),
        size_limit: dec("2"),
        ..Default::default()
    };
    for (n, u) in [
        ("HOVERT", hovert),
        ("FERRY", ferry),
        ("HOVER", hover_type()),
        ("HAULER", hauler),
    ] {
        r.sim.vehicles.insert(n.into(), u);
        r.registries
            .entry("vehicles".into())
            .or_default()
            .push(n.into());
    }
    r
}

/// The lake: the cells with −3 ≤ dx ≤ 3 and −6 ≤ dy ≤ 6.
fn lake(dx: i32, dy: i32) -> bool {
    (-3..=3).contains(&dx) && (-6..=6).contains(&dy)
}

fn world() -> World {
    let mut s = Scene::new();
    s.ruleset = rules();
    for dx in -8..=8 {
        for dy in -8..=8 {
            if lake(dx, dy) {
                s.set_cell(cell(dx, dy), code_tile(WATER_CODE), 0);
            }
        }
    }
    s.build()
}

fn at(w: &mut World, kind: Kind, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, 0, cell(dx, dy), 0)
}

fn click(w: &mut World, sel: &[ObjectId], on: ClickOn) {
    let r = targeting::click(w, ALPHA, sel, on, Modifiers::default());
    assert!(!r.orders.is_empty(), "the click gives an order");
    for o in r.orders {
        assert!(orders::issue(w, ALPHA, o));
    }
}

fn cursor(w: &World, sel: &[ObjectId], on: ClickOn) -> Action {
    targeting::cursor(w, ALPHA, sel, on, Modifiers::default())
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

fn land_of(w: &World, id: ObjectId) -> u8 {
    let c = w.obj(id).unwrap().pos.cell();
    w.cells.cell(c).unwrap().land
}

// spec: S-0507 claims 5 and 6 (infantry boards a hovering transport on land)
// spec: S-0507 claim 7 (unloading on water is refused for a player house)
// spec: S-0507 claims 8 to 10 (it unloads on the far shore)
#[test]
fn hover_transport_ferries_infantry_across_the_lake() {
    let mut w = world();
    let t = at(&mut w, Kind::Vehicle, "HOVERT", -5, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", -8, 0);
    testkit::run(&mut w, 5);
    assert_eq!(cursor(&w, &[p], ClickOn::Object(t)), Action::Enter);
    click(&mut w, &[p], ClickOn::Object(t));
    assert!(
        run_until(&mut w, 900, |w| transport::riding(w, p) == Some(t)),
        "boarded on land"
    );
    click(&mut w, &[t], ClickOn::Cell(cell(0, 0)));
    assert!(
        run_until(&mut w, 900, |w| {
            land_of(w, t) == land::WATER && w.obj(t).unwrap().dest.is_none()
        }),
        "on the lake"
    );
    assert_eq!(cursor(&w, &[t], ClickOn::Object(t)), Action::NoDeploy);
    click(&mut w, &[t], ClickOn::Cell(cell(6, 0)));
    assert!(
        run_until(&mut w, 1200, |w| {
            land_of(w, t) != land::WATER && w.obj(t).unwrap().dest.is_none()
        }),
        "across"
    );
    assert_eq!(cursor(&w, &[t], ClickOn::Object(t)), Action::Deploy);
    click(&mut w, &[t], ClickOn::Object(t));
    assert!(
        run_until(&mut w, 900, |w| transport::riding(w, p).is_none()),
        "let out"
    );
    let c = w.obj(p).unwrap().pos.cell();
    assert!(c.x > cell(3, 0).x, "on the far shore");
    assert_ne!(land_of(&w, p), land::WATER);
}

// spec: S-0507 claim 6 (a hovering vehicle boards a vehicle transport)
#[test]
fn hovering_vehicle_boards_a_transport() {
    let mut w = world();
    let t = at(&mut w, Kind::Vehicle, "HAULER", -6, 3);
    let v = at(&mut w, Kind::Vehicle, "HOVER", -6, -3);
    testkit::run(&mut w, 5);
    assert_eq!(cursor(&w, &[v], ClickOn::Object(t)), Action::Enter);
    click(&mut w, &[v], ClickOn::Object(t));
    assert!(
        run_until(&mut w, 900, |w| transport::riding(w, v) == Some(t)),
        "the hovering vehicle boarded"
    );
}

// spec: S-0507 claim 5 (a transport on water with no destination takes the requester as its destination; not an acceptance)
#[test]
fn ship_transport_approaches_a_requester() {
    let mut w = world();
    let t = at(&mut w, Kind::Vehicle, "FERRY", -2, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", -6, 0);
    testkit::run(&mut w, 5);
    assert_eq!(cursor(&w, &[p], ClickOn::Object(t)), Action::Enter);
    assert_eq!(
        transport::load_request(&mut w, t, p),
        transport::Reply::Approach
    );
    assert_eq!(
        w.obj(t).unwrap().dest,
        Some(glasnost_sim::world::Target::Object(p))
    );
}

// spec: S-0507 claim 7 (a player house's transport on water lacks deploy readiness)
#[test]
fn ship_transport_cannot_unload_on_water() {
    let mut w = world();
    let t = at(&mut w, Kind::Vehicle, "FERRY", 0, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", -6, 0);
    transport::board(&mut w, p, t);
    assert!(!transport::may_unload(&w, t));
    assert_eq!(cursor(&w, &[t], ClickOn::Object(t)), Action::NoDeploy);
}

