//! The factory additions of the jumpjet fixture (S-0417 claim 15, S-0207
//! claim 27): buildings VF (a vehicle factory), BAR (a barracks with the
//! first door kind), PLAIN (an infantry factory with neither), REFN (a
//! dock-unload refinery) and vehicles JVQUICK (a jumpjet whose climb passes
//! its cruise height in one step), TELE (a teleporting vehicle) and
//! TELEDRIVE (a driving vehicle with the teleport-capable flag). Every name
//! and number is invented.
#![allow(dead_code)]

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, MovementZone};

use super::d;

pub fn ruleset() -> Ruleset {
    let mut r = super::ruleset();
    // A vehicle factory with a 3 x 2 foundation and an invented door time, and a barracks with the first door kind.
    let mut vf = BuildingType {
        strength: Some(500),
        foundation: Some([3, 2]),
        factory: Some("vehicle".into()),
        weapons_factory: Some(true),
        ..Default::default()
    };
    vf.deploy_time = Some(d("0.01"));
    vf.combat.armor = None;
    let mut bar = BuildingType {
        strength: Some(500),
        foundation: Some([2, 2]),
        factory: Some("infantry".into()),
        ..Default::default()
    };
    bar.econ.door_first = Some(true);
    bar.combat.armor = None;
    // A building with neither door kind nor the vehicle factory flag.
    let mut plain = BuildingType {
        strength: Some(500),
        foundation: Some([2, 2]),
        factory: Some("infantry".into()),
        ..Default::default()
    };
    plain.combat.armor = None;
    // A refinery that permits dock unloading.
    let mut refn = BuildingType {
        strength: Some(500),
        foundation: Some([3, 3]),
        ..Default::default()
    };
    refn.econ.refinery = Some(true);
    refn.econ.dock_unload = Some(true);
    refn.combat.armor = None;
    let b = r.registries.get_mut("buildings").unwrap();
    b.extend(["VF", "BAR", "PLAIN", "REFN"].map(String::from));
    r.sim.buildings.insert("VF".into(), vf);
    r.sim.buildings.insert("BAR".into(), bar);
    r.sim.buildings.insert("PLAIN".into(), plain);
    r.sim.buildings.insert("REFN".into(), refn);
    // A jumpjet vehicle whose cruise height is tiny and whose climb passes it in one step (the skipped window).
    let mut quick_v = r.sim.vehicles.get("JV").unwrap().clone();
    quick_v.jumpjet_height = Some(4);
    quick_v.jumpjet_climb = Some(d("60.0"));
    r.sim.vehicles.insert("JVQUICK".into(), quick_v);
    // A teleporting vehicle and a driving vehicle that carries the teleport-capable flag.
    let mut tele = r.sim.vehicles.get("JV").unwrap().clone();
    tele.jumpjet = None;
    tele.locomotor = Some(Locomotor::Teleport);
    tele.locomotion_class = Some(LocomotionClass::Wheel);
    tele.movement_zone = Some(MovementZone::Normal);
    tele.teleporter = Some(true);
    tele.rot = Some(5);
    r.sim.vehicles.insert("TELE".into(), tele.clone());
    tele.locomotor = Some(Locomotor::Drive);
    r.sim.vehicles.insert("TELEDRIVE".into(), tele);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .extend(["JVQUICK", "TELE", "TELEDRIVE"].map(String::from));
    r
}
