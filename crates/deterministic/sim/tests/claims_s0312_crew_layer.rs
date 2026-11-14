//! S-0312 claim 13 with S-0209 Terms (cell layer): a crew put at the exact
//! position of a vehicle whose height differs from the terrain belongs to
//! the layer the vehicle was in. A vehicle hovering under a high bridge is
//! on the ground layer, so its crew is too (sim batch S13, #250).

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LocomotionClass, Locomotor, MovementZone, SequenceEntry, UnitType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::Layer;

const SIZE: i32 = 24;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("HOVER".into());
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("TECH".into());
    let s = &mut r.sim;
    s.vehicles.insert(
        "HOVER".into(),
        UnitType {
            strength: Some(100),
            speed: Some(6),
            rot: Some(5),
            locomotion_class: Some(LocomotionClass::Hover),
            locomotor: Some(Locomotor::Hover),
            movement_zone: Some(MovementZone::Amphibious),
            crewed: Some(true),
            ..Default::default()
        },
    );
    let mut tech = UnitType {
        strength: Some(60),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        ..Default::default()
    };
    for k in [
        "stand", "walk", "go_prone", "prone", "crawl", "get_up", "die_1", "die_2",
    ] {
        tech.sequences.insert(
            k.into(),
            SequenceEntry {
                start: 0,
                count: 4,
                multiplier: 0,
                direction: None,
            },
        );
    }
    s.infantry.insert("TECH".into(), tech);
    // Synthetic values: a certain crew escape, a hover height.
    s.general.crew_escape = dec("1");
    s.general.technician = Some("TECH".into());
    s.general.hover_height = Some(100);
    s.general.hover_bob = dec("0.02");
    s.general.hover_dampen = dec("0.75");
    r
}

// spec: S-0312 claim 13 (another height: the vehicle's exact position); S-0209 Terms (cell layer: an object on a bridge deck is in the deck layer, every other object in the ground layer)
#[test]
fn crew_of_a_hover_vehicle_under_a_high_bridge_is_on_the_ground_layer() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 11);
    let c = testkit::mid(SIZE, 0, 0);
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
    let id = testkit::spawn(&mut w, Kind::Vehicle, "HOVER", 0, c, 0);
    testkit::run(&mut w, 60);
    let o = w.obj(id).unwrap().clone();
    let surface = w.surface_at(o.pos.x, o.pos.y);
    assert_eq!(o.pos.cell(), c);
    assert_eq!(o.layer, Layer::Ground, "the vehicle is under the deck");
    assert!(o.pos.z > surface, "it hovers above the terrain");
    damage::apply(
        &mut w,
        id,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    let ti = w.rules().index_of(Kind::Infantry, "TECH").unwrap();
    let crew: Vec<_> = w
        .objects
        .values()
        .filter(|x| x.kind == Kind::Infantry && x.type_index == ti)
        .cloned()
        .collect();
    assert_eq!(crew.len(), 1, "one crew");
    assert_eq!(crew[0].pos, o.pos, "at the vehicle's exact position");
    assert_eq!(
        crew[0].layer,
        Layer::Ground,
        "the crew of a vehicle under the deck belongs to the ground layer"
    );
    assert!(w.cells.list(c, Layer::Ground).contains(&crew[0].id));
    assert!(!w.cells.list(c, Layer::Deck).contains(&crew[0].id));
}
