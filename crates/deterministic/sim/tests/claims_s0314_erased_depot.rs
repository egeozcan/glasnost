//! A repair depot removed by a completed erasure releases its contacts
//! (S-0314 claim 22, *destruction or other removal*; S-0514 claim 12), as a
//! destroyed one does (#222). On synthetic data: invented types, weapons
//! and numbers, no game data.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LocomotionClass, Locomotor, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{depot, econ, testkit, tick};

const SIZE: i32 = 30;
const ALPHA: HouseId = HouseId(0);

/// The combat ruleset plus TRUCK (an unarmed wheeled vehicle), DEPOT (a
/// 3 × 3 repair depot), a temporal warhead TEMP, weapon Chrono and ERASER,
/// an infantry type that fires it.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut push = |reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push("vehicles", "TRUCK");
    push("buildings", "DEPOT");
    push("infantry", "ERASER");
    let s = &mut r.sim;
    s.vehicles.insert(
        "TRUCK".into(),
        UnitType {
            strength: Some(600),
            speed: Some(8),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            cost: Some(900),
            ..Default::default()
        },
    );
    s.buildings.insert(
        "DEPOT".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([3, 3]),
            unit_repair: Some(true),
            number_impassable_rows: Some(0),
            cost: Some(800),
            ..Default::default()
        },
    );
    s.general.repair_step = Some(10);
    s.general.repair_percent = Some(Decimal("25%".into()));
    s.warheads.insert(
        "TEMP".into(),
        WarheadType {
            temporal: Some(true),
            verses: Some(vec![Decimal("0.1".into()); 11]),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Chrono".into(),
        WeaponType {
            damage: Some(30),
            rof: Some(40),
            range: Some(Decimal("40".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("TEMP".into()),
            ..Default::default()
        },
    );
    let mut e = s.infantry["WALKER"].clone();
    e.strength = Some(100_000);
    e.primary = Some("Chrono".into());
    s.infantry.insert("ERASER".into(), e);
    r
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn run_until(w: &mut World, n: i32, mut f: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..n {
        if f(w) {
            return true;
        }
        tick::step(w);
    }
    f(w)
}

/// A damaged truck clicked into the depot, run until it is asleep on it
/// and the depot is in repair.
fn arrived() -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 5);
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    let d = spawn(&mut w, Kind::Building, "DEPOT", 0, 0, 0);
    let v = spawn(&mut w, Kind::Vehicle, "TRUCK", 0, 6, 1);
    w.obj_mut(v).unwrap().health = 300;
    while w.tick < 4 {
        tick::step(&mut w);
    }
    let o = targeting::click(&w, ALPHA, &[v], ClickOn::Object(d), Modifiers::default()).orders;
    assert!(!o.is_empty(), "the click gives an enter order");
    for o in o {
        assert!(orders::issue(&mut w, ALPHA, o));
    }
    assert!(
        run_until(&mut w, 600, |w| w.obj(v).unwrap().contact == Some(d)
            && w.obj(d).unwrap().beh.current == Some(Behaviour::Repair)
            && w.obj(v).unwrap().beh.current == Some(Behaviour::Sleep)),
        "the truck arrives in sleep and the depot starts repair"
    );
    (w, d, v)
}

/// An enemy eraser erases `d` to completion.
fn erase_to_completion(w: &mut World, d: ObjectId) {
    let e = spawn(w, Kind::Infantry, "ERASER", 1, -6, 1);
    let stale = w.rules().quirks.stale_suspended_revert;
    let t = w.tick;
    w.obj_mut(e).unwrap().beh.force(Behaviour::Sleep, t, stale);
    let t0 = w.tick;
    assert!(glasnost_sim::combat::fire::shot(w, e, 0, Target::Object(d)));
    while !w.projectiles.is_empty() {
        tick::step(w);
        assert!(w.tick < t0 + 40, "the bullet did not detonate");
    }
    assert!(
        glasnost_sim::temporal::erased(w, d),
        "the depot is being erased"
    );
    let t = w.tick;
    if let Some(o) = w.obj_mut(e) {
        o.target = Some(Target::Object(d));
        o.beh.force(Behaviour::Attack, t, stale);
    }
    assert!(
        run_until(w, 4000, |w| w.obj(d).is_none()),
        "the erasure completes and removes the depot"
    );
}

// spec: S-0314 claim 22 (destruction or other removal: every contact is released without a run-away notice; an arrived vehicle in sleep stays in sleep) and S-0514 claim 12 (completion removes the target without death)
#[test]
fn erased_depot_releases_an_arrived_contact() {
    let (mut w, d, v) = arrived();
    erase_to_completion(&mut w, d);
    let o = w.obj(v).expect("the truck survives the erasure");
    assert_eq!(o.contact, None, "the erased depot's contact is released");
    assert!(depot::link(&w, v).is_none());
    assert_eq!(o.beh.current, Some(Behaviour::Sleep), "it stays in sleep");
}

// spec: S-0314 claim 22 (every contact is released) and S-0213 claim 8 (stop: an actor that is not tethered obeys it)
#[test]
fn erased_depot_leaves_no_tether() {
    let (mut w, d, v) = arrived();
    // The docked state a pad depot's exchange gives (S-0314 claim 17), set
    // directly as the project's other depot tests do.
    w.obj_mut(v).unwrap().depot.link.as_mut().unwrap().tethered = true;
    assert!(econ::tethered(&w, v));
    erase_to_completion(&mut w, d);
    assert!(!econ::tethered(&w, v), "no tether to a removed building");
    let mv = Order::Behaviour {
        actor: v,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(testkit::mid(SIZE, 12, 12))),
    };
    assert!(orders::issue(&mut w, ALPHA, mv));
    for _ in 0..5 {
        tick::step(&mut w);
    }
    assert!(orders::issue(&mut w, ALPHA, Order::Stop { actor: v }));
    for _ in 0..3 {
        tick::step(&mut w);
    }
    assert_eq!(w.obj(v).unwrap().dest, None, "the stop order is obeyed");
}
