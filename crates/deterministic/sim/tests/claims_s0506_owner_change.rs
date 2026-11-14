//! Claim tests for S-0506 claim 13 (#206): a spawner's controller clears its
//! slots when the spawner changes house, on the two paths that give a mobile
//! spawner a new owner besides mind control: a contact that follows a
//! captured building (S-0509 claim 14) and trigger action 14 (S-0309 claim
//! 30). Built on the synthetic rules of `structures_common` with an invented
//! launcher LNA and rocket RKA: invented types and numbers, no game data.

mod structures_common;

use glasnost_map::triggers::{Action, Event, TagDef, TriggerDef, Triggers};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{LocomotionClass, Locomotor, UnitType, WeaponType};
use glasnost_sim::rules::Kind;
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use structures_common::*;

/// [`ruleset`] with a rocket RKA and a driving launcher LNA with one RKA
/// slot and the spawner weapon Launch.
fn rules() -> Ruleset {
    let mut r = ruleset();
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("RKA".into());
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("LNA".into());
    r.sim.aircraft.insert(
        "RKA".into(),
        UnitType {
            strength: Some(40),
            speed: Some(50),
            rot: Some(20),
            missile_spawn: Some(true),
            spawned: Some(true),
            locomotor: Some(Locomotor::Rocket),
            ..Default::default()
        },
    );
    let mut t = UnitType {
        strength: Some(400),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        primary: Some("Launch".into()),
        missile_spawn: Some(true),
        ..Default::default()
    };
    t.combat.spawns = Some("RKA".into());
    t.combat.spawns_number = Some(1);
    t.combat.spawn_regen_rate = Some(50);
    t.combat.spawn_reload_rate = Some(0);
    r.sim.vehicles.insert("LNA".into(), t);
    r.sim.weapons.insert(
        "Launch".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(20),
            range: Some(d("30")),
            projectile: Some("Bullet".into()),
            warhead: Some("HE".into()),
            spawner: Some(true),
            ..Default::default()
        },
    );
    r
}

/// The docked object of the spawner's first slot.
fn docked(w: &World, spawner: ObjectId) -> ObjectId {
    w.obj(spawner).unwrap().spawn.clone().unwrap().slots[0]
        .object
        .expect("a docked slot object")
}

// spec: S-0506 claim 13 (the controller clears its slots when the spawner changes house: a docked slot object is removed silently); S-0509 claim 14 (a staying contact near the docking point changes owner together with the building)
#[test]
fn a_spawner_that_follows_a_captured_building_clears_its_slots() {
    let mut w = game_on(&rules(), triggers());
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    let rp = w.reference_point(b);
    let lna = testkit::spawn(&mut w, Kind::Vehicle, "LNA", 1, at(0, 0), 0);
    w.obj_mut(lna).unwrap().pos = rp;
    w.obj_mut(lna).unwrap().contact = Some(b);
    let old = docked(&w, lna);
    assert_eq!(owner(&w, old), Some(B));
    ownership::change(&mut w, b, Some(A), Cause::Capture);
    assert_eq!(owner(&w, lna), Some(A), "the contact follows the building");
    assert!(
        w.obj(old).is_none(),
        "the old house's docked slot object is removed (owner {:?})",
        owner(&w, old)
    );
}

/// A map whose tag `TAG` changes its carriers' owner to Beta (house value
/// 1) when event 8 fires.
fn change_owner_triggers() -> Triggers {
    let t = TriggerDef {
        events: vec![Event {
            code: 8,
            kind: 0,
            value: 0,
            type_name: None,
        }],
        actions: vec![Action {
            code: 14,
            number: 1,
            ..Default::default()
        }],
        ..TriggerDef::created(b"CHOWN".to_vec())
    };
    Triggers {
        triggers: vec![t],
        tags: vec![TagDef {
            id: b"TAG".to_vec(),
            repeat: 0,
            name: Vec::new(),
            first: Some(0),
            chain: vec![0],
        }],
        ..Default::default()
    }
}

// spec: S-0506 claim 13 (the controller clears its slots when the spawner changes house; after a house change, replacements for previously occupied slots appear for the new house at the next pass); S-0309 claim 30 (a trigger's change-owner action changes the owner) and S-0011 claim 24
#[test]
fn trigger_change_owner_clears_a_mobile_spawners_slots() {
    let mut w = game_on(&rules(), change_owner_triggers());
    let lna = put(&mut w, Kind::Vehicle, "LNA", 0, 0, 0);
    tag(&mut w, lna, "TAG");
    let old = docked(&w, lna);
    assert_eq!(owner(&w, old), Some(A));
    let tag_id = *w.triggers.objects.get(&lna).unwrap();
    glasnost_sim::triggers::raise(
        &mut w,
        tag_id,
        glasnost_sim::triggers::Raise {
            code: Some(8),
            causer: None,
            carrier: Some(glasnost_sim::triggers::Carrier::Object(lna)),
        },
    );
    assert_eq!(owner(&w, lna), Some(B), "the spawner changed house");
    assert!(
        w.obj(old).is_none(),
        "the old house's docked slot object is removed (owner {:?})",
        owner(&w, old)
    );
    // spec: S-0506 claim 13 (the replacement appears for the new house)
    testkit::run(&mut w, 60);
    let now = w.obj(lna).unwrap().spawn.clone().unwrap().slots[0].object;
    assert_eq!(now.and_then(|x| owner(&w, x)), Some(B));
}
