//! S-0616 claims 1 and 5 (#302): a smoke-kind system held as an effect
//! weapon's emitter keeps the displacement it had from the firer when it was
//! made at the fire point. Synthetic data: the testkit's combat rules (whose
//! CART has a primary fire offset) plus an invented smoke particle type
//! PUFF, smoke system type PLUME and a fire-flag weapon SMOKER that attaches
//! it; every name and number is made up.

use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType, WeaponType};
use glasnost_sim::combat::fire;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::Target;
use glasnost_sim::{particles, testkit};

// spec: S-0616 claim 5 (a smoke system's position, owner not a building: the owner's position plus the displacement at creation) and claim 1 (effect weapons: made at the fire point, the firer as owner object) (#302)
#[test]
fn smoke_kind_weapon_emitter_keeps_its_creation_displacement() {
    let mut r = testkit::combat_ruleset();
    for (reg, name) in [("particles", "PUFF"), ("particle_systems", "PLUME")] {
        r.registries
            .entry(reg.into())
            .or_default()
            .push(name.into());
    }
    r.sim.particles.insert(
        "PUFF".into(),
        ParticleType {
            behaves_like: Some("smoke".into()),
            max_ec: Some(1000),
            end_state_ai: Some(100),
            state_ai_advance: Some(4),
            ..Default::default()
        },
    );
    r.sim.particle_systems.insert(
        "PLUME".into(),
        ParticleSystemType {
            behaves_like: Some("smoke".into()),
            holds_what: Some("PUFF".into()),
            spawn_frames: Some(4),
            spawn_cutoff: Some(Decimal("100.0".into())),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "SMOKER".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(30),
            range: Some(Decimal("10".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            attached_particle_system: Some("PLUME".into()),
            use_fire_particles: Some(true),
            ..Default::default()
        },
    );
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("SMOKER".into());
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(30000);
    let n = 24;
    let mut w = testkit::game_with(&r, n, 7);
    let a = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, testkit::mid(n, -2, 0), 16384);
    let b = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, testkit::mid(n, 2, 0), 49152);
    let t = Target::Object(b);
    w.obj_mut(a).unwrap().target = Some(t);
    assert!(fire::shot(&mut w, a, 0, t));
    let e = w.obj(a).unwrap().combat.fire_emitter.expect("fire place");
    let fp = glasnost_sim::combat::position::fire_point(&w, a, 0, 0);
    let firer = w.obj(a).unwrap().pos;
    assert_ne!(fp, firer, "the fire point is displaced from the firer");
    assert_eq!(w.gas.systems[&e.system].pos, fp, "made at the fire point");
    // The firer does not move; the system's first turn.
    particles::smoke_turn(&mut w, e.system);
    assert_eq!(w.obj(a).unwrap().pos, firer);
    assert_eq!(
        w.gas.systems[&e.system].pos,
        fp,
        "claim 5: owner position plus the creation displacement; firer at {firer:?}"
    );
}
