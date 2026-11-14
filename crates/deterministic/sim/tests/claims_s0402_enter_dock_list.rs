//! S-0402 Terms and claim 2: an *enter* order starts docking directly only
//! on a refinery, a building of one of the harvester's dock types (#267).
//! Synthetic types from `tests/harvest_common`.

mod harvest_common;

use glasnost_sim::econ::harvest::{self, HarvestState};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use harvest_common::*;

fn enter(w: &mut World, h: ObjectId, b: ObjectId) {
    while w.tick <= w.orders_blocked_through {
        testkit::run(w, 1);
    }
    assert!(orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        }
    ));
}

// spec: S-0402 Terms (a refinery is any building of a dock type of the harvester in question) and claim 2 (an enter order on a refinery starts docking directly): HARV's dock list is REFN only; HUT is a plain building
#[test]
fn issue_267_enter_order_on_a_plain_building_takes_no_refinery_contact() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("HARV").unwrap().econ.dock = Some(vec!["REFN".into()]);
    let mut w = game(&r);
    let hut = place_building(&mut w, "HUT", 0, at(-6, -1));
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 2), 0);
    enter(&mut w, h, hut);
    testkit::run(&mut w, 100);
    let holders = harvest::docked(&w, hut);
    assert!(
        w.obj(h).unwrap().contact != Some(hut) && holders.is_empty(),
        "refinery contact with a building outside the dock list: contact {:?}, holders {:?}, behaviour {:?}",
        w.obj(h).unwrap().contact,
        holders,
        w.obj(h).unwrap().beh.current
    );
}

// spec: S-0402 Terms (a refinery is any building of a dock type of the harvester; only a dock-unload refinery directs a harvester to its docking cell) and claim 2: REF2 has the dock-unload flag but is not in HARV's dock list
#[test]
fn issue_267_enter_order_on_a_dock_unload_building_outside_the_dock_list_does_not_unload() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("HARV").unwrap().econ.dock = Some(vec!["REFN".into()]);
    let mut w = game(&r);
    let ref2 = place_building(&mut w, "REF2", 0, at(-6, -1));
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 2), 0);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load: vec![5, 0],
        ..Default::default()
    });
    let c0 = w.houses[0].credits;
    enter(&mut w, h, ref2);
    testkit::run(&mut w, 900);
    assert!(
        w.houses[0].credits == c0 && harvest::load(&w, h) == vec![5, 0],
        "unloaded at a building outside the dock list: credits {} -> {}, load {:?}",
        c0,
        w.houses[0].credits,
        harvest::load(&w, h)
    );
}

// spec: S-0402 claim 2 (control: an enter order on a building of a dock type still takes contact)
#[test]
fn issue_267_control_enter_order_on_a_dock_type_takes_contact() {
    let r = ruleset();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, at(-6, -1));
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 2), 0);
    enter(&mut w, h, refn);
    testkit::run(&mut w, 30);
    assert_eq!(w.obj(h).unwrap().contact, Some(refn));
    assert_eq!(harvest::docked(&w, refn), vec![h]);
}
