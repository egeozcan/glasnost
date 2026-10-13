//! Claim tests for turrets and facing for fire (S-0310), on synthetic data.
//!
//! The ruleset is `testkit::combat_ruleset()` with invented weapons added
//! here (no stock values). Facings are set by hand (`Turning::at`) and the
//! firing check (`combat::check::check`) and the vehicle's firing step
//! (`combat::behave::vehicle_step`) are called directly, so that each claim
//! is observed with every other reason absent.
//!
//! Not tested (seams or not implemented): installed upgrades (claim 4),
//! aircraft flight-control exemptions (claim 5), jumpjet infantry and the
//! locomotor's own refusals (claims 7 and 8), the deploy-to-fire replies
//! and *must deploy* (claim 9, the deploy specs), cloaking (claim 9), and
//! the visceroid's animation stage (claim 10). See
//! `docs/m4-claims/S-0310.md`.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, WeaponType};
use glasnost_sim::combat::behave;
use glasnost_sim::combat::check::{Answer, building_aim_point, check};
use glasnost_sim::combat::position::pixel_to_leptons;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

/// An optional pixel pair of the art data.
type Pixels = Option<[i32; 2]>;

/// Map size of every test here.
const N: i32 = 24;
/// Facing east (S-0204).
const EAST: u16 = 16384;
/// Facing west.
const WEST: u16 = 49152;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// An invented weapon: rate 30, range 10 cells, the invisible projectile
/// Bullet (not homing), warhead SA.
fn zap() -> WeaponType {
    WeaponType {
        damage: Some(1),
        rof: Some(30),
        range: Some(d("10")),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some("SA".into()),
        ..Default::default()
    }
}

/// The combat ruleset with ZAP on the CART (turreted) and on the WALKER, a
/// homing weapon HOMER (projectile Rocket, turn rate 12), and a durable
/// BUGGY as the target.
fn base() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.insert("ZAP".into(), zap());
    r.sim.weapons.insert("HOMER".into(), {
        let mut z = zap();
        z.projectile = Some("Rocket".into());
        z
    });
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("ZAP".into());
    r.sim.infantry.get_mut("WALKER").unwrap().primary = Some("ZAP".into());
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(30000);
    r
}

/// A world of `r` with an Alpha firer two cells west of the middle facing
/// east and a Beta BUGGY two cells east of it, set as the firer's target.
fn duel_of(r: &Ruleset, kind: Kind, name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, N, 7);
    let a = testkit::spawn(&mut w, kind, name, 0, testkit::mid(N, -2, 0), EAST);
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    (w, a, b)
}

fn duel(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    duel_of(r, Kind::Vehicle, "CART")
}

/// The firing check for `slot` at the firer's target, with range.
fn ask(w: &World, a: ObjectId) -> Answer {
    check(w, a, w.obj(a).unwrap().target, 0, true)
}

/// The direction from `p` to `b`'s reference point (S-0204 claim 8).
fn dir_from(w: &World, p: Pos, b: ObjectId) -> u16 {
    let q = w.reference_point(b);
    glasnost_num::geom::direction(
        i64::from(q.x - p.x),
        i64::from(q.y - p.y),
        w.rules().quirks.table_direction,
    )
}

/// The direction from `a`'s reference point to `b`'s.
fn dir(w: &World, a: ObjectId, b: ObjectId) -> u16 {
    dir_from(w, w.reference_point(a), b)
}

/// Sets the turret facing at rest.
fn turret_at(w: &mut World, a: ObjectId, f: u16) {
    w.obj_mut(a).unwrap().turret = Turning::at(f);
}

/// Sets the body facing at rest.
fn body_at(w: &mut World, a: ObjectId, f: u16) {
    w.obj_mut(a).unwrap().body = Turning::at(f);
}

/// Puts `a`'s turret (and body) at an aim error of `e` from its target,
/// at rest.
fn aim_off(w: &mut World, a: ObjectId, b: ObjectId, e: u16) {
    let f = dir(w, a, b).wrapping_add(e);
    turret_at(w, a, f);
    body_at(w, a, f);
}

// ---------------------------------------------------------------------------
// Claim 1: which facing aims.

// spec: S-0310 claim 1 (a vehicle with the turret flag aims with its turret facing, not its body)
#[test]
fn claim_1_turreted_vehicle_aims_with_its_turret() {
    let (mut w, a, b) = duel(&base());
    let d0 = dir(&w, a, b);
    turret_at(&mut w, a, d0);
    body_at(&mut w, a, d0.wrapping_add(16384));
    assert_eq!(ask(&w, a), Answer::CanFire);
    turret_at(&mut w, a, d0.wrapping_add(16384));
    body_at(&mut w, a, d0);
    assert_eq!(ask(&w, a), Answer::NotFacing);
}

// spec: S-0310 claim 1 (a vehicle without the turret flag aims with its body facing)
#[test]
fn claim_1_turretless_vehicle_aims_with_its_body() {
    let mut r = base();
    r.sim.vehicles.get_mut("CART").unwrap().turret = Some(false);
    let (mut w, a, b) = duel(&r);
    let d0 = dir(&w, a, b);
    body_at(&mut w, a, d0);
    turret_at(&mut w, a, d0.wrapping_add(16384));
    assert_eq!(ask(&w, a), Answer::CanFire);
    body_at(&mut w, a, d0.wrapping_add(16384));
    turret_at(&mut w, a, d0);
    assert_eq!(ask(&w, a), Answer::NotFacing);
}

// ---------------------------------------------------------------------------
// Claim 2 and Formulas F1, F2: vehicle tolerances.

// spec: S-0310 claim 2 and F1 (τ = 2048: an aim error of 2048 points, 2049 does not; both ways round)
#[test]
fn claim_2_vehicle_tolerance_2048() {
    let (mut w, a, b) = duel(&base());
    for (e, want) in [
        (0u16, Answer::CanFire),
        (2048, Answer::CanFire),
        (2049, Answer::NotFacing),
        (2304, Answer::NotFacing),
        (63488, Answer::CanFire),
        (63487, Answer::NotFacing),
        (32768, Answer::NotFacing),
    ] {
        aim_off(&mut w, a, b, e);
        assert_eq!(ask(&w, a), want, "aim error {e}");
    }
}

// spec: S-0310 claim 2 and F2 (a homing projectile: τ = 4096)
#[test]
fn claim_2_vehicle_tolerance_homing_4096() {
    let mut r = base();
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("HOMER".into());
    let (mut w, a, b) = duel(&r);
    for (e, want) in [
        (4096u16, Answer::CanFire),
        (4097, Answer::NotFacing),
        (4352, Answer::NotFacing),
        (61440, Answer::CanFire),
    ] {
        aim_off(&mut w, a, b, e);
        assert_eq!(ask(&w, a), want, "aim error {e}");
    }
}

// spec: S-0310 claim 2 (an omni-fire weapon, and every weapon of a visceroid type, is never not facing)
#[test]
fn claim_2_omni_fire_and_visceroids_need_no_aim() {
    for case in 0..3 {
        let mut r = base();
        match case {
            0 => r.sim.weapons.get_mut("ZAP").unwrap().omni_fire = Some(true),
            1 => r.sim.vehicles.get_mut("CART").unwrap().small_visceroid = Some(true),
            _ => r.sim.vehicles.get_mut("CART").unwrap().large_visceroid = Some(true),
        }
        let (mut w, a, b) = duel(&r);
        for e in [2304u16, 4352, 16384, 32768] {
            aim_off(&mut w, a, b, e);
            assert_eq!(ask(&w, a), Answer::CanFire, "case {case}, aim error {e}");
        }
    }
}

// ---------------------------------------------------------------------------
// Claim 3: still turning.

/// Starts a slow turret turn that ends on the target's direction and is
/// still running at the end of the previous tick.
fn turning_onto_target(w: &mut World, a: ObjectId, b: ObjectId) {
    let d0 = dir(w, a, b);
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.turret = Turning::at(d0.wrapping_add(8192));
    o.turret.turn_to(d0, t - 2, 1);
    assert!(o.turret.turning(t - 1));
}

// spec: S-0310 claim 3 (a turret turning at the end of the previous tick gives still turning, whatever its aim error)
#[test]
fn claim_3_still_turning() {
    let (mut w, a, b) = duel(&base());
    turning_onto_target(&mut w, a, b);
    assert_eq!(ask(&w, a), Answer::StillTurning);
    // Even within the tolerance (about 300 off the target), if the turn was still running at the end of the previous tick.
    let d0 = dir(&w, a, b);
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.turret = Turning::at(d0.wrapping_add(600));
    o.turret.turn_to(d0, t - 1, 1);
    assert!(o.turret.value(t) != d0 || o.turret.turning(t - 1));
    assert_eq!(ask(&w, a), Answer::StillTurning);
}

// spec: S-0310 claim 3 (not for a homing projectile: the aim test decides)
#[test]
fn claim_3_homing_projectile_is_never_still_turning() {
    let mut r = base();
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("HOMER".into());
    let (mut w, a, b) = duel(&r);
    turning_onto_target(&mut w, a, b);
    let e = glasnost_num::geom::delta(w.obj(a).unwrap().turret.value(w.tick), dir(&w, a, b)).abs();
    let want = if e <= 4096 {
        Answer::CanFire
    } else {
        Answer::NotFacing
    };
    assert_eq!(ask(&w, a), want);
}

// spec: S-0310 claim 3 (a vehicle without the turret flag, or with a spinning turret, never gets still turning)
#[test]
fn claim_3_no_turret_or_spinning_turret_never_still_turning() {
    for case in 0..2 {
        let mut r = base();
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        if case == 0 {
            cart.turret = Some(false);
        } else {
            cart.turret_spins = Some(true);
        }
        let (mut w, a, b) = duel(&r);
        turning_onto_target(&mut w, a, b);
        assert_ne!(ask(&w, a), Answer::StillTurning, "case {case}");
    }
}

// ---------------------------------------------------------------------------
// Claim 4: buildings.

/// Rules with an Alpha building TURRET1 armed with ZAP; `turret` and
/// `voxel` set its turret flag and voxel turret art flag.
fn tower_rules(turret: bool, voxel: bool) -> Ruleset {
    let mut r = base();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("TURRET1".into());
    r.sim.buildings.insert(
        "TURRET1".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([1, 1]),
            primary: Some("ZAP".into()),
            turret: Some(turret),
            rot: Some(5),
            turret_anim_is_voxel: Some(voxel),
            ..Default::default()
        },
    );
    r
}

fn tower_duel(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    duel_of(r, Kind::Building, "TURRET1")
}

/// Puts a building's turret at an aim error of `e` from the direction of
/// its target measured from its aiming point.
fn tower_off(w: &mut World, t: ObjectId, b: ObjectId, e: u16) {
    let f = dir_from(w, building_aim_point(w, t), b).wrapping_add(e);
    turret_at(w, t, f);
}

// spec: S-0310 claim 4 and F2 (a building with a sprite turret: τ = 2048, whatever the projectile)
#[test]
fn claim_4_sprite_turret_tolerance() {
    for homing in [false, true] {
        let mut r = tower_rules(true, false);
        if homing {
            r.sim.buildings.get_mut("TURRET1").unwrap().primary = Some("HOMER".into());
        }
        let (mut w, t, b) = tower_duel(&r);
        for (e, want) in [
            (2048u16, Answer::CanFire),
            (2049, Answer::NotFacing),
            (4096, Answer::NotFacing),
        ] {
            tower_off(&mut w, t, b, e);
            assert_eq!(ask(&w, t), want, "homing {homing}, aim error {e}");
        }
    }
}

// spec: S-0310 claim 4 and Edge cases (quirks.voxel_turret_exact_aim on: a voxel turret has τ = 0; off: the sprite tolerance 2048)
#[test]
fn quirk_voxel_turret_exact_aim() {
    for exact in [true, false] {
        let mut r = tower_rules(true, true);
        r.globals.quirks.voxel_turret_exact_aim = exact;
        let (mut w, t, b) = tower_duel(&r);
        tower_off(&mut w, t, b, 0);
        assert_eq!(ask(&w, t), Answer::CanFire, "exact {exact}, on target");
        tower_off(&mut w, t, b, 1);
        let want = if exact {
            Answer::NotFacing
        } else {
            Answer::CanFire
        };
        assert_eq!(ask(&w, t), want, "exact {exact}, aim error 1");
        tower_off(&mut w, t, b, 2048);
        assert_eq!(ask(&w, t), want, "exact {exact}, aim error 2048");
        tower_off(&mut w, t, b, 2049);
        assert_eq!(
            ask(&w, t),
            Answer::NotFacing,
            "exact {exact}, aim error 2049"
        );
    }
}

// spec: S-0310 claim 4 (a building that is not turreted fires in any direction)
#[test]
fn claim_4_unturreted_building_fires_in_any_direction() {
    let (mut w, t, b) = tower_duel(&tower_rules(false, false));
    for e in [0u16, 4096, 16384, 32768] {
        tower_off(&mut w, t, b, e);
        assert_eq!(ask(&w, t), Answer::CanFire, "aim error {e}");
    }
}

// spec: S-0310 claim 4 (the aiming point: the reference point shifted by the primary fire pixel shift; else by the turret anchor's when not zero; else none; S-0313 F3)
#[test]
fn claim_4_aiming_point_of_a_building() {
    let cases: [(Pixels, Pixels); 4] = [
        (Some([12, -7]), Some([3, 4])),
        (None, Some([3, 4])),
        (None, Some([0, 0])),
        (Some([65535, 65535]), Some([3, 4])),
    ];
    for (i, (fire, anim)) in cases.into_iter().enumerate() {
        let mut r = tower_rules(true, false);
        let bt = r.sim.buildings.get_mut("TURRET1").unwrap();
        bt.primary_fire_pixel_offset = fire;
        bt.turret_anim_xy = anim;
        let (w, t, _) = tower_duel(&r);
        let p = w.reference_point(t);
        let shift = match (i, fire, anim) {
            (0, Some([x, y]), _) => pixel_to_leptons(&w, x, y),
            (1 | 3, _, Some([x, y])) => pixel_to_leptons(&w, x, y),
            _ => (0, 0),
        };
        assert_eq!(
            building_aim_point(&w, t),
            Pos::new(p.x + shift.0, p.y + shift.1, p.z),
            "case {i}"
        );
    }
}

// spec: S-0310 claim 4 (scenario T4 b: the direction is measured from the aiming point, not from the reference point)
#[test]
fn claim_4_direction_from_the_aiming_point() {
    let mut r = tower_rules(true, false);
    // A large shift towards map north (screen right and up: Δx = Δy·(−1) pattern of F3).
    r.sim
        .buildings
        .get_mut("TURRET1")
        .unwrap()
        .primary_fire_pixel_offset = Some([0, -60]);
    let (mut w, t, b) = tower_duel(&r);
    let from_aim = dir_from(&w, building_aim_point(&w, t), b);
    let from_ref = dir(&w, t, b);
    assert!(
        glasnost_num::geom::delta(from_aim, from_ref).abs() > 2048,
        "the shift moves the direction by more than the tolerance"
    );
    turret_at(&mut w, t, from_aim);
    assert_eq!(ask(&w, t), Answer::CanFire);
    turret_at(&mut w, t, from_ref);
    assert_eq!(ask(&w, t), Answer::NotFacing);
}

// ---------------------------------------------------------------------------
// Claim 5: infantry and aircraft.

// spec: S-0310 claim 5 (infantry never get not facing)
#[test]
fn claim_5_infantry_never_not_facing() {
    let (mut w, a, b) = duel_of(&base(), Kind::Infantry, "WALKER");
    for e in [0u16, 4096, 32768] {
        aim_off(&mut w, a, b, e);
        assert_eq!(ask(&w, a), Answer::CanFire, "aim error {e}");
    }
}

/// An Alpha aircraft KITE armed with `weapon`, at height 512 above its cell.
fn kite_duel(weapon: &str) -> (World, ObjectId, ObjectId) {
    let mut r = base();
    r.registries.insert("aircraft".into(), vec!["KITE".into()]);
    r.sim.aircraft.insert(
        "KITE".into(),
        glasnost_rules::sim::UnitType {
            strength: Some(300),
            primary: Some(weapon.into()),
            ..Default::default()
        },
    );
    duel_of(&r, Kind::Aircraft, "KITE")
}

// spec: S-0310 claim 5 and F2 (an aircraft compares its turret facing with τ = 2048, whatever the projectile)
#[test]
fn claim_5_aircraft_tolerance_2048() {
    for weapon in ["ZAP", "HOMER"] {
        let (mut w, a, b) = kite_duel(weapon);
        let d0 = dir(&w, a, b);
        turret_at(&mut w, a, d0.wrapping_add(2048));
        body_at(&mut w, a, d0.wrapping_add(16384));
        assert_eq!(ask(&w, a), Answer::CanFire, "{weapon}, 2048");
        turret_at(&mut w, a, d0.wrapping_add(2049));
        body_at(&mut w, a, d0);
        assert_eq!(ask(&w, a), Answer::NotFacing, "{weapon}, 2049");
    }
}

// ---------------------------------------------------------------------------
// Claim 6: precedence.

// spec: S-0310 claim 6 (the general part outranks the aiming answers: a rearming vehicle never reports not facing; moving outranks still turning and not facing)
#[test]
fn claim_6_aiming_answers_rank_lowest() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().fire_while_moving = Some(false);
    let (mut w, a, b) = duel(&r);
    aim_off(&mut w, a, b, 16384);
    assert_eq!(ask(&w, a), Answer::NotFacing);
    turning_onto_target(&mut w, a, b);
    assert_eq!(ask(&w, a), Answer::StillTurning);
    w.obj_mut(a).unwrap().dest = Some(Target::Cell(testkit::mid(N, -6, 0)));
    assert_eq!(ask(&w, a), Answer::Moving);
    w.obj_mut(a).unwrap().combat.rearm_until = w.tick + 5;
    assert_eq!(ask(&w, a), Answer::Rearming);
}

// spec: S-0310 claim 6 and claim 11 (an infantry healer's illegal outranks the general part: rearming and out of range)
#[test]
fn claim_6_infantry_healer_illegal_outranks_the_general_part() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().damage = Some(-5);
    let (mut w, a, b) = duel_of(&r, Kind::Infantry, "WALKER");
    w.obj_mut(a).unwrap().combat.rearm_until = w.tick + 10;
    w.obj_mut(b).unwrap().pos = Pos::centre(testkit::mid(N, -11, -11), 0);
    // The target is a vehicle: not infantry.
    assert_eq!(ask(&w, a), Answer::Illegal);
}

// ---------------------------------------------------------------------------
// Claim 7: vehicles while moving.

fn dest(w: &mut World, a: ObjectId) {
    w.obj_mut(a).unwrap().dest = Some(Target::Cell(testkit::mid(N, -6, 0)));
}

// spec: S-0310 claim 7 (a type without the mobile-fire flag that has a destination: moving, for any weapon)
#[test]
fn claim_7_no_mobile_fire_with_a_destination() {
    let mut r = base();
    r.sim.vehicles.get_mut("CART").unwrap().mobile_fire = Some(false);
    let (mut w, a, _) = duel(&r);
    assert_eq!(ask(&w, a), Answer::CanFire);
    dest(&mut w, a);
    assert_eq!(ask(&w, a), Answer::Moving);
}

// spec: S-0310 claim 7 (a weapon without the fire-while-moving flag and a destination: moving; having a destination counts, not motion)
#[test]
fn claim_7_weapon_without_fire_while_moving() {
    for fwm in [false, true] {
        let mut r = base();
        r.sim.weapons.get_mut("ZAP").unwrap().fire_while_moving = Some(fwm);
        let (mut w, a, _) = duel(&r);
        dest(&mut w, a);
        assert!(
            !glasnost_sim::occupancy::in_progress(&w, a),
            "standing still"
        );
        let want = if fwm { Answer::CanFire } else { Answer::Moving };
        assert_eq!(ask(&w, a), want, "fire while moving {fwm}");
    }
}

// spec: S-0310 claim 7 (a balloon-hover type: the fire-while-moving condition reads "its locomotor is moving it", so a stopped balloon with a destination may fire)
#[test]
fn claim_7_balloon_hover_reads_locomotor_motion() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().fire_while_moving = Some(false);
    r.sim.vehicles.get_mut("CART").unwrap().balloon_hover = Some(true);
    let (mut w, a, _) = duel(&r);
    dest(&mut w, a);
    assert_eq!(ask(&w, a), Answer::CanFire);
    // The mobile-fire condition is not changed by the flag.
    let mut r2 = r.clone();
    r2.sim.vehicles.get_mut("CART").unwrap().mobile_fire = Some(false);
    let (mut w, a, _) = duel(&r2);
    dest(&mut w, a);
    assert_eq!(ask(&w, a), Answer::Moving);
}

// spec: S-0310 claim 7 (a spark-particle or fire-particle weapon with a destination: moving, even with mobile fire and fire while moving)
#[test]
fn claim_7_particle_weapons_with_a_destination() {
    for spark in [true, false] {
        let mut r = base();
        let wpn = r.sim.weapons.get_mut("ZAP").unwrap();
        wpn.fire_while_moving = Some(true);
        if spark {
            wpn.use_spark_particles = Some(true);
        } else {
            wpn.use_fire_particles = Some(true);
        }
        let (mut w, a, _) = duel(&r);
        assert_eq!(ask(&w, a), Answer::CanFire);
        dest(&mut w, a);
        assert_eq!(ask(&w, a), Answer::Moving, "spark {spark}");
    }
}

// spec: S-0310 claim 7 (a temporal primary weapon with a destination: moving, without any beam; a temporal secondary does not give it)
#[test]
fn claim_7_temporal_primary_with_a_destination() {
    let mut r = base();
    r.sim.weapons.insert("TIMER".into(), {
        let mut z = zap();
        z.temporal = Some(true);
        z.fire_while_moving = Some(true);
        z
    });
    // Primary temporal.
    let mut r1 = r.clone();
    r1.sim.vehicles.get_mut("CART").unwrap().primary = Some("TIMER".into());
    let (mut w, a, _) = duel(&r1);
    dest(&mut w, a);
    assert_eq!(ask(&w, a), Answer::Moving);
    // Secondary temporal, ordinary primary: the primary fires with a destination.
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.secondary = Some("TIMER".into());
    let (mut w, a, _) = duel(&r);
    dest(&mut w, a);
    assert_eq!(ask(&w, a), Answer::CanFire);
}

// ---------------------------------------------------------------------------
// Claim 8: infantry while moving.

// spec: S-0310 claim 8 (an infantry unit that is walking (speed fraction 1) gets moving)
#[test]
fn claim_8_walking_infantry_is_moving() {
    let (mut w, a, _) = duel_of(&base(), Kind::Infantry, "WALKER");
    assert_eq!(ask(&w, a), Answer::CanFire);
    let issued = {
        while w.tick <= w.orders_blocked_through {
            testkit::run(&mut w, 1);
        }
        glasnost_sim::orders::issue(
            &mut w,
            glasnost_sim::world::HouseId(0),
            glasnost_sim::orders::Order::Behaviour {
                actor: a,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(testkit::mid(N, -2, 8))),
            },
        )
    };
    assert!(issued);
    let mut seen = false;
    for _ in 0..40 {
        testkit::run(&mut w, 1);
        let walking = w.obj(a).unwrap().walk.as_ref().is_some_and(|s| s.fraction); // spec: S-0310 Terms (the speed fraction is 1 from a walking step, not from the choice of a walking target)
        if walking {
            // Every other reason absent: the countdown over, the walk the only one left.
            w.obj_mut(a).unwrap().combat.rearm_until = i32::MIN;
            let tg = w.obj(a).unwrap().target;
            assert_eq!(check(&w, a, tg, 0, false), Answer::Moving);
            seen = true;
            break;
        }
    }
    assert!(seen, "the soldier walked");
}

// spec: S-0310 claim 8 (a destination and a current sequence that is not interruptible: moving; an interruptible one: not)
#[test]
fn claim_8_destination_and_uninterruptible_sequence() {
    use glasnost_sim::infantry::Seq;
    let (mut w, a, _) = duel_of(&base(), Kind::Infantry, "WALKER");
    dest(&mut w, a);
    w.obj_mut(a).unwrap().infantry.as_mut().unwrap().seq = Some(Seq::Stand);
    assert_eq!(ask(&w, a), Answer::CanFire);
    // Go-prone is not interruptible (S-0107 T1).
    w.obj_mut(a).unwrap().infantry.as_mut().unwrap().seq = Some(Seq::GoProne);
    assert_eq!(ask(&w, a), Answer::Moving);
    w.obj_mut(a).unwrap().dest = None;
    assert_eq!(ask(&w, a), Answer::CanFire);
}

// spec: S-0310 claim 8 (a fire-particle weapon gets moving while the unit has a destination)
#[test]
fn claim_8_fire_particle_infantry_with_a_destination() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().use_fire_particles = Some(true);
    let (mut w, a, _) = duel_of(&r, Kind::Infantry, "WALKER");
    assert_eq!(ask(&w, a), Answer::CanFire);
    dest(&mut w, a);
    assert_eq!(ask(&w, a), Answer::Moving);
}

// ---------------------------------------------------------------------------
// Claim 9: the vehicle's replies.

// spec: S-0310 claim 9 (can fire: fires)
#[test]
fn claim_9_can_fire_fires() {
    let (mut w, a, _) = duel(&base());
    assert!(!behave::vehicle_step(&mut w, a));
    assert_eq!(w.obj(a).unwrap().last_fired, w.tick);
}

// spec: S-0310 claim 9 (not facing, with the turret flag: the turret's target facing becomes the direction of the target)
#[test]
fn claim_9_not_facing_turret_reaims() {
    let (mut w, a, b) = duel(&base());
    aim_off(&mut w, a, b, 16384);
    assert!(behave::vehicle_step(&mut w, a), "a facing error");
    let want = dir(&w, a, b);
    assert_eq!(behave::turret_target(&w, a, true), Some(want));
}

// spec: S-0310 claim 9 (not facing, without the turret flag, no destination, not moved: the body's target facing becomes the direction; with a destination: none)
#[test]
fn claim_9_not_facing_body_reaims_only_without_destination() {
    let mut r = base();
    r.sim.vehicles.get_mut("CART").unwrap().turret = Some(false);
    let (mut w, a, b) = duel(&r);
    aim_off(&mut w, a, b, 16384);
    assert!(!behave::vehicle_step(&mut w, a));
    let want = dir(&w, a, b);
    assert_eq!(w.obj(a).unwrap().body.target, want);
    assert_eq!(w.obj(a).unwrap().turret.target, want, "the turret follows");
    // With a destination: no reply.
    let (mut w, a, b) = duel(&r);
    aim_off(&mut w, a, b, 16384);
    let before = w.obj(a).unwrap().body.target;
    dest(&mut w, a);
    behave::vehicle_step(&mut w, a);
    assert_eq!(w.obj(a).unwrap().body.target, before);
}

// spec: S-0310 claim 9 (still turning, moving and rearming: no reply here)
#[test]
fn claim_9_other_answers_get_no_reply() {
    let mut r = base();
    r.sim.vehicles.get_mut("CART").unwrap().turret = Some(false);
    let (mut w, a, b) = duel(&r);
    aim_off(&mut w, a, b, 16384);
    w.obj_mut(a).unwrap().combat.rearm_until = w.tick + 5;
    let before = w.obj(a).unwrap().clone();
    assert!(!behave::vehicle_step(&mut w, a));
    let o = w.obj(a).unwrap();
    assert_eq!(o.body.target, before.body.target);
    assert_eq!(o.target, before.target);
    assert_eq!(o.last_fired, before.last_fired);
}

// ---------------------------------------------------------------------------
// Claim 11: healing weapons.

fn healer() -> Ruleset {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().damage = Some(-5);
    r
}

// spec: S-0310 claim 11 (a vehicle healer: a damaged vehicle target is legal and kept; at full health the check answers illegal and the vehicle drops it)
#[test]
fn claim_11_vehicle_healer_full_health_target() {
    let (mut w, a, b) = duel(&healer());
    w.obj_mut(b).unwrap().health -= 10;
    assert_eq!(ask(&w, a), Answer::CanFire);
    let (mut w, a, _) = duel(&healer());
    assert_eq!(ask(&w, a), Answer::Illegal);
    behave::vehicle_step(&mut w, a);
    assert_eq!(w.obj(a).unwrap().target, None);
}

// spec: S-0310 claim 11 (a vehicle healer with a target that is not a vehicle in the glossary sense: drops it)
#[test]
fn claim_11_vehicle_healer_drops_infantry() {
    let (mut w, a, _) = duel(&healer());
    let inf = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(N, 1, 0),
        0,
    );
    w.obj_mut(inf).unwrap().health -= 10;
    w.obj_mut(a).unwrap().target = Some(Target::Object(inf));
    assert_eq!(ask(&w, a), Answer::Illegal);
    behave::vehicle_step(&mut w, a);
    assert_eq!(w.obj(a).unwrap().target, None);
}

// spec: S-0310 claim 11 (a damaged vehicle whose type is marked non-vehicle: illegal, but the healer keeps it and does nothing)
#[test]
fn claim_11_vehicle_healer_keeps_a_damaged_non_vehicle() {
    let mut r = healer();
    r.sim.vehicles.get_mut("BUGGY").unwrap().non_vehicle = Some(true);
    let (mut w, a, b) = duel(&r);
    w.obj_mut(b).unwrap().health -= 10;
    assert_eq!(ask(&w, a), Answer::Illegal);
    behave::vehicle_step(&mut w, a);
    assert_eq!(w.obj(a).unwrap().target, Some(Target::Object(b)));
    assert_eq!(w.obj(a).unwrap().last_fired, -100);
}

// spec: S-0310 claim 11 (an infantry healer: illegal when its target is absent, not infantry, or at full health; a damaged infantry target is legal)
#[test]
fn claim_11_infantry_healer() {
    let (mut w, a, _) = duel_of(&healer(), Kind::Infantry, "WALKER");
    assert_eq!(ask(&w, a), Answer::Illegal, "a vehicle target");
    let inf = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(N, -1, 1),
        0,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(inf));
    assert_eq!(ask(&w, a), Answer::Illegal, "full health");
    w.obj_mut(inf).unwrap().health -= 10;
    assert_eq!(ask(&w, a), Answer::CanFire, "damaged infantry");
    assert_eq!(check(&w, a, None, 0, true), Answer::Illegal, "no target");
}
