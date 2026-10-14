//! Claim tests for S-0212 claim 27a (a harvester without contact on a
//! refinery's docking cell) on synthetic data: the invented harvest
//! ruleset of `harvest_common`, no stock data. House Alpha (0) of the
//! testkit's sandbox game, human.

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_sim::econ::harvest::DockState;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use harvest_common::*;

/// REFN covers (−6 … −4, −1 … 1) from the middle; its docking cell, (3, 1)
/// from the top-left cell (S-0402 claim 10), is (−3, 0), outside the
/// foundation.
fn refinery_cell() -> CellXY {
    at(-6, -1)
}

fn dock_cell() -> CellXY {
    at(-3, 0)
}

fn unblock(w: &mut World) {
    while w.tick <= w.orders_blocked_through {
        testkit::run(w, 1);
    }
}

/// REFN with harvester B holding its only dock (contact and slot) while
/// standing away from the docking cell, and harvester A without contact
/// at `a`.
fn scene(w: &mut World, a: CellXY) -> (ObjectId, ObjectId, ObjectId) {
    let refn = place_building(w, "REFN", 0, refinery_cell());
    let b = place_vehicle(w, "HARV", 0, at(-5, 4), 0);
    w.obj_mut(b).unwrap().contact = Some(refn);
    w.obj_mut(refn).unwrap().econ.dock = Some(DockState { holders: vec![b] });
    let a = place_vehicle(w, "HARV", 0, a, 0);
    (refn, a, b)
}

fn order(w: &mut World, actor: ObjectId, behaviour: Behaviour, dest: Option<CellXY>) {
    assert!(orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor,
            behaviour,
            attack_move: false,
            target: None,
            dest: dest.map(Target::Cell),
        }
    ));
}

// spec: S-0212 claim 27a (another harvester's contact does not reserve the docking cell: an ordinary harvester without contact enters the empty cell outside the foundation and rests there; reaching it gives neither contact nor unloading)
#[test]
fn claim_27a_entry_and_rest_without_contact() {
    let r = ruleset();
    let mut w = game(&r);
    let (refn, a, b) = scene(&mut w, at(-3, 4));
    unblock(&mut w);
    order(&mut w, a, Behaviour::Move, Some(dock_cell()));
    let mut arrived = false;
    for _ in 0..300 {
        testkit::run(&mut w, 1);
        if w.obj(a).unwrap().pos.cell() == dock_cell() && w.obj(a).unwrap().dest.is_none() {
            arrived = true;
            break;
        }
    }
    assert!(arrived, "entered the docking cell");
    for _ in 0..300 {
        testkit::run(&mut w, 1);
        let o = w.obj(a).unwrap();
        assert_eq!(o.pos.cell(), dock_cell(), "not evicted");
        assert_eq!(o.contact, None);
        assert_ne!(o.beh.current, Some(Behaviour::Unload));
    }
    assert_eq!(w.obj(b).unwrap().contact, Some(refn));
    assert_eq!(
        w.obj(refn).unwrap().econ.dock.as_ref().unwrap().holders,
        vec![b]
    );
}

// spec: S-0212 claim 27a (a harvest order from the docking cell finds an ordinary route to its field: the docking role forbids no exit direction and needs no dock ownership)
#[test]
fn claim_27a_departure_without_contact() {
    let r = ruleset();
    let mut ore = Vec::new();
    for dy in 0..3 {
        for dx in 0..3 {
            ore.push((at(4 + dx, -1 + dy), ORE, 5));
        }
    }
    let mut w = game_with_ore(&r, &ore, 7);
    let (refn, a, b) = scene(&mut w, dock_cell());
    unblock(&mut w);
    order(&mut w, a, Behaviour::Harvest, Some(at(5, 0)));
    let mut left = false;
    for _ in 0..400 {
        testkit::run(&mut w, 1);
        let o = w.obj(a).unwrap();
        assert_eq!(o.contact, None, "B's contact is not taken");
        if o.pos.cell() != dock_cell() {
            left = true;
            break;
        }
    }
    assert!(left, "left the docking cell");
    assert_eq!(w.obj(b).unwrap().contact, Some(refn));
}
