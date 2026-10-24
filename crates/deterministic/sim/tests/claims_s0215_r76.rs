//! Claim tests for S-0215 revision 76 (claim 32, the non-SAM *attack* of
//! a building, and scenario B16 case e): a building whose target leaves its
//! weapon's range drops it and returns to *guard* at the first processing
//! at which the firing check answers *out of range*. While its rearm
//! countdown runs that check answers *rearming* (S-0301 claim 9 rank 5,
//! ahead of the range test at rank 10), and the building keeps its target.
//! The pending animation-delayed shot's case is in `claims_s0301_delayed.rs`
//! (`pending_shot_keeps_a_target_driven_out_of_range`).
//!
//! Synthetic data: the testkit's combat ruleset with an invented turret-less
//! building `GRD` (an invisible instant shot, a long rate of fire) and the
//! cart made unarmed and durable. The target is moved out of range by hand
//! on the tick of a shot (B16 e drives it with an order; our test moves it,
//! so that it leaves range while the countdown runs). Ledger:
//! `docs/m8-claims/intake10-S-0519-S-0215.md`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{ArmourClass, BuildingType, Decimal, WeaponType};
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::combat::weapon::choose_slot;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, Target, World};

const SIZE: i32 = 24;
/// The weapon's rate of fire: long enough that many processings fall
/// inside one countdown.
const ROF: i32 = 120;

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.weapons.insert(
        "Pin".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(ROF),
            range: Some(Decimal("5".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    let mut grd = BuildingType {
        strength: Some(800),
        foundation: Some([1, 1]),
        primary: Some("Pin".into()),
        ..Default::default()
    };
    grd.combat.armor = Some(ArmourClass::Concrete);
    grd.combat.primary_fire_flh = Some([0, 0, 100]);
    grd.econ.sight = Some(12);
    s.buildings.insert("GRD".into(), grd);
    r.registries
        .get_mut("buildings")
        .expect("buildings registry")
        .push("GRD".into());
    let cart = s.vehicles.get_mut("CART").expect("the cart");
    cart.primary = None;
    cart.strength = Some(30000);
    r
}

/// Moves `x` to the centre of `c` by hand, keeping the cell listing.
fn relocate(w: &mut World, x: ObjectId, c: CellXY) {
    let from = w.obj(x).unwrap().pos;
    w.cells.leave(from.cell(), Layer::Ground, x);
    let p = Pos::centre(c, 0);
    let z = w.surface_at(p.x, p.y);
    w.obj_mut(x).unwrap().pos = Pos::new(p.x, p.y, z);
    w.cells.enter(c, Layer::Ground, x, false);
}

fn answer(w: &World, b: ObjectId, x: ObjectId) -> Answer {
    let t = Target::Object(x);
    let slot = choose_slot(w, b, Some(t));
    check(w, b, Some(t), slot, true)
}

/// What one tick of the building shows after it ran.
#[derive(Debug, PartialEq, Eq)]
struct State {
    target: Option<Target>,
    attack: bool,
    guard: bool,
}

fn state(w: &World, b: ObjectId) -> State {
    let o = w.obj(b).unwrap();
    State {
        target: o.target,
        attack: o.beh.current == Some(Behaviour::Attack),
        guard: o.beh.current == Some(Behaviour::Guard) || o.beh.queued == Some(Behaviour::Guard),
    }
}

/// A world with `GRD` of house 0 at the centre attacking the cart of house
/// 1 three cells east; runs to the tick of its first shot and returns the
/// world, the building, the cart and the countdown's end.
fn shot_fired() -> (World, ObjectId, ObjectId, i32) {
    let r = ruleset();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let b = testkit::spawn(&mut w, Kind::Building, "GRD", 0, at(0, 0), 0);
    let x = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(3, 0), 0);
    testkit::run(&mut w, 6);
    let order = Order::Behaviour {
        actor: b,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(x)),
        dest: None,
    };
    assert!(orders::issue(&mut w, HouseId(0), order), "order taken");
    let start = w.obj(b).unwrap().combat.rearm_until;
    for _ in 0..30 {
        testkit::run(&mut w, 1);
        let until = w.obj(b).unwrap().combat.rearm_until;
        if until != start {
            return (w, b, x, until);
        }
    }
    panic!("the building never fired");
}

// spec: S-0215 claim 32 (revision 76) and scenario B16 case e (the target leaves the weapon's range while the rearm countdown runs: at every processing whose firing check answers rearming the building keeps its target and attack; at the first one that answers out of range it drops the target and guard is current, within 2 ticks, claim 32's delay table); S-0301 claim 9 (rearming at rank 5 comes ahead of the range test at rank 10)
#[test]
fn b16e_target_kept_out_of_range_while_rearming() {
    let (mut w, b, x, until) = shot_fired();
    assert!(until - w.tick > 60, "a long countdown remains");
    // Out of range (5 cells), still in sight.
    relocate(&mut w, x, at(8, 0));
    let mut kept_ticks = 0;
    while w.tick + 1 < until {
        assert_eq!(answer(&w, b, x), Answer::Rearming, "tick {}", w.tick);
        testkit::run(&mut w, 1);
        let s = state(&w, b);
        assert_eq!(
            s,
            State {
                target: Some(Target::Object(x)),
                attack: true,
                guard: false
            },
            "kept while rearming, tick {}",
            w.tick
        );
        kept_ticks += 1;
    }
    assert!(kept_ticks > 60, "kept through the countdown");
    // The countdown over, the check answers out of range, and the next
    // processing (1 or 2 ticks, claim 32) drops the target.
    testkit::run(&mut w, 1);
    assert_eq!(answer(&w, b, x), Answer::OutOfRange);
    let mut dropped = None;
    for k in 0..3 {
        testkit::run(&mut w, 1);
        let s = state(&w, b);
        if s.target.is_none() {
            assert!(s.guard, "guard with the target dropped");
            dropped = Some(k);
            break;
        }
    }
    assert!(
        dropped.is_some(),
        "dropped at the first out-of-range answer"
    );
}

// spec: S-0215 claim 32 (revision 76; the control: a target still in range is kept and shot at again once the countdown ends)
#[test]
fn b16e_control_in_range_target_is_shot_again() {
    let (mut w, b, x, until) = shot_fired();
    while w.tick < until + 3 {
        testkit::run(&mut w, 1);
        assert_eq!(w.obj(b).unwrap().target, Some(Target::Object(x)));
    }
    assert_ne!(
        w.obj(b).unwrap().combat.rearm_until,
        until,
        "a second shot started a new countdown"
    );
}
