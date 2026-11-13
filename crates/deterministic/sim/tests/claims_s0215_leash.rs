//! The area-guard leash of S-0215 F5 for a type without a guard range:
//! *g* falls back to the larger of the primary and secondary weapon ranges
//! (#164). On the testkit's synthetic combat ruleset: invented types,
//! weapons and numbers, no game data.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{ArmourClass, Decimal, UnitType};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 30;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// The combat ruleset plus T0, an unarmed wheeled target vehicle.
fn base() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut t = UnitType {
        strength: Some(100),
        speed: Some(5),
        rot: Some(5),
        locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Wheel),
        locomotor: Some(glasnost_rules::sim::Locomotor::Drive),
        ..Default::default()
    };
    t.combat.armor = Some(ArmourClass::Light);
    r.sim.vehicles.insert("T0".into(), t);
    r.registries
        .get_mut("vehicles")
        .expect("vehicles registry")
        .push("T0".into());
    r
}

fn veh(w: &mut World, ty: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, ty, house, at(dx, dy), 0)
}

/// An armed CART (no guard range unless `guard_cells`) in area guard, 2
/// cells (512 leptons) from its guard spot, holding an enemy target, with
/// its targeting timer far from running out; one tick. Returns its
/// target, its destination, the enemy and its two weapon ranges.
fn run(guard_cells: Option<&str>) -> (Option<Target>, Option<Target>, Target, i32, i32) {
    let mut r = base();
    r.sim.vehicles.get_mut("CART").expect("CART").guard_range = guard_cells.map(d);
    let mut w = testkit::game_with(&r, SIZE, 11);
    let a = veh(&mut w, "CART", 0, 0, 0);
    let t = w.tick;
    w.obj_mut(a)
        .unwrap()
        .beh
        .force(Behaviour::AreaGuard, t, true);
    w.obj_mut(a).unwrap().guard_spot = Some(Target::Cell(at(-2, 0)));
    let b = veh(&mut w, "T0", 1, 2, 0);
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    w.obj_mut(a).unwrap().targeting_timer = 1000;
    let w0 = glasnost_sim::combat::range::firer_weapon_range(&w, a, 0);
    let w1 = glasnost_sim::combat::range::firer_weapon_range(&w, a, 1);
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    (o.target, o.dest, Target::Object(b), w0, w1)
}

// spec: S-0215 F5 and claim 26 (control: g = 2 cells, L = ⌊11 × 1024 / 10⌋ = 1126, the spot 512 leptons away: the leash does not hold)
#[test]
fn f5_guard_range_two_cells_keeps_the_target() {
    let (target, dest, b, _, _) = run(Some("2"));
    assert_eq!(target, Some(b));
    assert_ne!(dest, Some(Target::Cell(at(-2, 0))));
}

// spec: S-0215 F5 (g = 0: the larger of the primary and secondary weapon ranges in leptons) and claim 26
#[test]
fn f5_no_guard_range_uses_the_weapon_range() {
    let (target, dest, b, w0, w1) = run(None);
    let g = w0.max(w1);
    // spec: S-0215 F5 (L = ⌊11 × min(2g, 4096) / 10⌋)
    let l = 11 * (2 * g).clamp(0, 4096) / 10;
    assert!(l > 512, "weapon ranges {w0}, {w1}: L = {l}");
    assert_eq!(
        target,
        Some(b),
        "L = {l}: the leash does not hold, the target is kept"
    );
    assert_ne!(dest, Some(Target::Cell(at(-2, 0))));
}
