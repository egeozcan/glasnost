//! Claim tests of S-0503 revision 30 (Q-3263 answered), claim 20 and
//! Terms: the point an object destination gives a hovering vehicle (S-0502
//! claim 16's point: a building's foundation centre or docking point, a
//! mobile object's next movement point), an unchanged destination ignored
//! for a cell as for an object and after a stop request, and contact taken
//! after *D* is set. On a flat synthetic map with invented types (every
//! name and number here is made up; no game data). Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S42*.

mod common;

use common::{Scene, cell};
use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType,
};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{depot, econ, hover, testkit};

const ALPHA: HouseId = HouseId(0);

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn land_row() -> LandRow {
    LandRow {
        foot: dec("1"),
        track: dec("1"),
        wheel: dec("1"),
        hover: dec("1"),
        float: dec("0"),
        amphibious: dec("1"),
        float_beach: dec("0"),
        buildable: None,
    }
}

/// The combat ruleset with hover fields (invented values, no bob) and
/// - HOVER: a hovering vehicle of the plain amphibious zone;
/// - HUT2: a plain 2 × 2 building;
/// - PIT2: a 2 × 2 tank bunker;
/// - DEPOT2: a 2 × 2 repair depot with two docking places.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.land_types.insert(LandType::Clear, land_row());
    s.land_types.insert(LandType::Road, land_row());
    s.general.hover_height = Some(100);
    s.general.hover_bob = dec("0");
    s.general.hover_boost = dec("1.5");
    s.general.hover_acceleration = dec("0.02");
    s.general.hover_brake = dec("0.01");
    s.general.hover_dampen = dec("0.75");
    let hov = UnitType {
        strength: Some(100),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Hover),
        locomotor: Some(Locomotor::Hover),
        movement_zone: Some(MovementZone::Amphibious),
        ..Default::default()
    };
    s.vehicles.insert("HOVER".into(), hov);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("HOVER".into());
    let plain = BuildingType {
        strength: Some(500),
        foundation: Some([2, 2]),
        ..Default::default()
    };
    let pit = BuildingType {
        bunker: Some(true),
        number_impassable_rows: Some(0),
        ..plain.clone()
    };
    let mut dep = BuildingType {
        unit_repair: Some(true),
        number_impassable_rows: Some(0),
        ..plain.clone()
    };
    dep.econ.number_of_docks = Some(2);
    dep.econ.docking_offsets = Some(vec![[600, 40, 0], [40, 600, 0]]);
    for (n, b) in [("HUT2", plain), ("PIT2", pit), ("DEPOT2", dep)] {
        r.sim.buildings.insert(n.into(), b);
        r.registries
            .entry("buildings".into())
            .or_default()
            .push(n.into());
    }
    r
}

/// HOVER of Alpha's at (−8, 0), settled at its hover height.
fn world() -> (World, ObjectId) {
    let mut s = Scene::new();
    s.ruleset = rules();
    let mut w = s.build();
    let v = testkit::spawn(&mut w, Kind::Vehicle, "HOVER", 0, cell(-8, 0), 0);
    common::run(&mut w, 300);
    (w, v)
}

fn at(w: &mut World, kind: Kind, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, 0, cell(dx, dy), 0)
}

/// A point at the surface height at the point itself (S-0503 Terms: no
/// high bridge here).
fn on_surface(w: &World, p: Pos) -> Pos {
    Pos::new(p.x, p.y, w.surface_at(p.x, p.y))
}

fn d(w: &World, v: ObjectId) -> Option<Pos> {
    hover::destination_point(w, v)
}

// spec: S-0503 claim 20 (r30: the object's point is S-0502 claim 16's, with the hovering vehicle in place of the jumpjet: a building gives the centre of its foundation, its reference point of S-0204 claim 4) and Terms (the height at the point itself)
#[test]
fn r30_a_building_gives_its_foundation_centre() {
    let (mut w, v) = world();
    let hut = at(&mut w, Kind::Building, "HUT2", 4, 0);
    econ::assign_destination(&mut w, v, Some(Target::Object(hut)));
    let c = w.reference_point(hut);
    assert_ne!(
        c,
        w.obj(hut).unwrap().pos,
        "void: the centre is not the position"
    );
    assert_eq!(d(&w, v), Some(on_surface(&w, c)));
}

// spec: S-0503 claim 20 (r30: a vehicle, an infantry unit or an aircraft gives S-0607 claim 7's point for a lifter that is not a building: outside any transport its heading point, its next movement point, or its position when it has none)
#[test]
fn r30_a_walking_infantry_gives_its_next_movement_point() {
    let (mut w, v) = world();
    let i = at(&mut w, Kind::Infantry, "WALKER", 4, 4);
    common::order_move(&mut w, i, cell(4, -12));
    let moving = |w: &World| {
        let o = w.obj(i).unwrap();
        o.walk
            .as_ref()
            .and_then(|s| s.target)
            .is_some_and(|t| t != o.pos)
    };
    common::run_until(&mut w, 60, moving);
    assert!(moving(&w), "void: the walker has a walking target");
    let t = w.obj(i).unwrap().walk.as_ref().unwrap().target.unwrap();
    econ::assign_destination(&mut w, v, Some(Target::Object(i)));
    assert_eq!(d(&w, v), Some(on_surface(&w, t)), "its next movement point");
}

// spec: S-0503 claim 20 (r30: a building with the vehicle-bunker flag gives its docking point by S-0607 claim 7's rules, the quadrant point from the vehicle's position at the assignment, S-0606 claim 4, which does not depend on the contact)
#[test]
fn r30_a_bunker_gives_its_quadrant_point_without_contact() {
    let (mut w, v) = world();
    // North-west of the vehicle, so the quadrant point is not the bunker's position.
    let pit = at(&mut w, Kind::Building, "PIT2", -12, -6);
    econ::assign_destination(&mut w, v, Some(Target::Object(pit)));
    assert_eq!(w.obj(v).unwrap().contact, None, "void: no contact");
    let q = glasnost_sim::structures::bunker::docking_point(&w, pit, v);
    assert_ne!(
        on_surface(&w, q),
        on_surface(&w, w.obj(pit).unwrap().pos),
        "void: the quadrant point is not the position"
    );
    assert_eq!(d(&w, v), Some(on_surface(&w, q)));
}

// spec: S-0503 claim 20 (r30, Contact taken by the assignment: a contact taken after D is set does not change D; with more than one docking place and none held, the docking-place rule of S-0607 claim 7 gives the foundation centre)
#[test]
fn r30_a_later_contact_does_not_move_d() {
    let (mut w, v) = world();
    let k = at(&mut w, Kind::Building, "DEPOT2", 4, 0);
    econ::assign_destination(&mut w, v, Some(Target::Object(k)));
    let before = d(&w, v);
    assert_eq!(before, Some(on_surface(&w, w.reference_point(k))));
    assert!(
        depot::request(&mut w, v, k, false),
        "void: the depot accepts"
    );
    assert!(depot::link(&w, v).is_some(), "void: contact is taken");
    assert_eq!(d(&w, v), before, "D stays");
}

// spec: S-0503 claim 20 (r30, Contact taken by the assignment: an enter order on a repair depot that holds no contact makes its entry request at once, S-0314 claim 23, and D is the docking point with the contact taken, using the displacement of that place, S-0314 claim 18)
#[test]
fn r30_an_enter_order_on_a_depot_gives_its_places_docking_point() {
    let (mut w, v) = world();
    let k = at(&mut w, Kind::Building, "DEPOT2", 4, 0);
    let order = Order::Behaviour {
        actor: v,
        behaviour: Behaviour::Enter,
        attack_move: false,
        target: None,
        dest: Some(Target::Object(k)),
    };
    assert!(orders::issue(&mut w, ALPHA, order));
    orders::apply_pending(&mut w);
    let l = depot::link(&w, v).expect("void: contact is taken");
    let p = glasnost_sim::econ::release::pad_position_at(&w, k, l.place);
    assert_eq!(d(&w, v), Some(on_surface(&w, p)));
}

/// Gives `v` the destination `t`, runs 60 ticks and makes a stop request
/// (claim 10, by a switch-off): *D* is cleared, the order-level destination
/// stays.
fn stopped_towards(w: &mut World, v: ObjectId, t: Target) {
    econ::assign_destination(w, v, Some(t));
    common::run(w, 60);
    assert!(d(w, v).is_some(), "void: a D before");
    glasnost_sim::econ::power::apply_disabled(w, v, true);
    assert_eq!(
        w.obj(v).unwrap().dest,
        Some(t),
        "void: the destination stays"
    );
    assert_eq!(d(w, v), None, "void: D is cleared");
}

// spec: S-0503 claim 20 (r30, An unchanged destination: a cell is ignored in the same way as an object; the test reads the order-level destination, not D, so a vehicle whose D a stop request has cleared gets no new D when the same destination is given again, until it changes or is cleared)
#[test]
fn r30_after_a_stop_request_the_same_cell_leaves_d_cleared() {
    let (mut w, v) = world();
    let c = Target::Cell(cell(6, 0));
    stopped_towards(&mut w, v, c);
    econ::assign_destination(&mut w, v, Some(c));
    assert_eq!(d(&w, v), None, "the same cell is ignored");
    econ::assign_destination(&mut w, v, Some(Target::Cell(cell(6, 2))));
    assert!(d(&w, v).is_some(), "another cell gives a D");
}

// spec: S-0503 claim 20 (r30, An unchanged destination: after claim 10's stop request the same object given again gives no new D)
#[test]
fn r30_after_a_stop_request_the_same_object_leaves_d_cleared() {
    let (mut w, v) = world();
    let hut = at(&mut w, Kind::Building, "HUT2", 6, 0);
    let t = Target::Object(hut);
    stopped_towards(&mut w, v, t);
    econ::assign_destination(&mut w, v, Some(t));
    assert_eq!(d(&w, v), None, "the same object is ignored");
}

// spec: S-0503 claim 20 (r30, An unchanged destination: the ignored assignment sets nothing: D, the stored path, the heading point and the order-level destination stay, and no path search is made; a cell as an object)
#[test]
fn r30_a_repeated_move_order_keeps_the_stored_path() {
    let (mut w, v) = world();
    let c: CellXY = cell(10, 0);
    common::order_move(&mut w, v, c);
    common::run(&mut w, 30);
    let steps = w.obj(v).unwrap().path.steps.clone();
    let before = d(&w, v);
    assert!(!steps.is_empty(), "void: a stored path");
    common::order_move(&mut w, v, c);
    orders::apply_pending(&mut w);
    let o = w.obj(v).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(c)));
    assert_eq!(o.path.steps, steps, "the stored path stays");
    assert_eq!(d(&w, v), before, "D stays");
}
