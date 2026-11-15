//! The guard key gives *area guard* to a vehicle that controls slave
//! units, armed or not, also without the harvester flag (#356; S-0213 table
//! T2). On the synthetic rules of `harvest_common` (`HARV` a harvester with
//! no weapon, `SLAVEM` a slave miner with no harvester flag and no weapon),
//! no game data.

mod harvest_common;

use glasnost_sim::orders::Order;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::targeting::{self, KeyCommand};
use glasnost_sim::world::{HouseId, ObjectId, Target};
use harvest_common::*;

fn area_guard(id: ObjectId, cell: glasnost_map::CellXY) -> Order {
    Order::Behaviour {
        actor: id,
        behaviour: Behaviour::AreaGuard,
        attack_move: false,
        target: Some(Target::Cell(cell)),
        dest: None,
    }
}

// spec: S-0213 table T2 (guard: movable, able to fire and not a building; or a vehicle that is a harvester or controls slave units, movable or not: behaviour order area guard on its own cell)
#[test]
fn issue_356_guard_key_gives_area_guard_to_an_unarmed_slave_master_vehicle() {
    let r = ruleset();
    let mut w = game(&r);
    let harv = place_vehicle(&mut w, "HARV", 0, at(-3, 0), 0);
    let miner = place_vehicle(&mut w, "SLAVEM", 0, at(3, 0), 0);
    let house = HouseId(0);
    assert!(glasnost_sim::seams::is_slave_master(&w, miner), "set-up");
    assert!(targeting::movable(&w, house, miner), "set-up");
    assert!(!targeting::able_to_fire(&w, house, miner), "set-up");
    assert_eq!(
        targeting::key_command(&w, house, &[harv], KeyCommand::Guard),
        vec![area_guard(harv, at(-3, 0))],
        "control: the unarmed harvester"
    );
    assert_eq!(
        targeting::key_command(&w, house, &[miner], KeyCommand::Guard),
        vec![area_guard(miner, at(3, 0))],
        "the unarmed slave master"
    );
}
