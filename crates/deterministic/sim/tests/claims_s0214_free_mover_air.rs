//! A vehicle flying with the jumpjet locomotor is a free mover while it is
//! in the air (#359; S-0214 Terms, *free mover*, and claim 11). On the
//! shared `common` scene with invented types, no game data.

mod common;

use common::{FOOT, Scene, WHEEL, cell, code_tile, run, unit_type};
use glasnost_map::CellXY;
use glasnost_rules::sim::{Locomotor, UnitType};
use glasnost_sim::combat::weapon;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::orders::Order;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const ALPHA: HouseId = HouseId(0);

fn at(w: &World, c: CellXY) -> ObjectId {
    w.objects.values().find(|o| o.pos.cell() == c).unwrap().id
}

fn dest(r: &targeting::ClickResult) -> Option<Target> {
    match r.orders.as_slice() {
        [Order::Behaviour { dest, .. }] => *dest,
        other => panic!("one behaviour order expected: {other:?}"),
    }
}

// spec: S-0214 Terms (free mover: an actor that is in the air, is an aircraft, is of an underground type or is a jumpjet infantry type) and claim 11 (a free mover's move: the clicked cell itself, no search); S-0502 claim 30 (a unit with the jumpjet locomotor in the air)
#[test]
fn issue_359_a_jumpjet_vehicle_in_the_air_is_a_free_mover() {
    let mut s = Scene::new();
    s.vehicle(
        "TESTJETCAR",
        UnitType {
            locomotor: Some(Locomotor::Jumpjet),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    s.infantry(
        "TESTJETMAN",
        UnitType {
            jumpjet: Some(true),
            ..unit_type(FOOT, 20, 0)
        },
    );
    // An enclosed cell: a ring of rock (no land row: blocked for every class).
    let enclosed = cell(6, 0);
    for dy in -1..=1 {
        for dx in -1..=1 {
            if (dx, dy) != (0, 0) {
                s.set_cell(
                    CellXY::new(enclosed.x + dx, enclosed.y + dy),
                    code_tile(7),
                    0,
                );
            }
        }
    }
    s.place_unit(0, "TESTJETCAR", cell(0, 3), 64)
        .place_infantry(0, "TESTJETMAN", cell(0, 0), 2);
    let mut w = s.build();
    run(&mut w, 1);
    let car = at(&w, cell(0, 3));
    let man = at(&w, cell(0, 0));
    jump::start_in_flight(&mut w, car, Phase::Hovering, 500);
    run(&mut w, 1);
    assert!(jump::is_jumpjet(&w, car), "set-up");
    assert!(weapon::in_air(&w, car), "set-up: the vehicle is in the air");
    let on = ClickOn::Cell(enclosed);
    assert_eq!(
        targeting::action(&w, ALPHA, man, on, Modifiers::default()),
        Action::Move,
        "control"
    );
    let r = targeting::click(&w, ALPHA, &[man], on, Modifiers::default());
    assert_eq!(
        dest(&r),
        Some(Target::Cell(enclosed)),
        "control: jumpjet infantry"
    );
    assert_eq!(
        targeting::action(&w, ALPHA, car, on, Modifiers::default()),
        Action::Move,
        "set-up"
    );
    let r = targeting::click(&w, ALPHA, &[car], on, Modifiers::default());
    assert_eq!(
        dest(&r),
        Some(Target::Cell(enclosed)),
        "a free mover's move takes the clicked cell itself"
    );
}
