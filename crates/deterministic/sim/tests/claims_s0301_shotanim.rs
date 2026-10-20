//! Claim tests for shot animations (S-0301 claims 20 to 23, Formulas F7
//! and F8), on synthetic data: invented test animations T0 … T15 and test
//! weapons modelled on scenario F19's (W8, W16, W1, W0, WG, WG0, WP, WP8),
//! each firing the invisible, draw-free projectile Bullet.
//!
//! Shots are made with `combat::fire::shot` directly, with the firer's
//! facings set by hand. The shot animation is read from the presentation
//! event `Effect::ShotAnim`; draws from the scenario stream's `taken()`
//! counter and a clone of the stream.

mod structures_common;

use glasnost_num::geom::Facing;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, UnitType, WeaponType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::{fire, shotanim};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

/// Map size of every test here.
const N: i32 = 24;
/// Facing east.
const EAST: u16 = 16384;
/// Facing west.
const WEST: u16 = 49152;

/// A shot animation event: name, position, attachment, depth adjustment.
type Made = (String, Pos, Option<(ObjectId, Pos)>, Option<i32>);

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

/// The names T0 … T(n − 1).
fn names(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("T{i}")).collect()
}

/// A test weapon firing Bullet with animation list `list`.
fn weapon(list: Vec<String>) -> WeaponType {
    WeaponType {
        damage: Some(1),
        rof: Some(30),
        range: Some(dec("10")),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some("SA".into()),
        anim: (!list.is_empty()).then_some(list),
        ..Default::default()
    }
}

/// Registers the test animations T0 … T15 in `r` (one stage delay each, so
/// their creation takes no draw), and the test weapons.
fn add_test_data(r: &mut Ruleset) {
    for n in names(16) {
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.clone());
        r.sim.animations.insert(n, AnimationType::default());
    }
    let w = &mut r.sim.weapons;
    w.insert("W8".into(), weapon(names(8)));
    w.insert("W16".into(), weapon(names(16)));
    w.insert("W1".into(), weapon(names(1)));
    w.insert("W0".into(), weapon(Vec::new()));
    let mut wp = weapon(Vec::new());
    wp.open_topped_anim = Some("T10".into());
    w.insert("WP".into(), wp);
    let mut wp8 = weapon(names(8));
    wp8.open_topped_anim = Some("T10".into());
    w.insert("WP8".into(), wp8);
}

/// The combat ruleset with the test data; the CART and the WALKER carry
/// `primary`, the CART without the turret flag unless `turret`.
fn rules_with(primary: &str, turret: bool) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    add_test_data(&mut r);
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.primary = Some(primary.into());
    cart.turret = Some(turret);
    r.sim.infantry.get_mut("WALKER").unwrap().primary = Some(primary.into());
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(30000);
    r
}

/// A world of `r` with an Alpha firer of `kind`/`name` two cells west of
/// the middle facing east and a Beta BUGGY two cells east of it, the
/// firer's target.
fn duel(r: &Ruleset, kind: Kind, name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, N, 7);
    let a = testkit::spawn(&mut w, kind, name, 0, testkit::mid(N, -2, 0), EAST);
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    (w, a, b)
}

/// Makes one shot with slot 0 at `target` and returns the shot animations
/// it reported and the raw values it took.
fn shoot(w: &mut World, a: ObjectId, target: Target) -> (Vec<Made>, u64) {
    w.effects.clear();
    let before = w.rng.taken();
    fire::shot(w, a, 0, target);
    let taken = w.rng.taken() - before;
    let made = w
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::ShotAnim {
                name,
                pos,
                attached,
                z_adjust,
                ..
            } => Some((name.clone(), *pos, *attached, *z_adjust)),
            _ => None,
        })
        .collect();
    (made, taken)
}

/// The names of the shot animations of one shot at `a`'s target.
fn shot_names(w: &mut World, a: ObjectId) -> Vec<String> {
    let t = w.obj(a).unwrap().target.unwrap();
    // A fresh countdown each time: fire::shot does not ask the firing check.
    shoot(w, a, t).0.into_iter().map(|m| m.0).collect()
}

/// The facing of 8-direction `d` (S-0204 claim 9).
fn dir8(d: u16) -> Facing {
    d * 8192
}

// spec: S-0301 F7 (d = ⌊f / 8192 + ½⌋ mod 8, e = (d + 1) mod 8) and claim 21 (the double rounding: with the 32-step rounding turret facings 3072 to 4095 give d = 1, with the 256-step rounding facings 3968 to 4095 do, where a single rounding gives d = 0)
#[test]
fn f7_eight_entry_and_the_double_rounding() {
    // Claim 20's table: entry e for 8-direction d.
    for (e, d) in [
        (0, 7),
        (1, 0),
        (2, 1),
        (3, 2),
        (4, 3),
        (5, 4),
        (6, 5),
        (7, 6),
    ] {
        assert_eq!(shotanim::eight_entry(dir8(d), None), e, "d = {d}");
    }
    // Single rounding: the boundary of d = 0 and d = 1 is at 4096.
    assert_eq!(shotanim::eight_entry(4095, None), 1);
    assert_eq!(shotanim::eight_entry(4096, None), 2);
    assert_eq!(shotanim::eight_entry(3072, None), 1);
    // 32-step first: 3072 … 4095 give d = 1, 3071 still d = 0.
    assert_eq!(shotanim::eight_entry(3071, Some(32)), 1);
    for f in [3072, 3584, 4095] {
        assert_eq!(shotanim::eight_entry(f, Some(32)), 2, "f = {f}");
    }
    // 256-step first: 3968 … 4095 give d = 1, 3967 still d = 0.
    assert_eq!(shotanim::eight_entry(3967, Some(256)), 1);
    for f in [3968, 4000, 4095] {
        assert_eq!(shotanim::eight_entry(f, Some(256)), 2, "f = {f}");
    }
    // The wrap: a facing just below a full turn is north, entry 1.
    assert_eq!(shotanim::eight_entry(65535, None), 1);
    assert_eq!(shotanim::eight_entry(65535, Some(32)), 1);
}

// spec: S-0301 claim 20 a (n = 8: entry (d + 1) mod 8 of the shot facing; F19 part a) and claim 21 (a vehicle without the turret flag: its body facing)
#[test]
fn claim_20_a_eight_entries_by_a_vehicles_body_facing() {
    let r = rules_with("W8", false);
    let (mut w, a, _) = duel(&r, Kind::Vehicle, "CART");
    for d in 0..8u16 {
        let o = w.obj_mut(a).unwrap();
        o.body = Turning::at(dir8(d));
        // The turret facing does not count for a type without the turret flag.
        o.turret = Turning::at(dir8((d + 3) % 8));
        let e = (d + 1) % 8;
        assert_eq!(shot_names(&mut w, a), vec![format!("T{e}")], "d = {d}");
    }
}

// spec: S-0301 claim 21 (a vehicle whose own type has the turret flag: its current turret facing; F19 part a)
#[test]
fn claim_21_a_turreted_vehicle_uses_its_turret_facing() {
    let r = rules_with("W8", true);
    let (mut w, a, _) = duel(&r, Kind::Vehicle, "CART");
    for d in 0..8u16 {
        let o = w.obj_mut(a).unwrap();
        o.turret = Turning::at(dir8(d));
        o.body = Turning::at(dir8((d + 5) % 8));
        assert_eq!(shot_names(&mut w, a), vec![format!("T{}", (d + 1) % 8)]);
    }
    // A single rounding for a vehicle: 3584 is d = 0, entry 1.
    w.obj_mut(a).unwrap().turret = Turning::at(3584);
    assert_eq!(shot_names(&mut w, a), vec!["T1".to_owned()]);
}

// spec: S-0301 claim 21 (infantry: its current body facing; an aircraft: its current turret facing; F19 part b)
#[test]
fn claim_21_infantry_body_and_aircraft_turret() {
    let r = rules_with("W8", false);
    let (mut w, a, _) = duel(&r, Kind::Infantry, "WALKER");
    for d in 0..8u16 {
        w.obj_mut(a).unwrap().body = Turning::at(dir8(d));
        assert_eq!(shot_names(&mut w, a), vec![format!("T{}", (d + 1) % 8)]);
    }
    let mut r = rules_with("W8", false);
    r.registries.insert("aircraft".into(), vec!["KITE".into()]);
    r.sim.aircraft.insert(
        "KITE".into(),
        UnitType {
            strength: Some(300),
            primary: Some("W8".into()),
            ..Default::default()
        },
    );
    let (mut w, k, _) = duel(&r, Kind::Aircraft, "KITE");
    for d in 0..8u16 {
        let o = w.obj_mut(k).unwrap();
        o.turret = Turning::at(dir8(d));
        o.body = Turning::at(dir8((d + 2) % 8));
        assert_eq!(shotanim::shot_facing(&w, k), (dir8(d), None));
        assert_eq!(shot_names(&mut w, k), vec![format!("T{}", (d + 1) % 8)]);
    }
}

// spec: S-0301 claim 20 a (any other n ≥ 1, 1 and 16 included: entry 0 whatever the facing; n = 0: no type; F19 part c) and claim 23 (no type chosen: no shot animation and no placeholder)
#[test]
fn claim_20_a_other_list_lengths() {
    for (wpn, expect) in [("W16", Some("T0")), ("W1", Some("T0")), ("W0", None)] {
        let r = rules_with(wpn, false);
        let (mut w, a, _) = duel(&r, Kind::Vehicle, "CART");
        for d in 0..8u16 {
            w.obj_mut(a).unwrap().body = Turning::at(dir8(d));
            let got = shot_names(&mut w, a);
            assert_eq!(
                got,
                expect.map(str::to_owned).into_iter().collect::<Vec<_>>()
            );
        }
    }
}

// spec: S-0301 claim 20 (every shot of a burst makes its own choice at the moment it is fired)
#[test]
fn claim_20_every_shot_of_a_burst_chooses_anew() {
    let mut r = rules_with("W8", false);
    r.sim.weapons.get_mut("W8").unwrap().burst = Some(3);
    let (mut w, a, _) = duel(&r, Kind::Vehicle, "CART");
    for d in [0u16, 4, 6] {
        w.obj_mut(a).unwrap().body = Turning::at(dir8(d));
        assert_eq!(shot_names(&mut w, a), vec![format!("T{}", (d + 1) % 8)]);
    }
}

// spec: S-0301 claim 22 (created at the shot's launch point; a firer that is not a building: attached, at its displacement from the firer, height included; no depth replacement) and S-0109 claim 22
#[test]
fn claim_22_attached_to_a_vehicle_at_its_launch_point() {
    let r = rules_with("W1", false);
    let (mut w, a, b) = duel(&r, Kind::Vehicle, "CART");
    let fp = glasnost_sim::combat::position::fire_point(&w, a, 0, 0);
    let at = w.obj(a).unwrap().pos;
    let (made, _) = shoot(&mut w, a, Target::Object(b));
    assert_eq!(made.len(), 1);
    let (name, pos, attached, z) = made[0].clone();
    assert_eq!(name, "T0");
    assert_eq!(pos, fp, "at the launch point (S-0313 claim 1's fire point)");
    assert_eq!(
        attached,
        Some((a, Pos::new(fp.x - at.x, fp.y - at.y, fp.z - at.z)))
    );
    assert_eq!(z, None);
}

// spec: S-0301 Terms (launch point: the firer's own position for a dropping projectile, S-0306 claim 5) and claim 22
#[test]
fn claim_22_a_dropping_projectile_launches_from_the_firer() {
    let mut r = rules_with("W1", false);
    r.sim.projectiles.insert(
        "Drop".into(),
        glasnost_rules::sim::ProjectileType {
            dropping: Some(true),
            inviso: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.get_mut("W1").unwrap().projectile = Some("Drop".into());
    let (mut w, a, b) = duel(&r, Kind::Vehicle, "CART");
    let at = w.obj(a).unwrap().pos;
    let (made, _) = shoot(&mut w, a, Target::Object(b));
    assert_eq!(made.len(), 1, "one shot animation");
    assert_eq!(made[0].1, at);
    assert_eq!(made[0].2, Some((a, Pos::new(0, 0, 0))));
}

// spec: S-0301 claim 18 and claim 22, Ordering consequences (the shot animation's creation draws come after the delay draw; choosing takes no draw), S-0109 claim 3 (a random stage delay is the creation's draw)
#[test]
fn claim_22_creation_draws_follow_the_delay_draw() {
    let mut r = rules_with("W1", false);
    // A descending random rate pair: ρ = 2 and 1 give a stage delay drawn over 450 … 900.
    r.sim.animations.get_mut("T0").unwrap().random_rate = Some([2, 1]);
    let (mut w, a, b) = duel(&r, Kind::Vehicle, "CART");
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut expect = w.rng.clone();
    let _ = expect.ranged(0, 2, half); // the delay draw R(0, 2)
    let stage = expect.ranged(450, 900, half); // then the animation's stage delay
    w.effects.clear();
    fire::shot(&mut w, a, 0, Target::Object(b));
    assert_eq!(w.rng, expect, "delay draw, then the animation's draw");
    let delay = w.effects.iter().find_map(|e| match e {
        Effect::ShotAnim { delay, .. } => Some(*delay),
        _ => None,
    });
    assert_eq!(delay, Some(Some(stage)));
    // An empty list adds no draw: the delay draw alone.
    let r0 = rules_with("W0", false);
    let (mut w, a, b) = duel(&r0, Kind::Vehicle, "CART");
    let mut expect = w.rng.clone();
    let _ = expect.ranged(0, 2, half);
    fire::shot(&mut w, a, 0, Target::Object(b));
    assert_eq!(w.rng, expect);
}

/// The combat ruleset's TOWER armed with `primary`, turreted or not, with
/// a sprite or a voxel turret.
fn tower_rules(primary: &str, turret: bool, voxel: bool) -> Ruleset {
    let mut r = rules_with(primary, false);
    let t = r.sim.buildings.get_mut("TOWER").unwrap();
    t.primary = Some(primary.into());
    t.turret = Some(turret);
    t.turret_anim_is_voxel = Some(voxel);
    r
}

// spec: S-0301 claim 21 (a building with a target: turreted with a sprite turret, its turret facing rounded to 32 steps and again to 8; F19 part f: about 3584 gives entry 2)
#[test]
fn claim_21_sprite_turreted_building_rounds_twice() {
    let r = tower_rules("W8", true, false);
    let (mut w, t, _) = duel(&r, Kind::Building, "TOWER");
    w.obj_mut(t).unwrap().turret = Turning::at(3584);
    assert_eq!(shotanim::shot_facing(&w, t), (3584, Some(32)));
    assert_eq!(shot_names(&mut w, t), vec!["T2".to_owned()]);
    for d in 0..8u16 {
        w.obj_mut(t).unwrap().turret = Turning::at(dir8(d));
        assert_eq!(shot_names(&mut w, t), vec![format!("T{}", (d + 1) % 8)]);
    }
}

// spec: S-0301 claim 21 (any other building with a target, not turreted or with a voxel turret: the direction of the target from its aiming point, S-0310 claim 4; F19 part i)
#[test]
fn claim_21_other_buildings_use_the_target_direction() {
    for (turret, voxel) in [(false, false), (true, true)] {
        let r = tower_rules("W8", turret, voxel);
        let (mut w, t, b) = duel(&r, Kind::Building, "TOWER");
        // The turret facing points elsewhere; it does not count.
        w.obj_mut(t).unwrap().turret = Turning::at(dir8(6));
        let from = glasnost_sim::combat::check::building_aim_point(&w, t);
        let to = glasnost_sim::combat::range::target_point(&w, Target::Object(b));
        let dir = glasnost_num::geom::direction(
            i64::from(to.x - from.x),
            i64::from(to.y - from.y),
            w.rules().quirks.table_direction,
        );
        assert_eq!(shotanim::shot_facing(&w, t), (dir, None));
        let e = shotanim::eight_entry(dir, None);
        assert_eq!(shot_names(&mut w, t), vec![format!("T{e}")]);
    }
}

// spec: S-0301 claim 21 (a building without a target: its turret facing rounded to the 256-step facing, then to 8)
#[test]
fn claim_21_building_without_a_target_rounds_to_256_steps() {
    let r = tower_rules("W8", true, false);
    let (mut w, t, b) = duel(&r, Kind::Building, "TOWER");
    let o = w.obj_mut(t).unwrap();
    o.target = None;
    o.turret = Turning::at(3968);
    assert_eq!(shotanim::shot_facing(&w, t), (3968, Some(256)));
    let (made, _) = shoot(&mut w, t, Target::Object(b));
    assert_eq!(made[0].0, "T2");
}

// spec: S-0301 claim 22 (a building's shot animation is not attached; its creator replaces its depth adjustment with z_a of F8) and F8 (z_a = min(0, −trunc((y_l − y₀) / 4)) without occupants)
#[test]
fn claim_22_f8_a_buildings_animation_stays_with_its_depth() {
    let r = tower_rules("W1", true, false);
    let (mut w, t, b) = duel(&r, Kind::Building, "TOWER");
    let fp = glasnost_sim::combat::position::fire_point(&w, t, 0, 0);
    let (made, _) = shoot(&mut w, t, Target::Object(b));
    let (_, pos, attached, z) = made[0].clone();
    assert_eq!(pos, fp);
    assert_eq!(attached, None, "not attached");
    let y0 = 256 * w.obj(t).unwrap().pos.cell().y;
    assert_eq!(z, Some(0.min(-((fp.y - y0) / 4))));
    // F8 on chosen launch points: truncation toward zero, and never above 0.
    for (dy, za) in [(0, 0), (3, 0), (7, -1), (400, -100), (-1, 0), (-400, 0)] {
        assert_eq!(shotanim::building_depth(&w, t, y0 + dy), za, "dy = {dy}");
    }
}

// spec: S-0301 claim 23 (a spawner weapon's shot and a disc laser's firing make no shot animation)
#[test]
fn claim_23_spawner_and_disc_laser_shots_make_none() {
    for flag in ["spawner", "disc"] {
        let mut r = rules_with("W1", false);
        let wpn = r.sim.weapons.get_mut("W1").unwrap();
        match flag {
            "spawner" => wpn.spawner = Some(true),
            _ => wpn.disk_laser = Some(true),
        }
        let (mut w, a, b) = duel(&r, Kind::Vehicle, "CART");
        let (made, _) = shoot(&mut w, a, Target::Object(b));
        assert!(made.is_empty(), "{flag}");
    }
}

// spec: S-0301 claim 23 (a drain attempt, the firing of a suicide weapon and a void shot make no shot animation)
#[test]
fn claim_23_drain_suicide_and_void_shots_make_none() {
    for flag in ["drain", "suicide", "void"] {
        let mut r = rules_with("W1", false);
        if flag == "void" {
            // An arcing projectile with a turn rate at speed 1 has no launch pitch (S-0306 claim 6 a).
            r.sim.projectiles.insert(
                "SlowArc".into(),
                glasnost_rules::sim::ProjectileType {
                    arcing: Some(true),
                    rot: Some(1),
                    ..Default::default()
                },
            );
        }
        let wpn = r.sim.weapons.get_mut("W1").unwrap();
        match flag {
            "drain" => wpn.drain_weapon = Some(true),
            "suicide" => wpn.suicide = Some(true),
            _ => {
                wpn.projectile = Some("SlowArc".into());
                wpn.speed = Some(1);
            }
        }
        let (mut w, a, b) = duel(&r, Kind::Vehicle, "CART");
        let (made, _) = shoot(&mut w, a, Target::Object(b));
        assert!(made.is_empty(), "{flag}");
    }
}

// spec: S-0301 claim 20 c (no type chosen and a passenger firing out: the open-topped animation; a non-empty list plays its entry by the passenger's own facing, never the open-topped animation; F19 part h)
#[test]
fn claim_20_c_open_topped_fallback() {
    for (wpn, d, expect) in [("WP", 3u16, "T10"), ("WP8", 3, "T4"), ("WP8", 6, "T7")] {
        let mut r = rules_with(wpn, false);
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        cart.primary = None;
        cart.passengers = Some(3);
        cart.size_limit = Some(dec("2"));
        cart.open_topped = Some(true);
        let (mut w, t, b) = duel(&r, Kind::Vehicle, "CART");
        let p = testkit::spawn(
            &mut w,
            Kind::Infantry,
            "WALKER",
            0,
            testkit::mid(N, -2, 1),
            EAST,
        );
        glasnost_sim::transport::board(&mut w, p, t);
        assert!(glasnost_sim::transport::open_topped_passenger(&w, p));
        w.obj_mut(p).unwrap().body = Turning::at(dir8(d));
        let weapon = glasnost_sim::combat::weapon::slot_weapon(&w, p, 0).unwrap();
        assert_eq!(
            shotanim::choose(&w, p, weapon).as_deref(),
            Some(expect),
            "{wpn}"
        );
        let (made, _) = shoot(&mut w, p, Target::Object(b));
        assert_eq!(made.len(), 1);
        assert_eq!(made[0].0, expect);
        assert_eq!(made[0].2.map(|x| x.0), Some(p), "attached to the passenger");
    }
    // The same weapon outside a transport: no fallback.
    let r = rules_with("WP", false);
    let (mut w, a, _) = duel(&r, Kind::Infantry, "WALKER");
    assert!(shot_names(&mut w, a).is_empty());
}

mod garrisoned {
    use super::structures_common::*;
    use super::*;
    use glasnost_sim::structures::garrison;

    /// A hut of Alpha garrisoned by one GARRI whose occupy weapon has the
    /// list T0 … T7 and, when `occupant`, the occupant animation T9 (WG,
    /// WG0 of F19 part e).
    fn garrisoned(occupant: bool) -> (World, ObjectId, ObjectId) {
        let mut r = ruleset();
        add_test_data(&mut r);
        let g = r.sim.weapons.get_mut("OccGun").unwrap();
        g.anim = Some(names(8));
        g.occupant_anim = occupant.then(|| "T9".into());
        g.projectile = Some("Bullet".into());
        let mut w = game_on(&r, Default::default());
        let hut = put(&mut w, Kind::Building, "HUT", 0, 0, 0);
        let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, -6);
        assert!(garrison::attempt(&mut w, u, hut));
        let e = put(&mut w, Kind::Vehicle, "BUGGY", 1, 3, 0);
        (w, hut, e)
    }

    // spec: S-0301 claim 20 b (a garrisoned building: the occupant animation replaces the list's choice; without one, none even with a list) and claim 22 with F8 (an occupied building: not attached, z_a = −200)
    #[test]
    fn claim_20_b_occupant_animation_replaces_the_list() {
        for occupant in [true, false] {
            let (mut w, hut, e) = garrisoned(occupant);
            assert!(garrison::garrisoned(&w, hut));
            for d in 0..8u16 {
                w.obj_mut(hut).unwrap().turret = Turning::at(dir8(d));
                let (made, _) = shoot(&mut w, hut, Target::Object(e));
                if occupant {
                    assert_eq!(made.len(), 1);
                    assert_eq!(made[0].0, "T9");
                    assert_eq!(made[0].2, None);
                    assert_eq!(made[0].3, Some(-200));
                } else {
                    assert!(
                        made.is_empty(),
                        "no animation without an occupant animation"
                    );
                }
            }
        }
    }
}
