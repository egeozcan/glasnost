//! S-0616 claim 20 and S-0205 claim 1 (#304): the bad-data cases of claim 20
//! can put a particle's creation position anywhere in the 32-bit range, and
//! the arithmetic that follows wraps (S-0205 claim 1) instead of panicking
//! in a build with overflow checks. Synthetic data: the testkit's combat
//! rules plus an invented smoke particle type SOOT and smoke system type
//! PLUME with a spawn radius of −1; every name and number is made up.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType};
use glasnost_sim::world::Pos;
use glasnost_sim::{gas, particles, testkit};

const SIZE: i32 = 20;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    register(&mut r, "particles", "SOOT");
    r.sim.particles.insert(
        "SOOT".into(),
        ParticleType {
            behaves_like: Some("smoke".into()),
            max_ec: Some(1000),
            end_state_ai: Some(100),
            state_ai_advance: Some(4),
            velocity: Some(Decimal("4.0".into())),
            ..Default::default()
        },
    );
    register(&mut r, "particle_systems", "PLUME");
    r.sim.particle_systems.insert(
        "PLUME".into(),
        ParticleSystemType {
            behaves_like: Some("smoke".into()),
            holds_what: Some("SOOT".into()),
            spawn_frames: Some(2),
            spawn_radius: Some(-1),
            particle_cap: Some(50),
            ..Default::default()
        },
    );
    r
}

// spec: S-0616 F1 (v = Q − P per component) and S-0205 claim 1 (an integer result outside the range wraps) (#304)
#[test]
fn heading_from_a_position_at_the_range_edge_wraps() {
    let w = testkit::game_with(&rules(), SIZE, 7);
    let at = Pos::new(i32::MIN, 0, 0);
    let target = Pos::new(5248, 0, 0);
    let wrapped = Pos::new(5248i32.wrapping_sub(i32::MIN), 0, 0);
    assert_eq!(
        particles::heading(&w, at, target),
        particles::heading(&w, Pos::default(), wrapped),
        "the difference wraps"
    );
}

// spec: S-0616 claim 20 (a smoke spawn radius of −1: the remainder is the raw value itself, by S-0205 claim 3) and S-0205 claim 1 (an integer result outside the range wraps) (#304)
#[test]
fn smoke_emission_with_spawn_radius_minus_one() {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    let p = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    gas::warhead_system(&mut w, "PLUME", p, None);
    let sys = *w.gas.systems.keys().last().unwrap();
    assert_eq!(w.gas.systems[&sys].target, p);
    // A point of the simulation stream whose next raw value r1 makes
    // target.x - (x + r1) leave the 32-bit range.
    let mut g = w.rng.clone();
    loop {
        let before = g.clone();
        let r1 = g.raw() as i32;
        if p.x.checked_sub(p.x.wrapping_add(r1)).is_none() {
            w.rng = before;
            break;
        }
    }
    w.tick = 0;
    particles::smoke_turn(&mut w, sys);
    assert!(
        !w.gas.systems[&sys].particles.is_empty(),
        "the turn emitted its particle"
    );
}
