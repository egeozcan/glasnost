//! S-0308 Terms, *Airborne* (#178): an object that is not an aircraft is
//! airborne for the weapon choice by S-0307's height test, whatever its
//! locomotor. Synthetic data: the testkit's combat ruleset plus an invented
//! anti-air-only projectile and weapon; every name and number invented.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{ProjectileType, WeaponType};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::weapon;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 40;

/// The combat ruleset with `Flak` (the `Rifle` with an anti-air-only,
/// invisible projectile), CART firing `Rifle` then `Flak`, and GATT: a
/// gattling CART with the slots Rifle, Flak, Rifle, Flak.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.projectiles.insert(
        "AAOnly".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(false),
            inviso: Some(true),
            ..Default::default()
        },
    );
    let gun = r.sim.weapons["Rifle"].clone();
    r.sim.weapons.insert(
        "Flak".into(),
        WeaponType {
            projectile: Some("AAOnly".into()),
            ..gun
        },
    );
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.primary = Some("Rifle".into());
    cart.secondary = Some("Flak".into());
    let mut g = r.sim.vehicles["CART"].clone();
    let slots = ["Rifle", "Flak", "Rifle", "Flak"];
    g.turret_count = Some(slots.len() as i32);
    g.weapon_count = Some(slots.len() as i32);
    g.combat.weapon_slots = Some(slots.iter().map(|s| s.to_string()).collect());
    g.combat.is_gattling = Some(true);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("GATT".into());
    r.sim.vehicles.insert("GATT".into(), g);
    r
}

/// A firer of `firer` at the centre and a WALKER of the other house two
/// cells east, lifted `up` leptons and falling.
fn falling_walker(firer: &str, up: i32) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    let f = testkit::spawn(&mut w, Kind::Vehicle, firer, 0, testkit::mid(SIZE, 0, 0), 0);
    let t = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        1,
        testkit::mid(SIZE, 2, 0),
        0,
    );
    w.obj_mut(t).unwrap().pos.z += up;
    glasnost_sim::fall::start(&mut w, t, false);
    assert!(w.obj(t).unwrap().on_map);
    assert!(glasnost_sim::fall::falling(&w, t));
    assert!(!glasnost_sim::jump::is_jumpjet(&w, t));
    (w, f, t)
}

// spec: S-0308 Terms (Airborne: an object that is not an aircraft uses S-0307 Terms, the height test of 208 leptons) and table P row 19 (in the air and an anti-air secondary: 1); S-0306 claim 23 (the firing check reads the same test)
#[test]
fn a_falling_infantry_unit_high_up_gets_the_anti_air_secondary() {
    let (w, f, t) = falling_walker("CART", 300);
    assert!(weapon::airborne_by_height(&w, t));
    let tg = Some(Target::Object(t));
    let (slot, _) = weapon::would_use(&w, f, tg);
    assert_eq!(slot, 1, "table P row 19");
    assert_ne!(check::check(&w, f, tg, slot, false), Answer::Illegal);
}

// spec: S-0308 Terms (Airborne, S-0307 Terms: below 208 leptons an object is not airborne) and table P row 20
#[test]
fn a_falling_infantry_unit_low_down_gets_the_primary() {
    let (w, f, t) = falling_walker("CART", 100);
    assert!(!weapon::airborne_by_height(&w, t));
    assert_eq!(weapon::choose_slot(&w, f, Some(Target::Object(t))), 0);
}

// spec: S-0308 claim 7 (a gattling type at stage s uses slot 2s + 1 against an object in the air when the anti-air test passes) and Terms (Airborne)
#[test]
fn a_gattling_type_uses_slot_two_s_plus_one_against_a_falling_unit() {
    let (mut w, g, t) = falling_walker("GATT", 300);
    w.obj_mut(g).unwrap().combat.gattling_stage = 1;
    assert_eq!(weapon::choose_slot(&w, g, Some(Target::Object(t))), 3);
}
