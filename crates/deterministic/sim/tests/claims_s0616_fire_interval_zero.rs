//! S-0616 claim 20 (#303): a fire-kind system whose spawn interval is 0
//! passes the interval test only at tick 0 (S-0205 claim 3: a rem 0 = a).
//! Synthetic data: the testkit's combat rules plus an invented fire particle
//! type FLAME, fire system type BLAZE and a fire-flag weapon FLAMER that
//! attaches it; every name and number is made up.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType, WeaponType};
use glasnost_sim::combat::fire;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{Target, World};
use glasnost_sim::{particles, testkit};

const N: i32 = 24;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn rules(interval: i32) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    register(&mut r, "particles", "FLAME");
    r.sim.particles.insert(
        "FLAME".into(),
        ParticleType {
            behaves_like: Some("fire".into()),
            max_ec: Some(1000),
            end_state_ai: Some(100),
            state_ai_advance: Some(4),
            velocity: Some(Decimal("6.0".into())),
            ..Default::default()
        },
    );
    register(&mut r, "particle_systems", "BLAZE");
    r.sim.particle_systems.insert(
        "BLAZE".into(),
        ParticleSystemType {
            behaves_like: Some("fire".into()),
            holds_what: Some("FLAME".into()),
            spawn_frames: Some(interval),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "FLAMER".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(30),
            range: Some(Decimal("10".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            attached_particle_system: Some("BLAZE".into()),
            use_fire_particles: Some(true),
            ..Default::default()
        },
    );
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("FLAMER".into());
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(30000);
    r
}

/// The particles a fire emitter with spawn interval `interval` holds after
/// one turn in tick `tick`, with a live owner and no sweep.
fn emitted(interval: i32, tick: i32) -> usize {
    let mut w: World = testkit::game_with(&rules(interval), N, 7);
    let a = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        0,
        testkit::mid(N, -2, 0),
        16384,
    );
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        49152,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    let s = w.obj(a).unwrap().combat.fire_emitter.unwrap().system;
    w.tick = tick;
    particles::fire_turn(&mut w, s);
    w.gas.systems[&s].particles.len()
}

// spec: S-0616 claim 20 (fire spawn interval 0: the interval test passes only at tick 0), claim 11 (emits when the tick is a multiple of the spawn interval) and S-0205 claim 3 (a rem 0 = a) (#303)
#[test]
fn fire_interval_zero_emits_at_tick_zero() {
    assert_eq!(emitted(5, 0), 1, "control: interval 5 emits at tick 0");
    assert_eq!(
        emitted(0, 7),
        0,
        "interval 0, tick 7: 7 rem 0 = 7, no emission"
    );
    assert_eq!(emitted(0, 0), 1, "interval 0, tick 0: 0 rem 0 = 0, emits");
}
