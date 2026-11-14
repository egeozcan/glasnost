//! S-0503 claims 10 and 15 with S-0212 claim 20: a hover whose movement
//! destination a switch-off stop request cleared makes no fresh search for
//! its order-level destination when it reaches its heading point (#270).
//! Synthetic map, invented types and values.

mod common;

use common::{Scene, cell, code_tile};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType,
};
use glasnost_sim::hover;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{Target, World};
use glasnost_sim::{testkit, tick};

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

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.land_types.insert(LandType::Clear, land_row("1", "0"));
    s.land_types.insert(LandType::Road, land_row("1", "0"));
    s.land_types.insert(LandType::Water, land_row("0", "1"));
    s.general.hover_height = Some(100);
    s.general.hover_bob = dec("0.02");
    s.general.hover_boost = dec("1.5");
    s.general.hover_acceleration = dec("0.02");
    s.general.hover_brake = dec("0.01");
    s.general.hover_dampen = dec("0.75");
    // A short retry delay (invented).
    s.general.path_delay = dec("0.03");
    let u = UnitType {
        strength: Some(100),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Hover),
        locomotor: Some(Locomotor::Hover),
        movement_zone: Some(MovementZone::Amphibious),
        ..Default::default()
    };
    r.sim.vehicles.insert("HOVER".into(), u);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("HOVER".into());
    r
}

fn world() -> World {
    let mut s = Scene::new();
    s.ruleset = rules();
    for dx in -6..=6 {
        for dy in -6..=6 {
            s.set_cell(cell(dx, dy), code_tile(WATER_CODE), 0);
        }
    }
    s.build()
}

// spec: S-0503 claim 10 (the stop request clears D and the stored path and leaves the order-level destination), claim 15 (no fresh search is made without D), S-0212 claim 20 (a hover's searches use its movement destination, not its order-level destination)
#[test]
fn issue_270_no_search_on_arrival_after_a_switch_off_stop_request() {
    let mut w = world();
    // On land beside the lake, so the switch-off does not sink it.
    let id = testkit::spawn(&mut w, Kind::Vehicle, "HOVER", 0, cell(-8, -8), 16384);
    common::run(&mut w, 300);
    let c = cell(-8, 6);
    common::order_move(&mut w, id, c);
    common::run_until(&mut w, 20, |w| w.obj(id).unwrap().dest.is_some());
    // Travel until the retry delay of the trip's search has run out and the
    // hover is close to a heading point that is not its destination.
    let mut switched = false;
    for _ in 0..400 {
        tick::step(&mut w);
        let o = w.obj(id).unwrap();
        let h = o.hover.clone().unwrap();
        let Some(p) = h.point else { continue };
        let near = (p.x - o.pos.x).abs() + (p.y - o.pos.y).abs() <= 24;
        if near && w.tick >= o.path.retry_at && hover::destination_point(&w, id) != Some(p) {
            switched = true;
            break;
        }
    }
    assert!(switched, "a moment to switch it off was found");
    glasnost_sim::econ::power::apply_disabled(&mut w, id, true);
    let o = w.obj(id).unwrap();
    assert_eq!(
        o.dest,
        Some(Target::Cell(c)),
        "order-level destination stays"
    );
    assert!(
        o.path.steps.is_empty(),
        "the stop request cleared the stored path"
    );
    assert_eq!(hover::destination_point(&w, id), None, "D is cleared");
    assert!(o.hover.as_ref().unwrap().point.is_some(), "P is kept");
    let retry_before = o.path.retry_at;
    assert!(w.tick >= retry_before, "the retry delay has run out");
    // Run to the arrival at the heading point.
    common::run_until(&mut w, 200, |w| {
        w.obj(id).unwrap().hover.as_ref().unwrap().point.is_none()
    });
    let o = w.obj(id).expect("the hover is alive");
    assert!(o.hover.as_ref().unwrap().point.is_none(), "it arrived");
    assert_eq!(hover::destination_point(&w, id), None, "still no D");
    assert!(!hover::moving(&w, id), "not moving: no D and no P");
    assert_eq!(
        o.path.retry_at, retry_before,
        "no fresh search without D: the retry delay was not started again"
    );
    assert!(
        o.path.steps.is_empty(),
        "no fresh search without D: no stored path"
    );
}
