//! Claim tests of S-0214 revision 78 (claim 6 (e): the attack offered over
//! a reason of the moment, Q-2356 answered, and
//! `quirks.moment_reason_attack_cursor`), M8 step A3 intake 85. Synthetic
//! data only: every type, weapon, warhead and number is invented.
//!
//! A drop in tick *k* shows after `tick::step` has run that tick, when the
//! tick number reads *k* + 1. The ledger is `docs/m8-claims/intake85.md`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{ArmourClass, Decimal, Locomotor, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::teleport::TeleportState;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
const A: HouseId = HouseId(ALPHA);
/// The check period of S-0309 claim 29, written out for the expected ticks.
const PERIOD: i32 = 16;
/// The armour class *light*'s index in the versus list.
const LIGHT: usize = 3;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

/// Eleven versus values: `v` for class `class`, 1 for the others.
fn only(class: usize, v: &str) -> Option<Vec<Decimal>> {
    let mut l = vec![dec("1"); 11];
    l[class] = dec(v);
    Some(l)
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["BUGGY"].clone();
    t.strength = Some(1000);
    t.primary = None;
    t.secondary = None;
    t.combat.armor = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

fn warhead(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut WarheadType)) {
    let mut h = WarheadType {
        verses: only(LIGHT, "1"),
        ..Default::default()
    };
    f(&mut h);
    r.sim.warheads.insert(name.into(), h);
}

/// A weapon of range 5 cells firing every 30 ticks, with the testkit's
/// invisible `Bullet`, which is not anti-air.
fn weapon(r: &mut Ruleset, name: &str, warhead_name: &str, f: impl FnOnce(&mut WeaponType)) {
    let mut x = WeaponType {
        damage: Some(20),
        rof: Some(30),
        range: Some(dec("5")),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some(warhead_name.into()),
        ..Default::default()
    };
    f(&mut x);
    r.sim.weapons.insert(name.into(), x);
}

/// The rules of this file:
/// - warhead `H` (0 against *light*, S-0304 claim 1's zero-versus test)
///   with weapon `W`, and with weapon `WC`, which needs the firer
///   uncloaked; warhead `PLAIN` (1 everywhere) with weapon `Plain`;
/// - vehicles `DEF` (primary `W`), `CDEF` (primary `WC`), `GUN` (primary
///   `Plain`), `TARGET` (class *light*, unarmed: *Z* of T4 part F),
///   `PLAINV` (class *heavy*, unarmed: *A*), `JUMPER` (the teleporting
///   locomotor without the harvester flag, primary `Plain`) and `LANDER`
///   (as `JUMPER`, with land-targeting mode 1, S-0306 claim 23's land row);
/// - building `LTOWER`: the testkit's turreted `TOWER` with land-targeting
///   mode 1.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "H", |h| h.verses = only(LIGHT, "0"));
    warhead(&mut r, "PLAIN", |_| {});
    weapon(&mut r, "W", "H", |_| {});
    weapon(&mut r, "WC", "H", |x| x.decloak_to_fire = Some(true));
    weapon(&mut r, "Plain", "PLAIN", |_| {});
    vehicle(&mut r, "DEF", |t| t.primary = Some("W".into()));
    vehicle(&mut r, "CDEF", |t| t.primary = Some("WC".into()));
    vehicle(&mut r, "GUN", |t| t.primary = Some("Plain".into()));
    vehicle(&mut r, "TARGET", |t| {
        t.combat.armor = Some(ArmourClass::Light)
    });
    vehicle(&mut r, "PLAINV", |t| {
        t.combat.armor = Some(ArmourClass::Heavy)
    });
    vehicle(&mut r, "JUMPER", |t| {
        t.primary = Some("Plain".into());
        t.locomotor = Some(Locomotor::Teleport);
        t.harvester = Some(false);
    });
    vehicle(&mut r, "LANDER", |t| {
        t.primary = Some("Plain".into());
        t.locomotor = Some(Locomotor::Teleport);
        t.harvester = Some(false);
        t.combat.land_targeting = Some(1);
    });
    vehicle(&mut r, "NAVALER", |t| {
        t.primary = Some("Plain".into());
        t.locomotor = Some(Locomotor::Teleport);
        t.harvester = Some(false);
        t.combat.naval_targeting = Some(6);
    });
    let mut b = r.sim.buildings["TOWER"].clone();
    b.combat.land_targeting = Some(1);
    register(&mut r, "buildings", "LTOWER");
    r.sim.buildings.insert("LTOWER".into(), b);
    r
}

/// The rules with `quirks.moment_reason_attack_cursor` set to `on`.
fn rules_with(on: bool) -> Ruleset {
    let mut r = rules();
    r.globals.quirks.moment_reason_attack_cursor = on;
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn game(on: bool) -> World {
    testkit::game_with(&rules_with(on), SIZE, 1)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn obj(x: ObjectId) -> Option<Target> {
    Some(Target::Object(x))
}

fn ask(w: &World, id: ObjectId, t: Option<Target>) -> Answer {
    check::check(w, id, t, 0, false)
}

/// The action an ordinary hover over `t` shows for `actor` (S-0214 claim 4).
fn act(w: &World, actor: ObjectId, t: ObjectId) -> Action {
    targeting::action(w, A, actor, ClickOn::Object(t), Modifiers::default())
}

/// The orders an ordinary click over `t` gives `actor` (S-0214 claim 4).
fn click(w: &World, actor: ObjectId, t: ObjectId) -> Vec<Order> {
    targeting::click(w, A, &[actor], ClickOn::Object(t), Modifiers::default()).orders
}

fn is_attack_on(orders: &[Order], actor: ObjectId, t: ObjectId) -> bool {
    matches!(orders, [Order::Behaviour { actor: a, behaviour: Behaviour::Attack, target: Some(Target::Object(x)), .. }]
        if *a == actor && *x == t)
}

fn target(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).and_then(|o| o.target)
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

/// Runs whole ticks until the game's opening order block is over and the
/// tick number is `residue` modulo the check period.
fn step_to(w: &mut World, residue: i32) {
    // spec: S-0213 claim 1 and S-0201 claim 14 (an order block refuses input)
    while glasnost_sim::orders::blocked(w) || w.tick.rem_euclid(PERIOD) != residue {
        step(w);
    }
}

/// Makes `id` materialise (phase in, S-0207 claim 20 f) for `d` ticks
/// from now.
fn materialise(w: &mut World, id: ObjectId, d: i64) {
    let t = w.tick;
    w.obj_mut(id).unwrap().teleport = Some(TeleportState {
        phasing: Some((t, d)),
        ..Default::default()
    });
    assert!(glasnost_sim::teleport::phasing_in(w, id));
}

/// Issues `orders` for house ALPHA and runs whole ticks, at most 4, until
/// `id` holds `x`.
fn issue_until_held(w: &mut World, orders: Vec<Order>, id: ObjectId, x: Target) {
    for o in orders {
        assert!(glasnost_sim::orders::issue(w, A, o));
    }
    for _ in 0..4 {
        step(w);
        if target(w, id) == Some(x) {
            return;
        }
    }
    panic!("the attack order never gave the target");
}

/// Runs whole ticks, at most `limit`, until `id` no longer holds `x`;
/// returns the tick in which it dropped it.
fn drop_tick(w: &mut World, id: ObjectId, x: Target, limit: i32) -> Option<i32> {
    for _ in 0..limit {
        step(w);
        if target(w, id) != Some(x) {
            return Some(w.tick - 1);
        }
    }
    None
}

fn shots_by(w: &World, id: ObjectId) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Shot { firer, .. } if *firer == id))
        .count()
}

// ---------------------------------------------------------------------------
// The rearm countdown, out of ammunition and cloaked (T4 part F).
// ---------------------------------------------------------------------------

// spec: S-0214 claim 6 (e) (r78: part (e) takes the firing check's answer, the highest rank that holds, and fails only on illegal; while the rearm countdown runs, rank 5, the late target tests of rank 9, the zero-versus test of S-0304 claim 1 among them, are not asked, so the actor is offered the attack over such a target without force-fire) and Edge cases (quirks.moment_reason_attack_cursor: on, the attack is offered while the countdown runs; off, part (e) asks only whether a reason that gives illegal holds, so the action over the target no longer changes with the countdown); scenario T4 part F and its control
#[test]
fn t4f_the_countdown_offers_the_attack_over_a_target_its_warhead_cannot_affect() {
    for on in [true, false] {
        let mut w = game(on);
        let d = put(&mut w, Kind::Vehicle, "DEF", ALPHA, -1, 0);
        let z = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
        let a = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 2, 2);
        // Control: no countdown runs.
        assert_eq!(ask(&w, d, obj(z)), Answer::Illegal);
        assert_ne!(act(&w, d, z), Action::Attack, "control, on {on}");
        assert!(!is_attack_on(&click(&w, d, z), d, z), "control, on {on}");
        assert_eq!(act(&w, d, a), Action::Attack, "control over A, on {on}");
        // The countdown runs well past this tick.
        let t = w.tick;
        w.obj_mut(d).unwrap().combat.rearm_until = t + 50;
        assert_eq!(ask(&w, d, obj(z)), Answer::Rearming);
        assert_eq!(act(&w, d, a), Action::Attack, "over A, on {on}");
        if on {
            assert_eq!(act(&w, d, z), Action::Attack);
            assert!(is_attack_on(&click(&w, d, z), d, z));
        } else {
            assert_ne!(act(&w, d, z), Action::Attack);
            assert!(!is_attack_on(&click(&w, d, z), d, z));
        }
        // After the countdown: as the control.
        w.obj_mut(d).unwrap().combat.rearm_until = t;
        assert_ne!(act(&w, d, z), Action::Attack, "after, on {on}");
    }
}

// spec: S-0214 claim 6 (e) (r78: an actor given the attack order in its countdown makes no shot at the target and drops it at its first checking turn in a check tick at which no reason ranked above the refusal applies, S-0309 claim 29; its firing step makes no drop on illegal; a holder that takes a checking turn in every tick drops it within 16 ticks of the last such reason ending) and Edge cases (quirks.moment_reason_attack_cursor on); scenario T4 part F's expected keep, drop and no shot
#[test]
fn t4f_the_target_ordered_in_the_countdown_is_kept_to_its_end_and_dropped_unfired() {
    let mut w = game(true);
    let d = put(&mut w, Kind::Vehicle, "DEF", ALPHA, -1, 0);
    let z = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
    step_to(&mut w, 2);
    let n = w.tick;
    // Check ticks n + 14 and n + 30 fall in the countdown; n + 46 is the first after it.
    w.obj_mut(d).unwrap().combat.rearm_until = n + 40;
    let orders = click(&w, d, z);
    assert!(is_attack_on(&orders, d, z));
    w.effects.clear();
    issue_until_held(&mut w, orders, d, Target::Object(z));
    assert_eq!(drop_tick(&mut w, d, Target::Object(z), 80), Some(n + 46));
    assert_eq!(shots_by(&w, d), 0, "never fired at");
}

// spec: S-0214 claim 6 (e) (r78: while the actor is out of ammunition, rank 7, and while its answer is cloaked, rank 8, the late target tests of rank 9 are not asked either, so it is offered the attack over a target that only those tests refuse) and Edge cases (quirks.moment_reason_attack_cursor: off, part (e) passes over out of ammunition and cloaked, whatever the actor's ammunition)
#[test]
fn out_of_ammunition_and_cloaked_hide_the_zero_versus_test() {
    for on in [true, false] {
        let mut w = game(on);
        let d = put(&mut w, Kind::Vehicle, "DEF", ALPHA, -1, 0);
        let c = put(&mut w, Kind::Vehicle, "CDEF", ALPHA, -1, 2);
        let z = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
        let a = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 2, 2);
        assert_ne!(act(&w, c, z), Action::Attack, "control, on {on}");
        w.obj_mut(d).unwrap().combat.ammo = 0;
        w.obj_mut(c).unwrap().cloak.state = glasnost_sim::cloak::CloakState::Cloaked;
        assert_eq!(ask(&w, d, obj(z)), Answer::OutOfAmmo);
        assert_eq!(ask(&w, c, obj(z)), Answer::Cloaked);
        for x in [d, c] {
            assert_eq!(act(&w, x, z) == Action::Attack, on, "on {on}");
            assert_eq!(is_attack_on(&click(&w, x, z), x, z), on, "on {on}");
            assert_eq!(act(&w, x, a), Action::Attack, "over A, on {on}");
        }
    }
}

// ---------------------------------------------------------------------------
// A phasing-in actor (T4 part E).
// ---------------------------------------------------------------------------

// spec: S-0214 claim 6 (e) (r78, Q-2356 answered: a phasing-in actor gets rearming for every target and weapon ahead of the target tests, S-0301 claim 9, A materialising firer within rank 4, so part (e) holds over a target in the air its weapon cannot hit; a phasing-in actor is not movable, Terms, so part (d) still asks that the target be in range: over an enemy in range row G8 gives attack, over one out of range row G9 gives nothing) and Edge cases (quirks.moment_reason_attack_cursor: off, part (e) passes over the materialising rearming too, so over the target in the air the actor gets nothing from row G9 and no attack order); scenario T4 part E and its control
#[test]
fn t4e_a_phasing_in_actor_is_offered_the_attack_over_a_target_in_the_air_in_range() {
    for on in [true, false] {
        let mut w = game(on);
        let j = put(&mut w, Kind::Vehicle, "JUMPER", ALPHA, -2, 0);
        let up = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 0, 0);
        let near = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 0, 2);
        let far = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 10, 0);
        w.obj_mut(up).unwrap().pos.z += 600;
        // Control, not phasing in: no attack over the target in the air, the attack over both tanks.
        assert!(targeting::movable(&w, A, j));
        assert_eq!(ask(&w, j, obj(up)), Answer::Illegal);
        assert_ne!(act(&w, j, up), Action::Attack, "control, on {on}");
        assert_eq!(act(&w, j, near), Action::Attack, "control, on {on}");
        assert_eq!(act(&w, j, far), Action::Attack, "control, on {on}");
        materialise(&mut w, j, 100);
        // spec: S-0214 Terms (movable: not phasing in)
        assert!(!targeting::movable(&w, A, j));
        assert_eq!(ask(&w, j, obj(up)), Answer::Rearming);
        assert_eq!(act(&w, j, near), Action::Attack, "on {on}");
        assert!(is_attack_on(&click(&w, j, near), j, near), "on {on}");
        assert_eq!(act(&w, j, far), Action::Nothing, "on {on}");
        assert!(click(&w, j, far).is_empty(), "on {on}");
        if on {
            assert_eq!(act(&w, j, up), Action::Attack);
            assert!(is_attack_on(&click(&w, j, up), j, up));
        } else {
            assert_eq!(act(&w, j, up), Action::Nothing);
            assert!(click(&w, j, up).is_empty());
        }
    }
}

// spec: S-0214 Terms (movable: a mobile object that is phasing in is not movable) and S-0213 table T2 (scatter: movable; stop: movable or able to fire)
#[test]
fn a_phasing_in_unit_is_not_movable_for_the_key_commands() {
    let mut w = game(true);
    let j = put(&mut w, Kind::Vehicle, "JUMPER", ALPHA, -2, 0);
    let scatter = |w: &World| targeting::key_command(w, A, &[j], targeting::KeyCommand::Scatter);
    let stop = |w: &World| targeting::key_command(w, A, &[j], targeting::KeyCommand::Stop);
    assert_eq!(scatter(&w).len(), 1, "control");
    materialise(&mut w, j, 100);
    assert!(scatter(&w).is_empty());
    assert_eq!(stop(&w).len(), 1, "able to fire");
}

// spec: S-0214 claim 6 (e) (r78: a phasing-in actor that is not an aircraft and has no temporal beam attached keeps a target so ordered while phasing in; once phasing in ends it drops a target that the firing check then answers illegal for, without firing at it, at its first checking turn in a check tick, S-0309 claim 29) and Edge cases (quirks.moment_reason_attack_cursor on); scenario T4 part E's expected keep, drop and no shot
#[test]
fn t4e_the_target_in_the_air_is_kept_through_the_phase_in_and_dropped_unfired() {
    let mut w = game(true);
    let j = put(&mut w, Kind::Vehicle, "JUMPER", ALPHA, -2, 0);
    let up = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 0, 0);
    w.obj_mut(up).unwrap().pos.z += 600;
    step_to(&mut w, 6);
    let n = w.tick;
    // The phase-in ends in the movement update of tick n + 40; check ticks
    // n + 10 and n + 26 fall in it, n + 42 is the first after it.
    materialise(&mut w, j, 40);
    let orders = click(&w, j, up);
    assert!(is_attack_on(&orders, j, up));
    w.effects.clear();
    issue_until_held(&mut w, orders, j, Target::Object(up));
    let dropped = drop_tick(&mut w, j, Target::Object(up), 80);
    assert_eq!(
        dropped,
        Some(n + 42),
        "the first check tick after the phase-in"
    );
    assert!(!glasnost_sim::teleport::phasing_in(&w, j));
    assert_eq!(shots_by(&w, j), 0, "never fired at");
}

// spec: S-0214 claim 6 (e) (r78: a falling actor that is also phasing in is the exception: a falling vehicle that is phasing in gets the materialising rearming, which outranks its falling illegal, S-0301 claim 9, so part (e) holds for it as for any phasing-in actor) and Edge cases (quirks.moment_reason_attack_cursor: off, part (e) passes over the rearming and the falling illegal fails it)
#[test]
fn a_falling_vehicle_that_is_phasing_in_holds_part_e() {
    for on in [true, false] {
        let mut w = game(on);
        let j = put(&mut w, Kind::Vehicle, "JUMPER", ALPHA, -2, 0);
        let near = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 0, 0);
        w.obj_mut(j).unwrap().fall.falling = true;
        assert_eq!(ask(&w, j, obj(near)), Answer::Illegal);
        assert_ne!(act(&w, j, near), Action::Attack, "falling, on {on}");
        materialise(&mut w, j, 100);
        assert_eq!(ask(&w, j, obj(near)), Answer::Rearming);
        assert_eq!(act(&w, j, near) == Action::Attack, on, "on {on}");
    }
}

// ---------------------------------------------------------------------------
// The early rearming answers of rank 4 and S-0306 claim 23's land row.
// ---------------------------------------------------------------------------

// spec: S-0214 claim 6 (e) (r78: S-0306 claim 23's water and land rows come after the materialising rearming within rank 4, so a phasing-in actor is offered the attack over a target in range that the land row refuses; the same holds for an own building's pending delayed shot, S-0301 claim 14; the rearm countdown, rank 5, comes after both rows, so a firer that is only waiting out its countdown is not offered the attack over such a target) and Edge cases (quirks.moment_reason_attack_cursor reaches these answers too: off, no attack)
#[test]
fn the_land_row_outranks_the_countdown_but_not_the_early_rearming_answers() {
    for on in [true, false] {
        let mut w = game(on);
        let tower = put(&mut w, Kind::Building, "LTOWER", ALPHA, -2, 0);
        let lander = put(&mut w, Kind::Vehicle, "LANDER", ALPHA, -2, 4);
        let near = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 0, 0);
        let close = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 0, 4);
        // Control: the land row refuses both targets.
        assert_eq!(ask(&w, tower, obj(near)), Answer::Illegal);
        assert_eq!(ask(&w, lander, obj(close)), Answer::Illegal);
        assert_ne!(act(&w, tower, near), Action::Attack, "control, on {on}");
        assert_ne!(act(&w, lander, close), Action::Attack, "control, on {on}");
        // Only the countdown: the row outranks it.
        let t = w.tick;
        w.obj_mut(tower).unwrap().combat.rearm_until = t + 50;
        w.obj_mut(lander).unwrap().combat.rearm_until = t + 50;
        assert_eq!(ask(&w, tower, obj(near)), Answer::Illegal);
        assert_ne!(act(&w, tower, near), Action::Attack, "countdown, on {on}");
        assert_ne!(act(&w, lander, close), Action::Attack, "countdown, on {on}");
        // The building's pending delayed shot and the materialising rearming come before the row.
        w.obj_mut(tower).unwrap().combat.delayed_shot = Some(t + 20);
        materialise(&mut w, lander, 100);
        assert_eq!(ask(&w, tower, obj(near)), Answer::Rearming);
        assert_eq!(ask(&w, lander, obj(close)), Answer::Rearming);
        assert_eq!(act(&w, tower, near) == Action::Attack, on, "on {on}");
        assert_eq!(is_attack_on(&click(&w, tower, near), tower, near), on);
        assert_eq!(act(&w, lander, close) == Action::Attack, on, "on {on}");
        assert_eq!(is_attack_on(&click(&w, lander, close), lander, close), on);
    }
}

// spec: S-0214 claim 6 (e) (r78: S-0306 claim 23's water and land rows come after the materialising rearming within rank 4 and the rearm countdown, rank 5, comes after both, so a firer only waiting out its countdown is not offered the attack over a target the water row refuses, while a phasing-in one is) and Edge cases (quirks.moment_reason_attack_cursor: off, no attack); S-0306 claim 23 and S-0308 table N (the water row: a target not on a bridge, on a water or beach cell, with naval mode 6 giving "none" for every target)
#[test]
fn the_water_row_outranks_the_countdown_but_not_the_materialising_rearming() {
    use glasnost_map::terrain::land;
    for on in [true, false] {
        let mut w = game(on);
        let n = put(&mut w, Kind::Vehicle, "NAVALER", ALPHA, -2, 4);
        let wet = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 0, 4);
        let dry = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 0, 6);
        let c = w.obj(wet).unwrap().pos.cell();
        w.cells.grid.get_mut(c).unwrap().land = land::WATER;
        // Control: the water row refuses the target on water only.
        assert_eq!(ask(&w, n, obj(wet)), Answer::Illegal);
        assert_ne!(act(&w, n, wet), Action::Attack, "control, on {on}");
        assert_ne!(ask(&w, n, obj(dry)), Answer::Illegal, "dry land");
        // Only the countdown: the row outranks it.
        let t = w.tick;
        w.obj_mut(n).unwrap().combat.rearm_until = t + 50;
        assert_eq!(ask(&w, n, obj(wet)), Answer::Illegal, "countdown");
        assert_ne!(act(&w, n, wet), Action::Attack, "countdown, on {on}");
        // The materialising rearming comes before the row.
        materialise(&mut w, n, 100);
        assert_eq!(ask(&w, n, obj(wet)), Answer::Rearming);
        assert_eq!(act(&w, n, wet) == Action::Attack, on, "on {on}");
        assert_eq!(is_attack_on(&click(&w, n, wet), n, wet), on);
    }
}

// spec: S-0214 claim 6 (e) (r78: the early rearming answers of rank 4 rank above the tests of ranks 9 and 12 too, S-0301 claim 9, so a target that only those tests refuse gets the same as for the countdown: here an own building's pending delayed shot over a target its warhead's versus value of zero cannot affect) and Edge cases (quirks.moment_reason_attack_cursor: off, no attack)
#[test]
fn a_pending_delayed_shot_hides_the_zero_versus_test() {
    for on in [true, false] {
        let mut r = rules_with(on);
        r.sim.buildings.get_mut("TOWER").unwrap().primary = Some("W".into());
        let mut w = testkit::game_with(&r, SIZE, 1);
        let tower = put(&mut w, Kind::Building, "TOWER", ALPHA, -2, 0);
        let z = put(&mut w, Kind::Vehicle, "TARGET", BETA, 0, 0);
        assert_eq!(ask(&w, tower, obj(z)), Answer::Illegal);
        assert_ne!(act(&w, tower, z), Action::Attack, "control, on {on}");
        let t = w.tick;
        w.obj_mut(tower).unwrap().combat.delayed_shot = Some(t + 20);
        assert_eq!(ask(&w, tower, obj(z)), Answer::Rearming);
        assert_eq!(act(&w, tower, z) == Action::Attack, on, "on {on}");
    }
}
