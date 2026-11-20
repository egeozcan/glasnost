//! Issue #457: damage aimed at a vehicle in a tank bunker. Guard tests of
//! the damage paths as S-0302 r51 claim 8 (stage 2's two bunker rows,
//! *Hits without a warhead* and *A bunkered unit and its bunker in one
//! detonation*, Q-3302 answered) and S-0303 give them: a direct hit, area
//! detonations centred on the vehicle and on the bunker, a hit with no
//! warhead, real shots, a killing hit and a curtain. On an empty synthetic
//! map with invented types: every name and number here is made up; no game
//! data. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S42* and *Batch
//! S44*.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, WarheadType, WeaponType};
use glasnost_sim::combat::area::{self, Detonation, DetonationResult};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::bunker;
use glasnost_sim::superweapons::{ProtectKind, protect};
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
/// vehicle that may enter it; the warheads PEN (penetrates bunkers) and
/// FLAT (does not), both with a spread of one cell, FLAT0 (spread 0) and
/// FLATH (spread 0.5, a small detonation), neither penetrating; and, for
/// real shots, the vehicles SHOOTF, SHOOTP and SHOOTR armed with an
/// invisible-projectile FLAT weapon, an invisible-projectile PEN weapon
/// and a visible-rocket FLAT weapon.
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
    for (name, pen, spread) in [
        ("PEN", true, "1"),
        ("FLAT", false, "1"),
        ("FLAT0", false, "0"),
        ("FLATH", false, "0.5"),
    ] {
        r.sim.warheads.insert(
            name.into(),
            WarheadType {
                verses: Some(vec![dec("1"); 11]),
                penetrates_bunker: pen.then_some(true),
                cell_spread: Some(dec(spread)),
                percent_at_max: Some(dec("1")),
                ..Default::default()
            },
        );
    }
    for (shooter, weapon, projectile, wh) in [
        ("SHOOTF", "FSHOT", "Bullet", "FLAT"),
        ("SHOOTP", "PSHOT", "Bullet", "PEN"),
        ("SHOOTR", "RSHOT", "Rocket", "FLAT"),
    ] {
        r.sim.weapons.insert(
            weapon.into(),
            WeaponType {
                damage: Some(50),
                rof: Some(60),
                range: Some(dec("8")),
                speed: Some(40),
                projectile: Some(projectile.into()),
                warhead: Some(wh.into()),
                ..Default::default()
            },
        );
        let mut v = r.sim.vehicles["CRUSHER"].clone();
        v.crusher = None;
        v.primary = Some(weapon.into());
        register(&mut r, "vehicles", shooter);
        r.sim.vehicles.insert(shooter.into(), v);
    }
    r
}

/// The same PIT as `scene` builds, at the same cell and tick steps, with no
/// vehicle: the control for "the hits the same detonation would deal it if
/// it were empty".
fn empty_bunker() -> (World, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    let b = testkit::spawn(
        &mut w,
        Kind::Building,
        "PIT",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    testkit::run(&mut w, 3);
    (w, b)
}

fn detonate(w: &mut World, point: Pos, wh: &str, damage: i32) -> DetonationResult {
    let warhead = Some(warhead(w, wh));
    area::detonate(
        w,
        Detonation {
            point,
            damage,
            warhead,
            source: None,
            house: Some(HouseId(1)),
            resources: false,
        },
    )
}

/// The bunker's loss from one detonation at `point` with the bunker empty.
fn empty_loss(point: Pos, wh: &str, damage: i32) -> i32 {
    let (mut w, b) = empty_bunker();
    let hb = health(&w, b);
    detonate(&mut w, point, wh, damage);
    hb - health(&w, b)
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
    let b = testkit::spawn(
        &mut w,
        Kind::Building,
        "PIT",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    testkit::run(&mut w, 3);
    let u = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "TANK",
        0,
        testkit::mid(SIZE, 6, 1),
        0,
    );
    let free = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "TANK",
        0,
        testkit::mid(SIZE, -8, -8),
        0,
    );
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
    assert_eq!(
        bunker::bunkered_in(&w, u),
        Some(b),
        "void: the tank is bunkered"
    );
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

// spec: S-0302 r51 claim 8 (stage 2: a bunkered unit inside its bunker hit by a warhead that does not penetrate bunkers changes no health; *A bunkered unit and its bunker in one detonation*: nothing stopped at one of the two objects passes to the other) and S-0606 claim 9
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
    assert!(
        health(&w, free) < hf,
        "void: the same hit hurts a free vehicle"
    );
}

// spec: S-0302 r51 claim 8 (stage 2: no row stops a penetrating warhead on the bunkered unit; the tank-bunker row is for hits on the bunker; nothing passes to the other object) and S-0606 claim 9
#[test]
fn a_direct_penetrating_hit_hurts_the_vehicle_and_not_the_bunker() {
    let (mut w, b, u, _) = scene();
    let (hu, hb) = (health(&w, u), health(&w, b));
    let h = direct(&w, Some("PEN"), false);
    damage::apply(&mut w, u, h);
    assert!(health(&w, u) < hu, "the bunkered vehicle is damaged");
    assert_eq!(health(&w, b), hb, "the bunker takes nothing");
}

// spec: S-0303 (each object a detonation reaches takes its own hit through S-0302) and S-0302 r51 claim 8 (stage 2's two bunker rows, for each hit of the detonation; *A bunkered unit and its bunker in one detonation*; scenario D8 part d: the bunker loses health to a warhead that does not penetrate bunkers, and none to one that does)
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

// spec: S-0302 r51 claim 8 (*Hits without a warhead*: neither bunker row holds; one that does not ignore defences has m0 = 0 by claim 6 and the zero-amount row stops it; one that ignores defences keeps its delivered amount, claim 9)
#[test]
fn a_hit_without_a_warhead_on_the_bunkered_vehicle() {
    let (mut w, b, u, _) = scene();
    let (hu, hb) = (health(&w, u), health(&w, b));
    let h = direct(&w, None, false);
    damage::apply(&mut w, u, h);
    assert_eq!(health(&w, u), hu, "stopped by the zero-amount row");
    let h = direct(&w, None, true);
    damage::apply(&mut w, u, h);
    assert!(
        health(&w, u) < hu,
        "ignoring defences: the vehicle is damaged"
    );
    assert_eq!(health(&w, b), hb, "the bunker takes nothing");
}

// spec: S-0302 r51 claim 8 (the I column: a hit that ignores defences passes both bunker rows; *A bunkered unit and its bunker in one detonation*: the unit takes a hit that ignores defences)
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

// spec: S-0302 r51 claim 8 (*A bunkered unit and its bunker in one detonation*: with a warhead that does not penetrate bunkers the unit takes nothing and the bunker takes exactly the hits that the same detonation would deal it if it were empty) and S-0303 claims 7 and 8 (one hit per scanned foundation cell at that cell's distance)
#[test]
fn a_non_penetrating_detonation_deals_the_bunker_its_empty_bunker_hits() {
    for wh in ["FLAT", "FLAT0"] {
        for centre_on_vehicle in [true, false] {
            let (mut w, b, u, _) = scene();
            let (hu, hb) = (health(&w, u), health(&w, b));
            let point = if centre_on_vehicle {
                w.obj(u).unwrap().pos
            } else {
                w.reference_point(b)
            };
            let control = empty_loss(point, wh, 50);
            assert!(control > 0, "void: {wh} reaches the empty bunker");
            detonate(&mut w, point, wh, 50);
            let case = format!("{wh}, centred on the vehicle: {centre_on_vehicle}");
            assert_eq!(health(&w, u), hu, "the vehicle takes nothing, {case}");
            assert_eq!(hb - health(&w, b), control, "the empty-bunker hits, {case}");
        }
    }
}

// spec: S-0302 r51 claim 8 (*A bunkered unit and its bunker in one detonation*: with a warhead that penetrates bunkers the unit takes what it would take at that distance outside a bunker, and the bunker takes nothing while it holds the unit)
#[test]
fn a_penetrating_detonation_hurts_the_vehicle_as_outside_and_not_the_bunker() {
    let (mut w, b, u, free) = scene();
    let (hu, hb, hf) = (health(&w, u), health(&w, b), health(&w, free));
    let point = w.obj(u).unwrap().pos;
    detonate(&mut w, point, "PEN", 50);
    assert_eq!(health(&w, b), hb, "the bunker takes nothing");
    let lost = hu - health(&w, u);
    assert!(lost > 0, "the vehicle is damaged");
    // The free control vehicle at the epicentre: the same distance, 0.
    let point = w.obj(free).unwrap().pos;
    detonate(&mut w, point, "PEN", 50);
    assert_eq!(hf - health(&w, free), lost, "as outside a bunker");
}

// spec: S-0302 r51 claim 8 (*A bunkered unit and its bunker in one detonation*: the unit's hit comes before the bunker's hits in the same cell, S-0303 claim 11 and S-0209 claim 13; when it destroys the unit the bunker link ends with that death, S-0606 claim 14, and the bunker's later hits find an empty bunker and damage it by the ordinary rules)
#[test]
fn a_penetrating_killing_hit_leaves_the_bunker_empty_for_its_later_hits() {
    // Spread 0: only the centre cell is scanned, so every bunker hit there
    // comes after the unit's.
    let wh0 = "PEN0";
    let mut r = rules();
    let mut pen0 = r.sim.warheads["PEN"].clone();
    pen0.cell_spread = Some(dec("0"));
    r.sim.warheads.insert(wh0.into(), pen0);
    let build = |r: &Ruleset, with_tank: bool| {
        let mut w = testkit::game_with(r, SIZE, 7);
        testkit::run(&mut w, 2);
        let b = testkit::spawn(
            &mut w,
            Kind::Building,
            "PIT",
            0,
            testkit::mid(SIZE, 0, 0),
            0,
        );
        testkit::run(&mut w, 3);
        if !with_tank {
            return (w, b, None);
        }
        let u = testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "TANK",
            0,
            testkit::mid(SIZE, 6, 1),
            0,
        );
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
        assert_eq!(bunker::bunkered_in(&w, u), Some(b), "void: bunkered");
        (w, b, Some(u))
    };
    let (mut w, b, u) = build(&r, true);
    let u = u.unwrap();
    w.obj_mut(u).unwrap().health = 1;
    let point = w.obj(u).unwrap().pos;
    let hb = health(&w, b);
    let (mut e, eb, _) = build(&r, false);
    let he = health(&e, eb);
    detonate(&mut e, point, wh0, 50);
    let control = he - health(&e, eb);
    assert!(control > 0, "void: the detonation reaches the empty bunker");
    detonate(&mut w, point, wh0, 50);
    assert!(
        w.obj(u).is_none_or(|o| o.health <= 0),
        "the unit's hit kills it"
    );
    assert_eq!(bunker::bunkered_in(&w, u), None, "the bunker link ended");
    assert_eq!(hb - health(&w, b), control, "the empty-bunker hits");
}

// spec: S-0302 r51 claim 8 (*A bunkered unit and its bunker in one detonation*: a curtain-protected bunkered unit in the detonation's centre cell, closer to the epicentre than S-0303 claim 15's distance, nullifies a small detonation, also where the same detonation with the bunker empty would not be nullified; every object not under the invulnerability effect then takes nothing, the bunker included) and S-0303 claim 15
#[test]
fn a_curtained_bunkered_vehicle_nullifies_a_small_detonation() {
    let (mut w, b, u, _) = scene();
    let point = w.obj(u).unwrap().pos;
    assert!(
        empty_loss(point, "FLATH", 50) > 0,
        "void: with the bunker empty the detonation is not nullified"
    );
    let (mut c, cb, _, _) = scene();
    let hc = health(&c, cb);
    detonate(&mut c, point, "FLATH", 50);
    assert!(
        health(&c, cb) < hc,
        "void: with no curtain the bunker is hurt"
    );
    protect::start(&mut w, u, ProtectKind::Curtain, 100);
    let (hu, hb) = (health(&w, u), health(&w, b));
    let res = detonate(&mut w, point, "FLATH", 50);
    assert_eq!(res, DetonationResult::Nullified);
    assert_eq!(health(&w, b), hb, "the bunker takes nothing");
    assert_eq!(health(&w, u), hu, "the protection cancels the unit's hit");
}

/// A Beta shooter of type `shooter` at five cells west of the bunker,
/// ordered to attack the bunkered vehicle; steps until the vehicle's or the
/// bunker's health changes. Returns (vehicle loss, bunker loss).
fn shot(shooter: &str) -> (i32, i32) {
    let (mut w, b, u, _) = scene();
    let (hu, hb) = (health(&w, u), health(&w, b));
    let s = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        shooter,
        1,
        testkit::mid(SIZE, -5, 0),
        0,
    );
    orders::issue(
        &mut w,
        HouseId(1),
        Order::Behaviour {
            actor: s,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(u)),
            dest: None,
        },
    );
    for _ in 0..600 {
        tick::step(&mut w);
        if health(&w, u) != hu || health(&w, b) != hb {
            break;
        }
    }
    (hu - health(&w, u), hb - health(&w, b))
}

// spec: S-0302 r51 claim 8 (*A bunkered unit and its bunker in one detonation*: a shot aimed at the unit is no exception: a projectile's ordinary detonation, an invisible one included, is area damage like any other, S-0306 claim 21, and no separate hit goes to the object it was aimed at; each hit is judged by its own row)
#[test]
fn real_shots_at_the_bunkered_vehicle_follow_the_two_rows() {
    for shooter in ["SHOOTF", "SHOOTR"] {
        let (vehicle, bunker) = shot(shooter);
        assert_eq!(vehicle, 0, "{shooter}: the vehicle takes nothing");
        assert!(bunker > 0, "{shooter}: the bunker is hurt");
    }
    let (vehicle, bunker) = shot("SHOOTP");
    assert!(vehicle > 0, "a penetrating shot hurts the vehicle");
    assert_eq!(bunker, 0, "and not the bunker");
}
