//! Interaction-pair scenario tests of M7b (PLAN, cross-component
//! interactions): each pair runs through the fixed tick pipeline on
//! synthetic data (invented types, weapons and warheads, every number our
//! own). The ledger is `docs/m7-claims/pairs-m7b.md`.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, ParticleSystemType, ParticleType, WarheadType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::infantry::Seq;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Pos, World};
use glasnost_sim::{gas, lift, save, testkit};

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The special-hit animations every pair uses: BOOM (the explosion death,
/// kind 3) and BURN (the burning death, kind 4, a burning walker with 4
/// running frames, 80 frames and damage 0.25).
fn special_hits(r: &mut Ruleset) {
    register(r, "animations", "BOOM");
    r.sim.animations.insert(
        "BOOM".into(),
        AnimationType {
            frame_count: Some(10),
            ..Default::default()
        },
    );
    register(r, "animations", "BURN");
    r.sim.animations.insert(
        "BURN".into(),
        AnimationType {
            frame_count: Some(80),
            is_flaming_guy: Some(true),
            running_frames: Some(4),
            damage: Some(Decimal("0.25".into())),
            ..Default::default()
        },
    );
    r.sim.general.infantry_explode = Some("BOOM".into());
    r.sim.general.flaming_infantry = Some("BURN".into());
}

/// Names of the animations and walkers the effects since the last clear
/// show.
fn shown(w: &World) -> Vec<String> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, .. } => Some(name.clone()),
            Effect::Walker {
                name, made: true, ..
            } => Some(format!("walker {name}")),
            _ => None,
        })
        .collect()
}

fn round_trip(w: &World) {
    let h = save::state_hash(w);
    let l = save::load(&save::save(w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
}

// Pair: special-hit effects and the magnetic lift's fall (S-0615 × S-0607).
// spec: S-0607 claim 16 (a dropped unit's fall ending on infantry removes it at once, without death effects or death animations, whatever the crushing warhead's death kind)
// spec: S-0607 claims 16 to 18 (a fall ending on a vehicle: blocked land, crashed, two impact detonations, the first with the crushing warhead at ground level)
// spec: S-0615 claims 1, 2 and 4 (an infantry unit that a detonation of a kind-4 warhead kills becomes a burning walker at its position)
// spec: S-0303 claims 6 to 12 (ordinary area damage of the impact detonation reaches the neighbouring cell)
#[test]
fn a_lift_fall_crushes_silently_but_its_impact_makes_a_burning_walker() {
    use lift_common::*;
    let tweak = |r: &mut Ruleset| {
        special_hits(r);
        // The crushing warhead kills infantry by burning (kind 4); its spread reaches the next cell.
        let crush = r.sim.warheads.get_mut("CRUSH").unwrap();
        crush.inf_death = Some(4);
        crush.cell_spread = Some(d("1.5"));
        infantry(r, "FRAIL", |t| t.strength = Some(1));
    };
    // Part 1: the fall ends on an infantry unit, which is crushed silently.
    let mut w = world(tweak);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    let under = spawn(&mut w, Kind::Infantry, "FRAIL", 0, at(3, 0));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    assert!(run_until(&mut w, 400, |w| w
        .obj(v)
        .is_some_and(|o| o.pos.cell() == at(3, 0))
        && height(w, v) > 300));
    glasnost_sim::econ::assign_destination(
        &mut w,
        l,
        Some(glasnost_sim::world::Target::Cell(at(0, 1))),
    );
    let mut seen = Vec::new();
    assert!(run_until(&mut w, 400, |w| {
        seen.extend(shown(w));
        w.obj(under).is_none()
    }));
    assert!(
        !seen.iter().any(|n| n == "walker BURN" || n == "BOOM"),
        "crushed without a death animation: {seen:?}"
    );
    assert!(w.walkers.is_empty());
    round_trip(&w);

    // Part 2: the fall ends on a vehicle; the impact's crushing detonation kills an infantry unit
    // in the next cell, which burns.
    let mut w = world(tweak);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    spawn(&mut w, Kind::Vehicle, "VICT", 0, at(3, 0));
    let near = spawn(&mut w, Kind::Infantry, "FRAIL", 0, at(3, 1));
    beam(&mut w, l, v);
    assert!(run_until(&mut w, 400, |w| w
        .obj(v)
        .is_some_and(|o| o.pos.cell() == at(3, 0))
        && height(w, v) > 300));
    glasnost_sim::econ::assign_destination(
        &mut w,
        l,
        Some(glasnost_sim::world::Target::Cell(at(0, 1))),
    );
    let at_death = w.obj(near).unwrap().pos;
    let mut walker = None;
    assert!(run_until(&mut w, 400, |w| {
        for e in &w.effects {
            if let Effect::Walker {
                name,
                pos,
                made: true,
                ..
            } = e
            {
                walker = Some((name.clone(), *pos));
            }
        }
        w.obj(near).is_none()
    }));
    testkit::run(&mut w, 1);
    for e in &w.effects {
        if let Effect::Walker {
            name,
            pos,
            made: true,
            ..
        } = e
        {
            walker = walker.or(Some((name.clone(), *pos)));
        }
    }
    assert_eq!(
        walker,
        Some(("BURN".to_string(), at_death)),
        "a burning walker where it died"
    );
    assert_eq!(w.walkers.len(), 1);
    round_trip(&w);
    // The walker runs its course and leaves.
    assert!(run_until(&mut w, 2000, |w| w.walkers.is_empty()));
}

/// The gas-pair rules: testkit's combat rules with the special-hit
/// animations, the shared gas system GasCloudSys holding FOG (a gas particle
/// of damage 7 with the virus warhead VENOM, infantry-death kind 8), the
/// virus-death animation PLAGUE (μ = 2, 6 frames, 3 FOG particles) and
/// FRAIL, an infantry type of strength 1.
fn gas_rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    special_hits(&mut r);
    register(&mut r, "particles", "FOG");
    r.sim.particles.insert(
        "FOG".into(),
        ParticleType {
            behaves_like: Some("gas".into()),
            max_ec: Some(30000),
            max_dc: Some(5),
            damage: Some(7),
            warhead: Some("VENOM".into()),
            state_ai_advance: Some(4),
            end_state_ai: Some(100),
            ..Default::default()
        },
    );
    register(&mut r, "particle_systems", "GasCloudSys");
    r.sim.particle_systems.insert(
        "GasCloudSys".into(),
        ParticleSystemType {
            behaves_like: Some("gas".into()),
            holds_what: Some("FOG".into()),
            ..Default::default()
        },
    );
    register(&mut r, "warheads", "VENOM");
    r.sim.warheads.insert(
        "VENOM".into(),
        WarheadType {
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            inf_death: Some(8),
            ..Default::default()
        },
    );
    register(&mut r, "animations", "PLAGUE");
    r.sim.animations.insert(
        "PLAGUE".into(),
        AnimationType {
            middle_frame: Some(2),
            frame_count: Some(6),
            spawns_particle: Some("FOG".into()),
            num_particles: Some(3),
            ..Default::default()
        },
    );
    r.sim.general.infantry_virus = Some("PLAGUE".into());
    let mut frail = r.sim.infantry["WALKER"].clone();
    frail.strength = Some(1);
    register(&mut r, "infantry", "FRAIL");
    r.sim.infantry.insert("FRAIL".into(), frail);
    r
}

/// A victim's death: who, where, the other animations made, the new clouds
/// and the new lasting virus animations.
type Death = (ObjectId, Pos, Vec<(String, Pos)>, usize, usize);

fn cloud_count(w: &World) -> usize {
    let s = w.gas.shared.expect("a shared gas system");
    w.gas.systems[&s].particles.len()
}

// Pair: special-hit effects and gas deaths (S-0615 × S-0613).
// spec: S-0613 claims 2, 13 and 14 (a cloud's hit with the virus warhead kills an infantry unit: the virus-death animation stays and delivers its count later)
// spec: S-0107 claim 17 and S-0613 claim 3 (a parachutist killed by a virus death: the extra virus-death animation, removed at once, and one cloud)
// spec: S-0615 claim 1 (a parachuting unit's death is kind 3: the explosion death animation as well; each dispatch makes at most one animation)
// spec: S-0615 claim 2 (the explosion animation at the unit's position)
#[test]
fn a_gas_cloud_kills_a_parachutist_with_one_cloud_and_an_explosion() {
    let r = gas_rules();
    let mut w = testkit::game_with(&r, 20, 7);
    let walker = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "FRAIL",
        1,
        testkit::mid(20, -4, 0),
        0,
    );
    let chute = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "FRAIL",
        1,
        testkit::mid(20, 4, 0),
        0,
    );
    let draw = w
        .rules()
        .gas
        .particles
        .iter()
        .find(|p| p.name == "FOG")
        .unwrap()
        .draw;
    // One cloud on each victim in turn, so that the deaths fall in different ticks.
    let mut deaths: Vec<Death> = Vec::new();
    for id in [walker, chute] {
        let p = w.obj(id).unwrap().pos;
        gas::middle_event_particle(&mut w, draw, Some((p.x, p.y, p.z)));
        for _ in 0..200 {
            if let Some(o) = w.obj_mut(chute)
                && let Some(i) = o.infantry.as_mut()
            {
                i.seq = Some(Seq::Parachute);
            }
            let pos = w.obj(id).unwrap().pos;
            let clouds = cloud_count(&w);
            let anims = w.gas.anims.len();
            testkit::run(&mut w, 1);
            if w.obj(id).is_none() {
                let names = w
                    .effects
                    .iter()
                    .filter_map(|e| match e {
                        Effect::Anim { name, pos, .. } if name != "PLAGUE" => {
                            Some((name.clone(), *pos))
                        }
                        _ => None,
                    })
                    .collect();
                deaths.push((
                    id,
                    pos,
                    names,
                    cloud_count(&w) - clouds,
                    w.gas.anims.len() - anims,
                ));
                break;
            }
        }
    }
    assert_eq!(deaths.len(), 2, "both died in the clouds");
    for (id, pos, names, new_clouds, new_anims) in &deaths {
        if *id == chute {
            assert_eq!(
                names,
                &[("BOOM".to_string(), *pos)],
                "the parachutist explodes where it died"
            );
            assert_eq!(*new_clouds, 1, "one extra cloud at once");
            assert_eq!(
                *new_anims, 0,
                "the extra virus animation is removed at once"
            );
        } else {
            assert!(names.is_empty(), "no special-hit animation: {names:?}");
            assert_eq!(*new_anims, 1, "the virus-death animation stays");
            assert_eq!(*new_clouds, 0, "its count comes later");
        }
    }
    assert!(w.walkers.is_empty());
    round_trip(&w);
}
