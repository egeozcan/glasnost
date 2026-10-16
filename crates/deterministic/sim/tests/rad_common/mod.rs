//! Shared fixtures of the S-0525 claim tests: a synthetic ruleset with the
//! game-wide radiation values of the spec's measurement scenarios (M = 2,
//! A = 15, C_max = 10000, D_L = D_I = 60, k_L = 0.0625, k_I = 0.5,
//! k_T = 1.0, colour (0, 255, 0)), a radiation weapon of level 1000 whose
//! warhead has cell spread 2.0 and no damage, and probe units. Every name
//! and number is invented; no game data.
#![allow(dead_code)]

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::data::WeaponId;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

pub const SIZE: i32 = 40;

/// The cell X of the scenarios.
pub const X: (i32, i32) = (0, 0);

pub fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

pub fn every(v: &str) -> Option<Vec<Decimal>> {
    Some(vec![d(v); 11])
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

/// An unarmed test infantry type of strength 100000.
pub fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(100_000);
    t.primary = None;
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

/// An unarmed test vehicle type of strength 100000.
pub fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["BUGGY"].clone();
    t.strength = Some(100_000);
    t.primary = None;
    t.combat.armor = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

/// The ruleset: PROBE (infantry), IMMUNE (infantry immune to radiation),
/// VPROBE (a vehicle), the weapon RadGun (level 1000, damage 0, spread 2.0)
/// and the radiation-site warhead SITE (versus 100 % against everything).
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    infantry(&mut r, "PROBE", |_| {});
    infantry(&mut r, "IMMUNE", |t| t.immune_to_radiation = Some(true));
    vehicle(&mut r, "VPROBE", |_| {});
    let g = &mut r.sim.general;
    g.rad_duration_multiple = Some(2);
    g.rad_application_delay = Some(15);
    g.rad_level_max = Some(10_000);
    g.rad_level_delay = Some(60);
    g.rad_light_delay = Some(60);
    g.rad_level_factor = Some(d("0.0625"));
    g.rad_light_factor = Some(d("0.5"));
    g.rad_tint_factor = Some(d("1.0"));
    g.rad_color = Some([0, 255, 0]);
    g.rad_site_warhead = Some("SITE".into());
    r.sim.warheads.insert(
        "SITE".into(),
        WarheadType {
            verses: every("1"),
            radiation: Some(true),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "RADW".into(),
        WarheadType {
            cell_spread: Some(d("2.0")),
            verses: every("0"),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "RadGun".into(),
        WeaponType {
            damage: Some(0),
            rof: Some(10),
            range: Some(d("12")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("RADW".into()),
            rad_level: Some(1000),
            ..Default::default()
        },
    );
    r
}

pub fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 1)
}

pub fn weapon(w: &World, name: &str) -> WeaponId {
    w.rules().combat.weapon_index[name]
}

pub fn probe(w: &mut World, name: &str, c: CellXY) -> ObjectId {
    testkit::spawn(w, Kind::Infantry, name, 1, c, 0)
}

pub fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

/// Runs ticks until the tick number is `tick`.
pub fn run_to(w: &mut World, tick: i32) {
    while w.tick < tick {
        testkit::run(w, 1);
    }
}

/// The health lost by `id` in each of the next `n` ticks, as (tick, loss)
/// pairs for the ticks with a loss.
pub fn losses(w: &mut World, id: ObjectId, n: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for _ in 0..n {
        let before = health(w, id);
        let t = w.tick;
        testkit::run(w, 1);
        let loss = before - health(w, id);
        if loss != 0 {
            out.push((t, loss));
        }
    }
    out
}

/// A world with the base ruleset (`tweak` changes it first) and a shooter
/// of house 0 far from the probes' cells.
pub fn base(tweak: impl FnOnce(&mut Ruleset)) -> (World, ObjectId) {
    let mut r = ruleset();
    tweak(&mut r);
    let second = r
        .sim
        .weapons
        .contains_key("RadGun0")
        .then(|| "RadGun0".to_string());
    infantry(&mut r, "SHOOTER", |t| {
        t.primary = Some("RadGun".into());
        t.secondary = second;
    });
    let mut w = game(&r);
    let s = testkit::spawn(&mut w, Kind::Infantry, "SHOOTER", 0, at(-12, -12), 0);
    (w, s)
}

/// Fires the radiation weapon at a cell, as a shot made now; the inviso
/// bullet detonates in the turn pass of this tick.
pub fn fire_at(w: &mut World, shooter: ObjectId, c: (i32, i32)) {
    fire_slot(w, shooter, 0, c);
}

/// [`fire_at`] with the weapon of a slot.
pub fn fire_slot(w: &mut World, shooter: ObjectId, slot: usize, c: (i32, i32)) {
    let cell = at(c.0, c.1);
    assert!(glasnost_sim::combat::fire::shot(
        w,
        shooter,
        slot,
        Target::Cell(cell)
    ));
}

/// Fires at a cell and runs the tick in which the bullet detonates;
/// returns the tick the shot was fired in, which is the tick of the turn
/// pass that made or topped up the field.
pub fn detonate(w: &mut World, shooter: ObjectId, c: (i32, i32)) -> i32 {
    detonate_slot(w, shooter, 0, c)
}

/// [`detonate`] with the weapon of a slot.
pub fn detonate_slot(w: &mut World, shooter: ObjectId, slot: usize, c: (i32, i32)) -> i32 {
    let t0 = w.tick;
    fire_slot(w, shooter, slot, c);
    while !w.projectiles.is_empty() {
        testkit::run(w, 1);
        assert!(w.tick < t0 + 40, "the bullet did not detonate");
    }
    t0
}
