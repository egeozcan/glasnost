//! Claim tests for target acquisition (S-0309), and for the parts of the
//! behaviours spec (S-0215 claims 18, 26, 27 and 33) that decide searches
//! and retaliation, on synthetic data.
//!
//! Every type, weapon, warhead and number here is invented. The searches
//! are mostly called directly (`acquire::search`, `acquire::retaliate`) on a
//! world in which no tick has run, so that no other search, processing
//! draw or shot interferes. The ledger is `docs/m4-claims/S-0309.md`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{ArmourClass, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

/// Map size of the tests (cells around `mid(SIZE, 0, 0)` are legal).
const SIZE: i32 = 30;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// The combat ruleset of the testkit with a few invented additions:
/// - weapon `Half` (damage 10, range 5 cells, projectile Bullet, warhead
///   `Halver` with every versus value 0.5),
/// - vehicle types `T0` … `T9`: unarmed light targets of strength 100 whose
///   special threat value is set per test,
/// - the five threat coefficients of CART set to 0 explicitly.
fn base() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.warheads.insert(
        "Halver".into(),
        WarheadType {
            verses: Some(vec![d("0.5"); 11]),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Half".into(),
        WeaponType {
            damage: Some(10),
            rof: Some(20),
            range: Some(d("5")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("Halver".into()),
            ..Default::default()
        },
    );
    for i in 0..10 {
        let name = format!("T{i}");
        let mut t = UnitType {
            strength: Some(100),
            speed: Some(5),
            rot: Some(5),
            locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Wheel),
            locomotor: Some(glasnost_rules::sim::Locomotor::Drive),
            ..Default::default()
        };
        t.combat.armor = Some(ArmourClass::Light);
        s.vehicles.insert(name.clone(), t);
        r.registries
            .get_mut("vehicles")
            .expect("vehicles registry")
            .push(name);
    }
    coefficients(&mut r, "CART", ["0", "0", "0", "0", "0"]);
    r
}

/// Sets the five threat coefficients of a vehicle type (own-effectiveness,
/// target-effectiveness, special-threat, strength, distance).
fn coefficients(r: &mut Ruleset, ty: &str, c: [&str; 5]) {
    let u = r.sim.vehicles.get_mut(ty).expect("vehicle type");
    u.combat.coefficients = Some(c.map(d));
}

/// Sets the guard range of a vehicle type, in cells.
fn guard(r: &mut Ruleset, ty: &str, cells: &str) {
    r.sim
        .vehicles
        .get_mut(ty)
        .expect("vehicle type")
        .guard_range = Some(d(cells));
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 11)
}

fn put(w: &mut World, kind: Kind, ty: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, ty, house, at(dx, dy), 0)
}

fn veh(w: &mut World, ty: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    put(w, Kind::Vehicle, ty, house, dx, dy)
}

/// The per-tick range search of claim 1's first row, called directly.
fn range_search(w: &mut World, a: ObjectId) -> Option<ObjectId> {
    acquire::search(w, a, SearchKind::Range, SearchContext::PerTick)
}

fn target(w: &World, a: ObjectId) -> Option<Target> {
    w.obj(a).and_then(|o| o.target)
}

fn obj(x: ObjectId) -> Option<Target> {
    Some(Target::Object(x))
}

fn opportunity(w: &World, a: ObjectId) -> bool {
    w.obj(a).is_some_and(|o| o.combat.opportunity)
}

// ---------------------------------------------------------------------------
// Claim 24 and table "Search outcome and target state"

// spec: S-0309 claim 24 and table Search outcome row 1 (a dropped opportunity target that the same search picks again is not an opportunity target)
#[test]
fn repicked_opportunity_target_is_not_an_opportunity_target() {
    let mut r = base();
    // Guard range 8 cells (2048 leptons) reaches beyond the gun's 5.5 cells.
    guard(&mut r, "CART", "8");
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 7, 0);
    assert_eq!(range_search(&mut w, a), Some(b));
    assert!(opportunity(&w, a), "a new target of the per-tick search");
    // Second search: the opportunity target is out of the gun's range, so it is
    // dropped, and the same search picks it again.
    assert_eq!(range_search(&mut w, a), Some(b));
    assert_eq!(target(&w, a), obj(b));
    assert!(
        !opportunity(&w, a),
        "the object that was the target before the search is not an opportunity target"
    );
}

// spec: S-0309 claim 24 (setting the object itself gives its own cell)
#[test]
fn redirect_self_gives_own_cell() {
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    assert_eq!(
        acquire::redirect(&w, a, Target::Object(a)),
        Some(Target::Cell(at(0, 0)))
    );
}

// spec: S-0309 claim 24 (a dead object, or an infantry unit whose current sequence is a death kind, gives no target; others are held as set)
#[test]
fn redirect_dead_or_dying_gives_no_target() {
    use glasnost_sim::infantry::Seq;
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    let i = put(&mut w, Kind::Infantry, "WALKER", 1, 3, 0);
    assert_eq!(acquire::redirect(&w, a, Target::Object(b)), obj(b));
    assert_eq!(
        acquire::redirect(&w, a, Target::Cell(at(4, 4))),
        Some(Target::Cell(at(4, 4)))
    );
    for seq in [
        Seq::Die1,
        Seq::Die2,
        Seq::Die3,
        Seq::Die4,
        Seq::Die5,
        Seq::WetDie1,
        Seq::WetDie2,
        Seq::AirDeathStart,
        Seq::AirDeathFall,
        Seq::AirDeathEnd,
    ] {
        w.obj_mut(i).unwrap().infantry.as_mut().unwrap().seq = Some(seq);
        assert_eq!(acquire::redirect(&w, a, Target::Object(i)), None, "{seq:?}");
    }
    w.obj_mut(i).unwrap().infantry.as_mut().unwrap().seq = Some(Seq::Stand);
    assert_eq!(acquire::redirect(&w, a, Target::Object(i)), obj(i));
    w.obj_mut(b).unwrap().health = 0;
    assert_eq!(acquire::redirect(&w, a, Target::Object(b)), None, "dead");
}

// ---------------------------------------------------------------------------
// Claim 8 and F1

// spec: S-0309 F1 (R = 0: n = trunc(w / 256) + 1 + trunc(b / 256); a type that is underwater, organic and self-healing uses g for w)
#[test]
fn underwater_organic_self_healing_type_uses_guard_range_for_rings() {
    let mut r = base();
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.combat.underwater = Some(true);
    cart.combat.organic = Some(true);
    cart.combat.self_healing = Some(true);
    let mut w = game(&r);
    let a = veh(&mut w, "CART", 0, 0, 0);
    // Within the gun's 5.5 cells, but in ring 2: with g = 0, n = 0 + 1 + 0 = 1 looks at ring 0 only.
    veh(&mut w, "T0", 1, 2, 0);
    assert_eq!(range_search(&mut w, a), None);
    // Without the three flags, n = trunc(1408 / 256) + 1 = 6 reaches ring 2.
    let mut w = game(&base());
    let a = veh(&mut w, "CART", 0, 0, 0);
    let b = veh(&mut w, "T0", 1, 2, 0);
    assert_eq!(range_search(&mut w, a), Some(b));
}
