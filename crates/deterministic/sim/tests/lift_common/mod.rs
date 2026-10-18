//! Shared fixtures of the magnetic-lift claim tests (S-0607): a synthetic
//! ruleset with a magnetic lifter vehicle (LIFTER, a magnetic-beam weapon
//! whose warhead seizes with the jumpjet locomotor), victim vehicles of
//! strength 100 (VICT, HEAVY of size 10, CRASHER that can crash), a game-wide
//! death weapon and crushing and demolition warheads. Every name and number
//! is invented; no game data.
#![allow(dead_code)]

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, UnitType, WarheadType, WeaponType};
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

pub fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["BUGGY"].clone();
    t.strength = Some(100);
    t.primary = None;
    t.combat.armor = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

pub fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(100);
    t.primary = None;
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

fn warhead(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut WarheadType)) {
    let mut h = WarheadType {
        verses: Some(vec![d("1.0"); 11]),
        ..Default::default()
    };
    f(&mut h);
    r.sim.warheads.insert(name.into(), h);
}

/// The ruleset described in the module documentation.
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "MAG", |h| {
        h.is_locomotor = Some(true);
        h.locomotor = Some("jumpjet".into());
    });
    // The impact warheads: a small spread, full damage at the centre.
    warhead(&mut r, "CRUSH", |h| h.cell_spread = Some(d("0.5")));
    warhead(&mut r, "C4", |h| h.cell_spread = Some(d("0.5")));
    r.sim.weapons.insert(
        "MagBeam".into(),
        WeaponType {
            damage: Some(10),
            rof: Some(20),
            range: Some(d("12")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MAG".into()),
            is_mag_beam: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Doom".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(20),
            range: Some(d("1")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("C4".into()),
            ..Default::default()
        },
    );
    vehicle(&mut r, "LIFTER", |t| {
        t.strength = Some(1000);
        t.primary = Some("MagBeam".into());
    });
    vehicle(&mut r, "VICT", |_| {});
    vehicle(&mut r, "HEAVY", |t| t.size = Some(d("10")));
    vehicle(&mut r, "CRASHER", |t| t.crashable = Some(true));
    infantry(&mut r, "TROOPER", |_| {});
    let g = &mut r.sim.general;
    g.c4_warhead = Some("C4".into());
    g.crush_warhead = Some("CRUSH".into());
    g.death_weapon = Some("Doom".into());
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

/// A lifter that keeps attacking `target` (behaviour *attack*), as in play.
pub fn attack(w: &mut World, lifter: ObjectId, target: ObjectId) {
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(lifter) {
        o.target = Some(Target::Object(target));
        o.beh.force(Behaviour::Attack, tick, stale);
    }
}

/// Fires the lifter's beam at `target` and runs until the shot has
/// detonated; the lifter then keeps attacking the target.
pub fn beam(w: &mut World, lifter: ObjectId, target: ObjectId) {
    let t0 = w.tick;
    attack(w, lifter, target);
    assert!(glasnost_sim::combat::fire::shot(
        w,
        lifter,
        0,
        Target::Object(target)
    ));
    while !w.projectiles.is_empty() {
        testkit::run(w, 1);
        assert!(w.tick < t0 + 40, "the beam did not detonate");
    }
    attack(w, lifter, target);
}

pub fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

pub fn height(w: &World, id: ObjectId) -> i32 {
    w.height_above_surface(id)
}

pub fn run(w: &mut World, n: i32) {
    testkit::run(w, n);
}

/// Runs until `f` holds, at most `limit` ticks; whether it did.
pub fn run_until(w: &mut World, limit: i32, mut f: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..limit {
        if f(w) {
            return true;
        }
        testkit::run(w, 1);
    }
    f(w)
}

/// Makes `cells` water through a synthetic water tile, so that a cell
/// recompute keeps it water (S-0211 claim 5).
pub fn water(w: &mut World, cells: &[CellXY]) {
    let mut cat = (*testkit::catalogue()).clone();
    let first = cat.tiles.len() as i32;
    let mut tile = cat.tiles[0].clone();
    if let Some(Some(c)) = tile.cells.first_mut() {
        c.terrain_code = 9; // the synthetic catalogue's water terrain code
    }
    cat.tiles.push(tile);
    cat.allow_tiberium.push(false);
    w.catalogue = Some(std::sync::Arc::new(cat));
    for &c in cells {
        w.cells.grid.get_mut(c).unwrap().tile = first;
    }
    glasnost_sim::load::recalc_cells(w, cells);
}

/// A player's move order to `c` from the actor's owner.
pub fn order_move(w: &mut World, actor: ObjectId, c: CellXY) -> bool {
    let Some(house) = w.obj(actor).and_then(|o| o.owner) else {
        return false;
    };
    let order = glasnost_sim::orders::Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    };
    glasnost_sim::orders::issue(w, house, order)
}

/// Waits for a set-down seized unit to touch down, then gives it move
/// orders (S-0607 scenario L11 part a) until a fall-back to rest restores
/// its own locomotor (S-0607 claim 20); whether it did.
pub fn restored(w: &mut World, id: ObjectId) -> bool {
    if !run_until(w, 400, |w| {
        !glasnost_sim::lift::dropped(w, id)
            && w.height_above_surface(id) == 0
            && !glasnost_sim::jump::moving(w, id)
    }) {
        return false;
    }
    for _ in 0..10 {
        let c = w.obj(id).map(|o| o.pos.cell());
        if let Some(c) = c {
            order_move(w, id, CellXY { x: c.x + 2, y: c.y });
        }
        if run_until(w, 20, |w| !glasnost_sim::lift::seized(w, id)) {
            return true;
        }
    }
    false
}
