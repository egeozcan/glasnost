//! Shared fixtures of the S-0519 claim tests: planted bombs, suicide
//! weapons, charges and air-strike designators on synthetic data. Every
//! name and number is invented; no game data.
#![allow(dead_code)]

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

pub const SIZE: i32 = 40;

pub fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

pub fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

pub fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn weapon(damage: i32, range: &str, warhead: &str) -> WeaponType {
    WeaponType {
        damage: Some(damage),
        rof: Some(40),
        range: Some(d(range)),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some(warhead.into()),
        ..Default::default()
    }
}

fn warhead(f: impl FnOnce(&mut WarheadType)) -> WarheadType {
    let mut h = WarheadType {
        verses: Some(vec![d("1"); 11]),
        ..Default::default()
    };
    f(&mut h);
    h
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
    t.strength = Some(100_000);
    t.primary = None;
    t.combat.armor = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

/// The ruleset. Bomb: damage 100, timed delay 50, warhead BOOM (spread 1.5
/// cells), both under `[CombatDamage]`; charge delay 0.0111 minutes (K = 9);
/// the demolition warhead DEMO. Types: BOMBER (plants, IvanBomb weapon),
/// ENG (disarms), KAMI (suicide weapon, explodes, death weapon Boom), SABO
/// (explosive charges, second weapon with rate 30), VICT (strength 100000),
/// WEAK (strength 150), BLDG (a building of strength 800), HUT (a garrison
/// building).
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.warheads.insert("IVAN".into(), warhead(|h| h.ivan_bomb = Some(true)));
    r.sim.warheads.insert("DISARM".into(), warhead(|h| h.bomb_disarm = Some(true)));
    r.sim.warheads.insert(
        "BOOM".into(),
        warhead(|h| h.cell_spread = Some(d("1.5"))),
    );
    r.sim.warheads.insert("DEMO".into(), warhead(|_| {}));
    r.sim.weapons.insert("PlantW".into(), weapon(0, "20", "IVAN"));
    r.sim.weapons.insert("DisarmW".into(), weapon(0, "20", "DISARM"));
    r.sim.weapons.insert("Boom".into(), weapon(80, "5", "BOOM"));
    let mut suicide = weapon(5, "3", "DEMO");
    suicide.suicide = Some(true);
    r.sim.weapons.insert("SuicideW".into(), suicide);
    let mut slow = weapon(1, "3", "DEMO");
    slow.rof = Some(30);
    r.sim.weapons.insert("SaboGun".into(), slow);
    infantry(&mut r, "BOMBER", |t| {
        t.primary = Some("PlantW".into());
        t.ivan = Some(true);
    });
    infantry(&mut r, "ENG", |t| t.primary = Some("DisarmW".into()));
    infantry(&mut r, "KAMI", |t| {
        t.strength = Some(50);
        t.primary = Some("SuicideW".into());
        t.combat.explodes = Some(true);
        t.combat.death_weapon = Some("Boom".into());
    });
    infantry(&mut r, "SABO", |t| {
        t.c4 = Some(true);
        t.primary = Some("Rifle".into());
        t.secondary = Some("SaboGun".into());
    });
    infantry(&mut r, "PLAIN", |_| {});
    vehicle(&mut r, "VICT", |_| {});
    vehicle(&mut r, "WEAK", |t| t.strength = Some(150));
    register(&mut r, "buildings", "BLDG");
    r.sim.buildings.insert(
        "BLDG".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([1, 1]),
            ..Default::default()
        },
    );
    register(&mut r, "buildings", "HUT");
    r.sim.buildings.insert(
        "HUT".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([1, 1]),
            can_be_occupied: Some(true),
            max_number_occupants: Some(3),
            ..Default::default()
        },
    );
    // The air-strike part: the designation warhead, the strike aircraft JET
    // (normal) and JET2 (elite), the designator DESIG (strike of 2 JET,
    // recharge 100; elite 3 JET2, recharge 50).
    r.sim.warheads.insert("STRIKE".into(), warhead(|h| h.airstrike = Some(true)));
    r.sim.weapons.insert("Laser".into(), weapon(10, "20", "STRIKE"));
    r.sim.weapons.insert("JetGun".into(), weapon(10, "6", "DEMO"));
    for n in ["JET", "JET2"] {
        let mut u = UnitType {
            strength: Some(200),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(glasnost_rules::sim::Locomotor::Fly),
            primary: Some("JetGun".into()),
            ..Default::default()
        };
        u.combat.ammo = Some(2);
        register(&mut r, "aircraft", n);
        r.sim.aircraft.insert(n.into(), u);
    }
    vehicle(&mut r, "DESIG", |t| {
        t.primary = Some("Laser".into());
        t.combat.airstrike_team = Some(2);
        t.combat.elite_airstrike_team = Some(3);
        t.combat.airstrike_team_type = Some("JET".into());
        t.combat.elite_airstrike_team_type = Some("JET2".into());
        t.combat.airstrike_recharge_time = Some(100);
        t.combat.elite_airstrike_recharge_time = Some(50);
    });
    let g = &mut r.sim.general;
    g.ivan_warhead = Some("BOOM".into());
    g.ivan_damage = Some(100);
    g.ivan_timed_delay = Some(50);
    g.ivan_icon_flicker_rate = Some(8);
    g.can_detonate_time_bomb = Some(true);
    g.c4_warhead = Some("DEMO".into());
    g.c4_delay = Some(d("0.0111"));
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

/// An armed unit that does nothing by itself.
pub fn quiet(w: &mut World, kind: Kind, name: &str, house: u8, c: CellXY) -> ObjectId {
    let id = spawn(w, kind, name, house, c);
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(id) {
        o.beh.force(glasnost_sim::rules::Behaviour::Sleep, tick, stale);
    }
    id
}

/// Makes a shot of slot 0 at an object and runs until the projectile is gone.
pub fn shoot(w: &mut World, firer: ObjectId, target: ObjectId) -> bool {
    let made = glasnost_sim::combat::fire::shot(w, firer, 0, Target::Object(target));
    let t0 = w.tick;
    while !w.projectiles.is_empty() && w.tick < t0 + 40 {
        testkit::run(w, 1);
    }
    made
}

/// Sets the designator's target (the order a player would give) and fires
/// one designation shot at the object, running until it has detonated.
pub fn designate(w: &mut World, d: ObjectId, x: ObjectId) {
    if let Some(o) = w.obj_mut(d) {
        o.target = Some(Target::Object(x));
    }
    shoot(w, d, x);
}

/// The aircraft linked to the designator's strike.
pub fn linked(w: &World, d: ObjectId) -> Vec<ObjectId> {
    w.strike.strikes.get(&d).map(|s| s.aircraft.clone()).unwrap_or_default()
}
