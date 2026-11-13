//! The refinery tether of a harvester ends with its refinery contact
//! (S-0402 claims 10 and 11, glossary *tether link*; S-0213 table T3 (b)):
//! a later depot contact does not make it count as tethered (#185). On
//! synthetic data: invented types and numbers, no game data.

mod harvest_common;

use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::harvest;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{depot, econ, testkit};
use harvest_common::*;

/// The harvest ruleset plus DEPOT, a 3 × 3 repair depot without the pad
/// flag.
fn rules() -> glasnost_rules::schema::Ruleset {
    let mut r = ruleset();
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
            ..Default::default()
        },
    );
    r
}

fn until(w: &mut World, max: i32, mut f: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..max {
        testkit::run(w, 1);
        if f(w) {
            return true;
        }
    }
    false
}

fn enter(w: &mut World, h: ObjectId, b: ObjectId) {
    assert!(orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: Some(Target::Object(b)),
            dest: Some(Target::Object(b)),
        }
    ));
}

/// Docks HARV at REFN through an enter order, then orders it to DEPOT
/// while it is tethered.
fn ordered_to_depot() -> (World, ObjectId, ObjectId) {
    let r = rules();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, at(-6, -1));
    let dep = place_building(&mut w, "DEPOT", 0, at(4, 5));
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 2), 0);
    while w.tick <= w.orders_blocked_through {
        testkit::run(&mut w, 1);
    }
    enter(&mut w, h, refn);
    assert!(until(&mut w, 600, |w| {
        w.obj(h).unwrap().beh.current == Some(Behaviour::Unload)
    }));
    assert_eq!(w.obj(h).unwrap().contact, Some(refn));
    assert!(harvest::state_of(&w, h).tethered);
    assert!(econ::tethered(&w, h), "tethered to the refinery");
    // Damaged, so that the depot's repeated entry request accepts it.
    w.obj_mut(h).unwrap().health = 200;
    enter(&mut w, h, dep);
    assert!(until(&mut w, 3, |w| w.obj(h).unwrap().contact == Some(dep)));
    (w, h, dep)
}

// spec: S-0213 table T3 (b) and S-0402 claim 11 (an order to a harvester tethered to its refinery breaks off the contact and the tether); S-0314 claim 17 (entry to a depot without the pad flag sets no tether)
#[test]
fn order_from_the_dock_to_a_depot_leaves_no_tether() {
    let (w, h, dep) = ordered_to_depot();
    assert_eq!(depot::link(&w, h).map(|l| l.depot), Some(dep));
    assert!(!depot::tethered(&w, h), "the depot link has no tether");
    assert!(
        !econ::tethered(&w, h),
        "the harvester counts as tethered with only an untethered depot contact"
    );
}

// spec: S-0213 claim 8 and S-0314 claim 20 (an untethered vehicle obeys stop: its depot contact broken off, no destination)
#[test]
fn stop_order_on_the_way_to_the_depot_is_obeyed() {
    let (mut w, h, dep) = ordered_to_depot();
    assert!(until(&mut w, 200, |w| {
        let o = w.obj(h).unwrap();
        o.beh.current == Some(Behaviour::Enter) && o.contact == Some(dep)
    }));
    assert!(!depot::tethered(&w, h));
    assert!(orders::issue(&mut w, HouseId(0), Order::Stop { actor: h }));
    testkit::run(&mut w, 2);
    let o = w.obj(h).unwrap();
    assert_eq!(o.contact, None, "stop did not break off the depot contact");
    assert_eq!(o.dest, None, "stop did not clear the destination");
}
