//! S-0504 claims 10 and 13 through a detonation (#172): a vessel that sinks
//! after a killing hit keeps 1 health and still takes area damage, so a
//! later detonation can register another killing hit. Synthetic data: an
//! invented self-healing vessel on invented water cells.

mod common;

use common::{Scene, cell, code_tile};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType,
};
use glasnost_sim::combat::area::{self, Detonation, DetonationResult};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::naval;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, World};
use glasnost_sim::{testkit, tick};

const WATER_CODE: u8 = 9;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn land_row(ground: &str, float: &str) -> LandRow {
    LandRow {
        foot: dec(ground),
        track: dec(ground),
        wheel: dec(ground),
        hover: dec("1"),
        float: dec(float),
        amphibious: dec("1"),
        float_beach: dec(float),
        buildable: None,
    }
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.land_types.insert(LandType::Clear, land_row("1", "0"));
    s.land_types.insert(LandType::Road, land_row("1", "0"));
    s.land_types.insert(LandType::Water, land_row("0", "1"));
    let mut t = UnitType {
        strength: Some(300),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Float),
        locomotor: Some(Locomotor::Ship),
        movement_zone: Some(MovementZone::Water),
        weight: dec("4"),
        ..Default::default()
    };
    t.combat.naval = Some(true);
    t.combat.armor = Some(glasnost_rules::sim::ArmourClass::Light);
    t.combat.self_healing = Some(true);
    r.sim.vehicles.insert("SKIFF".into(), t);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("SKIFF".into());
    r
}

fn world() -> World {
    let mut s = Scene::new();
    s.ruleset = rules();
    for dx in -8..=10 {
        for dy in -6..=6 {
            s.set_cell(cell(dx, dy), code_tile(WATER_CODE), 0);
        }
    }
    s.build()
}

fn lethal() -> Hit {
    Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: true,
    }
}

fn he(w: &World) -> u32 {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "HE")
        .unwrap() as u32
}

/// Starts a vessel sinking (by a killing hit, or without one) and lets it
/// descend 30 ticks, self-healing above 1 health.
fn sinking(by_killing_hit: bool) -> (World, ObjectId) {
    let mut w = world();
    let skiff = testkit::spawn(&mut w, Kind::Vehicle, "SKIFF", 0, cell(0, 0), 0);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    if by_killing_hit {
        assert_eq!(damage::apply(&mut w, skiff, lethal()), HitResult::NowDead);
        assert_eq!(w.obj(skiff).unwrap().health, 1);
    } else {
        naval::start_sinking(&mut w, skiff);
        w.obj_mut(skiff).unwrap().health = 100;
    }
    for _ in 0..30 {
        tick::step(&mut w);
    }
    let o = w.obj(skiff).expect("still in play");
    assert!(naval::sinking(&w, skiff));
    assert!(o.on_map);
    assert!(o.health > 1, "self-healed above 1: {}", o.health);
    (w, skiff)
}

fn area_hit(w: &mut World, skiff: ObjectId, damage: i32) -> DetonationResult {
    let point = w.obj(skiff).unwrap().pos;
    let warhead = Some(he(w));
    area::detonate(
        w,
        Detonation {
            point,
            damage,
            warhead,
            source: None,
            house: None,
            resources: false,
        },
    )
}

// spec: S-0504 claim 10 (sinking is not immunity to damage delivered to the object; applicable area damage can damage it)
#[test]
fn area_damage_reaches_a_vessel_sinking_without_a_killing_hit() {
    let (mut w, skiff) = sinking(false);
    let before = w.obj(skiff).unwrap().health;
    assert_eq!(area_hit(&mut w, skiff, 40), DetonationResult::Hit);
    assert!(w.obj(skiff).unwrap().health < before);
}

// spec: S-0504 claim 10, S-0302 claim 8 (No-health targets: a sinking vehicle with health left takes hits by the other rows) and S-0303 claim 9 (a candidate that is alive)
#[test]
fn area_damage_reaches_a_vessel_sinking_after_a_killing_hit() {
    let (mut w, skiff) = sinking(true);
    let before = w.obj(skiff).unwrap().health;
    assert_eq!(area_hit(&mut w, skiff, 40), DetonationResult::Hit);
    assert!(w.obj(skiff).unwrap().health < before);
}

// spec: S-0504 claim 13 and S-0312 claim 32 (another killing hit on a positive-health sinking vessel leaves it at 1 health, still sinking)
#[test]
fn a_detonation_can_kill_a_sinking_vessel_again() {
    let (mut w, skiff) = sinking(true);
    assert_eq!(area_hit(&mut w, skiff, 100_000), DetonationResult::Hit);
    let o = w.obj(skiff).expect("still in play");
    assert_eq!(o.health, 1);
    assert!(naval::sinking(&w, skiff));
}
