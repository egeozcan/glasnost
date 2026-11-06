//! Tests of the selected turret index of a multi-turret vehicle (S-0105
//! claim 43 and state table S3) on the simulation's synthetic combat
//! ruleset. Every type name and number is made up.

use glasnost_convert::turrets::{EMPTY, TurretTable};
use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::weapon;
use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};

use super::build::{charge_turret_number, selected_turret_index};

const SIZE: i32 = 24;

/// The combat ruleset with the vehicle type `name`: CART's fields, a
/// turret, a turret count of 3, three weapon slots, and `f` applied.
fn with_multi(name: &str, f: impl FnOnce(&mut glasnost_rules::sim::UnitType)) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut t = r.sim.vehicles["CART"].clone();
    t.turret = Some(true);
    t.turret_count = Some(3);
    t.weapon_count = Some(3);
    t.combat.weapon_slots = Some(vec!["Gun".into(), "Gun".into(), "Gun".into()]);
    f(&mut t);
    r.sim.vehicles.insert(name.into(), t);
    r.registries.get_mut("vehicles").unwrap().push(name.into());
    r
}

fn spawn(w: &mut World, name: &str, house: u8, dx: i32) -> ObjectId {
    testkit::spawn(
        w,
        SimKind::Vehicle,
        name,
        house,
        testkit::mid(SIZE, dx, 0),
        16384,
    )
}

/// A turret table: weapon number 0 → 2, 1 → 0, 2 → 1, 3 → 5 (at or above
/// the count), 17 → none.
fn table() -> TurretTable {
    let mut t = EMPTY;
    for (w, k) in [(0, 2), (1, 0), (2, 1), (3, 5)] {
        t[w] = Some(k);
    }
    t
}

/// spec: S-0105 claim 43 and S3 (a gunner type: T[0] from creation, T[w] each time its weapon
/// number is set, even when it is unchanged; a request outside 0 … 17 gives weapon number 0;
/// the entry is kept when it lies outside 0 … N − 1; a type with neither flag: none for life,
/// whatever its table; a type that is not multi-turret: none); MS-33 parts a, b and c
#[test]
fn a_gunner_type_shows_the_entry_for_its_weapon_number() {
    let r = with_multi("MULTI", |t| t.gunner = Some(true));
    let mut w = testkit::game_with(&r, SIZE, 5);
    let m = spawn(&mut w, "MULTI", 0, 0);
    let t = table();
    let shown = |w: &World| (w.tick - 1).max(0);
    assert_eq!(
        selected_turret_index(&w, m, shown(&w), Some(&t)),
        Some(2),
        "T[0] at creation"
    );
    assert_eq!(
        selected_turret_index(&w, m, 0, None),
        None,
        "an all-none table"
    );
    // spec: S-0308 claim 6 (a hand-over sets the weapon number to the passenger's mode)
    for (mode, want) in [
        (1, Some(0)),
        (2, Some(1)),
        (3, Some(5)),
        (17, None),
        (20, Some(2)),
    ] {
        weapon::set_weapon_number(&mut w, m, mode);
        assert_eq!(
            selected_turret_index(&w, m, shown(&w), Some(&t)),
            want,
            "mode {mode}"
        );
    }
    // Kept through ticks and veterancy (nothing else changes it).
    weapon::set_weapon_number(&mut w, m, 1);
    testkit::run(&mut w, 3);
    let two = glasnost_num::F64::from_i64(2, glasnost_num::Rounding::TowardZero)
        .to_f32(glasnost_num::Rounding::TowardZero);
    w.obj_mut(m).unwrap().combat.experience = two;
    assert_eq!(selected_turret_index(&w, m, shown(&w), Some(&t)), Some(0));
    // Neither a gunner nor a charge-turret type: none for life, whatever its table.
    let r = with_multi("PLAIN", |_| {});
    let mut w = testkit::game_with(&r, SIZE, 5);
    let p = spawn(&mut w, "PLAIN", 0, 0);
    assert_eq!(selected_turret_index(&w, p, 0, Some(&t)), None);
    weapon::set_weapon_number(&mut w, p, 2);
    assert_eq!(selected_turret_index(&w, p, 0, Some(&t)), None);
    // A type with rotating-barrel weapon stages, or no turret count, is not multi-turret.
    let r = with_multi("SPIN", |t| {
        t.gunner = Some(true);
        t.combat.is_gattling = Some(true);
    });
    let mut w = testkit::game_with(&r, SIZE, 5);
    let s = spawn(&mut w, "SPIN", 0, 0);
    assert_eq!(selected_turret_index(&w, s, 0, Some(&t)), None);
    let cart = spawn(&mut w, "CART", 0, 3);
    assert_eq!(selected_turret_index(&w, cart, 0, Some(&t)), None);
}

/// spec: S-0301 claim 16 (c = trunc(N × r / D) clamped to 0 … N − 1 while the countdown runs; 0
/// with D ≤ 0 or once it is over); MS-33 part f (model 2 for r ≥ 2D/3, model 1 for
/// D/3 ≤ r < 2D/3, model 0 below and after)
#[test]
fn the_charge_turret_number_steps_down_with_the_countdown() {
    let (n, d, shot) = (3, 30, 100);
    let until = shot + d;
    let c = |t| charge_turret_number(n, until, d, t);
    for t in shot..until + 5 {
        let r = until - t;
        let want = if r >= 2 * d / 3 {
            2
        } else if r >= d / 3 && r > 0 {
            1
        } else {
            0
        };
        assert_eq!(c(t), want, "r = {r}");
    }
    assert_eq!(c(shot), n - 1, "right after the shot: N − 1");
    assert_eq!(charge_turret_number(n, until, 0, shot), 0, "D = 0");
    assert_eq!(charge_turret_number(n, until, -4, shot), 0, "D < 0");
    // spec: S-0301 claim 1 (the countdown starts over: rearm_until is the earliest tick at creation)
    assert_eq!(charge_turret_number(n, i32::MIN, 0, 0), 0, "from creation");
}

/// spec: S-0105 claim 43 (a charge-turret type: never from the turret table, because it ignores
/// every request to change its weapon number; in each of its turns S-0301 claim 16's c; index 0
/// from creation in our engine); MS-33 parts a and f
#[test]
fn a_charge_turret_type_shows_its_countdown() {
    let r = with_multi("CHARGE", |t| {
        t.gunner = Some(true);
        t.combat.is_charge_turret = Some(true);
    });
    let mut w = testkit::game_with(&r, SIZE, 5);
    let c = spawn(&mut w, "CHARGE", 0, -1);
    let t = table();
    let shown = |w: &World| (w.tick - 1).max(0);
    assert_eq!(
        selected_turret_index(&w, c, shown(&w), Some(&t)),
        Some(0),
        "0 from creation"
    );
    weapon::set_weapon_number(&mut w, c, 2);
    assert_eq!(
        selected_turret_index(&w, c, shown(&w), Some(&t)),
        Some(0),
        "not T[w]"
    );
    // Let it fire at a target in range and follow its countdown.
    let target = spawn(&mut w, "CART", 1, 2);
    let mut seen = Vec::new();
    for _ in 0..400 {
        testkit::run(&mut w, 1);
        let o = w.obj(c).unwrap();
        let (until, delay) = (o.combat.rearm_until, o.combat.rearm_delay);
        let k = selected_turret_index(&w, c, shown(&w), Some(&t));
        assert_eq!(
            k,
            Some(charge_turret_number(3, until, delay, shown(&w))),
            "tick {}",
            shown(&w)
        );
        seen.push(k);
        if w.obj(target).is_none() {
            break;
        }
    }
    for k in [0, 1, 2] {
        assert!(seen.contains(&Some(k)), "model {k} shown");
    }
}

/// spec: S-0105 claim 43 and S3 (the index a map vehicle drawn before any tick is created with:
/// T[0] for a gunner type, turret 0 for a charge-turret type, none for a type with neither
/// flag); without typed rules there is no index
#[test]
fn a_map_vehicle_before_any_tick_shows_its_creation_index() {
    use super::build::creation_turret_index;
    use glasnost_convert::kinds::Kind;
    let (mut import, _, _) = super::tests::context();
    import.turret_tables.insert(0, table());
    let index = |f: fn(&mut glasnost_rules::sim::UnitType)| {
        let w = testkit::game_with(&with_multi("CARX", f), SIZE, 5);
        creation_turret_index(Some(&w.rules_arc()), &import, Kind::Vehicles, 0)
    };
    assert_eq!(index(|t| t.gunner = Some(true)), Some(2), "T[0]");
    assert_eq!(
        index(|t| {
            t.gunner = Some(true);
            t.combat.is_charge_turret = Some(true);
        }),
        Some(0),
        "turret 0 from creation"
    );
    assert_eq!(index(|_| {}), None, "neither flag");
    assert_eq!(
        creation_turret_index(None, &import, Kind::Vehicles, 0),
        None
    );
}

/// spec: S-0111 claim 23 (a turret-tip vehicle's t: S-0105 claim 43's index, except that a type
/// with rotating-barrel weapon stages gets t by the gunner rules, the table entry for its weapon
/// number, and none when it is not a gunner type or has the charge-turret flag)
#[test]
fn the_name_tip_index_of_a_rotating_barrel_type() {
    use super::build::tip_turret_index;
    let t = table();
    let r = with_multi("SPIN", |t| {
        t.gunner = Some(true);
        t.combat.is_gattling = Some(true);
    });
    let mut w = testkit::game_with(&r, SIZE, 5);
    let s = spawn(&mut w, "SPIN", 0, 0);
    assert_eq!(
        selected_turret_index(&w, s, 0, Some(&t)),
        None,
        "not multi-turret"
    );
    assert_eq!(tip_turret_index(&w, s, 0, Some(&t)), Some(2), "T[0]");
    weapon::set_weapon_number(&mut w, s, 2);
    assert_eq!(tip_turret_index(&w, s, 0, Some(&t)), Some(1), "T[2]");
    assert_eq!(tip_turret_index(&w, s, 0, None), None, "an all-none table");
    for (name, f) in [
        (
            "SPINX",
            (|t| t.combat.is_gattling = Some(true)) as fn(&mut glasnost_rules::sim::UnitType),
        ),
        ("SPINC", |t| {
            t.gunner = Some(true);
            t.combat.is_gattling = Some(true);
            t.combat.is_charge_turret = Some(true);
        }),
    ] {
        let r = with_multi(name, f);
        let mut w = testkit::game_with(&r, SIZE, 5);
        let v = spawn(&mut w, name, 0, 0);
        assert_eq!(tip_turret_index(&w, v, 0, Some(&t)), None, "{name}");
        weapon::set_weapon_number(&mut w, v, 2);
        assert_eq!(tip_turret_index(&w, v, 0, Some(&t)), None, "{name}");
    }
    // Any other type: the drawing's index.
    let r = with_multi("MULTI", |t| t.gunner = Some(true));
    let mut w = testkit::game_with(&r, SIZE, 5);
    let m = spawn(&mut w, "MULTI", 0, 0);
    weapon::set_weapon_number(&mut w, m, 1);
    assert_eq!(
        tip_turret_index(&w, m, 0, Some(&t)),
        selected_turret_index(&w, m, 0, Some(&t))
    );
    assert_eq!(tip_turret_index(&w, m, 0, Some(&t)), Some(0));
}
