//! Claim test for S-0409 claim 8 and Terms, *Arrival in a cell* (#205): a
//! driving unit moves its sight at its cell-arrival update, not when its
//! position crosses a cell boundary. Synthetic data: an invented slow
//! driving vehicle SLOW with sight 2 on the testkit's flat map.

use glasnost_map::CellXY;
use glasnost_rules::sim::{LocomotionClass, Locomotor, UnitType};
use glasnost_sim::econ::shroud;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Target, World};

const SIZE: i32 = 40;
const A: HouseId = HouseId(0);

// spec: S-0409 claim 8 (a walking or driving unit removes and adds its sight at the position reached on each arrival in a cell; every moving object also every 15 ticks) and Terms (arrival in a cell: the movement arrival update, not merely crossing a cell boundary; driving arrivals follow S-0207); S-0207 claim 12 (reaching the track target is the cell-arrival update)
#[test]
fn a_driving_units_sight_moves_at_its_arrival_not_at_the_boundary_crossing() {
    let mut r = testkit::ruleset();
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("SLOW".into());
    r.sim.vehicles.insert(
        "SLOW".into(),
        UnitType {
            strength: Some(100),
            speed: Some(2),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            sight: Some(2),
            ..Default::default()
        },
    );
    let mut w = testkit::game_with(&r, SIZE, 3);
    testkit::run(&mut w, 4);
    let start = testkit::mid(SIZE, 0, 0);
    let e = testkit::spawn(&mut w, Kind::Vehicle, "SLOW", 0, start, 0);
    glasnost_sim::tick::step(&mut w);
    assert!(glasnost_sim::orders::issue(
        &mut w,
        A,
        glasnost_sim::orders::Order::Behaviour {
            actor: e,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(testkit::mid(SIZE, 6, 0))),
        }
    ));
    let count = |w: &World| {
        let mut n = 0;
        for dy in -12..=12 {
            for dx in -12..=12 {
                if shroud::explored(w, A, CellXY::new(start.x + dx, start.y + dy)) {
                    n += 1;
                }
            }
        }
        n
    };
    let mut early = Vec::new();
    let mut at_arrival = Vec::new();
    for _ in 0..600 {
        let before = w.obj(e).unwrap().clone();
        let n0 = count(&w);
        glasnost_sim::tick::step(&mut w);
        let o = w.obj(e).unwrap();
        let centre = o.pos.x.rem_euclid(256) == 128 && o.pos.y.rem_euclid(256) == 128;
        let periodic = o.econ.sight.refreshed != before.econ.sight.refreshed;
        let grew = count(&w) > n0;
        if grew && !centre && !periodic {
            early.push(w.tick);
        }
        if grew && centre && !periodic {
            at_arrival.push(w.tick);
        }
    }
    assert!(
        early.is_empty(),
        "the explored area grew on ticks {early:?}, with neither an arrival at a cell centre nor a 15-tick refresh"
    );
    assert!(
        !at_arrival.is_empty(),
        "some arrival comes before the next 15-tick refresh and moves the sight"
    );
}
