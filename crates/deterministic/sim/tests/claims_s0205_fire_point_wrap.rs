//! Claim tests for a building's pixel-shifted fire point at the end of the
//! 32-bit range (S-0205 claim 1 with S-0313 claims 7 and 8), on synthetic
//! data: the shift's lepton sum wraps (#153).
//!
//! The ruleset is `testkit::combat_ruleset()` with an invented weapon and
//! building added here (no stock values). The pixel shift is chosen by
//! search so that its lepton shift, added to the base point, passes the
//! range; the expected point is the wrapped sum.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, WeaponType};
use glasnost_sim::combat::position::{fire_point, pixel_to_leptons};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

/// Map size of every test here.
const N: i32 = 24;

/// An invented ruleset: the PAD building (1 × 1, turreted, muzzle
/// (120, 40, 60)) with an invented weapon, and a durable BUGGY target.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.insert(
        "ZAP".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(30),
            range: Some(Decimal("10".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(30000);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("PAD".into());
    r.sim.buildings.insert(
        "PAD".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([1, 1]),
            primary: Some("ZAP".into()),
            turret: Some(true),
            rot: Some(5),
            combat: glasnost_rules::sim::CombatFields {
                primary_fire_flh: Some([120, 40, 60]),
                turret_offset: Some(0),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    r
}

/// A world of `r` with PAD at the middle, its turret facing east, aiming
/// at a BUGGY three cells east.
fn scene(r: &Ruleset) -> (World, ObjectId) {
    let mut w = testkit::game_with(r, N, 7);
    let t = testkit::spawn(&mut w, Kind::Building, "PAD", 0, testkit::mid(N, 0, 0), 16384);
    let b = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, testkit::mid(N, 3, 0), 49152);
    let o = w.obj_mut(t).unwrap();
    o.target = Some(Target::Object(b));
    o.body = Turning::at(0);
    o.turret = Turning::at(16384);
    (w, t)
}

/// The base point (S-0313 Terms: the top corner of the foundation).
fn base_point(w: &World, t: ObjectId) -> Pos {
    let o = w.obj(t).unwrap();
    let c = o.pos.cell();
    Pos::new(256 * c.x, 256 * c.y, o.pos.z)
}

/// A pixel row q whose lepton shift (p = 0: equal in x and y, S-0313 F3)
/// passes the range when added to `room` or more leptons.
fn huge_shift(w: &World, room: i32) -> (i32, i32) {
    (251_000_000..252_000_000)
        .step_by(97)
        .map(|q| (q, pixel_to_leptons(w, 0, q).0))
        .find(|&(_, d)| d > i32::MAX - room)
        .expect("a pixel row whose shift passes the range")
}

// spec: S-0313 claim 7 (no dual-shift flag: the base point shifted by the pixel shift); S-0205 claim 1 (the sum wraps)
#[test]
fn a_primary_fire_pixel_shift_past_the_range_wraps() {
    let mut r = rules();
    let (w, t) = scene(&r);
    let bp = base_point(&w, t);
    let (q, d) = huge_shift(&w, bp.x.min(bp.y));
    r.sim.buildings.get_mut("PAD").unwrap().primary_fire_pixel_offset = Some([0, q]);
    let (w, t) = scene(&r);
    assert_eq!(
        fire_point(&w, t, 0, 0),
        Pos::new(bp.x.wrapping_add(d), bp.y.wrapping_add(d), bp.z)
    );
}

// spec: S-0313 claim 7 (with the dual-shift flag: claim 1's fire point shifted); S-0205 claim 1 (the sum wraps)
#[test]
fn a_dual_fire_pixel_shift_past_the_range_wraps() {
    let mut r = rules();
    let (w, t) = scene(&r);
    let plain = fire_point(&w, t, 0, 0);
    let (q, d) = huge_shift(&w, plain.x.min(plain.y));
    let b = r.sim.buildings.get_mut("PAD").unwrap();
    b.primary_fire_pixel_offset = Some([0, q]);
    b.primary_fire_dual_offset = Some(true);
    let (w, t) = scene(&r);
    assert_eq!(
        fire_point(&w, t, 0, 0),
        Pos::new(plain.x.wrapping_add(d), plain.y.wrapping_add(d), plain.z)
    );
}

// spec: S-0313 claim 8 (a voxel turret and no voxel barrel: claim 1's fire point shifted by the turret anchor's pixel shift); S-0205 claim 1 (the sum wraps)
#[test]
fn a_turret_anchor_shift_past_the_range_wraps() {
    let mut r = rules();
    let (w, t) = scene(&r);
    let plain = fire_point(&w, t, 0, 0);
    let (q, d) = huge_shift(&w, plain.x.min(plain.y));
    let b = r.sim.buildings.get_mut("PAD").unwrap();
    b.turret_anim_is_voxel = Some(true);
    b.turret_anim_xy = Some([0, q]);
    let (w, t) = scene(&r);
    assert_eq!(
        fire_point(&w, t, 0, 0),
        Pos::new(plain.x.wrapping_add(d), plain.y.wrapping_add(d), plain.z)
    );
}
