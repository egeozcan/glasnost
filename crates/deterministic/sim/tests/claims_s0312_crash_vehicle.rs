//! S-0312 revision 81 (Q-3112): the scope of `quirks.crash_death_weapon_twice`
//! for vehicles using jumpjet movement, and claim 16's *Vehicle crash end*
//! (#252). Synthetic types on the jumpjet fixtures of `tests/jump_common`:
//! a crashable balloon-hover vehicle and a crashable vehicle of another
//! kind, each with the fixtures' `Cannon` as its death weapon.

mod jump_common;

use glasnost_num::F64;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::World;
use jump_common::*;

/// The fixtures' ruleset with BALCRASH (balloon-hover, crashable) and
/// JVBOOM (crashable, not balloon-hover), both exploding when `explodes`.
fn game_of(twice: bool, explodes: bool) -> World {
    let mut r = ruleset();
    r.globals.quirks.crash_death_weapon_twice = twice;
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .extend(["BALCRASH", "JVBOOM"].map(String::from));
    let mut base = r.sim.vehicles.get("JV").unwrap().clone();
    base.combat.explodes = Some(explodes);
    base.combat.death_weapon = Some("Cannon".into());
    let mut bal = base.clone();
    bal.balloon_hover = Some(true);
    r.sim.vehicles.insert("BALCRASH".into(), bal);
    r.sim.vehicles.insert("JVBOOM".into(), base);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn detonations(w: &World) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Detonation { .. }))
        .count()
}

/// Kills a vehicle of type `name` cruising 500 leptons above flat ground:
/// the death-weapon detonations at the killing hit, then all of them once
/// its crash has ended.
fn shot_down(twice: bool, explodes: bool, name: &str) -> (usize, usize) {
    let mut w = game_of(twice, explodes);
    let u = put(&mut w, name, Kind::Vehicle, 0, -4, 0);
    go(&mut w, u, at(10, 0));
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    jump::start_in_flight(&mut w, u, Phase::Cruising, 500);
    let mode = w.rules().mode;
    w.obj_mut(u).unwrap().jump.as_mut().unwrap().vc = F64::from_i64(14, mode);
    w.effects.clear();
    let health = w.obj(u).unwrap().health;
    damage::apply(
        &mut w,
        u,
        Hit {
            amount: health * 10,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    assert!(jump::crashing(&w, u), "it starts to crash");
    let at_death = detonations(&w);
    let t0 = w.tick;
    while w.obj(u).is_some() && w.tick < t0 + 600 {
        testkit::run(&mut w, 1);
    }
    assert!(w.obj(u).is_none(), "the crash has ended");
    (at_death, detonations(&w))
}

// spec: S-0312 claim 6 and claim 16 (Vehicle crash end: a balloon-hover type sets off its death weapon at its crash end) and Edge cases (quirks.crash_death_weapon_twice on: an exploding object whose death starts a crash fires at death and again at the impact)
#[test]
fn issue_252_on_a_balloon_hover_vehicle_fires_at_death_and_at_its_crash_end() {
    assert_eq!(shot_down(true, true, "BALCRASH"), (1, 2));
}

// spec: S-0312 Edge cases (quirks.crash_death_weapon_twice off: such an object, the crashable vehicle of a balloon-hover type among them, fires no death weapon at death, only the one at the impact)
#[test]
fn issue_252_off_a_balloon_hover_vehicle_fires_only_at_its_crash_end() {
    assert_eq!(shot_down(false, true, "BALCRASH"), (0, 1));
}

// spec: S-0312 claim 16 (Vehicle crash end: a balloon-hover type sets off its death weapon whether or not it has the explodes flag) and claim 6 (an object that is not exploding fires no death weapon at death)
#[test]
fn issue_252_a_balloon_hover_vehicle_that_does_not_explode_fires_at_its_crash_end() {
    assert_eq!(shot_down(true, false, "BALCRASH"), (0, 1));
    assert_eq!(shot_down(false, false, "BALCRASH"), (0, 1));
}

// spec: S-0312 claim 16 (Vehicle crash end: a type of any other kind sets off no death weapon at this impact) and Edge cases (the switch reaches no other death: an exploding crashable vehicle of any other type fires its death weapon at death in both settings)
#[test]
fn issue_252_a_crashable_vehicle_of_another_kind_fires_only_at_death() {
    assert_eq!(shot_down(true, true, "JVBOOM"), (1, 1));
    assert_eq!(shot_down(false, true, "JVBOOM"), (1, 1));
}
