//! The loops of the links between rules types (Q-2996 to Q-2999 answered):
//! one importer warning for each loop, and a spawn link only for a spawn
//! count above 0. Synthetic data only: invented animation, piece,
//! projectile, weapon and particle types on the testkit's combat rules.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, ParticleType, ProjectileType, VoxelAnimationType, WarheadType, WeaponType,
};
use glasnost_sim::testkit;

const N: i32 = 24;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn anim(r: &mut Ruleset, name: &str, a: AnimationType) {
    register(r, "animations", name);
    r.sim.animations.insert(
        name.into(),
        AnimationType {
            frame_count: Some(200),
            rate: Some(900),
            ..a
        },
    );
}

fn trailer(to: &str) -> AnimationType {
    AnimationType {
        trailer_anim: Some(to.into()),
        trailer_separation: Some(1),
        ..Default::default()
    }
}

/// The warnings the rules build gives for `r`.
fn warnings(r: &Ruleset) -> Vec<String> {
    let w = testkit::game_with(r, N, 7);
    w.rules().loops.warnings.clone()
}

// spec: S-0319 claim 15 (the importer warns once for each loop: each largest group of types that all lead to one another through animation links, a type with a link to itself counting as a group)
#[test]
fn one_warning_for_each_loop_of_animation_types() {
    let mut r = testkit::combat_ruleset();
    anim(&mut r, "SELFA", trailer("SELFA"));
    anim(&mut r, "SELFB", trailer("SELFB"));
    anim(&mut r, "RINGA", trailer("RINGB"));
    anim(&mut r, "RINGB", trailer("RINGA"));
    // A chain into a loop is on no loop and adds no warning.
    anim(&mut r, "INTO", trailer("SELFA"));
    let got = warnings(&r);
    assert_eq!(got.len(), 3, "three loops: {got:?}");
    assert!(got.iter().any(|x| x.contains("RINGA") && x.contains("RINGB")));
    assert!(got.iter().all(|x| !x.contains("INTO")));
}

// spec: S-0319 Terms (an animation link to the spawn type only when the spawn count is above 0)
#[test]
fn a_spawn_type_with_a_count_of_0_is_no_link() {
    let mut r = testkit::combat_ruleset();
    anim(
        &mut r,
        "NOSPAWN",
        AnimationType {
            bouncer: Some(true),
            spawns: Some("NOSPAWN".into()),
            spawn_count: Some(0),
            ..Default::default()
        },
    );
    let w = testkit::game_with(&r, N, 7);
    assert!(!w.rules().loops.anims.any(), "no animation link, no loop");
    assert!(w.rules().loops.warnings.is_empty());
}

fn meteor(r: &mut Ruleset, name: &str, spawns: &str) {
    register(r, "voxel_animations", name);
    r.sim.voxel_animations.insert(
        name.into(),
        VoxelAnimationType {
            is_meteor: Some(true),
            duration: Some(1),
            spawns: Some(spawns.into()),
            spawn_count: Some(2),
            ..Default::default()
        },
    );
}

// spec: S-0317 claim 29 (the importer warns once for each loop of piece types)
#[test]
fn one_warning_for_each_loop_of_piece_types() {
    let mut r = testkit::combat_ruleset();
    meteor(&mut r, "PA", "PA");
    meteor(&mut r, "PB", "PB");
    let got = warnings(&r);
    assert_eq!(got.len(), 2, "two loops: {got:?}");
}

fn smoke(r: &mut Ruleset, name: &str, next: &str) {
    register(r, "particles", name);
    r.sim.particles.insert(
        name.into(),
        ParticleType {
            behaves_like: Some("smoke".into()),
            max_ec: Some(4),
            end_state_ai: Some(100),
            radius: Some(40),
            next_particle: Some(next.into()),
            ..Default::default()
        },
    );
}

// spec: S-0616 claim 8a (the importer warns once for each loop of next-type links)
#[test]
fn one_warning_for_each_loop_of_particle_types() {
    let mut r = testkit::combat_ruleset();
    smoke(&mut r, "SA", "SA");
    smoke(&mut r, "SB", "SB");
    let got = warnings(&r);
    assert_eq!(got.len(), 2, "two loops: {got:?}");
}

fn airburst(r: &mut Ruleset, weapon: &str, projectile: &str, bursts_into: &str) {
    r.sim.weapons.insert(
        weapon.into(),
        WeaponType {
            damage: Some(1),
            rof: Some(100),
            range: Some(glasnost_rules::sim::Decimal("10".into())),
            speed: Some(50),
            projectile: Some(projectile.into()),
            warhead: Some("Dud".into()),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        projectile.into(),
        ProjectileType {
            airburst: Some(true),
            airburst_weapon: Some(bursts_into.into()),
            ..Default::default()
        },
    );
    r.sim.warheads.insert("Dud".into(), WarheadType::default());
}

// spec: S-0306 claim 21a (the importer warns once for each loop of airburst links; no two loops share a projectile type)
#[test]
fn one_warning_for_each_loop_of_airburst_links() {
    let mut r = testkit::combat_ruleset();
    airburst(&mut r, "WA", "BA", "WA");
    airburst(&mut r, "WB", "BB", "WB");
    let got = warnings(&r);
    assert_eq!(got.len(), 2, "two loops: {got:?}");
}
