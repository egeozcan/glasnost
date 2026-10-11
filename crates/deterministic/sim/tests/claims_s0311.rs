//! Claim tests for S-0311 (veterancy) on synthetic data: invented types,
//! costs and game-wide values, no stock numbers. Kills are delivered
//! directly through damage resolution (`damage::apply`), so that no
//! fighting or random draw stands between a kill and its experience.

use std::sync::Arc;

use glasnost_map::objects::EntryFields;
use glasnost_map::{Infantry, Map, Unit};
use glasnost_num::{F32, F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, UnitType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::data::{Abilities, Ability};
use glasnost_sim::combat::veterancy::{self, Rank};
use glasnost_sim::combat::weapon;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{GameSetup, HouseId, ObjectId, World};

const SIZE: i32 = 24;

/// Bit patterns of singles used as experience values.
const ONE: u32 = 0x3F80_0000; // 1.0
const ONE_HALF: u32 = 0x3FC0_0000; // 1.5
const TWO: u32 = 0x4000_0000; // 2.0
const TWO_HALF: u32 = 0x4020_0000; // 2.5
const EIGHTH: u32 = 0x3E00_0000; // 0.125
const MINUS_QUARTER: u32 = 0xBE80_0000; // -0.25

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

/// A single n / d, rounded toward zero.
fn single(n: i64, d: i64) -> F32 {
    let rz = Rounding::TowardZero;
    F64::from_i64(n, rz)
        .div(F64::from_i64(d, rz), rz)
        .to_f32(rz)
}

/// The combat ruleset with an unarmed trainable killer of cost 1000 and an
/// unarmed target of cost 500 (both vehicles), veteran ratio `ratio` and
/// cap `cap` (absent when `None`).
fn ruleset(ratio: &str, cap: Option<&str>) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let reg = r.registries.get_mut("vehicles").unwrap();
    reg.push("KILLER".into());
    reg.push("TARGET".into());
    let unarmed = |strength: i32, cost: i32| UnitType {
        strength: Some(strength),
        speed: Some(5),
        rot: Some(5),
        locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Wheel),
        locomotor: Some(glasnost_rules::sim::Locomotor::Drive),
        cost: Some(cost),
        ..Default::default()
    };
    r.sim.vehicles.insert("KILLER".into(), unarmed(400, 1000));
    r.sim.vehicles.insert("TARGET".into(), unarmed(100, 500));
    r.sim.general.veteran_ratio = dec(ratio);
    r.sim.general.veteran_cap = cap.and_then(dec);
    r
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 7)
}

/// A game with houses 0 and 1 allied.
fn allied_game(r: &Ruleset) -> World {
    let mut setup: GameSetup = testkit::setup(7);
    setup.alliances.push((0, 1));
    glasnost_sim::load::new_game(
        Arc::new(SimRules::new(r, "claims-s0311")),
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &setup,
    )
    .expect("game loads")
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, 0), 0)
}

/// A lethal hit on `target` credited to `attacker` (no attacking house, so
/// S-0302 claim 14 credits the attacker object).
fn kill_by(w: &mut World, target: ObjectId, attacker: ObjectId) {
    let r = damage::apply(
        w,
        target,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: Some(attacker),
            house: None,
            ignore_defences: true,
            no_escape: true,
        },
    );
    assert_eq!(r, damage::HitResult::NowDead);
}

/// Spawns a fresh target of `house` and kills it with `killer`.
fn kill_one(w: &mut World, killer: ObjectId, house: u8, name: &str) {
    let t = spawn(w, Kind::Vehicle, name, house, 6);
    kill_by(w, t, killer);
}

fn exp(w: &World, id: ObjectId) -> F32 {
    w.obj(id).unwrap().combat.experience
}

fn set_exp(w: &mut World, id: ObjectId, e: F32) {
    w.obj_mut(id).unwrap().combat.experience = e;
}

fn score(w: &World, h: usize) -> i32 {
    w.houses[h].combat.score
}

/// The kill number after which the killer first reaches `rank`, within
/// `max` kills of rookie targets of house 1.
fn kills_to(w: &mut World, killer: ObjectId, rank: Rank, max: usize) -> Option<usize> {
    for n in 1..=max {
        kill_one(w, killer, 1, "TARGET");
        if veterancy::rank(w, killer) >= rank {
            return Some(n);
        }
    }
    None
}

// spec: S-0311 claim 1 (rookie below 1, veteran from 1 up to but not including 2, elite from 2; negative experience is rookie)
#[test]
fn claim_1_rank_thresholds() {
    assert_eq!(veterancy::rank_of(F32::ZERO), Rank::Rookie);
    assert_eq!(veterancy::rank_of(F32(MINUS_QUARTER)), Rank::Rookie);
    assert_eq!(veterancy::rank_of(F32(0x3F7F_FFFF)), Rank::Rookie); // just below 1
    assert_eq!(veterancy::rank_of(F32(ONE)), Rank::Veteran);
    assert_eq!(veterancy::rank_of(F32(0x3FFF_FFFF)), Rank::Veteran); // just below 2
    assert_eq!(veterancy::rank_of(F32(TWO)), Rank::Elite);
    assert_eq!(veterancy::rank_of(F32(TWO_HALF)), Rank::Elite);
}

// spec: S-0311 claim 1 (ranks have no other state: an object whose experience falls below a threshold loses the rank at once) and claim 10 (X1 part d: elite at 2.5, cap 1.5, one kill gives exactly 1.5)
#[test]
fn claim_1_rank_follows_experience_down() {
    let r = ruleset("4", Some("1.5"));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    set_exp(&mut w, k, F32(TWO_HALF));
    assert_eq!(veterancy::rank(&w, k), Rank::Elite);
    kill_one(&mut w, k, 1, "TARGET");
    assert_eq!(exp(&w, k), F32(ONE_HALF));
    assert_eq!(veterancy::rank(&w, k), Rank::Veteran);
}

// spec: S-0311 claim 1 (experience never decays with time) and claim 2 (experience changes only by the rules of this spec)
#[test]
fn claim_2_experience_does_not_decay() {
    let r = ruleset("4", Some("2.5"));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    let t = spawn(&mut w, Kind::Vehicle, "TARGET", 1, 4);
    set_exp(&mut w, k, F32(ONE_HALF));
    set_exp(&mut w, t, F32(EIGHTH));
    testkit::run(&mut w, 300);
    assert_eq!(exp(&w, k), F32(ONE_HALF));
    assert_eq!(exp(&w, t), F32(EIGHTH));
}

/// A ruleset with an armed vehicle whose weapon does 100 damage, the
/// firepower ability on the given lists and the veteran combat factor 2.
fn firepower_rules(veteran: bool, elite: bool) -> (Ruleset, &'static str) {
    let mut r = ruleset("4", Some("3"));
    r.sim.weapons.insert(
        "Hammer".into(),
        glasnost_rules::sim::WeaponType {
            damage: Some(100),
            rof: Some(30),
            range: dec("3"),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    let mut t = r.sim.vehicles["KILLER"].clone();
    t.primary = Some("Hammer".into());
    if veteran {
        t.veteran_abilities = vec!["FIREPOWER".into()];
    }
    if elite {
        t.elite_abilities = vec!["FIREPOWER".into()];
    }
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("SMASHER".into());
    r.sim.vehicles.insert("SMASHER".into(), t);
    r.sim.general.veteran_combat = dec("2");
    (r, "SMASHER")
}

/// Shot damage of the smasher at experience 0, 1 and 2.
fn shot_damages(veteran: bool, elite: bool) -> [i32; 3] {
    let (r, name) = firepower_rules(veteran, elite);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, name, 0, -4);
    let wp = w.rules().combat.weapon_index["Hammer"];
    let mut out = [0; 3];
    for (i, e) in [F32::ZERO, F32(ONE), F32(TWO)].into_iter().enumerate() {
        set_exp(&mut w, id, e);
        out[i] = damage::shot_damage(&w, id, wp);
    }
    out
}

// spec: S-0311 claim 3 (veteran: the veteran list; elite: both lists; rookie: none; an ability on both lists applies once) and claim 4 (firepower row; scenario X5)
#[test]
fn claim_3_abilities_by_rank_firepower() {
    assert_eq!(shot_damages(false, true), [100, 100, 200]);
    assert_eq!(shot_damages(true, false), [100, 200, 200]);
    assert_eq!(shot_damages(true, true), [100, 200, 200]);
}

// spec: S-0311 claim 3 (has_ability: the veteran list at veteran and elite, the elite list at elite only)
#[test]
fn claim_3_has_ability_lists() {
    let mut r = ruleset("4", Some("3"));
    let k = r.sim.vehicles.get_mut("KILLER").unwrap();
    k.veteran_abilities = vec!["STRONGER".into()];
    k.elite_abilities = vec!["SELF_HEAL".into()];
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    let has = |w: &World, a| veterancy::has_ability(w, id, a);
    assert!(!has(&w, Ability::Stronger) && !has(&w, Ability::SelfHeal));
    set_exp(&mut w, id, F32(ONE));
    assert!(has(&w, Ability::Stronger) && !has(&w, Ability::SelfHeal));
    set_exp(&mut w, id, F32(TWO));
    assert!(has(&w, Ability::Stronger) && has(&w, Ability::SelfHeal));
}

// spec: S-0311 claim 4 (stronger row: the damage the object takes is divided by the veteran armour factor, S-0302 F2)
#[test]
fn claim_4_stronger_divides_damage_taken() {
    let mut r = ruleset("4", Some("3"));
    r.sim.vehicles.get_mut("TARGET").unwrap().strength = Some(1000);
    r.sim.vehicles.get_mut("TARGET").unwrap().veteran_abilities = vec!["STRONGER".into()];
    r.sim.general.veteran_armor = dec("2");
    let mut w = game(&r);
    let sa = r.sim.warheads.keys().position(|k| k == "SA").unwrap() as u32;
    let t = spawn(&mut w, Kind::Vehicle, "TARGET", 1, 4);
    // SA's versus value for the armour class none (index 0) is 1.
    let hit = Hit {
        amount: 100,
        warhead: Some(sa),
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    damage::apply(&mut w, t, hit);
    assert_eq!(w.obj(t).unwrap().health, 900);
    set_exp(&mut w, t, F32(ONE));
    damage::apply(&mut w, t, hit);
    assert_eq!(w.obj(t).unwrap().health, 850);
}

// spec: S-0311 claim 4 (rate-of-fire row: the delay between shots is multiplied by the veteran rate-of-fire factor, S-0301 claim 6)
#[test]
fn claim_4_rate_of_fire_ability() {
    let (mut r, name) = firepower_rules(false, false);
    r.sim.vehicles.get_mut(name).unwrap().veteran_abilities = vec!["ROF".into()];
    r.sim.general.veteran_rof = dec("2");
    // Two identical games, the same draws: one firer a rookie, one a veteran.
    let delay = |veteran: bool| {
        let mut w = game(&r);
        let id = spawn(&mut w, Kind::Vehicle, name, 0, -4);
        if veteran {
            set_exp(&mut w, id, F32(ONE));
        }
        let at = glasnost_sim::world::Target::Cell(testkit::mid(SIZE, -1, 0));
        assert!(glasnost_sim::combat::fire::shot(&mut w, id, 0, at));
        w.obj(id).unwrap().combat.rearm_delay
    };
    let rookie = delay(false);
    assert!((30..=32).contains(&rookie));
    assert_eq!(delay(true), 2 * rookie);
}

// spec: S-0311 claim 5 (resource-proof and resource-heal have no effect: a damaged elite with resource-heal on both lists does not heal)
#[test]
fn claim_5_resource_heal_has_no_effect() {
    let mut r = ruleset("4", Some("3"));
    let t = r.sim.vehicles.get_mut("TARGET").unwrap();
    t.veteran_abilities = vec!["TIBERIUM_HEAL".into(), "TIBERIUM_PROOF".into()];
    t.elite_abilities = vec!["TIBERIUM_HEAL".into()];
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "TARGET", 0, 0);
    set_exp(&mut w, id, F32(TWO));
    w.obj_mut(id).unwrap().health = 40;
    testkit::run(&mut w, 400);
    assert_eq!(w.obj(id).unwrap().health, 40);
}

/// A ruleset where KILLER has primary Gun and elite primary Cannon, and
/// TARGET has primary Gun and no elite primary.
fn elite_weapon_rules() -> Ruleset {
    let mut r = ruleset("4", Some("3"));
    let k = r.sim.vehicles.get_mut("KILLER").unwrap();
    k.primary = Some("Gun".into());
    k.elite_primary = Some("Cannon".into());
    r.sim.vehicles.get_mut("TARGET").unwrap().primary = Some("Gun".into());
    r
}

// spec: S-0311 claim 6 (only the elite rank uses elite weapons; a slot with no elite weapon keeps its weapon; scenario X6)
#[test]
fn claim_6_elite_weapons_only_at_elite() {
    let r = elite_weapon_rules();
    let mut w = game(&r);
    let gun = w.rules().combat.weapon_index["Gun"];
    let cannon = w.rules().combat.weapon_index["Cannon"];
    let a = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    let b = spawn(&mut w, Kind::Vehicle, "TARGET", 0, -2);
    for e in [F32::ZERO, F32(ONE), F32(0x3FFF_FFFF)] {
        set_exp(&mut w, a, e);
        assert_eq!(weapon::slot_weapon(&w, a, 0), Some(gun));
    }
    set_exp(&mut w, a, F32(TWO));
    assert_eq!(weapon::slot_weapon(&w, a, 0), Some(cannon));
    assert_eq!(weapon::current_weapon(&w, a), Some(cannon));
    set_exp(&mut w, b, F32(TWO));
    assert_eq!(weapon::slot_weapon(&w, b, 0), Some(gun));
}

// spec: S-0311 claim 7 (V = m × A_v with m = 1, 2, 3 for a rookie, veteran or elite victim, added to the credited house's score; scenario X2)
#[test]
fn claim_7_kill_value_by_victim_rank() {
    let r = ruleset("4", Some("2.5"));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    for (bits, mult) in [(0u32, 1), (ONE, 2), (TWO, 3)] {
        let before = score(&w, 0);
        let t = spawn(&mut w, Kind::Vehicle, "TARGET", 1, 6);
        set_exp(&mut w, t, F32(bits));
        kill_by(&mut w, t, k);
        assert_eq!(score(&w, 0) - before, mult * 500);
    }
    // 0.125 + 0.25 + 0.375 = 0.75 of experience.
    assert_eq!(exp(&w, k), single(3, 4));
}

// spec: S-0311 claim 7 (X2: with veteran targets the killer is a veteran after the 4th kill, with elite targets after the 3rd)
#[test]
fn claim_7_veteran_and_elite_victims_promote_sooner() {
    for (bits, expected) in [(ONE, 4), (TWO, 3)] {
        let r = ruleset("4", Some("2.5"));
        let mut w = game(&r);
        let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
        let mut n = 0;
        while veterancy::rank(&w, k) < Rank::Veteran {
            let t = spawn(&mut w, Kind::Vehicle, "TARGET", 1, 6);
            set_exp(&mut w, t, F32(bits));
            kill_by(&mut w, t, k);
            n += 1;
        }
        assert_eq!(n, expected);
    }
}

// spec: S-0311 claim 7 (V = 0 when the credited object's house is allied with the destroyed object's: itself or an ally; the score rises by 0)
#[test]
fn claim_7_allied_victims_give_nothing() {
    let r = ruleset("4", Some("2.5"));
    let mut w = allied_game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    kill_one(&mut w, k, 0, "TARGET");
    kill_one(&mut w, k, 1, "TARGET");
    assert_eq!(exp(&w, k), F32::ZERO);
    assert_eq!(score(&w, 0), 0);
}

// spec: S-0311 claim 7 (a destroyed object with no house is never allied: it carries its full kill value)
#[test]
fn claim_7_ownerless_victim_is_never_allied() {
    let r = ruleset("4", Some("2.5"));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    let ti = w.rules().index_of(Kind::Vehicle, "TARGET").unwrap();
    let t = glasnost_sim::objects::create(&mut w, Kind::Vehicle, ti, None);
    let p = glasnost_sim::world::Pos::centre(testkit::mid(SIZE, 6, 0), 0);
    glasnost_sim::objects::place(&mut w, t, p, 0);
    kill_by(&mut w, t, k);
    assert_eq!(exp(&w, k), F32(EIGHTH));
    assert_eq!(score(&w, 0), 500);
}

// spec: S-0311 claim 7 (an unscored type gives no experience and adds nothing to the score)
#[test]
fn claim_7_unscored_victims() {
    let mut r = ruleset("4", Some("2.5"));
    r.sim.vehicles.get_mut("TARGET").unwrap().combat.dont_score = Some(true);
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    kill_one(&mut w, k, 1, "TARGET");
    assert_eq!(exp(&w, k), F32::ZERO);
    assert_eq!(score(&w, 0), 0);
}

// spec: S-0311 claim 8 (case 2: a trainable credited object gains; a credited object of a type that is not trainable gains nothing, while its house still adds V to its score)
#[test]
fn claim_8_untrainable_credited_object() {
    let mut r = ruleset("4", Some("2.5"));
    r.sim.vehicles.get_mut("KILLER").unwrap().combat.trainable = Some(false);
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    kill_one(&mut w, k, 1, "TARGET");
    assert_eq!(exp(&w, k), F32::ZERO);
    assert_eq!(score(&w, 0), 500);
}

// spec: S-0311 claim 9 (only a credited attacker object leads to experience: a kill credited to a house alone gives none)
#[test]
fn claim_9_house_credit_gives_no_experience() {
    let r = ruleset("4", Some("2.5"));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    // An attacking house that is not the attacker's owner credits the house (S-0302 claim 14).
    let t = spawn(&mut w, Kind::Vehicle, "TARGET", 1, 6);
    damage::apply(
        &mut w,
        t,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: Some(k),
            house: Some(HouseId(1)),
            ignore_defences: true,
            no_escape: true,
        },
    );
    assert!(w.obj(t).is_none());
    // No attacker at all.
    let t = spawn(&mut w, Kind::Vehicle, "TARGET", 1, 6);
    damage::apply(
        &mut w,
        t,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: Some(HouseId(0)),
            ignore_defences: true,
            no_escape: true,
        },
    );
    assert!(w.obj(t).is_none());
    assert_eq!(exp(&w, k), F32::ZERO);
}

// spec: S-0311 claim 10 and F3 (worked example: ρ = 4, A_r = 1000, a rookie victim with A_v = 500 adds 0.125; scenario X1 part a: veteran after the 8th kill, elite after the 16th, nothing after that with cap 2.5)
#[test]
fn f3_worked_example() {
    let r = ruleset("4", Some("2.5"));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    kill_one(&mut w, k, 1, "TARGET");
    assert_eq!(exp(&w, k), F32(EIGHTH));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    assert_eq!(kills_to(&mut w, k, Rank::Veteran, 40), Some(8));
    assert_eq!(kills_to(&mut w, k, Rank::Elite, 40), Some(8));
    for _ in 0..10 {
        kill_one(&mut w, k, 1, "TARGET");
    }
    assert_eq!(exp(&w, k), F32(TWO_HALF));
}

// spec: S-0311 claim 10 (X1 part b: with the cap's absent-key default 1.0, kills make a veteran after the 8th kill and never an elite)
#[test]
fn claim_10_default_cap_never_elite() {
    let r = ruleset("4", None);
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    assert_eq!(kills_to(&mut w, k, Rank::Veteran, 40), Some(8));
    assert_eq!(kills_to(&mut w, k, Rank::Elite, 30), None);
    assert_eq!(exp(&w, k), F32(ONE));
}

// spec: S-0311 claim 10 and F3 (X1 part c: ρ = 1.5 gives 0.33333331…, 0.66666662…, 0.99999994… and a veteran only after the 4th kill, under quirks.chop_rounding)
#[test]
fn f3_rounding_toward_zero() {
    let r = ruleset("1.5", Some("2.5"));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    let expected = [0x3EAA_AAAA_u32, 0x3F2A_AAAA, 0x3F7F_FFFF];
    for e in expected {
        kill_one(&mut w, k, 1, "TARGET");
        assert_eq!(exp(&w, k), F32(e));
        assert_eq!(veterancy::rank(&w, k), Rank::Rookie);
    }
    kill_one(&mut w, k, 1, "TARGET");
    assert_eq!(veterancy::rank(&w, k), Rank::Veteran);
}

// spec: S-0311 claim 10 (with quirks.chop_rounding off every step of F3 rounds to nearest: three kills at ρ = 1.5 reach exactly 1)
#[test]
fn f3_rounding_to_nearest_with_chop_rounding_off() {
    let mut r = ruleset("1.5", Some("2.5"));
    r.globals.quirks.chop_rounding = false;
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    assert_eq!(kills_to(&mut w, k, Rank::Veteran, 10), Some(3));
}

// spec: S-0311 claim 10 (the cap limits only the gain: experience above the cap set by claim 11 stays until a kill value, 0 included, is given, which makes it the cap)
#[test]
fn claim_10_cap_applies_only_at_a_kill() {
    let r = ruleset("4", Some("1.5"));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    set_exp(&mut w, k, F32(TWO_HALF));
    testkit::run(&mut w, 20);
    assert_eq!(exp(&w, k), F32(TWO_HALF));
    // A kill value of 0 (an own-house victim) still applies the cap.
    kill_one(&mut w, k, 0, "TARGET");
    assert_eq!(exp(&w, k), F32(ONE_HALF));
}

// spec: S-0311 State and ordering (several destructions in one tick add their gains one after another, each limited by the cap; the kill value uses the victim's rank at its destruction)
#[test]
fn ordering_gains_add_and_each_is_capped() {
    let r = ruleset("4", Some("1"));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    set_exp(&mut w, k, single(7, 8));
    let a = spawn(&mut w, Kind::Vehicle, "TARGET", 1, 6);
    let b = spawn(&mut w, Kind::Vehicle, "TARGET", 1, 8);
    set_exp(&mut w, b, F32(TWO));
    kill_by(&mut w, a, k);
    assert_eq!(exp(&w, k), F32(ONE));
    kill_by(&mut w, b, k);
    assert_eq!(exp(&w, k), F32(ONE));
}

// spec: S-0311 claim 11 (an object is created as a veteran, experience 1, when its type is in its house's country veteran list for its kind: infantry, vehicles, aircraft)
#[test]
fn claim_11_country_veteran_lists() {
    let mut r = ruleset("4", Some("2.5"));
    r.registries
        .get_mut("aircraft")
        .map(|v| v.push("FLYER".into()))
        .unwrap_or_else(|| {
            r.registries.insert("aircraft".into(), vec!["FLYER".into()]);
        });
    r.sim.aircraft.insert(
        "FLYER".into(),
        UnitType {
            strength: Some(100),
            cost: Some(700),
            ..Default::default()
        },
    );
    let c = r.sim.countries.get_mut("Alphaland").unwrap();
    c.veteran_units = Some(vec!["KILLER".into()]);
    c.veteran_infantry = Some(vec!["WALKER".into()]);
    c.veteran_aircraft = Some(vec!["FLYER".into()]);
    let mut w = game(&r);
    let v = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    let i = spawn(&mut w, Kind::Infantry, "WALKER", 0, -2);
    let a = spawn(&mut w, Kind::Aircraft, "FLYER", 0, 0);
    let other = spawn(&mut w, Kind::Vehicle, "TARGET", 0, 2);
    let beta = spawn(&mut w, Kind::Vehicle, "KILLER", 1, 4);
    assert_eq!(exp(&w, v), F32(ONE));
    assert_eq!(exp(&w, i), F32(ONE));
    assert_eq!(exp(&w, a), F32(ONE));
    assert_eq!(exp(&w, other), F32::ZERO);
    assert_eq!(exp(&w, beta), F32::ZERO);
}

/// A game whose map places one KILLER vehicle and one WALKER of house
/// Alpha per veterancy value (`None`: the field absent).
fn map_game(r: &Ruleset, values: &[Option<i32>]) -> World {
    let mut map: Map = testkit::empty_map(SIZE);
    for (k, v) in values.iter().enumerate() {
        let placed = |ty: &str, dy: i32| glasnost_map::objects::Placed {
            key: format!("{ty}{k}").into_bytes(),
            owner: b"Alpha".to_vec(),
            type_name: ty.as_bytes().to_vec(),
            health: 256,
            cell: testkit::mid(SIZE, 2 * k as i32 - 6, dy),
            facing: 0,
            raw: Vec::new(),
        };
        map.objects.units.push(Unit {
            placed: placed("KILLER", -3),
        });
        map.objects.unit_fields.push(EntryFields {
            veterancy: *v,
            ..Default::default()
        });
        map.objects.infantry.push(Infantry {
            placed: placed("WALKER", 3),
            sub_cell: 1,
        });
        map.objects.infantry_fields.push(EntryFields {
            veterancy: *v,
            ..Default::default()
        });
    }
    glasnost_sim::load::new_game(
        Arc::new(SimRules::new(r, "claims-s0311-map")),
        &map,
        testkit::catalogue(),
        &testkit::setup(7),
    )
    .expect("map game loads")
}

/// Experience of the map's vehicles and infantry, in map order.
fn placed_exp(w: &World, kind: Kind) -> Vec<F32> {
    w.objects
        .values()
        .filter(|o| o.kind == kind)
        .map(|o| o.combat.experience)
        .collect()
}

// spec: S-0311 claim 11 and F5 (e = RZ₂₄(RZ₅₃(n × 0.01)): n = 100 gives exactly 1, 200 exactly 2; it replaces the creation rules, so 0 makes a country-listed type a rookie; not limited by the cap: 250 is elite with cap 1.0; scenario X7 part a)
#[test]
fn f5_map_veterancy() {
    let mut r = ruleset("4", None);
    let c = r.sim.countries.get_mut("Alphaland").unwrap();
    c.veteran_units = Some(vec!["KILLER".into()]);
    c.veteran_infantry = Some(vec!["WALKER".into()]);
    let values = [
        Some(0),
        Some(50),
        Some(100),
        Some(150),
        Some(200),
        Some(250),
        None,
    ];
    let w = map_game(&r, &values);
    for kind in [Kind::Vehicle, Kind::Infantry] {
        let e = placed_exp(&w, kind);
        assert_eq!(e.len(), values.len());
        let rz = Rounding::TowardZero;
        let expect = |n: i64| {
            F64::from_i64(n, rz)
                .mul(glasnost_sim::rules::nearest_double("0.01"), rz)
                .to_f32(rz)
        };
        assert_eq!(e[0], F32::ZERO);
        assert_eq!(e[1], expect(50));
        assert_eq!(e[2], F32(ONE));
        assert_eq!(e[3], expect(150));
        assert_eq!(e[4], F32(TWO));
        assert_eq!(e[5], expect(250));
        // No veterancy field: the country list's veteran stays.
        assert_eq!(e[6], F32(ONE));
        let ranks: Vec<Rank> = e.iter().map(|&x| veterancy::rank_of(x)).collect();
        assert_eq!(
            ranks,
            [
                Rank::Rookie,
                Rank::Rookie,
                Rank::Veteran,
                Rank::Veteran,
                Rank::Elite,
                Rank::Elite,
                Rank::Veteran
            ]
        );
    }
}

// spec: S-0311 Edge cases (bad data: a negative map veterancy value gives negative experience, a rookie)
#[test]
fn edge_negative_map_veterancy() {
    let r = ruleset("4", None);
    let w = map_game(&r, &[Some(-25)]);
    let e = placed_exp(&w, Kind::Vehicle)[0];
    assert_eq!(e, F32(MINUS_QUARTER));
    assert_eq!(veterancy::rank_of(e), Rank::Rookie);
}

// spec: S-0311 Edge cases (quirks.zero_kill_value_caps on: a kill value of 0 against an actual cost of 0 sets the recipient's experience to the cap)
#[test]
fn quirk_zero_kill_value_caps_on() {
    let mut r = ruleset("4", Some("1.5"));
    r.sim.vehicles.get_mut("KILLER").unwrap().cost = Some(0);
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    kill_one(&mut w, k, 0, "TARGET");
    assert_eq!(exp(&w, k), F32(ONE_HALF));
}

// spec: S-0311 Edge cases (quirks.zero_kill_value_caps off: a kill value of 0 never changes experience)
#[test]
fn quirk_zero_kill_value_caps_off() {
    let mut r = ruleset("4", Some("1.5"));
    r.sim.vehicles.get_mut("KILLER").unwrap().cost = Some(0);
    r.globals.quirks.zero_kill_value_caps = false;
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    kill_one(&mut w, k, 0, "TARGET");
    assert_eq!(exp(&w, k), F32::ZERO);
    // The same applies above the cap: a value of 0 leaves 2.5 alone.
    set_exp(&mut w, k, F32(TWO_HALF));
    kill_one(&mut w, k, 0, "TARGET");
    assert_eq!(exp(&w, k), F32(TWO_HALF));
}

// spec: S-0311 Edge cases (an actual cost of 0 with a positive kill value reaches the cap in both settings of quirks.zero_kill_value_caps) and F3 (a divisor of 0)
#[test]
fn f3_zero_cost_recipient_reaches_cap() {
    for on in [true, false] {
        let mut r = ruleset("4", Some("1.5"));
        r.sim.vehicles.get_mut("KILLER").unwrap().cost = Some(0);
        r.globals.quirks.zero_kill_value_caps = on;
        let mut w = game(&r);
        let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
        kill_one(&mut w, k, 1, "TARGET");
        assert_eq!(exp(&w, k), F32(ONE_HALF));
    }
}

// spec: S-0311 Edge cases (bad data: a negative veteran ratio gives negative gains; a negative cap is a negative result of any kill)
#[test]
fn edge_negative_ratio_and_cap() {
    let r = ruleset("-4", Some("2.5"));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    kill_one(&mut w, k, 1, "TARGET");
    assert_eq!(exp(&w, k), single(-1, 8));
    let r = ruleset("4", Some("-0.25"));
    let mut w = game(&r);
    let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
    kill_one(&mut w, k, 1, "TARGET");
    assert_eq!(exp(&w, k), F32(MINUS_QUARTER));
}

// spec: S-0311 Edge cases (ability list items are compared ignoring letter case and are not trimmed: an item with a space names no ability)
#[test]
fn edge_ability_items_not_trimmed() {
    let l = |v: &[&str]| Abilities::from_list(&v.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    assert!(l(&["FIREPOWER"]).has(Ability::Firepower));
    assert!(l(&["firePower"]).has(Ability::Firepower));
    assert!(!l(&[" FIREPOWER"]).has(Ability::Firepower));
    assert!(!l(&["FIREPOWER "]).has(Ability::Firepower));
    assert!(l(&["NONSENSE", "C4"]).has(Ability::C4));
}

// spec: S-0311 Edge cases (quirks.experience_cost_victim_house: the house cost factors are not modelled, so both settings give the recipient's cost field and the same gain)
#[test]
fn quirk_experience_cost_victim_house_both_settings() {
    for on in [true, false] {
        let mut r = ruleset("4", Some("2.5"));
        r.globals.quirks.experience_cost_victim_house = on;
        let mut w = game(&r);
        let k = spawn(&mut w, Kind::Vehicle, "KILLER", 0, -4);
        kill_one(&mut w, k, 1, "TARGET");
        assert_eq!(exp(&w, k), F32(EIGHTH));
    }
}
