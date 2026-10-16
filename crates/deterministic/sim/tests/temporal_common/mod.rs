//! Shared fixtures of the S-0514 claim tests: a synthetic ruleset with
//! erasing infantry (ERASER with raw damage 30, ERASER2 with 20), target
//! vehicles of strength 100, a gunner transport and an open-topped one.
//! Every name and number is invented; no game data.
#![allow(dead_code)]

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind;
use glasnost_sim::temporal;
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

pub fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(100_000);
    t.primary = None;
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

pub fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["BUGGY"].clone();
    t.strength = Some(100);
    t.primary = None;
    t.combat.armor = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

fn chrono(damage: i32) -> WeaponType {
    WeaponType {
        damage: Some(damage),
        rof: Some(40),
        range: Some(d("40")),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some("TEMP".into()),
        ..Default::default()
    }
}

/// The ruleset: ERASER (raw damage 30, trainable, cost 1000), ERASER2 (20),
/// VICT (a vehicle of strength 100, cost 500), the transports GUNNER (a
/// gunner transport for 2) and OPEN (open-topped, for 2), the animations
/// WARP (the warp-away) and SPARK (the sparkle) and the warp distance 3.
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.warheads.insert(
        "TEMP".into(),
        WarheadType {
            temporal: Some(true),
            verses: Some(vec![d("0.1"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert("Chrono".into(), chrono(30));
    r.sim.weapons.insert("Chrono2".into(), chrono(20));
    infantry(&mut r, "ERASER", |t| {
        t.primary = Some("Chrono".into());
        t.combat.trainable = Some(true);
        t.cost = Some(1000);
    });
    infantry(&mut r, "ERASER2", |t| t.primary = Some("Chrono2".into()));
    infantry(&mut r, "BYSTANDER", |_| {});
    vehicle(&mut r, "VICT", |t| t.cost = Some(500));
    vehicle(&mut r, "GUNNER", |t| {
        t.passengers = Some(2);
        t.size_limit = Some(d("2"));
        t.gunner = Some(true);
    });
    vehicle(&mut r, "OPEN", |t| {
        t.passengers = Some(2);
        t.size_limit = Some(d("2"));
        t.open_topped = Some(true);
    });
    for n in ["WARP", "SPARK"] {
        register(&mut r, "animations", n);
        r.sim.animations.insert(
            n.into(),
            AnimationType {
                rate: Some(450),
                end: Some(3),
                start: Some(0),
                ..Default::default()
            },
        );
    }
    let g = &mut r.sim.general;
    g.warp_away = Some("WARP".into());
    g.chrono_sparkle = Some("SPARK".into());
    g.open_topped_warp_distance = Some(3);
    g.veteran_ratio = Some(d("4"));
    g.veteran_cap = Some(d("2.5"));
    r
}

pub fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 1)
}

/// A world with the ruleset (`tweak` changes it first).
pub fn world(tweak: impl FnOnce(&mut Ruleset)) -> World {
    let mut r = ruleset();
    tweak(&mut r);
    game(&r)
}

pub fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, c: CellXY) -> ObjectId {
    testkit::spawn(w, kind, name, house, c, 0)
}

/// An armed unit that does nothing by itself (behaviour *sleep*), so that
/// only the shots a test makes act.
pub fn attacker(w: &mut World, name: &str, house: u8, c: CellXY) -> ObjectId {
    let id = testkit::spawn(w, Kind::Infantry, name, house, c, 0);
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(id) {
        o.beh
            .force(glasnost_sim::rules::Behaviour::Sleep, tick, stale);
    }
    id
}

pub fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

pub fn run_to(w: &mut World, tick: i32) {
    while w.tick < tick {
        testkit::run(w, 1);
    }
}

/// Fires the shooter's weapon of `slot` at an object and runs the tick in
/// which the bullet detonates; returns the tick of the shot.
pub fn erase(w: &mut World, shooter: ObjectId, target: ObjectId) -> i32 {
    let t0 = w.tick;
    assert!(glasnost_sim::combat::fire::shot(
        w,
        shooter,
        0,
        Target::Object(target)
    ));
    while !w.projectiles.is_empty() {
        testkit::run(w, 1);
        assert!(w.tick < t0 + 40, "the bullet did not detonate");
    }
    // The attacker keeps the target it fired at, as in play (falling back to rest would let go).
    if temporal::holds(w, shooter, target) {
        let tick = w.tick;
        let stale = w.rules().quirks.stale_suspended_revert;
        if let Some(o) = w.obj_mut(shooter) {
            o.target = Some(Target::Object(target));
            o.beh
                .force(glasnost_sim::rules::Behaviour::Attack, tick, stale);
        }
    }
    t0
}

/// Fires at a cell and runs the tick of the detonation.
pub fn erase_cell(w: &mut World, shooter: ObjectId, c: CellXY) {
    let t0 = w.tick;
    assert!(glasnost_sim::combat::fire::shot(
        w,
        shooter,
        0,
        Target::Cell(c)
    ));
    while !w.projectiles.is_empty() {
        testkit::run(w, 1);
        assert!(w.tick < t0 + 40);
    }
}

pub fn amount(w: &World, target: ObjectId) -> Option<i32> {
    w.temporal.locks.get(&target).map(|l| l.amount)
}

pub fn erased(w: &World, id: ObjectId) -> bool {
    temporal::erased(w, id)
}

/// The sparkle and warp-away animations among the effects, by name and tick.
pub fn anims(w: &World) -> Vec<String> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect()
}
