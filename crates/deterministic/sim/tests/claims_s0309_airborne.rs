//! The airborne scan of automatic searches against aircraft in flight
//! (S-0309 claim 9; S-0303 Terms, *tracked flying object*), on synthetic
//! data, through the real order and tick path (M8 step A4 round 3, the
//! lead's play note 15: anti-air defences and units did not engage enemy
//! aircraft on their own).
//!
//! Every type, weapon and number here is invented.

mod common;
mod econ_common;

use econ_common::{SIZE, dec, game, wheel};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Locomotor, ProjectileType, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::combat::flying;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// The rules: the economy ruleset with the combat ruleset's weapons, plus
/// - projectile `Flak` (anti-air, not anti-ground) and `Both` (anti-air
///   and anti-ground),
/// - weapon `Sam` (range 8 cells, `Flak`) and `Duo` (range 8 cells,
///   `Both`),
/// - building `SAMS` (1 × 1, primary `Sam`): an anti-air-only defence,
/// - vehicle `FLAKV` (primary `Duo`): an anti-air and anti-ground unit,
/// - vehicle `SAMV` (primary `Sam`): an anti-air-only unit,
/// - aircraft `KITE`: an unarmed landable aircraft.
fn rules() -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    let s = &mut r.sim;
    s.projectiles.insert(
        "Flak".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(false),
            inviso: Some(true),
            ..Default::default()
        },
    );
    s.projectiles.insert(
        "Both".into(),
        ProjectileType {
            aa: Some(true),
            inviso: Some(true),
            ..Default::default()
        },
    );
    for (n, p) in [("Sam", "Flak"), ("Duo", "Both")] {
        s.weapons.insert(
            n.into(),
            WeaponType {
                damage: Some(5),
                rof: Some(30),
                range: dec("8"),
                speed: Some(100),
                projectile: Some(p.into()),
                warhead: Some("HE".into()),
                ..Default::default()
            },
        );
    }
    let mut sams = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        primary: Some("Sam".into()),
        ..Default::default()
    };
    sams.combat.armor = Some(glasnost_rules::sim::ArmourClass::Concrete);
    push(&mut r, "buildings", "SAMS");
    r.sim.buildings.insert("SAMS".into(), sams);
    let mut flakv = wheel(500);
    flakv.primary = Some("Duo".into());
    push(&mut r, "vehicles", "FLAKV");
    r.sim.vehicles.insert("FLAKV".into(), flakv);
    let mut samv = wheel(500);
    samv.primary = Some("Sam".into());
    push(&mut r, "vehicles", "SAMV");
    r.sim.vehicles.insert("SAMV".into(), samv);
    let mut kite = UnitType {
        strength: Some(5000),
        speed: Some(20),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        ..Default::default()
    };
    kite.econ.landable = Some(true);
    push(&mut r, "aircraft", "KITE");
    r.sim.aircraft.insert("KITE".into(), kite);
    r
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

/// Orders `actor` of house `h` to move to the cell at (dx, dy), as a
/// person's move order.
fn move_to(w: &mut World, h: u8, actor: ObjectId, dx: i32, dy: i32) {
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    assert!(orders::issue(
        w,
        HouseId(h),
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(testkit::mid(SIZE, dx, dy))),
        }
    ));
}

/// What happened while an enemy aircraft flew across the searcher's range.
#[derive(Debug, Default)]
struct Pass {
    /// Ticks the aircraft was in flight (S-0501 Terms).
    in_flight: usize,
    /// Of them, ticks it was a tracked flying object.
    tracked_in_flight: usize,
    /// Ticks it was tracked while on the ground.
    tracked_on_ground: usize,
    /// The first tick the searcher held it as its target.
    first_target: Option<i32>,
    /// Whether it was in flight then.
    target_in_flight: bool,
    /// The first tick a projectile of the searcher flew at it.
    first_shot: Option<i32>,
    /// Whether the aircraft took damage.
    damaged: bool,
}

/// Puts the searcher `name` of `kind` (house 0, human-controlled unless
/// `computer`) at the centre and an enemy KITE 14 cells west of it, orders
/// the KITE east across the searcher to 14 cells east, and watches up to
/// `n` ticks.
fn pass(kind: Kind, name: &str, computer: bool, n: usize) -> Pass {
    let mut w = game(&rules());
    if computer {
        // spec: S-0309 Terms (Test-only exception: a marked computer-controlled player house)
        w.houses[0].human = false;
    }
    let s = put(&mut w, kind, name, 0, 0, 0);
    let k = put(&mut w, Kind::Aircraft, "KITE", 1, -14, 0);
    let full = w.obj(k).unwrap().health;
    move_to(&mut w, 1, k, 14, 0);
    let mut p = Pass::default();
    let mut last = w.obj(s).unwrap().last_fired;
    for _ in 0..n {
        glasnost_sim::tick::step(&mut w);
        let flying = !air::on_ground(&w, k);
        let tracked = flying::tracked(&w, k);
        if flying {
            p.in_flight += 1;
            p.tracked_in_flight += usize::from(tracked);
        } else {
            p.tracked_on_ground += usize::from(tracked);
        }
        if p.first_target.is_none() && w.obj(s).unwrap().target == Some(Target::Object(k)) {
            p.first_target = Some(w.tick);
            p.target_in_flight = flying;
        }
        // A shot at the aircraft: the searcher's tick of the last shot (S-0206
        // claim 9) moved while it held the aircraft as its target.
        let so = w.obj(s).unwrap();
        if so.last_fired != last && so.target == Some(Target::Object(k)) {
            p.first_shot.get_or_insert(so.last_fired);
        }
        last = so.last_fired;
        if w.obj(k).is_some_and(|o| o.health < full) {
            p.damaged = true;
        }
    }
    p
}

// spec: S-0309 claims 5 and 9 and S-0303 Terms (an aircraft in ordinary flight is a tracked flying object; an anti-air-only building's range search looks at the airborne candidates only, so it acquires and fires at an enemy aircraft flying through its range on a move order, with no damage to it first)
#[test]
fn anti_air_building_engages_an_aircraft_flying_past() {
    for computer in [false, true] {
        let p = pass(Kind::Building, "SAMS", computer, 400);
        assert!(p.in_flight > 0, "the aircraft flies: {p:?}");
        assert_eq!(
            p.tracked_in_flight, p.in_flight,
            "tracked on every tick in flight: {p:?}"
        );
        assert!(
            p.first_target.is_some() && p.target_in_flight,
            "the building acquires the aircraft in flight (computer {computer}): {p:?}"
        );
        assert!(p.first_shot.is_some(), "and fires at it: {p:?}");
        assert!(p.damaged, "and hits it: {p:?}");
    }
}

// spec: S-0309 claims 5 and 9 and S-0303 Terms (a unit with an anti-air and anti-ground weapon acquires an aircraft in flight through the airborne scan: an aircraft in flight is listed in no cell, S-0209 claim 13)
#[test]
fn anti_air_unit_engages_an_aircraft_flying_past() {
    for computer in [false, true] {
        let p = pass(Kind::Vehicle, "FLAKV", computer, 400);
        assert!(
            p.first_target.is_some() && p.target_in_flight,
            "the unit acquires the aircraft in flight (computer {computer}): {p:?}"
        );
        assert!(p.first_shot.is_some(), "and fires at it: {p:?}");
    }
}

// spec: S-0309 claims 5 and 9 and S-0303 Terms (a unit whose only weapon is anti-air looks for aircraft only, so its range search scans the airborne candidates and no cell ring; it acquires an aircraft in flight)
#[test]
fn anti_air_only_unit_engages_an_aircraft_flying_past() {
    for computer in [false, true] {
        let p = pass(Kind::Vehicle, "SAMV", computer, 400);
        assert!(
            p.first_target.is_some() && p.target_in_flight,
            "the unit acquires the aircraft in flight (computer {computer}): {p:?}"
        );
        assert!(p.first_shot.is_some(), "and fires at it: {p:?}");
    }
}

// spec: S-0303 Terms (an aircraft in ordinary flight is tracked) and S-0209 claim 13 (a landed aircraft is listed in a cell); the boundaries are this engine's interim rule of Q-1845: tracked exactly while S-0501's in-flight test holds
#[test]
fn aircraft_is_tracked_only_in_flight() {
    let mut w = game(&rules());
    let k = put(&mut w, Kind::Aircraft, "KITE", 1, -6, 0);
    assert!(air::on_ground(&w, k), "placed on the ground");
    assert!(!flying::tracked(&w, k), "not tracked on the ground");
    move_to(&mut w, 1, k, 6, 0);
    let mut flew = false;
    let mut landed_after = false;
    for _ in 0..1500 {
        glasnost_sim::tick::step(&mut w);
        let in_flight = !air::on_ground(&w, k);
        assert_eq!(
            flying::tracked(&w, k),
            in_flight,
            "tracked exactly in flight at tick {}",
            w.tick
        );
        if in_flight {
            flew = true;
            let o = w.obj(k).unwrap();
            assert_eq!(
                w.flying.sector_of.get(&k),
                Some(&flying::sector(&w, o.pos.cell())),
                "its sector follows its cell"
            );
        } else if flew {
            landed_after = true;
            break;
        }
    }
    assert!(flew && landed_after, "takes off and lands again");
}

/// A detonation of the combat ruleset's warhead HE (spread 0.75 cells, R
/// 192 leptons) at `at`, with no source.
fn blast(w: &mut World, at: glasnost_sim::world::Pos) {
    let wh = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "HE")
        .unwrap() as u32;
    glasnost_sim::combat::area::detonate(
        w,
        glasnost_sim::combat::area::Detonation {
            point: at,
            damage: 50,
            warhead: Some(wh),
            source: None,
            house: None,
            resources: false,
        },
    );
}

// spec: S-0303 claims 6 and 10 (a detonation above the terrain reaches the tracked flying objects; an aircraft in the air counts at half distance, so a burst 300 leptons from it, beyond R = 192 but within 2R, hits it, and one 400 leptons from it misses); the tracking is Q-1845's interim rule
#[test]
fn area_damage_reaches_an_aircraft_in_flight_at_half_distance() {
    let mut w = game(&rules());
    let k = put(&mut w, Kind::Aircraft, "KITE", 1, -6, 0);
    move_to(&mut w, 1, k, 6, 0);
    let up = (0..600).any(|_| {
        glasnost_sim::tick::step(&mut w);
        // spec: S-0303 claim 10 (in the air by S-0308's test)
        glasnost_sim::combat::weapon::in_air(&w, k)
    });
    assert!(up && flying::tracked(&w, k), "in the air and tracked");
    let o = w.obj(k).unwrap();
    let (p, full) = (o.pos, o.health);
    blast(&mut w, glasnost_sim::world::Pos::new(p.x, p.y + 300, p.z));
    assert!(
        w.obj(k).unwrap().health < full,
        "hit at half of 300 leptons"
    );
    let h = w.obj(k).unwrap().health;
    blast(&mut w, glasnost_sim::world::Pos::new(p.x, p.y + 400, p.z));
    assert_eq!(w.obj(k).unwrap().health, h, "missed at half of 400 leptons");
}
