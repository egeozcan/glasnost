//! Claim tests for a building's pending delayed shot against a target that
//! becomes refused during the delay (S-0301 revision 36, claims 9 and 14,
//! scenario F21; Q-1770 answered), with S-0306 claim 23's rows and S-0308
//! claim 14's weapon choice, on synthetic data (M8 step A3 intake 8).
//!
//! Every type, weapon, warhead and number here is invented. A shore
//! crossing is made by changing the land type of the cell the target stands
//! on (the check reads only that cell's land type, S-0306 claim 23), and a
//! lift-off by setting the target's height by hand; F21 itself permits no
//! state writes, these tests are of our engine. The render-layer row of
//! S-0306 claim 23 is not modelled (every unit is in the ground layer), so
//! part e is tested through the airborne row only. The ledger is
//! `docs/m8-claims/intake8-S-0201-S-0301.md`.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    ArmourClass, BuildingType, Decimal, LocomotionClass, Locomotor, ProjectileType, UnitType,
    WarheadType, WeaponType,
};
use glasnost_sim::combat::behave;
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 30;
/// The delay field of every part but f (F21's 30 ticks).
const DELAY: i32 = 30;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// A slow visible projectile weapon of range 8 cells, damage 1, with a
/// rate of fire longer than any run here.
fn weapon(projectile: &str) -> WeaponType {
    WeaponType {
        damage: Some(1),
        rof: Some(2000),
        range: Some(d("8")),
        speed: Some(8),
        projectile: Some(projectile.into()),
        warhead: Some("Even".into()),
        ..Default::default()
    }
}

/// The testkit's combat ruleset plus
/// - projectiles `SlowA` and `SlowB` (visible, steering, anti-ground, not
///   anti-air: a slow shot stays in flight long enough to be seen),
///   warhead `Even` (every versus value 1),
/// - weapons `ShotA` (projectile `SlowA`) and `ShotB` (projectile `SlowB`),
///   of equal range,
/// - the vehicle `BOAT` (unarmed, durable),
/// - the building `DLY` (animation-delayed, delay field `delay`, primary
///   `ShotA`, secondary `ShotB`, land mode `lm`, naval mode `nm`).
fn rules(delay: i32, lm: i32, nm: i32) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    for name in ["SlowA", "SlowB"] {
        s.projectiles.insert(
            name.into(),
            ProjectileType {
                aa: Some(false),
                ag: Some(true),
                rot: Some(12),
                acceleration: Some(4),
                ..Default::default()
            },
        );
    }
    s.warheads.insert(
        "Even".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    s.weapons.insert("ShotA".into(), weapon("SlowA"));
    s.weapons.insert("ShotB".into(), weapon("SlowB"));
    let mut boat = UnitType {
        strength: Some(30000),
        speed: Some(5),
        rot: Some(5),
        sight: Some(9),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    };
    boat.combat.armor = Some(ArmourClass::Light);
    s.vehicles.insert("BOAT".into(), boat);
    r.registries
        .get_mut("vehicles")
        .expect("vehicles registry")
        .push("BOAT".into());
    let mut dly = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        primary: Some("ShotA".into()),
        secondary: Some("ShotB".into()),
        is_anim_delayed_fire: Some(true),
        delayed_fire_delay: Some(delay),
        ..Default::default()
    };
    dly.combat.armor = Some(ArmourClass::Light);
    // Fire points above the ground, so that a shot is not at once a hit on the ground.
    dly.combat.primary_fire_flh = Some([0, 0, 200]);
    dly.combat.secondary_fire_flh = Some([0, 0, 200]);
    dly.combat.land_targeting = Some(lm);
    dly.combat.naval_targeting = Some(nm);
    dly.econ.sight = Some(9);
    s.buildings.insert("DLY".into(), dly);
    r.registries
        .get_mut("buildings")
        .expect("buildings registry")
        .push("DLY".into());
    r
}

fn set_land(w: &mut World, c: CellXY, l: u8) {
    w.cells.grid.get_mut(c).expect("legal cell").land = l;
}

fn lift(w: &mut World, x: ObjectId, dz: i32) {
    let p = w.obj(x).unwrap().pos;
    let ground = w.surface_at(p.x, p.y);
    w.obj_mut(x).unwrap().pos.z = ground + dz;
}

fn weapon_id(w: &World, name: &str) -> u32 {
    w.rules().combat.weapon_index[name]
}

/// The projectiles that `b` has fired: their weapons and targets.
fn shots_of(w: &World, b: ObjectId) -> Vec<(u32, Option<Target>)> {
    w.projectiles
        .values()
        .filter(|p| p.firer == Some(b))
        .map(|p| (p.weapon, p.target))
        .collect()
}

/// A world of `r` with the building `DLY` of house 0 at the centre and an
/// enemy `BOAT` of house 1 three cells east, on a cell of land type `l`.
/// A few ticks run first, so that orders are taken (S-0213 claim 1).
fn fixture(r: &Ruleset, l: u8) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, SIZE, 13);
    let b = testkit::spawn(&mut w, Kind::Building, "DLY", 0, at(0, 0), 0);
    let x = testkit::spawn(&mut w, Kind::Vehicle, "BOAT", 1, at(3, 0), 0);
    set_land(&mut w, at(3, 0), l);
    testkit::run(&mut w, 6);
    (w, b, x)
}

/// Orders `b` to attack `x` (S-0213 claim 5).
fn order_attack(w: &mut World, b: ObjectId, x: ObjectId) {
    let order = Order::Behaviour {
        actor: b,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(x)),
        dest: None,
    };
    assert!(
        orders::issue(w, HouseId(0), order),
        "the attack order is taken"
    );
}

/// Runs whole ticks until `b`'s delayed shot is pending; returns its due
/// tick.
fn until_pending(w: &mut World, b: ObjectId) -> i32 {
    for _ in 0..20 {
        if let Some(due) = w.obj(b).unwrap().combat.delayed_shot {
            return due;
        }
        testkit::run(w, 1);
    }
    panic!("no delayed shot started");
}

/// What a run of F21 shows from the start of the delay.
struct Run {
    /// Every tick of the delay before the one that ends it kept the
    /// target and *attack*.
    kept: bool,
    /// In the tick that ends the delay: the projectiles made, the target
    /// and the behaviour after it, and whether a rearm countdown started.
    end_shots: Vec<(u32, Option<Target>)>,
    end_target: Option<Target>,
    end_attack: bool,
    end_rearm: bool,
    /// Two ticks after that tick: the target and whether *attack* is gone.
    after_target: Option<Target>,
    after_guard: bool,
}

/// Orders `b` to attack `x`, then runs whole ticks through the delay,
/// calling `during(w, k)` before the `k`-th tick of the delay.
fn run_delay(
    w: &mut World,
    b: ObjectId,
    x: ObjectId,
    mut during: impl FnMut(&mut World, i32),
) -> Run {
    order_attack(w, b, x);
    let due = until_pending(w, b);
    let mut kept = true;
    let mut k = 0;
    while w.tick < due {
        during(w, k);
        testkit::run(w, 1);
        let o = w.obj(b).unwrap();
        kept &= o.combat.delayed_shot == Some(due) && o.beh.current == Some(Behaviour::Attack);
        kept &= o.target.is_some();
        k += 1;
    }
    // The tick that ends the delay (S-0301 claim 14: max(d − 1, 0) ticks
    // after the turn that started it).
    assert_eq!(w.tick, due);
    let rearm_before = w.obj(b).unwrap().combat.rearm_until;
    during(w, k);
    testkit::run(w, 1);
    let o = w.obj(b).unwrap();
    assert_eq!(o.combat.delayed_shot, None, "no longer pending");
    let target = w.obj(x).expect("the target is alive");
    assert!(target.on_map, "and on the map");
    assert_eq!(target.pos.cell(), at(3, 0), "in its cell");
    let end_shots = shots_of(w, b);
    let end_target = o.target;
    let end_attack = o.beh.current == Some(Behaviour::Attack);
    let end_rearm = o.combat.rearm_until != rearm_before;
    testkit::run(w, 2);
    let o = w.obj(b).unwrap();
    Run {
        kept,
        end_shots,
        end_target,
        end_attack,
        end_rearm,
        after_target: o.target,
        after_guard: o.beh.current == Some(Behaviour::Guard)
            || o.beh.queued == Some(Behaviour::Guard),
    }
}

/// The refused-at-the-end outcome of F21 parts a, b and e.
fn assert_refused(run: &Run, x: ObjectId, what: &str) {
    assert!(run.kept, "{what}: target and attack kept through the delay");
    assert!(run.end_shots.is_empty(), "{what}: no projectile");
    assert!(!run.end_rearm, "{what}: no rearm countdown");
    assert_eq!(
        run.end_target,
        Some(Target::Object(x)),
        "{what}: the processing in the end tick keeps the target"
    );
    assert!(run.end_attack, "{what}: and attack");
    assert_eq!(run.after_target, None, "{what}: dropped 2 ticks later");
    assert!(run.after_guard, "{what}: guard 2 ticks later");
}

/// The made-shot outcome: one projectile of `weapon` at `x`.
fn assert_made(run: &Run, weapon: u32, x: ObjectId, what: &str) {
    assert!(run.kept, "{what}: target and attack kept through the delay");
    assert_eq!(
        run.end_shots,
        vec![(weapon, Some(Target::Object(x)))],
        "{what}: one shot at the target"
    );
    assert!(run.end_rearm, "{what}: the countdown starts with the shot");
}

// spec: S-0301 claims 9 and 14 (scenario F21 part a: land mode 1, naval mode 0; the target drives from water onto clear land during the delay: the building keeps its target and attack in every tick of the delay, the processing in the end tick comes first and keeps them, the end-of-delay check refuses by S-0306 claim 23's land row and makes no projectile or countdown, and 2 ticks later the processing drops the target and queues guard, S-0215 claim 32)
#[test]
fn f21_a_water_to_land_drops_the_shot_and_keeps_the_target() {
    for cross in [0, 7, 15, DELAY - 2] {
        let (mut w, b, x) = fixture(&rules(DELAY, 1, 0), land::WATER);
        let run = run_delay(&mut w, b, x, |w, k| {
            if k == cross {
                set_land(w, at(3, 0), land::CLEAR);
            }
        });
        assert_refused(&run, x, &format!("part a, crossing at {cross}"));
    }
}

// spec: S-0301 claims 9 and 14 (scenario F21 part b: land mode 0, naval mode 6; the target drives from clear land onto water during the delay: kept through the delay, refused by S-0306 claim 23's water row at the end, dropped 2 ticks later)
#[test]
fn f21_b_land_to_water_drops_the_shot_and_keeps_the_target() {
    for cross in [0, 11, DELAY - 2] {
        let (mut w, b, x) = fixture(&rules(DELAY, 0, 6), land::CLEAR);
        let run = run_delay(&mut w, b, x, |w, k| {
            if k == cross {
                set_land(w, at(3, 0), land::WATER);
            }
        });
        assert_refused(&run, x, &format!("part b, crossing at {cross}"));
    }
}

// spec: S-0301 claims 9 and 14 (scenario F21 parts a and b: a crossing after the end of the delay leaves the shot to be made; part c: with the target staying on its side the shot is made, an ordinary shot of slot 0 at the target, the countdown starting with it)
#[test]
fn f21_c_control_and_late_crossing_make_the_shot() {
    for (lm, nm, l) in [(1, 0, land::WATER), (0, 6, land::CLEAR)] {
        let (mut w, b, x) = fixture(&rules(DELAY, lm, nm), l);
        let a = weapon_id(&w, "ShotA");
        let run = run_delay(&mut w, b, x, |_, _| {});
        assert_made(&run, a, x, &format!("part c, modes {lm}/{nm}"));
        assert_eq!(run.end_target, Some(Target::Object(x)));
    }
}

// spec: S-0301 claim 14 and S-0308 claim 14 (scenario F21 part d: land mode 2, naval mode 5, two weapons; S-0308 table P gives slot 0 against the target on water, row 17 with table N mode 5, and slot 1 on land, row 18; the target crosses onto land during the delay and the shot at the end of the delay is of slot 0, the slot chosen when the delay started, not chosen again)
#[test]
fn f21_d_the_slot_chosen_at_the_start_is_kept() {
    let r = rules(DELAY, 2, 5);
    // The fresh choice on each side: slot 0 on water, slot 1 on land.
    for (l, want) in [(land::WATER, "ShotA"), (land::CLEAR, "ShotB")] {
        let (mut w, b, x) = fixture(&r, l);
        let id = weapon_id(&w, want);
        let run = run_delay(&mut w, b, x, |_, _| {});
        assert_made(&run, id, x, &format!("fresh choice on land type {l}"));
    }
    for cross in [0, 14, DELAY - 2] {
        let (mut w, b, x) = fixture(&r, land::WATER);
        let a = weapon_id(&w, "ShotA");
        let run = run_delay(&mut w, b, x, |w, k| {
            if k == cross {
                set_land(w, at(3, 0), land::CLEAR);
            }
        });
        assert_made(&run, a, x, &format!("part d, crossing at {cross}"));
    }
}

// spec: S-0301 claims 9 and 14 (scenario F21 part f: an attack order that gives the building another target while its shot is pending changes the target but starts no new delay, the due tick unchanged, and the one shot at the end of the delay is at the new target)
#[test]
fn f21_f_a_new_target_during_the_delay_starts_no_new_delay() {
    // An order takes effect after the turn pass (S-0203 claim 7), so the last
    // one that reaches the shot is given in the tick before the delay ends.
    for when in [1, 40, 147] {
        let (mut w, b, x) = fixture(&rules(150, 0, 0), land::CLEAR);
        let y = testkit::spawn(&mut w, Kind::Vehicle, "BOAT", 1, at(4, 0), 0);
        let a = weapon_id(&w, "ShotA");
        let mut due = None;
        let run = run_delay(&mut w, b, x, |w, k| {
            let now = w.obj(b).unwrap().combat.delayed_shot;
            if k == 0 {
                due = now;
            }
            assert_eq!(now, due, "the delay counts on unchanged, order at {when}");
            if k == when {
                order_attack(w, b, y);
            }
        });
        assert_made(&run, a, y, &format!("part f, order at {when}"));
        assert_eq!(run.end_target, Some(Target::Object(y)));
    }
}

// ---------------------------------------------------------------------------
// The check's answers while the shot is pending, called directly.

/// The building attacking the target, its delayed shot started by one
/// attack processing; returns the world, the building, the target and the
/// due tick.
fn pending(r: &Ruleset, l: u8) -> (World, ObjectId, ObjectId, i32) {
    let mut w = testkit::game_with(r, SIZE, 13);
    let b = testkit::spawn(&mut w, Kind::Building, "DLY", 0, at(0, 0), 0);
    let x = testkit::spawn(&mut w, Kind::Vehicle, "BOAT", 1, at(3, 0), 0);
    set_land(&mut w, at(3, 0), l);
    w.obj_mut(b).unwrap().target = Some(Target::Object(x));
    let tick = w.tick;
    w.obj_mut(b)
        .unwrap()
        .beh
        .force(Behaviour::Attack, tick, true);
    behave::building_process(&mut w, b);
    let due = w.obj(b).unwrap().combat.delayed_shot.expect("pending");
    (w, b, x, due)
}

fn ask(w: &World, b: ObjectId, slot: usize, range: bool) -> Answer {
    check(w, b, w.obj(b).unwrap().target, slot, range)
}

// spec: S-0301 claim 14 (scenario F21 part e, through S-0306 claim 23's airborne row: a target that lifts to 208 leptons above the surface or higher during the delay is refused at the end of it, no projectile and no countdown; one still below 208 leptons is not refused by that row and the shot is made; while pending, the check answers rearming either way, claim 9)
#[test]
fn f21_e_a_lifting_target_at_and_below_208_leptons() {
    for (dz, made) in [(208, false), (300, false), (207, true), (0, true)] {
        let (mut w, b, x, due) = pending(&rules(DELAY, 0, 0), land::CLEAR);
        lift(&mut w, x, dz);
        w.tick = due - 1;
        assert_eq!(ask(&w, b, 0, true), Answer::Rearming, "lift {dz}: pending");
        behave::delayed_shot_turn(&mut w, b);
        assert!(w.obj(b).unwrap().combat.delayed_shot.is_some());
        w.tick = due;
        let rearm = w.obj(b).unwrap().combat.rearm_until;
        behave::delayed_shot_turn(&mut w, b);
        let o = w.obj(b).unwrap();
        assert_eq!(o.combat.delayed_shot, None);
        assert_eq!(!shots_of(&w, b).is_empty(), made, "lift {dz}");
        assert_eq!(o.combat.rearm_until != rearm, made, "lift {dz}: countdown");
        assert_eq!(
            o.target,
            Some(Target::Object(x)),
            "the check keeps the target"
        );
    }
}

// spec: S-0301 claim 9 (a building's pending delayed shot answers rearming at rank 4 ahead of every row of S-0306 claim 23, the land and water rows and the airborne row, of the absent-weapon impossible and of the range test; without the pending shot the same checks answer illegal, impossible and out of range)
#[test]
fn claim_9_pending_shot_comes_ahead_of_the_rows_and_later_ranks() {
    // The land row (mode 1, clear land) and the water row (mode 6, water).
    for (lm, nm, l) in [(1, 0, land::CLEAR), (0, 6, land::WATER)] {
        // Started against a target the rows admit, then refused.
        let start = if l == land::CLEAR {
            land::WATER
        } else {
            land::CLEAR
        };
        let (mut w, b, _, _) = pending(&rules(DELAY, lm, nm), start);
        set_land(&mut w, at(3, 0), l);
        assert_eq!(ask(&w, b, 0, true), Answer::Rearming, "modes {lm}/{nm}");
        w.obj_mut(b).unwrap().combat.delayed_shot = None;
        assert_eq!(ask(&w, b, 0, true), Answer::Illegal, "modes {lm}/{nm}");
    }
    // The airborne row.
    let (mut w, b, x, _) = pending(&rules(DELAY, 0, 0), land::CLEAR);
    lift(&mut w, x, 208);
    assert_eq!(ask(&w, b, 0, true), Answer::Rearming, "airborne");
    w.obj_mut(b).unwrap().combat.delayed_shot = None;
    assert_eq!(
        ask(&w, b, 0, true),
        Answer::Illegal,
        "airborne, not pending"
    );
    // Out of range.
    let (mut w, b, x, _) = pending(&rules(DELAY, 0, 0), land::CLEAR);
    w.obj_mut(x).unwrap().pos = glasnost_sim::world::Pos::centre(at(12, 0), 0);
    assert_eq!(ask(&w, b, 0, true), Answer::Rearming, "out of range");
    w.obj_mut(b).unwrap().combat.delayed_shot = None;
    assert_eq!(ask(&w, b, 0, true), Answer::OutOfRange, "not pending");
    // The absent-weapon impossible: a type with no secondary, asked for slot 1.
    let mut r = rules(DELAY, 0, 0);
    r.sim.buildings.get_mut("DLY").unwrap().secondary = None;
    let (mut w, b, _, _) = pending(&r, land::CLEAR);
    assert_eq!(ask(&w, b, 1, true), Answer::Rearming, "absent weapon");
    w.obj_mut(b).unwrap().combat.delayed_shot = None;
    assert_eq!(
        ask(&w, b, 1, true),
        Answer::Impossible,
        "absent, not pending"
    );
}

// spec: S-0301 claim 9 (a building's pending delayed shot yields to the building's own answers of S-0405 table T1: a building being sold answers illegal, pending shot or not)
#[test]
fn claim_9_table_t1_answers_come_ahead_of_the_pending_shot() {
    let (mut w, b, _, _) = pending(&rules(DELAY, 0, 0), land::CLEAR);
    assert_eq!(ask(&w, b, 0, true), Answer::Rearming);
    let tick = w.tick;
    w.obj_mut(b)
        .unwrap()
        .beh
        .force(Behaviour::Selling, tick, true);
    assert!(w.obj(b).unwrap().combat.delayed_shot.is_some());
    assert_eq!(ask(&w, b, 0, true), Answer::Illegal, "selling, pending");
}
