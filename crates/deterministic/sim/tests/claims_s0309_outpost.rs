//! A captured turret building whose weapon hits air and ground targets
//! engages an enemy aircraft flying through its range (S-0309 claims 5 and
//! 9; S-0308; S-0509 claim 12) (#431, first step). On synthetic data:
//! invented types, weapons and numbers, no game data.

mod econ_common;

use econ_common::{SIZE, dec, game};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    ArmourClass, BuildingType, Locomotor, ProjectileType, UnitType, WeaponType,
};
use glasnost_sim::air;
use glasnost_sim::combat::acquire;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// The economy ruleset with the combat ruleset's weapons, plus projectile
/// `Both` (anti-air and anti-ground), weapon `Duo` (range 8 cells),
/// building `POST` (2 × 2, a turret, primary `Duo`, capturable) and
/// aircraft `KITE` (an unarmed landable aircraft).
fn rules() -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    let s = &mut r.sim;
    s.projectiles.insert(
        "Both".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(true),
            rot: Some(20),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Duo".into(),
        WeaponType {
            damage: Some(5),
            rof: Some(30),
            range: dec("8"),
            speed: Some(60),
            projectile: Some("Both".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    let mut post = BuildingType {
        strength: Some(900),
        foundation: Some([2, 2]),
        primary: Some("Duo".into()),
        turret: Some(true),
        rot: Some(10),
        capturable: Some(true),
        ..Default::default()
    };
    post.combat.armor = Some(ArmourClass::Concrete);
    post.combat.primary_fire_flh = Some([60, 0, 200]);
    push(&mut r, "buildings", "POST");
    r.sim.buildings.insert("POST".into(), post);
    let mut kite = UnitType {
        strength: Some(5000),
        speed: Some(20),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        ..Default::default()
    };
    kite.econ.landable = Some(true);
    push(&mut r, "aircraft", "KITE");
    r.sim.aircraft.insert("KITE".into(), kite);
    r
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

/// What a POST did while an enemy KITE flew across it: the first tick it
/// held the aircraft as its target, the first tick it fired at it, and
/// whether the aircraft was hit.
#[derive(Debug, Default)]
struct Pass {
    aircraft_class: bool,
    first_target: Option<i32>,
    first_shot: Option<i32>,
    damaged: bool,
}

/// A POST of house 2 (or of house 0 when `own`), captured by house 0
/// unless `own`, and a KITE of house 1 flying from 14 cells west to 14
/// cells east of it.
fn pass(own: bool) -> Pass {
    let mut w = game(&rules());
    let s = put(
        &mut w,
        Kind::Building,
        "POST",
        if own { 0 } else { 2 },
        0,
        0,
    );
    if !own {
        ownership::change(&mut w, s, Some(HouseId(0)), Cause::Capture);
        assert_eq!(w.obj(s).unwrap().owner, Some(HouseId(0)), "captured");
    }
    let k = put(&mut w, Kind::Aircraft, "KITE", 1, -14, 0);
    let full = w.obj(k).unwrap().health;
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    assert!(orders::issue(
        &mut w,
        HouseId(1),
        Order::Behaviour {
            actor: k,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(testkit::mid(SIZE, 14, 0))),
        }
    ));
    let mut p = Pass {
        aircraft_class: acquire::classes(&w, s).aircraft,
        ..Default::default()
    };
    let mut last = w.obj(s).unwrap().last_fired;
    for _ in 0..500 {
        glasnost_sim::tick::step(&mut w);
        let so = w.obj(s).unwrap();
        if p.first_target.is_none()
            && so.target == Some(Target::Object(k))
            && !air::on_ground(&w, k)
        {
            p.first_target = Some(w.tick);
        }
        if so.last_fired != last && so.target == Some(Target::Object(k)) {
            p.first_shot.get_or_insert(so.last_fired);
        }
        last = so.last_fired;
        if w.obj(k).is_some_and(|o| o.health < full) {
            p.damaged = true;
        }
    }
    p
}

// spec: S-0309 claims 5 and 9 (a turret building whose slot-0 projectile hits air takes the aircraft class, and its range search scans the tracked flying objects: it acquires and fires at an enemy aircraft flying through its range)
#[test]
fn turret_building_with_air_and_ground_weapon_engages_an_aircraft() {
    let p = pass(true);
    assert!(p.aircraft_class, "{p:?}");
    assert!(
        p.first_target.is_some(),
        "acquires the aircraft in flight: {p:?}"
    );
    assert!(p.first_shot.is_some(), "fires at it: {p:?}");
    assert!(p.damaged, "hits it: {p:?}");
}

// spec: S-0509 claim 12 (a captured building's search follows its new owner) and S-0309 claims 5 and 9
#[test]
fn captured_turret_building_engages_an_aircraft() {
    let p = pass(false);
    assert!(p.aircraft_class, "{p:?}");
    assert!(
        p.first_target.is_some(),
        "acquires the aircraft in flight: {p:?}"
    );
    assert!(p.first_shot.is_some(), "fires at it: {p:?}");
    assert!(p.damaged, "hits it: {p:?}");
}
