//! S-0209 claim 5, part 5d: row VP3 stands above rows VP5 to VP7 (issue
//! 322). Synthetic data only.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal};
use glasnost_sim::occupancy::{self, Outcome};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};
use glasnost_sim::{testkit, tick};

const SIZE: i32 = 24;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    // A 2 x 2 vehicle absorber with an impassable-rows value of 0: none of
    // its foundation cells is a blocking cell (S-0209 Terms).
    let mut t = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        unit_absorb: Some(true),
        passengers: Some(2),
        size_limit: Some(Decimal("3".into())),
        number_impassable_rows: Some(0),
        ..Default::default()
    };
    t.econ.power = Some(10);
    register(&mut r, "buildings", "VBIO0");
    r.sim.buildings.insert("VBIO0".into(), t);
    let mut car = r.sim.vehicles["BUGGY"].clone();
    car.primary = None;
    register(&mut r, "vehicles", "CAR");
    r.sim.vehicles.insert("CAR".into(), car);
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn out(w: &World, id: ObjectId, c: CellXY) -> Outcome {
    occupancy::vehicle_outcome(w, id, c, occupancy::reference_level(w, id))
}

// spec: S-0209 claim 5 (part 5d: each occupant fits the first row from the top; row VP3 stands above row VP5; row VP20)
#[test]
fn issue_322_vp3_before_vp5_for_a_vehicle_in_contact_with_its_absorber() {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    // An idle allied vehicle on the cell that becomes foundation cell (1, 0),
    // placed before the building.
    let idle = testkit::spawn(&mut w, Kind::Vehicle, "CAR", 0, at(1, 0), 0);
    let b = testkit::spawn(&mut w, Kind::Building, "VBIO0", 0, at(0, 0), 0);
    testkit::run(&mut w, 3);
    let f = at(1, 0);
    assert_eq!(w.obj(idle).unwrap().pos.cell(), f);
    assert!(!occupancy::blocking_cell(&w, b, f), "not a blocking cell");
    let mover = testkit::spawn(&mut w, Kind::Vehicle, "CAR", 0, at(5, 0), 0);
    testkit::run(&mut w, 2);
    orders::issue(
        &mut w,
        glasnost_sim::world::HouseId(0),
        Order::Behaviour {
            actor: mover,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    );
    // The ordinary flow of S-0605 claims 3 and 4: wait in line, then contact,
    // enter and the building as destination.
    let mut ready = false;
    for _ in 0..800 {
        tick::step(&mut w);
        let o = w.obj(mover).unwrap();
        if o.on_map
            && o.contact == Some(b)
            && o.dest == Some(Target::Object(b))
            && o.beh.seen() == Some(Behaviour::Enter)
        {
            ready = true;
            break;
        }
    }
    assert!(ready, "the mover reached enter with contact and destination");
    assert_eq!(w.obj(idle).unwrap().pos.cell(), f, "the idle vehicle stayed");
    assert!(w.obj(idle).unwrap().dest.is_none());
    // Row VP3 (none) for the building, row VP20 for the idle allied vehicle.
    assert_eq!(out(&w, mover, f), Outcome::StationaryFriendly);
}
