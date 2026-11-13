//! Claim test for S-0506 claim 14 (#191, Q-3044): the heading point of a
//! moving hovering vehicle is the point its locomotor moves it to (S-0503's
//! *P*), not its position; a switched-off one that keeps *P* is aimed at its
//! position. Synthetic rules and map; every name and number is invented.

mod common;

use common::cell;
use glasnost_rules::sim::{
    Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType,
};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Target, World};
use glasnost_sim::{rocket, testkit};

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

/// A world with one hovering vehicle type, HOVER, and one of them at
/// cell (0, 0), settled.
fn hover_world() -> (World, ObjectId) {
    let mut r = testkit::combat_ruleset();
    let row = LandRow {
        foot: dec("1"),
        track: dec("1"),
        wheel: dec("1"),
        hover: dec("1"),
        float: dec("0"),
        amphibious: dec("1"),
        float_beach: dec("0"),
        buildable: None,
    };
    r.sim.land_types.insert(LandType::Clear, row);
    r.sim.general.hover_height = Some(100);
    r.sim.general.hover_bob = dec("0.02");
    r.sim.general.hover_boost = dec("1.5");
    r.sim.general.hover_acceleration = dec("0.02");
    r.sim.general.hover_brake = dec("0.01");
    r.sim.general.hover_dampen = dec("0.75");
    r.sim.vehicles.insert(
        "HOVER".into(),
        UnitType {
            strength: Some(100),
            speed: Some(6),
            rot: Some(5),
            locomotion_class: Some(LocomotionClass::Hover),
            locomotor: Some(Locomotor::Hover),
            movement_zone: Some(MovementZone::Amphibious),
            ..Default::default()
        },
    );
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("HOVER".into());
    assert!(r.globals.quirks.rocket_aim_at_heading_point);
    let mut s = common::Scene::new();
    s.ruleset = r;
    let mut w = s.build();
    let id = testkit::spawn(&mut w, Kind::Vehicle, "HOVER", 0, cell(0, 0), 16384);
    common::run(&mut w, 300);
    (w, id)
}

// spec: S-0506 claim 14 (a unit's heading point: the point its locomotor is moving it to, the end of the step it is on); S-0207 Terms and S-0503 Terms (a hovering vehicle's heading point P, the next cell centre); Q-3044 (interim: P while it is powered)
#[test]
fn rocket_aim_at_a_moving_hover_is_its_heading_point() {
    let (mut w, id) = hover_world();
    common::order_move(&mut w, id, cell(5, 0));
    // Run until the hover has a heading point and is away from it.
    common::run_until(&mut w, 400, |w| {
        let o = w.obj(id).unwrap();
        let p = o.hover.as_ref().and_then(|h| h.point);
        p.is_some_and(|p| (p.x - o.pos.x).abs() + (p.y - o.pos.y).abs() > 64)
            && o.pos.cell() != cell(0, 0)
    });
    let o = w.obj(id).unwrap();
    let p = o.hover.as_ref().unwrap().point.expect("a heading point");
    assert_ne!(
        (p.x, p.y),
        (o.pos.x, o.pos.y),
        "the hover is short of its heading point"
    );
    assert!(!glasnost_sim::combat::weapon::in_air(&w, id));
    let aim = rocket::heading_point(&w, Target::Object(id)).unwrap();
    assert_eq!(
        (aim.x, aim.y, aim.z),
        (p.x, p.y, p.z),
        "the rocket aim is the hover's heading point, not its position"
    );
}

// spec: S-0506 claim 14 (its position when it is not moving); S-0503 claim 10 (a switch-off's stop request keeps the heading point), Terms (powered) and F1 (T = 0 when not powered); Q-3044 (interim: P only while powered, so a settled switched-off hover that keeps P is aimed at its position, as M-0016's stock rows for a switched-off hover against rockets show)
#[test]
fn rocket_aim_at_a_switched_off_hover_is_its_position() {
    let (mut w, id) = hover_world();
    common::order_move(&mut w, id, cell(5, 0));
    // Run until the hover has a heading point and is away from it.
    common::run_until(&mut w, 400, |w| {
        let o = w.obj(id).unwrap();
        let p = o.hover.as_ref().and_then(|h| h.point);
        p.is_some_and(|p| (p.x - o.pos.x).abs() + (p.y - o.pos.y).abs() > 64)
    });
    glasnost_sim::hover::switched(&mut w, id, true);
    // Let it settle; it keeps its heading point and stops travelling.
    common::run(&mut w, 300);
    let o = w.obj(id).unwrap();
    let p = o
        .hover
        .as_ref()
        .unwrap()
        .point
        .expect("the heading point is kept");
    assert_ne!((p.x, p.y), (o.pos.x, o.pos.y), "it never reaches it");
    assert_eq!(glasnost_sim::hover::hover_height(&w, id), 0, "settled");
    assert!(!glasnost_sim::hover::powered(&w, id), "not powered");
    assert!(glasnost_sim::hover::moving(&w, id), "moving: it keeps P");
    let aim = rocket::heading_point(&w, Target::Object(id)).unwrap();
    assert_eq!(
        (aim.x, aim.y, aim.z),
        (o.pos.x, o.pos.y, o.pos.z),
        "the rocket aim is the switched-off hover's position"
    );
}
