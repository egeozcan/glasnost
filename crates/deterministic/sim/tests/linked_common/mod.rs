//! Shared fixtures of the S-0518 claim tests: a prism building type PRISM
//! (primary weapon 100 damage, secondary weapon of range 8 cells, delayed
//! fire 5 ticks), a plain tower of the same weapons, an overpowerable coil,
//! an infantry unit with an electric-charge secondary and a tough target
//! vehicle. Every name and number is invented; no game data.
#![allow(dead_code)]

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

pub const SIZE: i32 = 40;

pub fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

pub fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn weapon(damage: i32, range: &str, warhead: &str) -> WeaponType {
    WeaponType {
        damage: Some(damage),
        rof: Some(60),
        range: Some(d(range)),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some(warhead.into()),
        ..Default::default()
    }
}

fn building(primary: &str, secondary: &str, delay: i32) -> BuildingType {
    BuildingType {
        strength: Some(800),
        foundation: Some([1, 1]),
        primary: Some(primary.into()),
        secondary: Some(secondary.into()),
        delayed_fire_delay: Some(delay),
        ..Default::default()
    }
}

pub fn building_type(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut b = building("PrismGun", "PrismReach", 5);
    f(&mut b);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), b);
}

/// The ruleset. General: prism type PRISM, modifier 50 %, cap 3, rearm 100,
/// duration 15. Target VICT has strength 100000 and is hit for the weapon's
/// damage at versus 1.
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.warheads.insert(
        "PW".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "ELEC".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            electric_assault: Some(true),
            ..Default::default()
        },
    );
    r.sim
        .weapons
        .insert("PrismGun".into(), weapon(100, "20", "PW"));
    r.sim
        .weapons
        .insert("PrismReach".into(), weapon(1, "8", "PW"));
    r.sim
        .weapons
        .insert("CoilLow".into(), weapon(7, "20", "PW"));
    r.sim
        .weapons
        .insert("CoilHigh".into(), weapon(13, "20", "PW"));
    r.sim.weapons.insert("Zap".into(), weapon(0, "6", "ELEC"));
    building_type(&mut r, "PRISM", |_| {});
    building_type(&mut r, "TOWER2", |_| {});
    building_type(&mut r, "COIL", |b| {
        b.primary = Some("CoilLow".into());
        b.secondary = Some("CoilHigh".into());
        b.overpowerable = Some(true);
    });
    let mut v = r.sim.vehicles["BUGGY"].clone();
    v.strength = Some(100_000);
    v.primary = None;
    v.combat.armor = None;
    register(&mut r, "vehicles", "VICT");
    r.sim.vehicles.insert("VICT".into(), v);
    let mut zapper = r.sim.vehicles["BUGGY"].clone();
    zapper.strength = Some(100_000);
    zapper.primary = Some("CoilLow".into());
    zapper.secondary = Some("Zap".into());
    register(&mut r, "vehicles", "ZAPCAR");
    r.sim.vehicles.insert("ZAPCAR".into(), zapper);
    let mut c = r.sim.infantry["WALKER"].clone();
    c.strength = Some(100_000);
    c.primary = Some("CoilLow".into());
    c.secondary = Some("Zap".into());
    register(&mut r, "infantry", "CHARGER");
    r.sim.infantry.insert("CHARGER".into(), c);
    let g = &mut r.sim.general;
    g.prism_type = Some("PRISM".into());
    g.prism_support_modifier = Some(50);
    g.prism_support_max = Some(3);
    g.prism_support_delay = Some(100);
    g.prism_support_duration = Some(15);
    r
}

pub fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 1)
}

pub fn world(tweak: impl FnOnce(&mut Ruleset)) -> World {
    let mut r = ruleset();
    tweak(&mut r);
    game(&r)
}

pub fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, c: CellXY) -> ObjectId {
    testkit::spawn(w, kind, name, house, c, 0)
}

pub fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

pub fn run_to(w: &mut World, tick: i32) {
    while w.tick < tick {
        testkit::run(w, 1);
    }
}

/// Gives a building the target and the *attack* behaviour.
pub fn attack(w: &mut World, id: ObjectId, target: ObjectId) {
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(id) {
        o.target = Some(Target::Object(target));
        o.beh.force(Behaviour::Attack, tick, stale);
    }
}

pub fn support(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(-1, |o| o.combat.support)
}

/// The ticks of the support beams drawn during the next `n` ticks, with
/// their endpoints.
pub fn beams(
    w: &mut World,
    n: i32,
) -> Vec<(i32, glasnost_sim::world::Pos, glasnost_sim::world::Pos)> {
    let mut out = Vec::new();
    for _ in 0..n {
        w.effects.clear();
        let t = w.tick;
        testkit::run(w, 1);
        for e in &w.effects {
            if let Effect::SupportBeam { from, to, .. } = e {
                out.push((t, *from, *to));
            }
        }
    }
    out
}

pub fn unit_type(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut v = r.sim.vehicles["VICT"].clone();
    f(&mut v);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), v);
}
