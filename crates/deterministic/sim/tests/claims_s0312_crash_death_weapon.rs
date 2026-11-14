//! S-0312 Edge cases, `quirks.crash_death_weapon_twice` (sim batch S13,
//! #252): an exploding non-rocket aircraft shot down above the terrain
//! fires its death weapon at death and again at the impact of its crash
//! (claim 16) with the switch on; with it off, only at the impact.
//! Synthetic data: an invented aircraft type with the combat ruleset's
//! `Cannon` as its death weapon.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType};
use glasnost_sim::air;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{Pos, World};

const SIZE: i32 = 24;

fn rules(twice: bool) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.globals.quirks.crash_death_weapon_twice = twice;
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("JET".into());
    let mut jet = UnitType {
        strength: Some(150),
        speed: Some(20),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        ..Default::default()
    };
    jet.combat.explodes = Some(true);
    jet.combat.death_weapon = Some("Cannon".into());
    r.sim.aircraft.insert("JET".into(), jet);
    r
}

fn lethal() -> Hit {
    Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

fn detonations(w: &World) -> Vec<Pos> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Detonation { pos, .. } => Some(*pos),
            _ => None,
        })
        .collect()
}

/// Shoots down an exploding aircraft 600 leptons above the terrain:
/// the detonations at the killing hit, then all of them once the crash has
/// reached the ground.
fn shot_down(twice: bool) -> (usize, usize) {
    let r = rules(twice);
    let mut w = testkit::game_with(&r, SIZE, 11);
    let id = testkit::spawn(&mut w, Kind::Aircraft, "JET", 0, testkit::mid(SIZE, 0, 0), 0);
    w.obj_mut(id).unwrap().pos.z += 600;
    assert!(air::height(&w, id) > 0, "in the air");
    w.effects.clear();
    damage::apply(&mut w, id, lethal());
    assert!(air::crashing(&w, id), "it starts to crash");
    let at_death = detonations(&w).len();
    let t0 = w.tick;
    while w.obj(id).is_some() && w.tick < t0 + 600 {
        testkit::run(&mut w, 1);
    }
    assert!(w.obj(id).is_none(), "the crash has reached the ground");
    (at_death, detonations(&w).len())
}

// spec: S-0312 claim 16 (shot down: the death weapon goes off at the impact point, even if it already went off at the moment of death) and Edge cases (quirks.crash_death_weapon_twice on: at death and again at the impact)
#[test]
fn switch_on_fires_at_death_and_at_the_impact() {
    assert_eq!(shot_down(true), (1, 2));
}

// spec: S-0312 Edge cases (quirks.crash_death_weapon_twice off: an exploding non-rocket aircraft fires its death weapon only at the impact of its crash)
#[test]
fn switch_off_fires_only_at_the_impact() {
    assert_eq!(shot_down(false), (0, 1));
}
