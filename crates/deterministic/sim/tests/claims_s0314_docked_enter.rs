//! S-0314 claims 20 and 23 (sim batch S13, #257): a docked vehicle keeps
//! its contact and tether whatever order it is given, and otherwise the
//! order runs normally; an enter order on another depot that holds a
//! contact therefore takes claim 23's queued entry: no destination, the
//! depot remembered, a queued entry. Synthetic types only.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{depot, testkit, tick};

const SIZE: i32 = 30;
const ALPHA: HouseId = HouseId(0);

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("TRUCK".into());
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("PAD".into());
    r.sim.vehicles.insert(
        "TRUCK".into(),
        UnitType {
            strength: Some(600),
            speed: Some(8),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            cost: Some(900),
            ..Default::default()
        },
    );
    // A 3 x 3 one-place repair depot with the pad flag.
    r.sim.buildings.insert(
        "PAD".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([3, 3]),
            unit_repair: Some(true),
            number_impassable_rows: Some(0),
            cost: Some(800),
            helipad: Some(true),
            ..Default::default()
        },
    );
    r.sim.general.repair_step = Some(10);
    r.sim.general.repair_percent = Some(Decimal("25%".into()));
    r
}

fn truck(w: &mut World, dx: i32, dy: i32, health: i32) -> ObjectId {
    let id = testkit::spawn(w, Kind::Vehicle, "TRUCK", 0, testkit::mid(SIZE, dx, dy), 0);
    w.obj_mut(id).unwrap().health = health;
    id
}

fn enter(actor: ObjectId, d: ObjectId) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Enter,
        attack_move: false,
        target: None,
        dest: Some(Target::Object(d)),
    }
}

/// Depots A and B; damaged V1 docked at A, damaged V2 holding B's only
/// place, and V1 ordered to enter B with health `v1_health`.
fn ordered(v1_health: i32) -> (World, ObjectId, ObjectId, ObjectId) {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 5);
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    let a = testkit::spawn(
        &mut w,
        Kind::Building,
        "PAD",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    let b = testkit::spawn(
        &mut w,
        Kind::Building,
        "PAD",
        0,
        testkit::mid(SIZE, -10, 0),
        0,
    );
    let v1 = truck(&mut w, 1, 1, 300);
    let v2 = truck(&mut w, -5, 1, 300);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    assert!(depot::request(&mut w, v1, a, false));
    w.obj_mut(v1).unwrap().beh.current = Some(Behaviour::Enter);
    depot::process_enter(&mut w, v1);
    assert!(depot::tethered(&w, v1), "V1 is docked at A");
    assert!(orders::issue(&mut w, ALPHA, enter(v2, b)));
    tick::step(&mut w);
    assert_eq!(w.obj(v2).unwrap().contact, Some(b), "V2 holds B's place");
    assert!(depot::tethered(&w, v1), "V1 still docked at A");
    w.obj_mut(v1).unwrap().health = v1_health;
    assert!(orders::issue(&mut w, ALPHA, enter(v1, b)));
    tick::step(&mut w);
    (w, v1, a, b)
}

// spec: S-0314 claim 20 (a docked vehicle keeps contact and tether; otherwise the order runs normally) and claim 23 (a later enter order on a depot that holds any contact: a damaged vehicle gets no destination and no contact, the depot is remembered and it gets a queued entry)
#[test]
fn a_damaged_docked_vehicle_queues_for_another_held_depot() {
    let (w, v1, a, b) = ordered(300);
    let o1 = w.obj(v1).unwrap();
    assert_eq!(o1.contact, Some(a), "claim 20: it keeps its contact");
    assert!(depot::tethered(&w, v1), "claim 20: and its tether");
    assert_eq!(o1.dest, None, "claim 23: no depot destination");
    assert_eq!(o1.depot.queued, Some(b), "claim 23: a queued entry for B");
    assert_eq!(o1.depot.entering, Some(b), "claim 23: B remembered");
}

// spec: S-0314 claim 20 (a docked vehicle keeps contact and tether) and claim 23 (a vehicle at full health gets no destination and no queued entry: the order ends)
#[test]
fn a_full_health_docked_vehicle_gets_nothing_from_another_held_depot() {
    let (w, v1, a, _) = ordered(600);
    let o1 = w.obj(v1).unwrap();
    assert_eq!(o1.contact, Some(a), "claim 20: it keeps its contact");
    assert_eq!(o1.dest, None, "claim 23: no destination");
    assert_eq!(o1.depot.queued, None, "claim 23: no queued entry");
}
