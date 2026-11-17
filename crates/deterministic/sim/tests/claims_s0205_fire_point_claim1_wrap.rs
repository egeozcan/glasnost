//! Claim tests for the fire point's remaining integer adds at the ends of
//! the 32-bit range (foundations F4, F3's seams in `combat::position`), on
//! synthetic data: a garrisoned building's muzzle shift (S-0313 claim 6),
//! claim 1's shift added to the base point, and the lateral part σ*L* of
//! F1. Each is an integer result, and S-0205 claim 1 says it wraps; S-0313
//! states no clamp. A debug build must give the wrapped result, as a
//! release build does, not panic.
//!
//! The rulesets are `testkit::combat_ruleset()` and the structure tests'
//! `structures_common::ruleset()`, with invented types added here (no stock
//! values).

mod structures_common;

use glasnost_map::triggers::Triggers;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, WeaponType};
use glasnost_sim::combat::position::{fire_point, pixel_to_leptons};
use glasnost_sim::rules::Kind;
use glasnost_sim::structures::garrison;
use glasnost_sim::testkit;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

/// Map size of the PAD scenes.
const N: i32 = 24;

/// An invented ruleset: the PAD building (1 × 1, turreted) with an invented
/// weapon whose muzzle triple is `flh`, and a durable BUGGY target.
fn pad_rules(flh: [i32; 3]) -> Ruleset {
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
                primary_fire_flh: Some(flh),
                turret_offset: Some(0),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    r
}

/// A world of `r` with PAD at the middle, its turret facing east (θ = 0,
/// S-0313 F2: a₃₂ = 8), aiming at a BUGGY three cells east, and PAD's world
/// height set to `z`.
fn pad_scene(r: &Ruleset, z: i32) -> (World, ObjectId) {
    let mut w = testkit::game_with(r, N, 7);
    let t = testkit::spawn(
        &mut w,
        Kind::Building,
        "PAD",
        0,
        testkit::mid(N, 0, 0),
        16384,
    );
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 3, 0),
        49152,
    );
    let o = w.obj_mut(t).unwrap();
    o.target = Some(Target::Object(b));
    o.body = Turning::at(0);
    o.turret = Turning::at(16384);
    o.pos.z = z;
    (w, t)
}

/// The base point (S-0313 Terms: the top corner of the foundation).
fn base_point(w: &World, t: ObjectId) -> Pos {
    let o = w.obj(t).unwrap();
    let c = o.pos.cell();
    Pos::new(256 * c.x, 256 * c.y, o.pos.z)
}

/// `v` narrowed to single precision toward zero (S-0313 F1: the triple is
/// converted from integers to single precision first; S-0205 claim 5),
/// for the values used here: below 2³¹ in magnitude and above 2²⁴, where a
/// single holds 24 significant bits.
fn single(v: i32) -> i32 {
    let m = i64::from(v).abs();
    let bits = 64 - m.leading_zeros() as i64;
    let drop = (bits - 24).max(0);
    let t = (m >> drop) << drop;
    (if v < 0 { -t } else { t }) as i32
}

// spec: S-0313 claim 1 and F1 (shift = (trunc u, trunc(−v), trunc w), added to the base point; with θ = 0 and δ = 0, u = F and v = σL); S-0205 claim 1 (the sums wrap)
#[test]
fn claim_1s_shift_past_the_range_wraps() {
    let bp = |r: &Ruleset| {
        let (w, t) = pad_scene(r, 1000);
        base_point(&w, t)
    };
    // F: u = F, added to the base point's x.
    let f = 2_147_483_000;
    let r = pad_rules([f, 0, 0]);
    let b = bp(&r);
    let (w, t) = pad_scene(&r, 1000);
    assert_eq!(fire_point(&w, t, 0, 0).x, b.x.wrapping_add(single(f)));
    // L: trunc(−v) = −L, added to the base point's y.
    let l = -2_147_483_000;
    let r = pad_rules([0, l, 0]);
    let (w, t) = pad_scene(&r, 1000);
    assert_eq!(fire_point(&w, t, 0, 0).y, b.y.wrapping_add(-single(l)));
    // H: w = H, added to the base point's height.
    let h = 2_147_483_000;
    let r = pad_rules([0, 0, h]);
    let (w, t) = pad_scene(&r, 1000);
    assert_eq!(fire_point(&w, t, 0, 0).z, 1000_i32.wrapping_add(single(h)));
}

// spec: S-0313 claim 2 and F1 (an odd burst counter negates the lateral part; σL is an integer converted to single precision); S-0205 claim 1 (−1 × −2147483648 wraps to −2147483648)
#[test]
fn the_lateral_part_of_an_odd_burst_counter_wraps() {
    let r = pad_rules([0, i32::MIN, 0]);
    let (w, t) = pad_scene(&r, 0);
    // σL wraps to L itself, so the odd counter fires from the even
    // counter's point.
    assert_eq!(fire_point(&w, t, 0, 1), fire_point(&w, t, 0, 0));
}

// spec: S-0313 claim 6 (a garrisoned building fires from its base point shifted by its muzzle point, F3, map x and y only; the height is the base point's); S-0205 claim 1 (the sums wrap)
#[test]
fn a_garrison_muzzle_shift_past_the_range_wraps() {
    use structures_common::*;
    let fixture = |q: i32| {
        let mut r = ruleset();
        r.sim.buildings.get_mut("HUT3").unwrap().garrison_muzzles = Some(vec![[0, q]]);
        let mut w = game_on(&r, Triggers::default());
        let b = put(&mut w, Kind::Building, "HUT3", 0, 0, 0);
        let u = put(&mut w, Kind::Infantry, "GARRI", 0, -8, -8);
        assert!(garrison::attempt(&mut w, u, b), "GARRI went inside");
        (w, b)
    };
    let (w, b) = fixture(0);
    let bp = base_point(&w, b);
    let room = bp.x.min(bp.y);
    assert!(room > 0, "the base point is east and south of the origin");
    // A pixel row q whose lepton shift (p = 0: equal in x and y, S-0313
    // F3) passes the range when added to the base point.
    let (q, d) = (251_000_000..252_000_000)
        .step_by(97)
        .map(|q| (q, pixel_to_leptons(&w, 0, q).0))
        .find(|&(_, d)| d > i32::MAX - room)
        .expect("a pixel row whose shift passes the range");
    let (w, b) = fixture(q);
    assert_eq!(garrison::muzzle_shift(&w, b, 0), (d, d));
    assert_eq!(
        fire_point(&w, b, 0, 0),
        Pos::new(bp.x.wrapping_add(d), bp.y.wrapping_add(d), bp.z)
    );
}
