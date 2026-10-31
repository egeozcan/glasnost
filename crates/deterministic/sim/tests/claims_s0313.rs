//! Claim tests for firing positions (S-0313), on synthetic data.
//!
//! The ruleset is `testkit::combat_ruleset()` with invented weapons and art
//! values added here (no stock values). Fire points are read from
//! `combat::position::fire_point` and from the shots that real firings
//! make (the presentation event of a shot carries its fire point).
//! Expected shifts are computed independently from the claims: with the
//! table trigonometry of S-0105 F1 and the angle of F2 in double precision,
//! within the 1-lepton tolerance of scenarios P1 and P3, and exactly where
//! the angle is 0 or the spec gives the numbers (F3, F4).
//!
//! Not tested (seams): passengers of open-topped transports (claim 5) and
//! garrisoned buildings (claim 6). The voxel barrel's own rule (claim 8) is
//! an open question of the spec; only its "not the voxel-turret row" part
//! is checked. See `docs/m4-claims/S-0313.md`.

use glasnost_num::geom::n_step;
use glasnost_num::trig::{cos_t, sin_t};
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, WeaponType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::fire;
use glasnost_sim::combat::position::{SPRAY, fire_point, pixel_to_leptons};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

/// Map size of every test here.
const N: i32 = 24;
/// The four map directions in facing units (north, east, south, west).
const FACINGS: [u16; 4] = [0, 16384, 32768, 49152];

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// An invented weapon: rate 30, range 10 cells, the invisible projectile
/// Bullet, warhead SA.
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

/// The combat ruleset with ZAP on the CART (turreted, muzzle `flh`, turret
/// displacement `delta`) and a durable BUGGY.
fn base(flh: [i32; 3], delta: i32) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.insert("ZAP".into(), zap());
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.primary = Some("ZAP".into());
    cart.combat.primary_fire_flh = Some(flh);
    cart.combat.turret_offset = Some(delta);
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.primary = Some("ZAP".into());
    walker.combat.primary_fire_flh = Some(flh);
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(30000);
    r
}

/// A world of `r` with an Alpha firer of `kind`/`name` at the middle and,
/// when `target`, a Beta BUGGY three cells east of it as its target.
fn scene(r: &Ruleset, kind: Kind, name: &str, target: bool) -> (World, ObjectId, Option<ObjectId>) {
    let mut w = testkit::game_with(r, N, 7);
    let a = testkit::spawn(&mut w, kind, name, 0, testkit::mid(N, 0, 0), 16384);
    let b = target.then(|| {
        testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "BUGGY",
            1,
            testkit::mid(N, 3, 0),
            49152,
        )
    });
    if let Some(b) = b {
        w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    }
    (w, a, b)
}

fn mode(w: &World) -> Rounding {
    w.rules().mode
}

/// θ of S-0313 F2 for a 32-step facing: (x₃₂ − 8) × c, c the double
/// nearest −π/16.
fn angle(f: u16, m: Rounding) -> F64 {
    let c = glasnost_num::decimal::nearest_double("-0.19634954084936207");
    F64::from_i64(i64::from(n_step(f, 32)) - 8, m).mul(c, m)
}

/// (cos, sin) of an angle narrowed to single precision, by S-0105's tables.
fn cs(a: F64, m: Rounding) -> (F64, F64) {
    let a = a.to_f32(m).to_f64();
    (cos_t(a, m), sin_t(a, m))
}

fn int(v: i32, m: Rounding) -> F64 {
    F64::from_i64(i64::from(v), m)
}

/// The shift of F1 as a model-frame (u, v, w) computed in double
/// precision: the turret displacement `delta` rotated by `tb`, the muzzle
/// position (F, σL, H) rotated by `ta`. Returns the map shift
/// (trunc u, trunc(−v), H).
fn shift(m: Rounding, tb: F64, ta: F64, delta: i32, flh: [i32; 3], sigma: i32) -> (i32, i32, i32) {
    let (cb, sb) = cs(tb, m);
    let (ca, sa) = cs(ta, m);
    let (f, l) = (int(flh[0], m), int(sigma * flh[1], m));
    let u = int(delta, m)
        .mul(cb, m)
        .add(f.mul(ca, m), m)
        .sub(l.mul(sa, m), m);
    let v = int(delta, m)
        .mul(sb, m)
        .add(f.mul(sa, m), m)
        .add(l.mul(ca, m), m);
    (u.to_i32_common(), v.negated().to_i32_common(), flh[2])
}

/// Asserts that `got` is `base` shifted by `want`, within 1 lepton per
/// axis (scenarios P1 and P3).
fn near(got: Pos, base: Pos, want: (i32, i32, i32), what: &str) {
    let s = (got.x - base.x, got.y - base.y, got.z - base.z);
    assert!(
        (s.0 - want.0).abs() <= 1 && (s.1 - want.1).abs() <= 1 && s.2 == want.2,
        "{what}: shift {s:?}, want {want:?}"
    );
}

/// The firer's body and turret at rest.
fn pose(w: &mut World, a: ObjectId, body: u16, turret: u16) {
    let o = w.obj_mut(a).unwrap();
    o.body = Turning::at(body);
    o.turret = Turning::at(turret);
}

// ---------------------------------------------------------------------------
// Claim 1 and Formula F1: the fire point.

// spec: S-0313 claim 1 and F1 (a firer without a turret: base point plus the muzzle position rotated with the body; exact at the body facing east, where θ is 0)
#[test]
fn claim_1_turretless_vehicle_rotates_with_the_body() {
    let flh = [100, 40, 50];
    let mut r = base(flh, 0);
    r.sim.vehicles.get_mut("CART").unwrap().turret = Some(false);
    let (mut w, a, _) = scene(&r, Kind::Vehicle, "CART", false);
    let m = mode(&w);
    let p = w.obj(a).unwrap().pos;
    pose(&mut w, a, 16384, 16384);
    assert_eq!(
        fire_point(&w, a, 0, 0),
        Pos::new(p.x + 100, p.y - 40, p.z + 50),
        "east: +x forward, +y (left) is map north"
    );
    for body in FACINGS {
        // The turret facing does not matter without the turret flag.
        pose(&mut w, a, body, body.wrapping_add(12345));
        let tb = angle(body, m);
        near(
            fire_point(&w, a, 0, 0),
            p,
            shift(m, tb, tb, 0, flh, 1),
            "body",
        );
    }
}

// spec: S-0313 claims 1 and 3 and F1, F2 (a turreted vehicle: M = the body rotation, δ along the body's forward axis, the muzzle rotated by the turret facing)
#[test]
fn claim_1_turreted_vehicle() {
    let flh = [100, 40, 50];
    let (mut w, a, _) = scene(&base(flh, 64), Kind::Vehicle, "CART", false);
    let m = mode(&w);
    let p = w.obj(a).unwrap().pos;
    for body in FACINGS {
        for turret in FACINGS {
            pose(&mut w, a, body, turret);
            let want = shift(m, angle(body, m), angle(turret, m), 64, flh, 1);
            near(
                fire_point(&w, a, 0, 0),
                p,
                want,
                &format!("body {body}, turret {turret}"),
            );
        }
    }
}

// spec: S-0313 claim 1 (the shift's map y is −v: a muzzle to the left of a vehicle facing east lies towards map north)
#[test]
fn claim_1_left_is_map_north_when_facing_east() {
    let (mut w, a, _) = scene(&base([0, 30, 0], 0), Kind::Vehicle, "CART", false);
    pose(&mut w, a, 16384, 16384);
    let p = w.obj(a).unwrap().pos;
    assert_eq!(fire_point(&w, a, 0, 0), Pos::new(p.x, p.y - 30, p.z));
}

// spec: S-0313 claim 1 (a real shot starts at the fire point)
#[test]
fn claim_1_a_shot_starts_at_the_fire_point() {
    let (mut w, a, _) = scene(&base([100, 40, 50], 64), Kind::Vehicle, "CART", true);
    let want = fire_point(&w, a, 0, 0);
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    let from = w.effects.iter().rev().find_map(|e| match e {
        Effect::Shot { firer, from, .. } if *firer == a => Some(*from),
        _ => None,
    });
    assert_eq!(from, Some(want));
}

// ---------------------------------------------------------------------------
// Claim 2: burst alternation.

// spec: S-0313 claim 2 (an even burst counter uses +L, an odd one −L)
#[test]
fn claim_2_lateral_part_alternates_with_the_counter() {
    let (mut w, a, _) = scene(&base([0, 30, 0], 0), Kind::Vehicle, "CART", false);
    pose(&mut w, a, 16384, 16384);
    let p = w.obj(a).unwrap().pos;
    for k in 0..4 {
        let side = if k % 2 == 0 { -30 } else { 30 };
        assert_eq!(
            fire_point(&w, a, 0, k),
            Pos::new(p.x, p.y + side, p.z),
            "k = {k}"
        );
    }
}

/// The fire points of `n` consecutive shots of the CART with a burst of
/// `burst`, the countdown cleared between shots.
fn burst_sides(burst: i32, n: usize) -> Vec<i32> {
    let mut r = base([0, 30, 0], 0);
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(burst);
    let (mut w, a, _) = scene(&r, Kind::Vehicle, "CART", true);
    pose(&mut w, a, 16384, 16384);
    let p = w.obj(a).unwrap().pos;
    let t = w.obj(a).unwrap().target.unwrap();
    let mut sides = Vec::new();
    for _ in 0..n {
        assert!(fire::shot(&mut w, a, 0, t));
        let from = w
            .effects
            .iter()
            .rev()
            .find_map(|e| match e {
                Effect::Shot { from, .. } => Some(*from),
                _ => None,
            })
            .unwrap();
        // −30 in map y is +L (to the left, map north).
        sides.push(if from.y - p.y < 0 { 1 } else { -1 });
        w.tick = w.obj(a).unwrap().combat.rearm_until;
    }
    sides
}

// spec: S-0313 claim 2 and scenario P2 (burst 2: +L, −L, +L, −L; burst 3: +L, −L, +L, then +L again; burst 1: always +L)
#[test]
fn claim_2_burst_alternation_in_real_shots() {
    assert_eq!(burst_sides(2, 4), vec![1, -1, 1, -1]);
    assert_eq!(burst_sides(3, 4), vec![1, -1, 1, 1]);
    assert_eq!(burst_sides(1, 3), vec![1, 1, 1]);
}

// ---------------------------------------------------------------------------
// Claim 3: the rotation angle.

// spec: S-0313 claim 3 (infantry: θ = 0, the muzzle rotates with the body only)
#[test]
fn claim_3_infantry_angle_is_zero() {
    let flh = [100, 40, 50];
    let (mut w, a, _) = scene(&base(flh, 0), Kind::Infantry, "WALKER", false);
    let m = mode(&w);
    let p = w.obj(a).unwrap().pos;
    for body in FACINGS {
        pose(&mut w, a, body, body.wrapping_add(16384));
        let tb = angle(body, m);
        near(
            fire_point(&w, a, 0, 0),
            p,
            shift(m, tb, tb, 0, flh, 1),
            "infantry",
        );
    }
}

/// Rules with an Alpha building PAD (foundation `fw` × `fh`) armed with
/// ZAP, muzzle `flh`, turret displacement `delta`, turret flag `turret`.
fn pad_rules(fw: i32, fh: i32, flh: [i32; 3], delta: i32, turret: bool) -> Ruleset {
    let mut r = base([0, 0, 0], 0);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("PAD".into());
    r.sim.buildings.insert(
        "PAD".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([fw, fh]),
            primary: Some("ZAP".into()),
            turret: Some(turret),
            rot: Some(5),
            combat: glasnost_rules::sim::CombatFields {
                primary_fire_flh: Some(flh),
                turret_offset: Some(delta),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    r
}

/// The base point of a building: the top corner of its foundation at its
/// world height (S-0313 Terms).
fn base_point(w: &World, t: ObjectId) -> Pos {
    let o = w.obj(t).unwrap();
    let c = o.pos.cell();
    Pos::new(256 * c.x, 256 * c.y, o.pos.z)
}

// spec: S-0313 claim 3 and F2 (a turreted building: θ from its turret facing, M the identity)
#[test]
fn claim_3_turreted_building_angle() {
    let flh = [120, 40, 60];
    let (mut w, t, _) = scene(&pad_rules(1, 1, flh, 0, true), Kind::Building, "PAD", true);
    let m = mode(&w);
    let bp = base_point(&w, t);
    for f in FACINGS {
        pose(&mut w, t, 0, f);
        let want = shift(m, F64::ZERO, angle(f, m), 0, flh, 1);
        near(fire_point(&w, t, 0, 0), bp, want, &format!("turret {f}"));
    }
}

// spec: S-0313 claim 3 (a building that is not turreted: θ from the direction of its target; with no target, from its turret facing)
#[test]
fn claim_3_unturreted_building_angle() {
    let flh = [120, 40, 60];
    let (mut w, t, b) = scene(&pad_rules(1, 1, flh, 0, false), Kind::Building, "PAD", true);
    let m = mode(&w);
    let bp = base_point(&w, t);
    let b = b.unwrap();
    // The target to the south of the building; the turret facing north.
    let south = Pos::centre(testkit::mid(N, 0, 3), 0);
    w.obj_mut(b).unwrap().pos = south;
    pose(&mut w, t, 0, 0);
    let rp = w.reference_point(t);
    let dt = glasnost_num::geom::direction(
        i64::from(south.x - rp.x),
        i64::from(south.y - rp.y),
        w.rules().quirks.table_direction,
    );
    let want = shift(m, F64::ZERO, angle(dt, m), 0, flh, 1);
    near(fire_point(&w, t, 0, 0), bp, want, "with a target");
    w.obj_mut(t).unwrap().target = None;
    let want = shift(m, F64::ZERO, angle(0, m), 0, flh, 1);
    near(
        fire_point(&w, t, 0, 0),
        bp,
        want,
        "no target: the turret facing",
    );
}

// ---------------------------------------------------------------------------
// Claim 4: the building turret displacement.

// spec: S-0313 claim 4 and Edge cases (quirks.building_fire_point_shift_fixed on: δ is along map east whatever the turret facing; scenario P3: 512 leptons east of the top corner)
#[test]
fn quirk_building_fire_point_shift_fixed_on() {
    let (mut w, t, _) = scene(
        &pad_rules(1, 1, [0, 0, 0], 512, true),
        Kind::Building,
        "PAD",
        true,
    );
    let bp = base_point(&w, t);
    for f in FACINGS {
        pose(&mut w, t, 0, f);
        assert_eq!(
            fire_point(&w, t, 0, 0),
            Pos::new(bp.x + 512, bp.y, bp.z),
            "turret {f}"
        );
    }
}

// spec: S-0313 Edge cases (quirks.building_fire_point_shift_fixed off: T = Rz(θ) · Tr(δ, 0, 0) · Tr(F, σL, H), the displacement turns with the turret)
#[test]
fn quirk_building_fire_point_shift_fixed_off() {
    let mut r = pad_rules(1, 1, [0, 0, 0], 512, true);
    r.globals.quirks.building_fire_point_shift_fixed = false;
    let (mut w, t, _) = scene(&r, Kind::Building, "PAD", true);
    let m = mode(&w);
    let bp = base_point(&w, t);
    for f in FACINGS {
        pose(&mut w, t, 0, f);
        let want = shift(m, F64::ZERO, angle(f, m), 0, [512, 0, 0], 1);
        near(fire_point(&w, t, 0, 0), bp, want, &format!("turret {f}"));
    }
}

// spec: S-0313 Terms (a building's base point is the top corner of its foundation, at the building's world height)
#[test]
fn terms_building_base_point_is_the_top_corner() {
    for (fw, fh) in [(1, 1), (2, 2), (3, 2)] {
        let (mut w, t, _) = scene(
            &pad_rules(fw, fh, [0, 0, 0], 0, false),
            Kind::Building,
            "PAD",
            false,
        );
        pose(&mut w, t, 0, 0);
        let c = testkit::mid(N, 0, 0);
        let z = w.obj(t).unwrap().pos.z;
        assert_eq!(
            fire_point(&w, t, 0, 0),
            Pos::new(256 * c.x, 256 * c.y, z),
            "{fw} × {fh}"
        );
    }
}

// ---------------------------------------------------------------------------
// Formula F3: pixel shifts.

// spec: S-0313 F3 (Δx = trunc(βq + αp), Δy = trunc(βq − αp), α near 256/60, β near 512/60: a pixel right moves east and north equally, a pixel down east and south)
#[test]
fn f3_pixel_shift_to_leptons() {
    let (w, _, _) = scene(&base([0, 0, 0], 0), Kind::Vehicle, "CART", false);
    for ((p, q), want) in [
        ((0, 0), (0, 0)),
        ((1, 0), (4, -4)),
        ((-1, 0), (-4, 4)),
        ((0, 1), (8, 8)),
        ((60, 0), (256, -256)),
        ((0, 30), (256, 256)),
        ((15, 30), (320, 192)),
        ((0, -30), (-256, -256)),
    ] {
        assert_eq!(pixel_to_leptons(&w, p, q), want, "pixels ({p}, {q})");
    }
}

// ---------------------------------------------------------------------------
// Claims 7 and 8: building pixel shifts and voxel turrets.

// spec: S-0313 claim 7 (a primary fire pixel shift without the dual-shift flag: the base point shifted by it, at the base point's height, the muzzle position not used)
#[test]
fn claim_7_primary_fire_pixel_shift() {
    let mut r = pad_rules(1, 1, [120, 40, 60], 0, true);
    r.sim
        .buildings
        .get_mut("PAD")
        .unwrap()
        .primary_fire_pixel_offset = Some([15, 30]);
    let (mut w, t, _) = scene(&r, Kind::Building, "PAD", true);
    pose(&mut w, t, 0, 4096);
    let bp = base_point(&w, t);
    assert_eq!(
        fire_point(&w, t, 0, 0),
        Pos::new(bp.x + 320, bp.y + 192, bp.z)
    );
}

// spec: S-0313 claim 7 (with the dual-shift flag: the fire point of claim 1 shifted by the pixel shift)
#[test]
fn claim_7_dual_shift_flag() {
    let flh = [120, 40, 60];
    let mut r = pad_rules(1, 1, flh, 0, true);
    let b = r.sim.buildings.get_mut("PAD").unwrap();
    b.primary_fire_pixel_offset = Some([15, 30]);
    b.primary_fire_dual_offset = Some(true);
    let (mut w, t, _) = scene(&r, Kind::Building, "PAD", true);
    let m = mode(&w);
    pose(&mut w, t, 0, 16384);
    let bp = base_point(&w, t);
    let (u, v, h) = shift(m, F64::ZERO, angle(16384, m), 0, flh, 1);
    assert_eq!(
        fire_point(&w, t, 0, 0),
        Pos::new(bp.x + u + 320, bp.y + v + 192, bp.z + h)
    );
}

// spec: S-0313 claim 7 (a pixel shift of (65535, 65535) counts as not given)
#[test]
fn claim_7_not_given_pixel_shift() {
    let flh = [120, 40, 60];
    let mut r = pad_rules(1, 1, flh, 0, true);
    r.sim
        .buildings
        .get_mut("PAD")
        .unwrap()
        .primary_fire_pixel_offset = Some([65535, 65535]);
    let (mut w, t, _) = scene(&r, Kind::Building, "PAD", true);
    pose(&mut w, t, 0, 16384);
    let bp = base_point(&w, t);
    assert_eq!(
        fire_point(&w, t, 0, 0),
        Pos::new(bp.x + 120, bp.y - 40, bp.z + 60)
    );
}

// spec: S-0313 claim 8 (a voxel turret and no voxel barrel: the fire point of claim 1 shifted by the turret anchor's pixel shift, map x and y only; neither: the fire point of claim 1)
#[test]
fn claim_8_voxel_turret_anchor_shift() {
    let flh = [120, 40, 60];
    for voxel in [true, false] {
        let mut r = pad_rules(1, 1, flh, 0, true);
        let b = r.sim.buildings.get_mut("PAD").unwrap();
        b.turret_anim_is_voxel = Some(voxel);
        b.turret_anim_xy = Some([0, 30]);
        let (mut w, t, _) = scene(&r, Kind::Building, "PAD", true);
        pose(&mut w, t, 0, 16384);
        let bp = base_point(&w, t);
        let (ax, ay) = if voxel { (256, 256) } else { (0, 0) };
        assert_eq!(
            fire_point(&w, t, 0, 0),
            Pos::new(bp.x + 120 + ax, bp.y - 40 + ay, bp.z + 60),
            "voxel {voxel}"
        );
    }
}

// spec: S-0313 claim 8 and scenario P6 part b (a voxel barrel: its shots do not start where the voxel-turret row puts them)
#[test]
fn claim_8_voxel_barrel_is_not_the_voxel_turret_row() {
    let flh = [120, 40, 60];
    let mut r = pad_rules(1, 1, flh, 0, true);
    let b = r.sim.buildings.get_mut("PAD").unwrap();
    b.turret_anim_is_voxel = Some(true);
    b.barrel_anim_is_voxel = Some(true);
    b.turret_anim_xy = Some([0, 30]);
    let (mut w, t, _) = scene(&r, Kind::Building, "PAD", true);
    pose(&mut w, t, 0, 16384);
    let bp = base_point(&w, t);
    assert_ne!(
        fire_point(&w, t, 0, 0),
        Pos::new(bp.x + 120 + 256, bp.y - 40 + 256, bp.z + 60)
    );
}

// ---------------------------------------------------------------------------
// Claim 9 and Formula F4: spray attack.

// spec: S-0313 F4 (the eight spray shifts in leptons)
#[test]
fn f4_spray_shift_table() {
    assert_eq!(
        SPRAY,
        [
            (256, 0),
            (180, 180),
            (0, 256),
            (-180, 180),
            (-256, 0),
            (-180, -180),
            (0, -256),
            (180, -180),
        ]
    );
}

/// The cell targets of `n` shots of a spray-attack CART with a burst of
/// `burst`, and the first draw R(0, 7).
fn spray_cells(burst: i32, n: usize) -> (Vec<Target>, i32, Pos) {
    let mut r = base([0, 0, 0], 0);
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(burst);
    r.sim.vehicles.get_mut("CART").unwrap().spray_attack = Some(true);
    let (mut w, a, _) = scene(&r, Kind::Vehicle, "CART", true);
    let half = w.rules().quirks.ranged_draw_half_span;
    let s1 = w.rng.clone().ranged(0, 7, half);
    let p = w.obj(a).unwrap().pos;
    let t = w.obj(a).unwrap().target.unwrap();
    let mut cells = Vec::new();
    for _ in 0..n {
        let before: Vec<ObjectId> = w.projectiles.keys().copied().collect();
        assert!(fire::shot(&mut w, a, 0, t));
        let new = w
            .projectiles
            .values()
            .find(|x| !before.contains(&x.id))
            .expect("a projectile");
        cells.push(new.target.unwrap());
        w.tick = w.obj(a).unwrap().combat.rearm_until;
    }
    (cells, s1, p)
}

/// The cell holding the firer's position shifted by spray shift `s`.
fn spray_cell(p: Pos, s: i32) -> Target {
    let (dx, dy) = SPRAY[s.rem_euclid(8) as usize];
    Target::Cell(Pos::new(p.x + dx, p.y + dy, p.z).cell())
}

// spec: S-0313 claim 9 and scenario P7 (burst 4: the first shot at s₁ = R(0, 7), the next three at s₁ + 2, + 4, + 6 mod 8; the cell replaces the target)
#[test]
fn claim_9_spray_steps() {
    let (cells, s1, p) = spray_cells(4, 5);
    let want: Vec<Target> = [0, 2, 4, 6].iter().map(|k| spray_cell(p, s1 + k)).collect();
    assert_eq!(cells[..4], want[..]);
    // The fifth shot opens a new burst: a fresh draw (its cell is one of the eight).
    assert!((0..8).any(|s| cells[4] == spray_cell(p, s)));
}

// spec: S-0313 Edge cases (a burst count above 8 gives a step of 0: every shot of the burst at the same shift)
#[test]
fn edge_spray_long_bursts_do_not_step() {
    let (cells, s1, p) = spray_cells(9, 4);
    assert!(cells.iter().all(|c| *c == spray_cell(p, s1)));
}

// ---------------------------------------------------------------------------
// Claim 1a (revision 37): absent muzzle positions.

/// Elite (experience 2.0, S-0311 claim 1).
fn make_elite(w: &mut World, a: ObjectId) {
    w.obj_mut(a).unwrap().combat.experience = glasnost_num::F32(0x4000_0000);
    assert_eq!(
        glasnost_sim::combat::veterancy::rank(w, a),
        glasnost_sim::combat::veterancy::Rank::Elite
    );
}

/// The fire point and the weapon of the shot `fire::shot` makes with
/// `slot` at the firer's target.
fn shot_from(w: &mut World, a: ObjectId, slot: usize) -> (Pos, u32) {
    let t = w.obj(a).unwrap().target.unwrap();
    let before: Vec<ObjectId> = w.projectiles.keys().copied().collect();
    assert!(fire::shot(w, a, slot, t));
    let from = w
        .effects
        .iter()
        .rev()
        .find_map(|e| match e {
            Effect::Shot { firer, from, .. } if *firer == a => Some(*from),
            _ => None,
        })
        .expect("a shot");
    let weapon = w
        .projectiles
        .values()
        .find(|x| !before.contains(&x.id))
        .expect("a projectile")
        .weapon;
    w.tick = w.obj(a).unwrap().combat.rearm_until;
    (from, weapon)
}

/// Rules with ZAP, ZAP2 and ZAP3 (distinct weapons) and a turretless CART
/// without turret displacement.
fn elite_rules() -> Ruleset {
    let mut r = base([0, 0, 0], 0);
    for name in ["ZAP2", "ZAP3", "ZAP4"] {
        r.sim.weapons.insert(name.into(), zap());
    }
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.turret = Some(false);
    cart.combat.turret_offset = None;
    r
}

// spec: S-0313 claim 1a and scenario P1 part d (an elite primary or secondary slot whose art gives no elite muzzle position fires from F1 of the same slot's non-elite muzzle position, not the other slot's; the elite weapons are fired, S-0308 claim 2)
#[test]
fn claim_1a_absent_elite_muzzle_takes_the_same_slots() {
    let (pf, sf) = ([100, 40, 50], [60, -30, 45]);
    let mut r = elite_rules();
    {
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        cart.secondary = Some("ZAP".into());
        cart.elite_primary = Some("ZAP2".into());
        cart.elite_secondary = Some("ZAP3".into());
        cart.combat.primary_fire_flh = Some(pf);
        cart.combat.secondary_fire_flh = Some(sf);
        cart.combat.elite_primary_fire_flh = None;
        cart.combat.elite_secondary_fire_flh = None;
    }
    for body in [0u16, 16384] {
        let (mut w, a, _) = scene(&r, Kind::Vehicle, "CART", true);
        make_elite(&mut w, a);
        pose(&mut w, a, body, body);
        let m = mode(&w);
        let p = w.obj(a).unwrap().pos;
        let tb = angle(body, m);
        let zap2 = w.rules().combat.weapon_index["ZAP2"];
        let zap3 = w.rules().combat.weapon_index["ZAP3"];
        let (from0, w0) = shot_from(&mut w, a, 0);
        assert_eq!(w0, zap2, "the elite primary is fired");
        near(from0, p, shift(m, tb, tb, 0, pf, 1), "elite primary");
        let (from1, w1) = shot_from(&mut w, a, 1);
        assert_eq!(w1, zap3, "the elite secondary is fired");
        // The first shot of a new burst: +L.
        near(from1, p, shift(m, tb, tb, 0, sf, 1), "elite secondary");
        assert_ne!(from0, from1);
    }
    // A given elite muzzle position is used as it is.
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .elite_primary_fire_flh = Some([20, 10, 90]);
    let (mut w, a, _) = scene(&r, Kind::Vehicle, "CART", true);
    make_elite(&mut w, a);
    pose(&mut w, a, 16384, 16384);
    let p = w.obj(a).unwrap().pos;
    assert_eq!(
        shot_from(&mut w, a, 0).0,
        Pos::new(p.x + 20, p.y - 10, p.z + 90)
    );
}

// spec: S-0313 claim 1a and scenario P1 part d (numbered weapons: the elite version of weapon n with no elite muzzle position takes weapon n's, not weapon 0's; an elite muzzle position given for weapon 0 only leaves weapon 1 on its own)
#[test]
fn claim_1a_absent_elite_numbered_muzzle_takes_weapon_ns() {
    let (f0, f1, f2, e0) = ([100, 40, 50], [70, -20, 60], [30, 15, 75], [10, 5, 80]);
    for elite_given in [None, Some(vec![e0])] {
        let mut r = elite_rules();
        {
            let cart = r.sim.vehicles.get_mut("CART").unwrap();
            cart.turret_count = Some(3);
            cart.weapon_count = Some(3);
            cart.combat.weapon_slots = Some(vec!["ZAP".into(), "ZAP".into(), "ZAP".into()]);
            cart.combat.elite_weapon_slots =
                Some(vec!["ZAP2".into(), "ZAP3".into(), "ZAP4".into()]);
            cart.combat.weapon_flh = Some(vec![f0, f1, f2]);
            cart.combat.elite_weapon_flh = elite_given.clone();
        }
        let (mut w, a, _) = scene(&r, Kind::Vehicle, "CART", true);
        make_elite(&mut w, a);
        pose(&mut w, a, 16384, 16384);
        let p = w.obj(a).unwrap().pos;
        let zap3 = w.rules().combat.weapon_index["ZAP3"];
        let (from1, w1) = shot_from(&mut w, a, 1);
        assert_eq!(w1, zap3, "the elite second weapon is fired");
        assert_eq!(
            from1,
            Pos::new(p.x + f1[0], p.y - f1[1], p.z + f1[2]),
            "weapon 1's muzzle position ({elite_given:?})"
        );
        let (from2, _) = shot_from(&mut w, a, 2);
        assert_eq!(
            from2,
            Pos::new(p.x + f2[0], p.y - f2[1], p.z + f2[2]),
            "weapon 2's muzzle position ({elite_given:?})"
        );
        let want0 = if elite_given.is_some() { e0 } else { f0 };
        let (from0, _) = shot_from(&mut w, a, 0);
        assert_eq!(
            from0,
            Pos::new(p.x + want0[0], p.y - want0[1], p.z + want0[2]),
            "weapon 0 ({elite_given:?})"
        );
    }
}

// spec: S-0313 claim 1a and scenario P1 part e (an absent numbered muzzle position is 0, 0, 0: the second weapon fires from the vehicle's own x and y, not from weapon 0's or an alternate muzzle position) and S-0306 claim 8 (its start 1 lepton above the surface)
#[test]
fn claim_1a_absent_numbered_muzzle_is_the_zero_triple() {
    let mut r = elite_rules();
    {
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        cart.turret_count = Some(2);
        cart.weapon_count = Some(2);
        cart.combat.weapon_slots = Some(vec!["ZAP".into(), "ZAP2".into()]);
        cart.combat.weapon_flh = Some(vec![[100, 40, 50]]);
        cart.combat.alternate_flh = Some(vec![
            [10, 20, 30],
            [11, 21, 31],
            [12, 22, 32],
            [13, 23, 33],
            [14, 24, 34],
        ]);
    }
    for body in [0u16, 16384] {
        let (mut w, a, _) = scene(&r, Kind::Vehicle, "CART", true);
        pose(&mut w, a, body, body);
        let p = w.obj(a).unwrap().pos;
        let before: Vec<ObjectId> = w.projectiles.keys().copied().collect();
        let (from, _) = shot_from(&mut w, a, 1);
        assert_eq!(from, p, "body {body}");
        let pr = w
            .projectiles
            .values()
            .find(|x| !before.contains(&x.id))
            .unwrap();
        // An invisible projectile is moved to K at once (S-0306 claim 9);
        // its placement is shown by the stored fire point.
        assert_eq!(pr.fire_point, p);
    }
}

// spec: S-0313 claim 1b (revision 42, Q-2146: a turret count above 0 reads no primary or secondary muzzle field and no elite form of them, so with no numbered field numbered weapons 0 and 1 and their elite versions fire from 0, 0, 0)
#[test]
fn claim_1b_numbered_type_ignores_the_primary_fields() {
    let mut r = elite_rules();
    {
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        cart.turret_count = Some(2);
        cart.weapon_count = Some(2);
        cart.combat.weapon_slots = Some(vec!["ZAP".into(), "ZAP2".into()]);
        cart.combat.elite_weapon_slots = Some(vec!["ZAP3".into(), "ZAP4".into()]);
        cart.combat.weapon_flh = None;
        cart.combat.primary_fire_flh = Some([100, 40, 50]);
        cart.combat.secondary_fire_flh = Some([60, -30, 45]);
        cart.combat.elite_primary_fire_flh = Some([20, 10, 90]);
    }
    for elite in [false, true] {
        let (mut w, a, _) = scene(&r, Kind::Vehicle, "CART", true);
        if elite {
            make_elite(&mut w, a);
        }
        pose(&mut w, a, 16384, 16384);
        let p = w.obj(a).unwrap().pos;
        for slot in [0, 1] {
            assert_eq!(
                shot_from(&mut w, a, slot).0,
                p,
                "slot {slot}, elite {elite}"
            );
        }
    }
    // With a turret count of 0 the same fields are read.
    {
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        cart.turret_count = Some(0);
        cart.combat.weapon_slots = None;
        cart.combat.elite_weapon_slots = None;
    }
    let (mut w, a, _) = scene(&r, Kind::Vehicle, "CART", true);
    pose(&mut w, a, 16384, 16384);
    let p = w.obj(a).unwrap().pos;
    assert_eq!(
        shot_from(&mut w, a, 0).0,
        Pos::new(p.x + 100, p.y - 40, p.z + 50)
    );
}
