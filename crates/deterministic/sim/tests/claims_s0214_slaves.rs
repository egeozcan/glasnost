//! S-0214 Terms, *movable*, and claim 19 (#209): a slave unit and an object
//! being erased are not movable, and an ordinary band box does not take a
//! slave. Synthetic data: an invented slave infantry type SLAVE and a 3 × 3
//! slave-master building SMB on the helpers of `tests/harvest_common`.

mod harvest_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{LocomotionClass, Locomotor, UnitType};
use glasnost_sim::econ::slaves;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, KeyCommand, Modifiers};
use glasnost_sim::temporal::Lock;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};
use harvest_common::*;

const SLAVES: i32 = 3;
const HOUSE: HouseId = HouseId(0);

fn rules() -> Ruleset {
    let mut r = ruleset();
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("SLAVE".into());
    let mut slave = UnitType {
        strength: Some(100),
        speed: Some(6),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        storage: Some(2),
        ..Default::default()
    };
    slave.econ.harvest_rate = Some(3);
    r.sim.infantry.insert("SLAVE".into(), slave);
    let mut b = building(3, 3);
    b.econ.enslaves = Some("SLAVE".into());
    b.econ.slaves_number = Some(SLAVES);
    b.econ.slave_regen_rate = Some(50);
    b.econ.slave_reload_rate = Some(40);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("SMB".into());
    r.sim.buildings.insert("SMB".into(), b);
    r
}

fn on_map(w: &World, m: ObjectId) -> Vec<ObjectId> {
    slaves::slaves_of(w, m)
        .into_iter()
        .filter(|&s| w.obj(s).is_some_and(|o| o.on_map))
        .collect()
}

/// A slave master with its slaves out at work on an ore field; returns the
/// first slave on the map.
fn working_slave() -> (World, ObjectId) {
    let r = rules();
    let mut ore = Vec::new();
    for dy in 0..3 {
        for dx in 0..3 {
            ore.push((at(5 + dx, -1 + dy), ORE, 5));
        }
    }
    let mut w = game_with_ore(&r, &ore, 7);
    let b = place_building(&mut w, "SMB", 0, at(0, -1));
    for _ in 0..40 {
        testkit::run(&mut w, 1);
        if on_map(&w, b).len() == SLAVES as usize {
            break;
        }
    }
    let out = on_map(&w, b);
    assert_eq!(out.len(), SLAVES as usize, "the slaves emerged");
    let s = out[0];
    assert!(slaves::is_slave(&w, s));
    assert_eq!(slaves::master_of(&w, s), Some(b));
    assert!(targeting::selectable(&w, s), "a click may select it");
    (w, s)
}

fn moves(orders: &[Order]) -> bool {
    orders.iter().any(|o| {
        matches!(
            o,
            Order::Behaviour {
                behaviour: Behaviour::Move,
                ..
            }
        )
    })
}

// spec: S-0214 Terms (movable: it is not a slave unit) and claim 19 (an ordinary band box takes units not controlled by a slave master); table GC rows GC9 and GC12 (no move order for an actor that is not movable)
#[test]
fn a_working_slave_is_not_movable_and_not_taken_by_a_band_box() {
    let (w, s) = working_slave();
    assert!(
        !targeting::box_eligible(&w, HOUSE, &[], s),
        "claim 19: band box"
    );
    assert!(!targeting::movable(&w, HOUSE, s), "Terms: movable");
    let click = targeting::click(
        &w,
        HOUSE,
        &[s],
        ClickOn::Cell(at(-6, -6)),
        Modifiers::default(),
    );
    assert!(!moves(&click.orders), "cell click: {:?}", click.orders);
    let scatter = targeting::key_command(&w, HOUSE, &[s], KeyCommand::Scatter);
    assert!(scatter.is_empty(), "scatter: {scatter:?}");
}

// spec: S-0214 Terms (movable: it is not being erased) and S-0213 table T2 (scatter: movable)
#[test]
fn a_unit_being_erased_gets_no_scatter() {
    let r = testkit::combat_ruleset();
    let mut w = testkit::game_with(&r, 32, 7);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, testkit::mid(32, 0, 0), 0);
    let holder = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, testkit::mid(32, 3, 0), 0);
    assert_eq!(
        targeting::key_command(&w, HOUSE, &[v], KeyCommand::Scatter),
        vec![Order::Scatter { actor: v }],
        "control"
    );
    w.temporal.locks.insert(
        v,
        Lock {
            leader: holder,
            joiners: Vec::new(),
            amount: 0,
            was_off: false,
            suspended: false,
        },
    );
    assert!(glasnost_sim::superweapons::being_erased(&w, v));
    assert!(!targeting::movable(&w, HOUSE, v));
    let k = targeting::key_command(&w, HOUSE, &[v], KeyCommand::Scatter);
    assert!(k.is_empty(), "scatter: {k:?}");
}
