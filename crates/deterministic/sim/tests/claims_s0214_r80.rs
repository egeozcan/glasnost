//! Claim tests of S-0214 revision 80 (Edge cases, *Attack offered over a
//! reason of the moment*: which answers part (e) passes over with
//! `quirks.moment_reason_attack_cursor` off, Q-2426 answered), M8 step A3
//! intake 90. Synthetic data only: every type, weapon, warhead and number
//! is invented. The ledger is `docs/m8-claims/intake90.md`.
//!
//! Each test sets up one of the four cases the Edge case names. With the
//! switch on (the original), an answer that is not *illegal* is the
//! highest rank that holds, so part (e) holds and the click gives the
//! attack order. With the switch off, part (e) passes over that answer and
//! a reason that gives *illegal* ranked below it fails part (e).

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    ArmourClass, Decimal, LandRow, LandType, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::spawn::{Slot, SlotState, SpawnController};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
const A: HouseId = HouseId(ALPHA);
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
///   with weapon `W`; warhead `PLAIN` (1 everywhere) with weapon `Plain`
///   and with the healing weapon `Heal` (damage below 0, S-0310 claim 11);
/// - vehicles `DEF` (primary `W`), `GUN` (primary `Plain`), `TARGET`
///   (class *light*, unarmed), `PLAINV` (class *heavy*, unarmed) and
///   `MEDV` (the deploy-to-fire and attack-friendlies flags, primary
///   `Heal`);
/// - clear land with the buildable flag.
fn rules(on: bool) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    warhead(&mut r, "H", |h| h.verses = only(LIGHT, "0"));
    warhead(&mut r, "PLAIN", |_| {});
    weapon(&mut r, "W", "H", |_| {});
    weapon(&mut r, "Plain", "PLAIN", |_| {});
    weapon(&mut r, "Heal", "PLAIN", |x| x.damage = Some(-20));
    vehicle(&mut r, "DEF", |t| t.primary = Some("W".into()));
    vehicle(&mut r, "GUN", |t| t.primary = Some("Plain".into()));
    vehicle(&mut r, "TARGET", |t| {
        t.combat.armor = Some(ArmourClass::Light)
    });
    vehicle(&mut r, "PLAINV", |t| {
        t.combat.armor = Some(ArmourClass::Heavy)
    });
    vehicle(&mut r, "MEDV", |t| {
        t.primary = Some("Heal".into());
        t.deploy_to_fire = Some(true);
        t.attack_friendlies = Some(true);
    });
    // Clear land is buildable, so a flat clear cell passes the deploy-to-fire cell test (S-0517 claim 14).
    r.sim.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    r.globals.quirks.moment_reason_attack_cursor = on;
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn game(on: bool) -> World {
    testkit::game_with(&rules(on), SIZE, 1)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn obj(x: ObjectId) -> Option<Target> {
    Some(Target::Object(x))
}

/// The firing check's answer without the range test (part (e) with the
/// switch on).
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

/// Raises `x` by `dz` leptons above the ground under it.
fn raise(w: &mut World, x: ObjectId, dz: i32) {
    let p = w.obj(x).unwrap().pos;
    let ground = w.surface_at(p.x, p.y);
    w.obj_mut(x).unwrap().pos.z = ground + dz;
}

/// Asserts what part (e) gives for `actor` over `t` in each position of
/// the switch: on, the answer `on_answer` (not *illegal*) holds part (e)
/// and the click gives the attack order; off, an *illegal* reason at a
/// lower rank fails it, and no attack order is given.
fn assert_part_e(w: &World, on: bool, actor: ObjectId, t: ObjectId, on_answer: Answer) {
    assert_eq!(ask(w, actor, obj(t)), on_answer, "the check's answer");
    assert_ne!(on_answer, Answer::Illegal);
    assert!(
        check::illegal_at_any_rank(w, actor, obj(t), 0),
        "an illegal reason holds at a lower rank"
    );
    assert_eq!(act(w, actor, t) == Action::Attack, on, "action, on {on}");
    assert_eq!(
        is_attack_on(&click(w, actor, t), actor, t),
        on,
        "click, on {on}"
    );
}

// spec: S-0214 r80 Edge cases (Q-2426 answered, question 1: with quirks.moment_reason_attack_cursor off, part (e) passes over every rearming of rank 4, the own-gripped-unit rearming of S-0607 claim 11 among them, so part (e) fails for a lifter over its own gripped unit in the air whose weapon cannot hit a unit in the air; on, the grip's rearming outranks the projectile refusal and part (e) holds) and S-0306 claim 23 (the firer's gripped unit)
#[test]
fn off_a_lifter_over_its_gripped_unit_in_the_air_fails_part_e() {
    for on in [true, false] {
        let mut w = game(on);
        let f = put(&mut w, Kind::Vehicle, "GUN", ALPHA, -1, 0);
        let up = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 1, 0);
        raise(&mut w, up, 600);
        // Control: not gripped, the projectile refusal is the answer.
        assert_eq!(ask(&w, f, obj(up)), Answer::Illegal, "control");
        assert_ne!(act(&w, f, up), Action::Attack, "control, on {on}");
        w.obj_mut(f).unwrap().lift.gripped = Some(up);
        w.obj_mut(up).unwrap().lift.lifter = Some(f);
        assert!(glasnost_sim::lift::grips(&w, f, up));
        assert_part_e(&w, on, f, up, Answer::Rearming);
    }
}

// spec: S-0214 r80 Edge cases (Q-2426 answered, question 2: with quirks.moment_reason_attack_cursor off, part (e) passes over every impossible of rank 4, the hidden target's of S-0516 claim 24 among them, so it fails over a hidden target that a later test rules illegal, here the zero-versus test of S-0304 claim 1; on, the hidden target's impossible is the answer and part (e) holds)
#[test]
fn off_a_hidden_target_that_a_later_test_rules_illegal_fails_part_e() {
    for on in [true, false] {
        let mut w = game(on);
        let d = put(&mut w, Kind::Vehicle, "DEF", ALPHA, -1, 0);
        let z = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
        // Control: seen, the zero-versus test answers illegal.
        assert_eq!(ask(&w, d, obj(z)), Answer::Illegal, "control");
        w.obj_mut(z).unwrap().cloak.state = glasnost_sim::cloak::CloakState::Cloaked;
        assert!(glasnost_sim::cloak::hidden_from(&w, z, Some(A)));
        assert_part_e(&w, on, d, z, Answer::Impossible);
    }
}

// spec: S-0214 r80 Edge cases (Q-2426 answered, question 2: with quirks.moment_reason_attack_cursor off, part (e) passes over busy, rank 2 of S-0301 claim 9, so it fails for a vehicle that is launching spawned aircraft over a target that a later reason rules illegal, here the zero-versus test; on, busy is the answer and part (e) holds) and S-0506 claim 10 (a launching count above 0: busy)
#[test]
fn off_a_launching_vehicle_over_an_illegal_target_fails_part_e() {
    for on in [true, false] {
        let mut w = game(on);
        let d = put(&mut w, Kind::Vehicle, "DEF", ALPHA, -1, 0);
        let z = put(&mut w, Kind::Vehicle, "TARGET", BETA, 2, 0);
        let a = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 2, 2);
        assert_eq!(ask(&w, d, obj(z)), Answer::Illegal, "control");
        assert_ne!(act(&w, d, z), Action::Attack, "control, on {on}");
        // One slot on its launch rail: the launching count is 1.
        w.obj_mut(d).unwrap().spawn = Some(SpawnController {
            slots: vec![Slot {
                state: SlotState::OnRail,
                object: None,
                until: 0,
            }],
            ..Default::default()
        });
        assert_eq!(glasnost_sim::spawn::launching_count(&w, d), 1);
        assert_part_e(&w, on, d, z, Answer::Busy);
        // A target no reason rules illegal: the attack in both positions.
        assert_eq!(ask(&w, d, obj(a)), Answer::Busy);
        assert!(!check::illegal_at_any_rank(&w, d, obj(a), 0));
        assert_eq!(act(&w, d, a), Action::Attack, "over A, on {on}");
    }
}

// spec: S-0214 r80 Edge cases (Q-2426 answered, question 2: with quirks.moment_reason_attack_cursor off, part (e) passes over must deploy, rank 11 of S-0301 claim 9, so it fails for a deploy-to-fire healer vehicle whose cell fails the deploy test over a target that rank 12's healer illegal rules out, S-0310 claim 11; on, must deploy is the answer and part (e) holds) and S-0517 claim 14 (the deploy-to-fire cell test: a cell that is not flat fails)
#[test]
fn off_a_deploy_to_fire_healer_on_a_failing_cell_fails_part_e() {
    for on in [true, false] {
        let mut w = game(on);
        let m = put(&mut w, Kind::Vehicle, "MEDV", ALPHA, -1, 0);
        // An own vehicle at full health: the healer's illegal of rank 12.
        // MEDV may attack as a friend, so the click reaches the attack rule
        // (claim 6 (a)); over an enemy a healing vehicle gets nothing from
        // table V row b1 whatever part (e) gives.
        let v = put(&mut w, Kind::Vehicle, "PLAINV", ALPHA, 2, 0);
        // Control: the cell passes, so the healer's illegal is the answer.
        assert!(glasnost_sim::econ::inplace::deploy_test(&w, m));
        assert_eq!(ask(&w, m, obj(v)), Answer::Illegal, "control");
        assert_ne!(act(&w, m, v), Action::Attack, "control, on {on}");
        let c = w.obj(m).unwrap().pos.cell();
        w.cells.grid.get_mut(c).unwrap().slope = 1;
        assert!(!glasnost_sim::econ::inplace::deploy_test(&w, m));
        assert_part_e(&w, on, m, v, Answer::MustDeploy);
    }
}

// spec: S-0214 r80 Edge cases (Q-2426 answered: two stops remain with quirks.moment_reason_attack_cursor off: with no weapon in the slot nothing further is asked and part (e) holds, and the range test is not asked, part (d) being the range part)
#[test]
fn off_the_two_stops_remain() {
    let mut w = game(false);
    let g = put(&mut w, Kind::Vehicle, "GUN", ALPHA, -1, 0);
    let far = put(&mut w, Kind::Vehicle, "PLAINV", BETA, 12, 0);
    // Out of range, nothing illegal: part (e) does not ask the range test.
    assert_eq!(check::check(&w, g, obj(far), 0, true), Answer::OutOfRange);
    assert!(!check::illegal_at_any_rank(&w, g, obj(far), 0));
    assert_eq!(act(&w, g, far), Action::Attack, "movable: part (d) holds");
    // No weapon in the slot: no illegal answer.
    let none = put(&mut w, Kind::Vehicle, "PLAINV", ALPHA, -1, 2);
    assert!(!check::illegal_at_any_rank(&w, none, obj(far), 0));
}
