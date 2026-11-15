//! S-0616 claim 1 (#301): a particle system made by the trigger action that
//! creates one has the waypoint cell's centre as its target point, so a
//! railgun-kind system so made has a line of length 0 (claim 17, F12) and
//! makes no particle and takes no ranged draw. Synthetic rules: a
//! railgun-kind particle type LINE and a railgun-kind system type RAIL that
//! holds it; every name and number is made up.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType};
use glasnost_sim::{emitters, gas, testkit};

const N: i32 = 24;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

// spec: S-0616 claim 1 (trigger action row: target point the waypoint cell's centre, at ground surface height), claim 17 and F12 (n = trunc(L × p); n ≤ 0: no particle and no ranged draw) (#301)
#[test]
fn trigger_made_railgun_system_has_its_own_position_as_target() {
    let mut r = testkit::combat_ruleset();
    register(&mut r, "particles", "LINE");
    r.sim.particles.insert(
        "LINE".into(),
        ParticleType {
            behaves_like: Some("railgun".into()),
            max_ec: Some(1000),
            end_state_ai: Some(100),
            state_ai_advance: Some(4),
            ..Default::default()
        },
    );
    register(&mut r, "particle_systems", "RAIL");
    r.sim.particle_systems.insert(
        "RAIL".into(),
        ParticleSystemType {
            behaves_like: Some("railgun".into()),
            holds_what: Some("LINE".into()),
            particles_per_coord: Some(Decimal("0.015625".into())),
            ..Default::default()
        },
    );
    let mut w = testkit::game_with(&r, N, 7);
    let c = testkit::mid(N, 1, 1);
    let ti = w.rules().gas.system("RAIL").unwrap() as i32;
    emitters::trigger_create(&mut w, ti, c);
    let s = *w.gas.systems.keys().last().unwrap();
    let pos = w.gas.systems[&s].pos;
    let target = w.gas.systems[&s].target;
    let before = w.rng.taken();
    gas::system_turn(&mut w, s);
    let made = w.gas.systems.get(&s).map_or(0, |x| x.particles.len());
    let draws = w.rng.taken() - before;
    assert_eq!(target, pos, "target point: the waypoint cell's centre");
    assert_eq!(made, 0, "L = 0: no particle");
    assert_eq!(draws, 0, "L = 0: no ranged draw");
}
