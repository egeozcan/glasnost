//! S-0309 claim 27 against a missile: an anti-air unit that holds a rocket
//! as its target drops it when the rocket is removed at its impact. The
//! lead's play note 36 (M8 A4 round 6, `docs/m8-claims/a4-play-notes.md`).
//! Synthetic data: invented types and numbers.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LocomotionClass, Locomotor, ProjectileType, UnitType, WeaponType,
};
use glasnost_sim::rules::Kind;
use glasnost_sim::spawn;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 40;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push(&mut r, "aircraft", "RKA");
    for n in ["LNA", "VICTIM", "SAMV"] {
        push(&mut r, "vehicles", n);
    }
    let s = &mut r.sim;
    s.aircraft.insert(
        "RKA".into(),
        UnitType {
            strength: Some(400),
            speed: Some(30),
            rot: Some(20),
            missile_spawn: Some(true),
            spawned: Some(true),
            locomotor: Some(Locomotor::Rocket),
            ..Default::default()
        },
    );
    let mut lna = UnitType {
        strength: Some(400),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        primary: Some("Launch".into()),
        missile_spawn: Some(true),
        ..Default::default()
    };
    lna.combat.spawns = Some("RKA".into());
    lna.combat.spawns_number = Some(1);
    lna.combat.spawn_regen_rate = Some(50);
    lna.combat.spawn_reload_rate = Some(0);
    lna.combat.primary_fire_flh = Some([60, 0, 80]);
    s.vehicles.insert("LNA".into(), lna);
    let wheel = |strength: i32| UnitType {
        strength: Some(strength),
        speed: Some(4),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    };
    s.vehicles.insert("VICTIM".into(), wheel(5000));
    let mut samv = wheel(500);
    samv.primary = Some("Sam".into());
    s.vehicles.insert("SAMV".into(), samv);
    s.projectiles.insert(
        "Flak".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(false),
            inviso: Some(true),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Sam".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(10),
            range: dec("12"),
            speed: Some(100),
            projectile: Some("Flak".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Launch".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(20),
            range: dec("30"),
            projectile: Some("Bullet".into()),
            warhead: Some("HE".into()),
            spawner: Some(true),
            ..Default::default()
        },
    );
    let g = &mut s.general;
    g.v3_rocket_type = Some("RKA".into());
    g.v3_rocket_pause_frames = Some(4);
    g.v3_rocket_tilt_frames = Some(10);
    g.v3_rocket_altitude = Some(600);
    g.v3_rocket_pitch_final = dec("0.6");
    g.v3_rocket_acceleration = dec("3");
    g.v3_rocket_damage = Some(30);
    g.v3_rocket_elite_damage = Some(30);
    g.v3_warhead = Some("HE".into());
    g.v3_elite_warhead = Some("HE".into());
    r
}

fn put(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, name, house, testkit::mid(SIZE, dx, dy), 0)
}

// spec: S-0309 claim 27 (r51, Q-2006: every infantry unit, vehicle, building and aircraft that holds an object as its target drops it at the moment the object is removed, inside the removing action; no holder keeps it until its own next turn)
#[test]
fn claim_27_a_removed_missile_is_dropped_by_its_anti_air_holder() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lna = put(&mut w, "LNA", 1, -8, 0);
    let victim = put(&mut w, "VICTIM", 0, 8, 0);
    let samv = put(&mut w, "SAMV", 0, 8, 3);
    let rocket = w.obj(lna).unwrap().spawn.clone().unwrap().slots[0]
        .object
        .unwrap();
    spawn::shot(&mut w, lna, Target::Object(victim));
    let mut engaged = false;
    let mut last_health = 0;
    for _ in 0..400 {
        testkit::run(&mut w, 1);
        let held = w.obj(samv).unwrap().target;
        if let Some(x) = w.obj(rocket) {
            engaged |= held == Some(Target::Object(rocket));
            last_health = x.health;
            continue;
        }
        assert!(engaged, "the anti-air unit took the rocket as its target");
        assert!(
            last_health > 0,
            "the rocket ended at its impact, removed, not destroyed by the anti-air fire"
        );
        assert_ne!(
            held,
            Some(Target::Object(rocket)),
            "tick {}: the removed rocket is still held",
            w.tick
        );
        return;
    }
    panic!("the rocket never reached its target");
}
