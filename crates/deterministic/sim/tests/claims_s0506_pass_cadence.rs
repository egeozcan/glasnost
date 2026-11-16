//! The launch pass cadence after a stretch in which the spawner makes no
//! pass (#343; S-0506 claim 2, *Passes and the spawner's turns*, Q-3134).
//! On synthetic data: an invented launcher LNA with one slot of an
//! invented rocket type RKA, a building SILO with the same slot, and the
//! testkit's combat ruleset; no game data.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LocomotionClass, Locomotor, UnitType, WeaponType,
};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

const SIZE: i32 = 32;

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("RKA".into());
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("LNA".into());
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("SILO".into());
    let s = &mut r.sim;
    s.aircraft.insert(
        "RKA".into(),
        UnitType {
            strength: Some(40),
            speed: Some(50),
            rot: Some(20),
            missile_spawn: Some(true),
            spawned: Some(true),
            locomotor: Some(Locomotor::Rocket),
            ..Default::default()
        },
    );
    let mut lna = UnitType {
        strength: Some(400),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        primary: Some("Launch".into()),
        missile_spawn: Some(true),
        ..Default::default()
    };
    lna.combat.spawns = Some("RKA".into());
    lna.combat.spawns_number = Some(1);
    lna.combat.spawn_regen_rate = Some(50);
    lna.combat.spawn_reload_rate = Some(0);
    s.vehicles.insert("LNA".into(), lna);
    let mut silo = BuildingType {
        strength: Some(1000),
        foundation: Some([1, 1]),
        primary: Some("Launch".into()),
        ..Default::default()
    };
    silo.combat.spawns = Some("RKA".into());
    silo.combat.spawns_number = Some(1);
    silo.combat.spawn_regen_rate = Some(50);
    silo.combat.spawn_reload_rate = Some(0);
    s.buildings.insert("SILO".into(), silo);
    s.weapons.insert(
        "Launch".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(20),
            range: Some(Decimal("30".into())),
            projectile: Some("Bullet".into()),
            warhead: Some("HE".into()),
            spawner: Some(true),
            ..Default::default()
        },
    );
    r
}

fn next_pass(w: &World, id: ObjectId) -> i32 {
    w.obj(id).unwrap().spawn.as_ref().unwrap().next_pass
}

/// A launcher created off the map, as production creates it (S-0404 claim
/// 11), and placed `late` ticks later; returns it and its creation tick.
fn placed_after(w: &mut World, late: i32) -> (ObjectId, i32) {
    let created = w.tick;
    let ti = w.rules().index_of(Kind::Vehicle, "LNA").unwrap();
    let id = glasnost_sim::objects::create(w, Kind::Vehicle, ti, Some(HouseId(0)));
    assert_eq!(next_pass(w, id), created + 20, "set-up: the first pass");
    testkit::run(w, late);
    assert_eq!(
        next_pass(w, id),
        created + 20,
        "set-up: no pass off the map"
    );
    let p = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let z = w.surface_at(p.x, p.y);
    glasnost_sim::objects::place(w, id, Pos::new(p.x, p.y, z), 0);
    (id, created)
}

/// The ticks, counted from `from`, in which a pass ran (the pass timer
/// moved) over the next `n` ticks.
fn passes(w: &mut World, id: ObjectId, from: i32, n: i32) -> Vec<i32> {
    let mut v = Vec::new();
    for _ in 0..n {
        let t = w.tick;
        let before = next_pass(w, id);
        testkit::run(w, 1);
        if next_pass(w, id) != before {
            v.push(t - from);
        }
    }
    v
}

// spec: S-0506 claim 2, Passes and the spawner's turns (Q-3134: after a stretch with no pass the first turn at T makes one pass and the next falls due at T + 10; the missed passes are not made up) and its Production stretch (no pass before it enters play)
#[test]
fn issue_343_passes_after_a_long_stretch_are_ten_ticks_apart() {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    let (id, created) = placed_after(&mut w, 40);
    let p = passes(&mut w, id, created, 25);
    assert_eq!(p.len(), 3, "passes at creation + {p:?}");
    let gaps: Vec<i32> = p.windows(2).map(|x| x[1] - x[0]).collect();
    assert_eq!(gaps, vec![10, 10], "passes at creation + {p:?}");
}

// spec: S-0506 claim 2, Passes and the spawner's turns (Q-3134: a pass made late by fewer than 10 ticks also sets the next one 10 ticks after it; the earlier phase is not kept)
#[test]
fn issue_343_a_pass_late_by_less_than_ten_ticks_keeps_no_old_phase() {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    let (id, created) = placed_after(&mut w, 25);
    let p = passes(&mut w, id, created, 25);
    assert!(p.len() >= 2, "passes at creation + {p:?}");
    let gaps: Vec<i32> = p.windows(2).map(|x| x[1] - x[0]).collect();
    assert!(
        gaps.iter().all(|&g| g == 10),
        "passes at creation + {p:?}, gaps {gaps:?}"
    );
    assert_ne!(
        p[0] + 10,
        40,
        "the old phase (creation + 20, + 30, + 40) is not kept"
    );
}

// spec: S-0506 claim 2, Passes and the spawner's turns, its Erasure stretch (a building makes no pass while it is being erased, S-0514 claim 17)
#[test]
fn issue_343_a_building_being_erased_makes_no_pass() {
    let mut r = rules();
    r.sim.warheads.get_mut("HE").unwrap().temporal = Some(true);
    let mut w = testkit::game_with(&r, SIZE, 7);
    let b = testkit::spawn(
        &mut w,
        Kind::Building,
        "SILO",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    let firer = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        1,
        testkit::mid(SIZE, 3, 0),
        0,
    );
    testkit::run(&mut w, 30);
    let before = next_pass(&w, b);
    glasnost_sim::temporal::deliver(&mut w, Some(firer), Some(Target::Object(b)));
    assert!(
        glasnost_sim::temporal::building_erased(&w, b),
        "set-up: the building is being erased"
    );
    let mut moved = Vec::new();
    for _ in 0..15 {
        let t = w.tick;
        let np = next_pass(&w, b);
        testkit::run(&mut w, 1);
        if glasnost_sim::temporal::building_erased(&w, b) && next_pass(&w, b) != np {
            moved.push(t);
        }
    }
    assert_eq!(
        moved,
        Vec::<i32>::new(),
        "no pass while being erased (timer was {before})"
    );
}
