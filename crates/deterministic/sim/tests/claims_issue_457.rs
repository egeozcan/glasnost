//! Issue #457: damage aimed at a vehicle in a tank bunker. Guard tests of
//! the damage paths as S-0302 claim 8 (stage 2's two bunker rows and *Hits
//! without a warhead*) and S-0303 give them: a direct hit, splash centred
//! on the vehicle, splash centred on the bunker, and a hit with no warhead.
//! Whether some of that damage should land on the bunker is asked in
//! Q-3302. On an empty synthetic map with invented types: every name and
//! number here is made up; no game data. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S42*.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, WarheadType};
use glasnost_sim::combat::area::{self, Detonation};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::bunker;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{testkit, tick};

const SIZE: i32 = 24;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The combat ruleset with PIT, a 2 × 2 tank bunker; TANK, a turreted
/// vehicle that may enter it; and the warheads PEN (penetrates bunkers)
/// and FLAT (does not), both with a spread of one cell.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut pit = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        bunker: Some(true),
        number_impassable_rows: Some(0),
        ..Default::default()
    };
    pit.econ.buildup = Some(true);
    pit.econ.buildup_frames = Some(4);
    register(&mut r, "buildings", "PIT");
    r.sim.buildings.insert("PIT".into(), pit);
    let mut tank = r.sim.vehicles["CRUSHER"].clone();
    tank.crusher = None;
    register(&mut r, "vehicles", "TANK");
    r.sim.vehicles.insert("TANK".into(), tank);
    for (name, pen) in [("PEN", true), ("FLAT", false)] {
        r.sim.warheads.insert(
            name.into(),
            WarheadType {
                verses: Some(vec![dec("1"); 11]),
                penetrates_bunker: pen.then_some(true),
                cell_spread: Some(dec("1")),
                percent_at_max: Some(dec("1")),
                ..Default::default()
            },
        );
    }
    r
}

fn warhead(w: &World, name: &str) -> u32 {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .expect("a synthetic warhead") as u32
}

/// An Alpha PIT at the middle cell holding an Alpha TANK, and a free Alpha
/// TANK well away from it as a control. Returns (world, bunker, vehicle
/// inside, free vehicle).
fn scene() -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    let b = testkit::spawn(&mut w, Kind::Building, "PIT", 0, testkit::mid(SIZE, 0, 0), 0);
    testkit::run(&mut w, 3);
    let u = testkit::spawn(&mut w, Kind::Vehicle, "TANK", 0, testkit::mid(SIZE, 6, 1), 0);
    let free = testkit::spawn(&mut w, Kind::Vehicle, "TANK", 0, testkit::mid(SIZE, -8, -8), 0);
    orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    );
    for _ in 0..1500 {
        if bunker::bunkered_in(&w, u) == Some(b) {
            break;
        }
        tick::step(&mut w);
    }
    assert_eq!(bunker::bunkered_in(&w, u), Some(b), "void: the tank is bunkered");
    (w, b, u, free)
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).unwrap().health
}

fn direct(w: &World, wh: Option<&str>, ignore_defences: bool) -> Hit {
    Hit {
        amount: 50,
        warhead: wh.map(|n| warhead(w, n)),
        distance: 0,
        attacker: None,
        house: Some(HouseId(1)),
        ignore_defences,
        no_escape: false,
    }
}

fn splash(w: &mut World, point: Pos, wh: &str) {
    let warhead = Some(warhead(w, wh));
    area::detonate(
        w,
        Detonation {
            point,
            damage: 50,
            warhead,
            source: None,
            house: Some(HouseId(1)),
            resources: false,
        },
    );
}

// spec: S-0302 claim 8 (stage 2: a bunkered unit inside its bunker hit by a warhead that does not penetrate bunkers changes no health; nothing passes the stopped hit on to the bunker) and S-0606 claim 9
#[test]
fn a_direct_non_penetrating_hit_hurts_neither_the_vehicle_nor_the_bunker() {
    let (mut w, b, u, free) = scene();
    let (hu, hb, hf) = (health(&w, u), health(&w, b), health(&w, free));
    let h = direct(&w, Some("FLAT"), false);
    assert_eq!(damage::apply(&mut w, u, h), HitResult::Unaffected);
    assert_eq!(health(&w, u), hu, "the bunkered vehicle takes nothing");
    assert_eq!(health(&w, b), hb, "the bunker takes nothing");
    let h = direct(&w, Some("FLAT"), false);
    damage::apply(&mut w, free, h);
    assert!(health(&w, free) < hf, "void: the same hit hurts a free vehicle");
}

// spec: S-0302 claim 8 (stage 2: no row stops a penetrating warhead on the bunkered unit; the tank-bunker row is for hits on the bunker) and S-0606 claim 9
#[test]
fn a_direct_penetrating_hit_hurts_the_vehicle_and_not_the_bunker() {
    let (mut w, b, u, _) = scene();
    let (hu, hb) = (health(&w, u), health(&w, b));
    let h = direct(&w, Some("PEN"), false);
    damage::apply(&mut w, u, h);
    assert!(health(&w, u) < hu, "the bunkered vehicle is damaged");
    assert_eq!(health(&w, b), hb, "the bunker takes nothing");
}

// spec: S-0303 (each object a detonation reaches takes its own hit through S-0302) and S-0302 claim 8 (stage 2's two bunker rows, for each hit of the detonation; scenario D8 part d: the bunker loses health to a warhead that does not penetrate bunkers, and none to one that does)
#[test]
fn splash_on_the_vehicle_and_on_the_bunker_follows_the_two_rows() {
    for centre_on_vehicle in [true, false] {
        for (wh, bunker_hurt, vehicle_hurt) in [("FLAT", true, false), ("PEN", false, true)] {
            let (mut w, b, u, _) = scene();
            let (hu, hb) = (health(&w, u), health(&w, b));
            let point = if centre_on_vehicle {
                w.obj(u).unwrap().pos
            } else {
                w.reference_point(b)
            };
            splash(&mut w, point, wh);
            let case = format!("{wh}, centred on the vehicle: {centre_on_vehicle}");
            assert_eq!(health(&w, b) < hb, bunker_hurt, "bunker, {case}");
            assert_eq!(health(&w, u) < hu, vehicle_hurt, "vehicle, {case}");
        }
    }
}

// spec: S-0302 claim 8 (*Hits without a warhead*: neither bunker row holds; one that does not ignore defences has m0 = 0 by claim 6 and the zero-amount row stops it; one that ignores defences keeps its delivered amount, claim 9)
#[test]
fn a_hit_without_a_warhead_on_the_bunkered_vehicle() {
    let (mut w, b, u, _) = scene();
    let (hu, hb) = (health(&w, u), health(&w, b));
    let h = direct(&w, None, false);
    damage::apply(&mut w, u, h);
    assert_eq!(health(&w, u), hu, "stopped by the zero-amount row");
    let h = direct(&w, None, true);
    damage::apply(&mut w, u, h);
    assert!(health(&w, u) < hu, "ignoring defences: the vehicle is damaged");
    assert_eq!(health(&w, b), hb, "the bunker takes nothing");
}

// spec: S-0302 claim 8 (the I column: a hit that ignores defences passes both bunker rows)
#[test]
fn a_hit_that_ignores_defences_passes_both_rows() {
    let (mut w, b, u, _) = scene();
    let (hu, hb) = (health(&w, u), health(&w, b));
    let h = direct(&w, Some("FLAT"), true);
    damage::apply(&mut w, u, h);
    assert!(health(&w, u) < hu, "the bunkered-unit row does not hold");
    let h = direct(&w, Some("PEN"), true);
    damage::apply(&mut w, b, h);
    assert!(health(&w, b) < hb, "the tank-bunker row does not hold");
}
