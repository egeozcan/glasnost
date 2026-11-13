//! Claim test for S-0514 claim 3 (#214): the contact refusal names the
//! vehicle factory, so a vehicle standing on a repair depot it is in
//! contact with is locked. Synthetic data (`temporal_common`); every name
//! and number is invented.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use glasnost_rules::sim::{BuildingType, Decimal};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, Target};
use temporal_common::*;

// spec: S-0514 claim 3 (the refusal names the vehicle factory the vehicle is in contact with; a repair depot is not one)
#[test]
fn a_vehicle_on_a_repair_depot_is_locked() {
    let mut w = world(|r| {
        r.registries
            .entry("buildings".into())
            .or_default()
            .push("DEPOT".into());
        r.sim.buildings.insert(
            "DEPOT".into(),
            BuildingType {
                strength: Some(800),
                foundation: Some([3, 3]),
                unit_repair: Some(true),
                number_impassable_rows: Some(0),
                cost: Some(800),
                ..Default::default()
            },
        );
        r.sim.general.repair_step = Some(1);
        r.sim.general.repair_percent = Some(Decimal("25%".into()));
    });
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    let depot = spawn(&mut w, Kind::Building, "DEPOT", 1, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 1));
    let other = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(10, 10));
    w.obj_mut(v).unwrap().health = 40;
    while w.tick < 4 {
        tick::step(&mut w);
    }
    assert!(orders::issue(
        &mut w,
        HouseId(1),
        Order::Behaviour {
            actor: v,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(depot)),
        }
    ));
    let mut arrived = false;
    for _ in 0..600 {
        tick::step(&mut w);
        let o = w.obj(v).unwrap();
        if o.contact == Some(depot)
            && o.pos.cell() == at(1, 1)
            && w.obj(depot).unwrap().beh.current == Some(Behaviour::Repair)
        {
            arrived = true;
            break;
        }
    }
    assert!(
        arrived,
        "the vehicle stands on the depot, in contact, under repair"
    );
    assert!(
        !glasnost_sim::econ::release::vehicle_factory(&w, depot),
        "the depot is not a vehicle factory"
    );
    assert!(
        !glasnost_sim::superweapons::protected(&w, v),
        "no curtain or shield"
    );
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    erase(&mut w, a, v);
    let o = w.obj(v).unwrap();
    let still_on = o.contact == Some(depot) && o.pos.cell() == at(1, 1);
    let on_depot_locked = erased(&w, v);
    // Control: the same type away from any building is locked by the same firer.
    erase(&mut w, a, other);
    assert!(
        erased(&w, other),
        "control: the same type elsewhere is locked"
    );
    assert!(still_on, "the vehicle was still on the depot at the hit");
    assert!(
        on_depot_locked,
        "S-0514 claim 3: no refusal holds for a vehicle on a repair depot, the lock attaches"
    );
}
