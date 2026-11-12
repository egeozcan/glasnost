//! S-0302 claim 18 and F8 (#217): the expected damage starts from F1's E₁,
//! without the garrison and bunker multipliers that the shot damage takes.
//! Synthetic data: the garrison fixtures of `structures_common` and an
//! invented bunkered shooter; every name and number invented.

mod structures_common;

use glasnost_map::triggers::Triggers;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::damage;
use glasnost_sim::rules::Kind;
use glasnost_sim::structures::garrison;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use structures_common::*;

const BOX_SIZE: i32 = 24;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The shot damage and the expected damage of a damage-100 weapon, all
/// factors 1, fired by a vehicle that is bunkered or not, with the bunker
/// multiplier `m_b`.
fn bunkered(bunkered: bool, m_b: &str) -> (i32, i32) {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.insert(
        "Zap".into(),
        WeaponType {
            projectile: Some("Bullet".into()),
            damage: Some(100),
            warhead: Some("FLAT".into()),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "FLAT".into(),
        WarheadType {
            verses: Some(vec![dec("1"); 11]),
            ..Default::default()
        },
    );
    let mut t: UnitType = r.sim.vehicles["BUGGY"].clone();
    t.strength = Some(1000);
    t.primary = Some("Zap".into());
    t.combat.armor = None;
    register(&mut r, "vehicles", "SHOOTER");
    r.sim.vehicles.insert("SHOOTER".into(), t.clone());
    t.primary = None;
    register(&mut r, "vehicles", "BOX");
    r.sim.vehicles.insert("BOX".into(), t);
    r.sim.general.bunker_damage_multiplier = Some(dec(m_b));
    let mut w = testkit::game_with(&r, BOX_SIZE, 1);
    let s = testkit::spawn(&mut w, Kind::Vehicle, "SHOOTER", 0, testkit::mid(BOX_SIZE, -3, 0), 0);
    let x = testkit::spawn(&mut w, Kind::Vehicle, "BOX", 1, testkit::mid(BOX_SIZE, 3, 0), 0);
    let holder = testkit::spawn(&mut w, Kind::Vehicle, "BOX", 0, testkit::mid(BOX_SIZE, -5, 0), 0);
    if bunkered {
        w.obj_mut(s).unwrap().structure.bunkered_in = Some(holder);
    }
    let z = w.rules().combat.weapon_index["Zap"];
    (
        damage::shot_damage(&w, s, z),
        damage::expected_damage(&w, s, z, x),
    )
}

// spec: S-0302 F1 (the shot damage takes M_b for a bunkered unit), claim 18 and F8 (the expected damage is E₁ without M_g, M_b, M_t)
#[test]
fn claim_18_expected_damage_leaves_out_the_bunker_multiplier() {
    assert_eq!(bunkered(false, "0.5"), (100, 100));
    let (shot, exp) = bunkered(true, "0.5");
    assert_eq!(shot, 50, "F1: the shot damage takes M_b");
    assert_eq!(exp, 100, "F8: the expected damage does not take M_b");
}

fn garrisoned(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = game_on(r, Triggers::default());
    let b = put(&mut w, Kind::Building, "HUT3", 0, 0, 0);
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -8, -8);
    assert!(garrison::attempt(&mut w, u, b));
    let cart = put(&mut w, Kind::Vehicle, "CART", 1, 3, 0);
    (w, b, cart)
}

fn occ_gun(w: &World) -> glasnost_sim::combat::data::WeaponId {
    let r = w.rules();
    (0..r.combat.weapons.len() as u32)
        .find(|&i| r.combat.weapon(i).name == "OccGun")
        .expect("OccGun")
}

// spec: S-0302 F1 (the shot damage takes M_g for a garrisoned building), claim 18 and F8 (the expected damage is E₁ of F1, without the garrison multiplier)
#[test]
fn claim_18_expected_damage_leaves_out_the_garrison_multiplier() {
    let plain = ruleset();
    let mut scaled = ruleset();
    scaled.sim.general.occupy_damage_multiplier = Some(d("1.5"));
    let (w1, b1, c1) = garrisoned(&plain);
    let (w2, b2, c2) = garrisoned(&scaled);
    let (g1, g2) = (occ_gun(&w1), occ_gun(&w2));
    assert!(garrison::garrisoned(&w2, b2));
    assert_eq!(damage::shot_damage(&w1, b1, g1), 20);
    assert_eq!(damage::shot_damage(&w2, b2, g2), 30);
    assert_eq!(
        damage::expected_damage(&w2, b2, g2, c2),
        damage::expected_damage(&w1, b1, g1, c1),
        "the expected damage changed with the garrison multiplier"
    );
}
