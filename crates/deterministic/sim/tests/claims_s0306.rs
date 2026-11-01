//! Claim tests for S-0306 (projectiles), on invented rules and a flat
//! synthetic map. Launches are driven through `combat::projectile::launch`
//! with an explicit fire point, flights through `combat::projectile::turn`.
//! The reference formulas below (F1, F4, F5, F7, F13) are written from the
//! spec's text.

use glasnost_map::CellXY;
use glasnost_num::decimal::nearest_double;
use glasnost_num::geom::{
    delta, direction, direction_constants, root_floor, table_angle_real, table_root,
    table_root_real,
};
use glasnost_num::trig::{acos_t, cos_t, sin_t};
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, OverlayType, ProjectileType, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::projectile::{angle_facing, d2, d3, launch, theta, turn};
use glasnost_sim::combat::state::Projectile;
use glasnost_sim::combat::{ballistics, lof};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

const N: i32 = 40;
const RZ: Rounding = Rounding::TowardZero;
const G: i32 = 10;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn n(v: i64) -> F64 {
    F64::from_i64(v, RZ)
}

fn nd(s: &str) -> F64 {
    nearest_double(s)
}

// ---------------------------------------------------------------- rules

/// The combat ruleset with gravity 10, a test weapon `Shot` (straight
/// projectile `Ball`, warhead `Dud` without animations) on the cart, an
/// aircraft type `PLANE` and a wall-damaging warhead.
fn base() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.gravity = Some(G);
    r.sim.weapons.insert(
        "Shot".into(),
        WeaponType {
            damage: Some(50),
            rof: Some(100),
            range: Some(d("10")),
            speed: Some(50),
            projectile: Some("Ball".into()),
            warhead: Some("Dud".into()),
            ..Default::default()
        },
    );
    r.sim
        .projectiles
        .insert("Ball".into(), ProjectileType::default());
    r.sim.warheads.insert("Dud".into(), WarheadType::default());
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("Shot".into());
    r.registries.insert("aircraft".into(), vec!["PLANE".into()]);
    r.sim.aircraft.insert(
        "PLANE".into(),
        UnitType {
            strength: Some(200),
            ..Default::default()
        },
    );
    r
}

fn weapon(r: &mut Ruleset) -> &mut WeaponType {
    r.sim.weapons.get_mut("Shot").unwrap()
}

fn ball(r: &mut Ruleset) -> &mut ProjectileType {
    r.sim.projectiles.get_mut("Ball").unwrap()
}

/// Registers 244 overlay types; those at the given indices are walls.
fn overlays(r: &mut Ruleset, walls: &[usize]) {
    let names: Vec<String> = (0..244).map(|i| format!("OV{i:03}")).collect();
    for (i, nm) in names.iter().enumerate() {
        r.sim.overlays.insert(
            nm.clone(),
            OverlayType {
                wall: Some(walls.contains(&i)),
                ..Default::default()
            },
        );
    }
    r.registries.insert("overlays".into(), names);
}

// ---------------------------------------------------------------- world

struct S {
    w: World,
    /// The firer, a cart of house 0 at the middle cell.
    a: ObjectId,
    /// The target, an unarmed buggy of house 1.
    t: ObjectId,
    /// The weapon `Shot`.
    wp: u32,
}

fn cell_of(dx: i32, dy: i32) -> CellXY {
    testkit::mid(N, dx, dy)
}

/// The firer at the middle cell facing `0`, the buggy at (dx, dy) cells,
/// and the firer's target set to the buggy.
fn setup(r: &Ruleset, dx: i32, dy: i32) -> S {
    let mut w = testkit::game_with(r, N, 7);
    let a = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, cell_of(0, 0), 0);
    let t = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell_of(dx, dy), 0);
    w.obj_mut(a).unwrap().target = Some(Target::Object(t));
    let wp = w.rules().combat.weapon_index["Shot"];
    S { w, a, t, wp }
}

fn wid(s: &S, name: &str) -> u32 {
    s.w.rules().combat.weapon_index[name]
}

fn pos(s: &S, id: ObjectId) -> Pos {
    s.w.obj(id).unwrap().pos
}

fn put(w: &mut World, id: ObjectId, p: Pos) {
    w.obj_mut(id).unwrap().pos = p;
}

fn set_level(w: &mut World, c: CellXY, level: i8) {
    w.cells.grid.get_mut(c).unwrap().level = level;
}

fn ground(w: &mut World, id: ObjectId) {
    let p = w.obj(id).unwrap().pos;
    let z = w.surface_at(p.x, p.y);
    put(w, id, Pos::new(p.x, p.y, z));
}

fn set_deck(w: &mut World, c: CellXY) {
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
}

/// Places a high-bridge overlay (index 24) in `c`, which gives the cell the
/// elevation-deck mark (S-0307 Terms), and marks it as a high-bridge cell.
fn set_bridge_overlay(w: &mut World, c: CellXY) {
    set_deck(w, c);
    w.cells.grid.get_mut(c).unwrap().overlay = 24; // spec: S-0211 claim 15 (high-bridge overlay index 24)
}

/// Gives the catalogue a water role: tiles `first` to `first + 13` are the
/// theater's water tiles (tile 0 is every cell's tile).
fn water_role(w: &mut World, first: i32) {
    let mut cat = (*testkit::catalogue()).clone();
    cat.roles.water = first;
    w.catalogue = Some(std::sync::Arc::new(cat));
}

fn set_overlay(w: &mut World, c: CellXY, o: u8) {
    w.cells.grid.get_mut(c).unwrap().overlay = o;
}

/// The firer's position raised by `dz`, used as the fire point.
fn fp(s: &S, dz: i32) -> Pos {
    let p = pos(s, s.a);
    Pos::new(p.x, p.y, p.z + dz)
}

fn fire_with(s: &mut S, wp: u32, shot: Target, f: Pos) -> Option<ObjectId> {
    let a = s.a;
    launch(&mut s.w, a, wp, shot, f, 50)
}

fn fire(s: &mut S, shot: Target, f: Pos) -> Option<ObjectId> {
    let wp = s.wp;
    fire_with(s, wp, shot, f)
}

fn pr(s: &S, id: ObjectId) -> &Projectile {
    s.w.projectiles.get(&id).expect("projectile in play")
}

fn half(w: &World) -> bool {
    w.rules().quirks.ranged_draw_half_span
}

fn set_turret(w: &mut World, id: ObjectId, f: u16) {
    let o = w.obj_mut(id).unwrap();
    o.turret = Turning::at(f);
    o.body = Turning::at(f);
}

// ---------------------------------------------------------------- reference formulas

/// F1: the initial velocity from a launch facing, a pitch facing and a
/// start speed (zero speed launch on).
fn f1(yaw: i32, pitch: i32, s: i32) -> [F64; 3] {
    f1q(yaw, pitch, s, true)
}

fn f1q(yaw: i32, pitch: i32, s: i32, zero_speed_launch: bool) -> [F64; 3] {
    let s0 = if s > 0 || !zero_speed_launch { s } else { 100 };
    let l = table_root((i64::from(s0) * i64::from(s0)) as u64, RZ).to_f64();
    let ty = theta(yaw, RZ);
    let cx = cos_t(ty, RZ).mul(l, RZ);
    let cy = sin_t(ty, RZ).mul(l, RZ).negated();
    let m = table_root_real(cx.mul(cx, RZ).add(cy.mul(cy, RZ), RZ), true, RZ);
    let tp = theta(pitch, RZ);
    [
        cos_t(tp, RZ).mul(cx, RZ),
        cos_t(tp, RZ).mul(cy, RZ),
        sin_t(tp, RZ).mul(m, RZ),
    ]
}

/// F4's solve for one distance: θ, or none.
fn f4_solve(v: i32, dist: i32, z: i32, g: F64, steep: bool) -> Option<F64> {
    let (v, dd, z) = (n(v.into()), n(dist.into()), n(z.into()));
    let v2 = v.mul(v, RZ);
    let disc = v2
        .mul(v2, RZ)
        .sub(n(2).mul(g, RZ).mul(z, RZ).mul(v2, RZ), RZ)
        .sub(g.mul(g, RZ).mul(dd, RZ).mul(dd, RZ), RZ);
    let omega = n(2).mul(n(1).add(z.mul(z, RZ).div(dd.mul(dd, RZ), RZ), RZ), RZ);
    if !disc.ge(F64::ZERO) || omega.is_nan() || omega.eq_num(F64::ZERO) {
        return None;
    }
    let rd = table_root_real(disc, true, RZ);
    let base = v2.sub(g.mul(z, RZ), RZ);
    let num = if steep {
        base.sub(rd, RZ)
    } else {
        base.add(rd, RZ)
    };
    let u = num.div(omega, RZ);
    if !u.ge(F64::ZERO) {
        return None;
    }
    Some(acos_t(table_root_real(u, true, RZ).div(v, RZ), RZ))
}

/// F4: the pitch facing of an arcing launch (uphill rule on), or none.
fn f4(v: i32, dist: i32, z: i32, g: i32, lob: bool) -> Option<i32> {
    let gg = n(g.into());
    let (c1, _) = direction_constants();
    if z > 0 {
        return Some(angle_facing(c1.div(n(2), RZ), RZ));
    }
    f4_off(v, dist, z, gg, lob)
}

/// F4 solved for every z (the uphill rule off).
fn f4_off(v: i32, dist: i32, z: i32, g: F64, lob: bool) -> Option<i32> {
    let th = f4_solve(v, dist, z, g, lob)?;
    let angle = if lob {
        th
    } else {
        match f4_solve(v, dist + 1, z, g, false) {
            Some(tp) if tp.lt(th) => th.negated(),
            _ => th,
        }
    };
    Some(angle_facing(angle, RZ))
}

/// F5: a displacement (or point) moved by radius σ in the direction of
/// draw ξ.
fn f5(p: (i32, i32), sigma: i32, xi: i32) -> (i32, i32) {
    let k = nd("4.656612877414201e-10");
    let (c1, c2) = direction_constants();
    let two_pi = c1.mul(n(4), RZ);
    let fx = n(xi.into())
        .mul(k, RZ)
        .mul(two_pi, RZ)
        .sub(c1, RZ)
        .mul(c2, RZ)
        .trunc_i128()
        .unwrap() as i64 as i16;
    let th = theta(i32::from(fx), RZ);
    let s = n(sigma.into());
    (
        n(p.0.into())
            .add(cos_t(th, RZ).mul(s, RZ), RZ)
            .to_i32_common(),
        n(p.1.into())
            .sub(sin_t(th, RZ).mul(s, RZ), RZ)
            .to_i32_common(),
    )
}

/// F13: the straight launch pitch for Z and τ over the planar distance d′.
fn f13(z: i32, tau: i32, dprime: i32) -> i32 {
    let x = n(dprime.into()).max(nd("0.05"));
    let y = n((z.abs() - tau).into());
    let (a, _) = table_angle_real(x, y, RZ);
    let tc = if z < 0 { a.negated() } else { a };
    angle_facing(tc, RZ)
}

/// |V| = r(Vₓ² + V_y² + V_z²).
fn speed(v: [F64; 3]) -> F64 {
    table_root_real(
        v[0].mul(v[0], RZ)
            .add(v[1].mul(v[1], RZ), RZ)
            .add(v[2].mul(v[2], RZ), RZ),
        true,
        RZ,
    )
}

fn yaw_to(from: Pos, to: Pos) -> i32 {
    i32::from(direction(
        i64::from(to.x - from.x),
        i64::from(to.y - from.y),
        true,
    ))
}

fn arc(dist: i32) -> i32 {
    ballistics::arc_speed(dist, G, false, RZ)
}

fn close(a: F64, b: F64) -> bool {
    a.sub(b, RZ).abs().lt(nd("0.0001"))
}

// ================================================================= launch

// spec: S-0306 F2 (v(L, g) = trunc(r(RZ₅₃(RZ₅₃(L × g) × 1.2))); scenario P1: v(2048, 10) = 156)
#[test]
fn f2_arc_speed_example() {
    assert_eq!(arc(2048), 156);
    // Scenario P8: v(1536, 10) = 135.
    assert_eq!(arc(1536), 135);
    // Floater: g/2.
    assert_eq!(
        ballistics::arc_speed(2048, G, true, RZ),
        ballistics::arc_speed_g(2048, nd("5"), RZ)
    );
}

/// A ground shot at a cell `cells` along the map x axis from a fire point
/// at the firer's position: the first velocity.
fn long_shot(r: &Ruleset, cells: i32) -> [F64; 3] {
    let mut s = setup(r, 3, 0);
    let f = fp(&s, 0);
    let c = CellXY::new(f.cell().x + cells, f.cell().y);
    s.w.obj_mut(s.a).unwrap().target = Some(Target::Cell(c));
    let id = fire(&mut s, Target::Cell(c), f).unwrap();
    pr(&s, id).vel
}

// spec: S-0306 claim 1 (quirks.shot_distance_wrap on: Δ = (51200, 0) gives n = −1673527296 and d = 40908)
#[test]
fn claim1_wrapped_shot_distance_example() {
    assert_eq!((51200i64 * 51200) as i32, -1673527296);
    assert_eq!(root_floor(1673527296, true, RZ), 40908);
    let v = long_shot(&base(), 200);
    let east = i32::from(direction(51200, 0, true));
    // Level (Δz = 0), cap 25600 does not act.
    assert_eq!(v, f1(east, 16383, arc(40908)));
}

// spec: S-0306 Edge cases (quirks.shot_distance_wrap off: the exact sum, d = 51200)
#[test]
fn quirk_shot_distance_wrap_off() {
    let mut r = base();
    r.globals.quirks.shot_distance_wrap = false;
    let v = long_shot(&r, 200);
    let east = i32::from(direction(51200, 0, true));
    assert_eq!(v, f1(east, 16383, arc(51200)));
}

// spec: S-0306 claim 1 (below 2³¹ the sum is exact: the 150-cell control of scenario P39; the imported speed field plays no part)
#[test]
fn claim1_shot_speed_of_rho_zero_ignores_the_speed_field() {
    let east = i32::from(direction(38400, 0, true));
    let v1 = long_shot(&base(), 150);
    assert_eq!(v1, f1(east, 16383, arc(38400)));
    let mut r = base();
    weapon(&mut r).speed = Some(3);
    assert_eq!(long_shot(&r, 150), v1);
}

// spec: S-0306 claim 1 (the gravity is g_f = g / 2 for a floater projectile; scenario P1 run d)
#[test]
fn claim1_floater_shot_speed() {
    let mut r = base();
    ball(&mut r).floater = Some(true);
    let v = long_shot(&r, 8);
    let east = i32::from(direction(2048, 0, true));
    assert_eq!(v, f1(east, 16383, ballistics::arc_speed(2048, G, true, RZ)));
}

// spec: S-0306 claim 1 (ρ ≠ 0: the shot speed is the weapon's speed field)
#[test]
fn claim1_negative_turn_rate_uses_the_speed_field() {
    let mut r = base();
    ball(&mut r).rot = Some(-1);
    weapon(&mut r).speed = Some(20); // stored 51
    let mut s = setup(&r, 8, 0);
    set_turret(&mut s.w, s.a, 16384);
    let f = fp(&s, 0);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    // The launch direction of ρ ≠ 0 is the aiming facing (claim 5).
    assert_eq!(pr(&s, id).vel, f1(16384, 16383, 51));
}

// spec: S-0306 claim 2 (the shot speed is capped at trunc(d₃(F, Q) / 2); scenario P1 run c)
#[test]
fn claim2_cap_at_half_the_distance() {
    let mut r = base();
    ball(&mut r).rot = Some(-1);
    weapon(&mut r).speed = Some(100); // stored 255
    let mut s = setup(&r, 1, 0);
    set_turret(&mut s.w, s.a, 16384);
    let f = fp(&s, 30);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let cap = d3(&s.w, f, pos(&s, s.t)) / 2;
    assert!(cap < 255);
    assert_eq!(pr(&s, id).vel, f1(16384, 16383, cap));
}

// spec: S-0306 claim 2 (a vertical projectile without radial fire segments starts at 1)
#[test]
fn claim2_vertical_starts_at_one() {
    let mut r = base();
    ball(&mut r).vertical = Some(true);
    let mut s = setup(&r, 5, 0);
    let f = fp(&s, 0);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let yaw = yaw_to(f, pos(&s, s.t));
    assert_eq!(pr(&s, id).vel, f1(yaw, 16383, 1));
}

// spec: S-0306 claim 2 (with radial fire segments a vertical projectile keeps the capped speed; scenario P20 variant b)
#[test]
fn claim2_vertical_with_radial_segments_keeps_the_capped_speed() {
    let mut r = base();
    ball(&mut r).vertical = Some(true);
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .radial_fire_segments = Some(1);
    let mut s = setup(&r, 5, 0);
    let f = fp(&s, 0);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    // Body facing 0, one segment: shot k = 0 leaves a quarter turn clockwise (claim 7).
    let s1 = arc(d2(&s.w, f, pos(&s, s.t)));
    let start = s1.min(d3(&s.w, f, pos(&s, s.t)) / 2);
    assert!(start > 1);
    assert_eq!(pr(&s, id).vel, f1(radial_yaw(0, 1, 0), 16383, start));
}

// spec: S-0306 claim 2 and claim 9 (a steering projectile's speed after placement is 1, with or without radial segments; scenario P20 variant c)
#[test]
fn claim2_steering_speed_is_one_after_placement() {
    for radial in [0, 2] {
        let mut r = base();
        ball(&mut r).rot = Some(5);
        weapon(&mut r).speed = Some(25);
        r.sim
            .vehicles
            .get_mut("CART")
            .unwrap()
            .combat
            .radial_fire_segments = Some(radial);
        let mut s = setup(&r, 5, 0);
        let f = fp(&s, 0);
        let t = Target::Object(s.t);
        let id = fire(&mut s, t, f).unwrap();
        let sp = speed(pr(&s, id).vel);
        assert!(
            sp.ge(nd("0.999")) && sp.le(n(1)),
            "radial {radial}: speed after placement"
        );
    }
}

// spec: S-0306 claim 2 and Terms (weapon speed S_w: v(R₀, g) for ρ = 0, the imported field for ρ ≠ 0)
#[test]
fn claim2_top_speeds() {
    let top = |rot: i32, vertical: bool| {
        let mut r = base();
        ball(&mut r).rot = Some(rot);
        ball(&mut r).vertical = Some(vertical);
        weapon(&mut r).speed = Some(20);
        let mut s = setup(&r, 5, 0);
        let f = fp(&s, 0);
        let t = Target::Object(s.t);
        let id = fire(&mut s, t, f).unwrap();
        pr(&s, id).top_speed
    };
    // Range 10 cells = 2560 leptons: v(2560, 10).
    assert_eq!(top(0, true), arc(2560));
    assert_eq!(top(-1, true), 51);
    assert_eq!(top(5, false), 51);
}

/// The yaw of radial segment k of n for body facing b (F1).
fn radial_yaw(b: i32, nseg: i32, k: i32) -> i32 {
    let (c1, c2) = direction_constants();
    let pi = c1.mul(n(2), RZ);
    let phi = pi.div(n(nseg.into()), RZ).mul(n(k.into()), RZ).sub(c1, RZ);
    let v = theta(b, RZ)
        .add(phi, RZ)
        .sub(c1, RZ)
        .mul(c2, RZ)
        .trunc_i128()
        .unwrap();
    i32::from(v as i64 as u16)
}

// spec: S-0306 claim 3 (the aim point is the reference point of the firer's target, without the aim shift; the shot speed uses the shot target's target point, with it)
#[test]
fn claim3_aim_point_without_the_aim_shift() {
    let mut r = base();
    r.sim
        .buildings
        .get_mut("TOWER")
        .unwrap()
        .target_coord_offset = Some([0, 700, 0]);
    let mut s = setup(&r, 3, 0);
    let tw = testkit::spawn(&mut s.w, Kind::Building, "TOWER", 1, cell_of(6, 0), 0);
    s.w.obj_mut(s.a).unwrap().target = Some(Target::Object(tw));
    let f = fp(&s, 0);
    let id = fire(&mut s, Target::Object(tw), f).unwrap();
    let rp = s.w.reference_point(tw);
    let tp = Pos::new(rp.x, rp.y + 700, rp.z);
    let s1 = arc(d2(&s.w, f, tp));
    let start = s1.min(d3(&s.w, f, rp) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, rp), 16383, start));
    // The stored target point is the reference point (claim 9).
    assert_eq!(pr(&s, id).stored, rp);
}

// spec: S-0306 claim 3 and Edge cases (quirks.aim_origin_without_target on: no current target aims at (0, 0, 0))
#[test]
fn quirk_aim_origin_without_target_on() {
    let mut s = setup(&base(), 5, 0);
    s.w.obj_mut(s.a).unwrap().target = None;
    let f = fp(&s, 0);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let o = Pos::new(0, 0, 0);
    let s1 = arc(d2(&s.w, f, pos(&s, s.t)));
    let start = s1.min(d3(&s.w, f, o) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, o), 16383, start));
}

// spec: S-0306 Edge cases (quirks.aim_origin_without_target off: the aim point is the shot target's reference point)
#[test]
fn quirk_aim_origin_without_target_off() {
    let mut r = base();
    r.globals.quirks.aim_origin_without_target = false;
    let mut s = setup(&r, 5, 0);
    s.w.obj_mut(s.a).unwrap().target = None;
    let f = fp(&s, 0);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let tp = pos(&s, s.t);
    let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, tp), 16383, start));
}

/// An area-fire shot: shot target the firer's own cell, the firer's target
/// the buggy 5 cells away.
fn area_fire(r: &mut Ruleset) -> (S, Pos, ObjectId) {
    weapon(r).area_fire = Some(true);
    let mut s = setup(r, 5, 0);
    let p = pos(&s, s.a);
    let f = Pos::new(p.x + 90, p.y, 0);
    let c = Target::Cell(p.cell());
    let id = fire(&mut s, c, f).unwrap();
    (s, f, id)
}

// spec: S-0306 claim 3 (a) and Edge cases (quirks.redirected_shot_aim on: the launch points at the firer's target, the speed and K follow the redirected cell)
#[test]
fn quirk_redirected_shot_aim_on() {
    let mut r = base();
    let (s, f, id) = area_fire(&mut r);
    let cell = Pos::centre(pos(&s, s.a).cell(), 0);
    let tp = pos(&s, s.t);
    let s1 = arc(d2(&s.w, f, cell));
    let start = s1.min(d3(&s.w, f, tp) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, tp), 16383, start));
    assert_eq!(pr(&s, id).stored, cell);
    assert_eq!(pr(&s, id).target, Some(Target::Cell(cell.cell())));
}

// spec: S-0306 Edge cases (quirks.redirected_shot_aim off: the aim point is the redirected cell)
#[test]
fn quirk_redirected_shot_aim_off() {
    let mut r = base();
    r.globals.quirks.redirected_shot_aim = false;
    let (s, f, id) = area_fire(&mut r);
    let cell = Pos::centre(pos(&s, s.a).cell(), 0);
    let s1 = arc(d2(&s.w, f, cell));
    let start = s1.min(d3(&s.w, f, cell) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, cell), 16383, start));
}

/// An arcing, inaccurate weapon fired at a cell 6 cells away from a fire
/// point 10 leptons below the cell (so that F4 gives the 45° launch for any
/// impact point), ballistic scatter 2 cells.
fn scatter_setup(r: &mut Ruleset, seed_shift: u32) -> (S, Pos, Pos) {
    ball(r).arcing = Some(true);
    ball(r).inaccurate = Some(true);
    r.sim.general.ballistic_scatter = Some(d("2"));
    let mut s = setup(r, 3, 0);
    for _ in 0..seed_shift {
        s.w.rng.raw();
    }
    let c = cell_of(6, 0);
    s.w.obj_mut(s.a).unwrap().target = Some(Target::Cell(c));
    let f = fp(&s, -10);
    (s, f, Pos::centre(c, 0))
}

// spec: S-0306 claim 4 and F5 (σ = R(trunc(B/2), B), then one direction draw; the impact point is F plus the scattered displacement)
#[test]
fn claim4_scatter_of_an_inaccurate_arcing_projectile() {
    for shift in 0..6 {
        let mut r = base();
        let (mut s, f, aim) = scatter_setup(&mut r, shift);
        let mut c = s.w.rng.clone();
        let h = half(&s.w);
        let sigma = c.ranged(256, 512, h);
        let xi = c.ranged(0, 2147483646, h);
        let id = fire(&mut s, Target::Cell(aim.cell()), f).unwrap();
        assert_eq!(s.w.rng, c, "exactly the two draws of claim 4");
        let (qx, qy) = f5((aim.x - f.x, aim.y - f.y), sigma, xi);
        let q = Pos::new(f.x + qx, f.y + qy, aim.z);
        // The horizontal distance from the aim point is σ (within one lepton).
        let dist = d2(&s.w, q, aim);
        assert!((dist - sigma).abs() <= 1, "σ {sigma}, distance {dist}");
        let start = arc(d2(&s.w, f, aim)).min(d3(&s.w, f, q) / 2);
        assert_eq!(pr(&s, id).vel, f1(yaw_to(f, q), 8191, start));
    }
}

// spec: S-0306 claim 4 (flak-scattering: σ = trunc(R(0, B) × D / R_W), D the distance to the aim point, R_W the firer's weapon range for that weapon)
#[test]
fn claim4_flak_scatter_radius() {
    let mut r = base();
    ball(&mut r).flak_scatter = Some(true);
    let (mut s, f, aim) = scatter_setup(&mut r, 0);
    let mut c = s.w.rng.clone();
    let h = half(&s.w);
    let rr = c.ranged(0, 512, h);
    let xi = c.ranged(0, 2147483646, h);
    let sigma = rr * d3(&s.w, f, aim) / 2560;
    let id = fire(&mut s, Target::Cell(aim.cell()), f).unwrap();
    assert_eq!(s.w.rng, c);
    let (qx, qy) = f5((aim.x - f.x, aim.y - f.y), sigma, xi);
    let q = Pos::new(f.x + qx, f.y + qy, aim.z);
    let start = arc(d2(&s.w, f, aim)).min(d3(&s.w, f, q) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, q), 8191, start));
}

// spec: S-0306 claim 4 and S-0307 claim 13 (R_W is the firer's weapon range of the weapon fired, here the secondary weapon in slot 1)
#[test]
fn claim4_flak_scatter_uses_the_range_of_the_weapon_fired() {
    let mut r = base();
    ball(&mut r).flak_scatter = Some(true);
    r.sim.weapons.insert(
        "Flak".into(),
        WeaponType {
            range: Some(d("4")),
            ..r.sim.weapons["Shot"].clone()
        },
    );
    r.sim.vehicles.get_mut("CART").unwrap().secondary = Some("Flak".into());
    let (mut s, f, aim) = scatter_setup(&mut r, 0);
    let flak = wid(&s, "Flak");
    let mut c = s.w.rng.clone();
    let h = half(&s.w);
    let rr = c.ranged(0, 512, h);
    let xi = c.ranged(0, 2147483646, h);
    let sigma = rr * d3(&s.w, f, aim) / 1024;
    let id = fire_with(&mut s, flak, Target::Cell(aim.cell()), f).unwrap();
    let (qx, qy) = f5((aim.x - f.x, aim.y - f.y), sigma, xi);
    let q = Pos::new(f.x + qx, f.y + qy, aim.z);
    let start = ballistics::arc_speed(d2(&s.w, f, aim), G, false, RZ).min(d3(&s.w, f, q) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, q), 8191, start));
}

// spec: S-0306 claim 4 (inaccurate projectiles of other kinds do not scatter at launch: no draw)
#[test]
fn claim4_no_scatter_without_the_arcing_flag() {
    let mut r = base();
    ball(&mut r).inaccurate = Some(true);
    r.sim.general.ballistic_scatter = Some(d("2"));
    let mut s = setup(&r, 5, 0);
    let before = s.w.rng.clone();
    let f = fp(&s, 0);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    assert_eq!(s.w.rng, before);
    let tp = pos(&s, s.t);
    let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, tp), 16383, start));
}

// spec: S-0306 claim 24 and claims 4 and 9 (an inaccurate arcing invisible flak projectile: the claim 4 radius R(B/2, B) and direction, then the claim 9 radius and direction; scenario P3 variant e)
#[test]
fn claim24_launch_draws_in_order() {
    let mut r = base();
    ball(&mut r).flak_scatter = Some(true);
    ball(&mut r).inviso = Some(true);
    let (mut s, f, aim) = scatter_setup(&mut r, 0);
    let mut c = s.w.rng.clone();
    let h = half(&s.w);
    let _sigma4 = c.ranged(256, 512, h);
    let _xi4 = c.ranged(0, 2147483646, h);
    let r9 = c.ranged(0, 1024, h);
    let xi9 = c.ranged(0, 2147483646, h);
    let id = fire(&mut s, Target::Cell(aim.cell()), f).unwrap();
    assert_eq!(s.w.rng, c, "four draws, in the order of claim 24");
    let sigma9 = r9 * d3(&s.w, f, aim) / 2560;
    let k = f5((aim.x, aim.y), sigma9, xi9);
    assert_eq!(pr(&s, id).stored, Pos::new(k.0, k.1, aim.z));
}

// spec: S-0306 claim 5 (ρ = 0, not dropping: the direction from the fire point towards the impact point) and claim 8 (revision 45: a fire point at the surface starts 1 lepton above it; the direction uses the unraised point)
#[test]
fn claim5_launch_direction_towards_the_impact_point() {
    let mut s = setup(&base(), 4, 3);
    set_turret(&mut s.w, s.a, 40000);
    let f = fp(&s, 0);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let tp = pos(&s, s.t);
    let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, tp), 16383, start));
    assert_eq!(pr(&s, id).pos, Pos::new(f.x, f.y, f.z + 1));
}

// spec: S-0306 claim 5 (a dropping projectile starts from the firer's own position, in the aiming facing) and claim 8 (revision 45: that position, at the surface here, raised 1 lepton)
#[test]
fn claim5_dropping_starts_at_the_firer() {
    let mut r = base();
    ball(&mut r).dropping = Some(true);
    let mut s = setup(&r, 4, 3);
    set_turret(&mut s.w, s.a, 12000);
    let f = fp(&s, 40);
    let f = Pos::new(f.x + 100, f.y, f.z);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let tp = pos(&s, s.t);
    let own = pos(&s, s.a);
    assert_eq!(pr(&s, id).pos, Pos::new(own.x, own.y, own.z + 1));
    let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
    let pitch = if (tp.z - f.z).abs() <= 200 { 16383 } else { 0 };
    assert_eq!(pr(&s, id).vel, f1(12000, pitch, start));
}

// spec: S-0306 claim 5 (a steering projectile starts at the fire point, in the aiming facing)
#[test]
fn claim5_steering_starts_at_the_fire_point_in_the_aiming_facing() {
    let mut r = base();
    ball(&mut r).rot = Some(5);
    weapon(&mut r).speed = Some(25);
    let mut s = setup(&r, 4, 3);
    set_turret(&mut s.w, s.a, 12000);
    let f = fp(&s, 40);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    assert_eq!(pr(&s, id).pos, f);
    // The velocity, rescaled to length 1, keeps the aiming facing's direction.
    let v0 = f1(12000, 16383, 1);
    let v = pr(&s, id).vel;
    assert!(close(v[0], v0[0]) && close(v[1], v0[1]));
}

// spec: S-0306 claim 6 a and F4 (z > 0: 45°, pitch facing 8191, quirks.uphill_arc_45; F4 Constants)
#[test]
fn claim6a_uphill_arc_is_45_degrees() {
    let (c1, _) = direction_constants();
    assert_eq!(angle_facing(c1.div(n(2), RZ), RZ), 8191);
    let mut r = base();
    ball(&mut r).arcing = Some(true);
    let mut s = setup(&r, 5, 0);
    set_level(&mut s.w, cell_of(5, 0), 2);
    ground(&mut s.w, s.t);
    let f = fp(&s, 30);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let tp = pos(&s, s.t);
    let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, tp), 8191, start));
}

// spec: S-0306 Edge cases (quirks.uphill_arc_45 off: F4 is solved for Δz > 0 as for Δz ≤ 0)
#[test]
fn quirk_uphill_arc_45_off() {
    let mut r = base();
    r.globals.quirks.uphill_arc_45 = false;
    ball(&mut r).arcing = Some(true);
    let mut s = setup(&r, 5, 0);
    set_level(&mut s.w, cell_of(5, 0), 2);
    ground(&mut s.w, s.t);
    let f = fp(&s, 30);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let tp = pos(&s, s.t);
    let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
    let pitch = f4_off(start, d2(&s.w, f, tp), tp.z - f.z, n(G.into()), false).unwrap();
    assert_ne!(pitch, 8191);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, tp), pitch, start));
}

/// An arcing launch at a target `dx` cells away on level `level`, from a
/// fire point `dz` above the firer; returns (velocity, expected F4 pitch).
fn arc_launch(
    r: &mut Ruleset,
    dx: i32,
    level: i8,
    dz: i32,
    lob: bool,
) -> (Option<[F64; 3]>, Option<i32>, i32, i32) {
    ball(r).arcing = Some(true);
    weapon(r).lobber = Some(lob);
    let mut s = setup(r, dx, 0);
    set_level(&mut s.w, cell_of(dx, 0), level);
    ground(&mut s.w, s.t);
    let f = fp(&s, dz);
    let t = Target::Object(s.t);
    let tp = pos(&s, s.t);
    let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
    let height_lob = tp.z - pos(&s, s.a).z > d2(&s.w, pos(&s, s.a), tp);
    let pitch = f4(start, d2(&s.w, f, tp), tp.z - f.z, G, lob || height_lob);
    let id = fire(&mut s, t, f);
    let v = id.map(|id| pr(&s, id).vel);
    (v, pitch, yaw_to(f, tp), start)
}

// spec: S-0306 claim 6 a and F4 (level and lower ground: the flatter solution; the lobber flag: the steeper one; scenario P5)
#[test]
fn claim6a_flatter_and_lob_solutions() {
    for (level, lob) in [(0, false), (-2, false), (0, true), (-2, true)] {
        let mut r = base();
        let (v, pitch, yaw, start) = arc_launch(&mut r, 5, level, 30, lob);
        let pitch = pitch.expect("F4 has a solution");
        assert_eq!(
            v.unwrap(),
            f1(yaw, pitch, start),
            "level {level}, lob {lob}"
        );
        if lob {
            assert!(pitch < 8191, "a lob is steeper than 45°");
        } else {
            assert!(
                pitch > 8191 && pitch < 16383,
                "the flatter shot is below 45°"
            );
        }
    }
}

// spec: S-0306 claim 6 a (the void shot: F4 has no solution, no projectile; scenario P5 run d)
#[test]
fn claim6a_void_shot() {
    let mut r = base();
    ball(&mut r).arcing = Some(true);
    ball(&mut r).rot = Some(-1);
    weapon(&mut r).speed = Some(10);
    let mut s = setup(&r, 5, 0);
    let f = fp(&s, 30);
    let before = s.w.rng.clone();
    let t = Target::Object(s.t);
    assert!(fire(&mut s, t, f).is_none());
    assert!(s.w.projectiles.is_empty());
    assert_eq!(s.w.rng, before, "no draw");
    // The inaccurate variant takes the two scatter draws of claim 4 first.
    let mut r2 = r.clone();
    ball(&mut r2).inaccurate = Some(true);
    let mut s = setup(&r2, 5, 0);
    let mut c = s.w.rng.clone();
    let h = half(&s.w);
    let b = s.w.rules().combat.general.ballistic_scatter;
    c.ranged(b / 2, b, h);
    c.ranged(0, 2147483646, h);
    let t = Target::Object(s.t);
    assert!(fire(&mut s, t, f).is_none());
    assert_eq!(s.w.rng, c);
}

// spec: S-0306 claim 6 a (a lob: the target higher above the firer than it is far horizontally; scenario P5 run e)
#[test]
fn claim6a_height_based_lob() {
    // 1 cell away, 3 levels (312) higher, fire point 500 up: Δz < 0, lob.
    let mut r = base();
    let (v, pitch, yaw, start) = arc_launch(&mut r, 1, 3, 500, false);
    let p = pitch.unwrap();
    assert!(p < 8191);
    assert_eq!(v.unwrap(), f1(yaw, p, start));
    // Control: 2 cells away, the flatter solution.
    let mut r = base();
    let (v, pitch, yaw, start) = arc_launch(&mut r, 2, 3, 500, false);
    let p = pitch.unwrap();
    assert!(p > 8191);
    assert_eq!(v.unwrap(), f1(yaw, p, start));
}

/// A shot from a fire point `h` exactly above the centre of the target's
/// cell (zero planar distance, scenario P37).
fn zero_distance(r: &Ruleset, h: i32) -> [F64; 3] {
    let mut s = setup(r, 3, 0);
    let tp = pos(&s, s.t);
    let f = Pos::new(tp.x, tp.y, tp.z + h);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).expect("not a void shot");
    pr(&s, id).vel
}

fn arcing_zero(rot: i32, speed: i32, lob: bool) -> Ruleset {
    let mut r = base();
    ball(&mut r).arcing = Some(true);
    ball(&mut r).rot = Some(rot);
    weapon(&mut r).speed = Some(speed);
    weapon(&mut r).lobber = Some(lob);
    r
}

// spec: S-0306 F4 (zero planar distance, z < 0, v = 0: the lob comes out at pitch facing −16383, horizontally backwards at 100; scenario P37 run a)
#[test]
fn f4_zero_distance_zero_speed_lob_goes_backwards() {
    let (c1, _) = direction_constants();
    let th = c1.sub(nd("-1.5707963705062866"), RZ);
    assert_eq!(angle_facing(th, RZ), -16383);
    let v = zero_distance(&arcing_zero(0, 50, true), 300);
    let east = i32::from(direction(0, 0, true));
    assert_eq!(v, f1(east, -16383, 0));
    // Map west, horizontal, length 100.
    assert!(v[0].lt(nd("-99")) && v[2].abs().lt(nd("1")));
}

// spec: S-0306 F4 (zero planar distance, z < 0, v > 0: a lob goes straight up, pitch facing 0; scenario P37 run b)
#[test]
fn f4_zero_distance_lob_goes_straight_up() {
    let v = zero_distance(&arcing_zero(-1, 10, true), 300);
    let yaw = 0; // the cart's aiming facing (ρ ≠ 0, claim 5)
    assert_eq!(v, f1(yaw, 0, 25));
}

// spec: S-0306 F4 (zero planar distance, the flatter shot: index 2049 or more points straight down, 32766; v = 20, g = 10, z = −300 gives index 2065)
#[test]
fn f4_zero_distance_flatter_shot_straight_down() {
    let (c1, c2) = direction_constants();
    let pi = c1.mul(n(2), RZ);
    assert_eq!(pi.negated().mul(c2, RZ).trunc_i128(), Some(32766));
    // The index of the solve for distance 1.
    let g = n(10);
    let (v, z) = (n(20), n(-300));
    let v2 = v.mul(v, RZ);
    let disc = v2
        .mul(v2, RZ)
        .sub(n(2).mul(g, RZ).mul(z, RZ).mul(v2, RZ), RZ)
        .sub(g.mul(g, RZ), RZ);
    let omega = n(2).mul(n(1).add(z.mul(z, RZ), RZ), RZ);
    let u = v2
        .sub(g.mul(z, RZ), RZ)
        .add(table_root_real(disc, true, RZ), RZ)
        .div(omega, RZ);
    let idx = table_root_real(u, true, RZ)
        .div(v, RZ)
        .add(n(1), RZ)
        .mul(n(2048), RZ)
        .trunc_i128();
    assert_eq!(idx, Some(2065));
    // Speed field 8: stored 20. The flatter shot from 300 above.
    let v = zero_distance(&arcing_zero(-1, 8, false), 300);
    assert_eq!(v, f1(0, 32766, 20));
}

// spec: S-0306 F4 (zero planar distance, the flatter shot with an index below 2049: straight up)
#[test]
fn f4_zero_distance_flatter_shot_straight_up() {
    // Gravity 0, speed field 100 (255), 2100 leptons above: U′ = v² / (1 + z²), so r(U′)/v is
    // about 1/2100 and the index trunc(2048 × (1 + 1/2100)) is 2048.
    let mut r = arcing_zero(-1, 100, false);
    r.sim.general.gravity = Some(0);
    let v = zero_distance(&r, 2100);
    assert_eq!(v, f1(0, 0, 255));
}

// spec: S-0306 F4 (zero planar distance with Δz = 0: ω is not a number, no solution, a void shot)
#[test]
fn f4_zero_distance_level_is_void() {
    let r = arcing_zero(-1, 10, true);
    let mut s = setup(&r, 3, 0);
    let tp = pos(&s, s.t);
    let t = Target::Object(s.t);
    assert!(fire(&mut s, t, tp).is_none());
}

// spec: S-0306 Edge cases (quirks.zero_distance_arc off: a lob straight up, the flatter shot straight down)
#[test]
fn quirk_zero_distance_arc_off() {
    let mut r = arcing_zero(0, 50, true);
    r.globals.quirks.zero_distance_arc = false;
    let east = i32::from(direction(0, 0, true));
    // ρ = 0: start speed 0, pitch 0 (straight up), F1 at 100.
    assert_eq!(zero_distance(&r, 300), f1(east, 0, 0));
    let mut r = arcing_zero(-1, 100, false);
    r.globals.quirks.zero_distance_arc = false;
    assert_eq!(zero_distance(&r, 30), f1(0, 32766, 15));
}

// spec: S-0306 claim 6 b (no arcing flag, drawn as a voxel model: 32768 when the target is lower, else 16384)
#[test]
fn claim6b_voxel_projectile_pitch() {
    let mut r = base();
    ball(&mut r).voxel = Some(true);
    // Lower target: the firer on a 2-level cell.
    let mut s = setup(&r, 5, 0);
    set_level(&mut s.w, cell_of(0, 0), 2);
    ground(&mut s.w, s.a);
    let f = fp(&s, 30);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let tp = pos(&s, s.t);
    let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
    let v = pr(&s, id).vel;
    assert_eq!(v, f1(yaw_to(f, tp), 32768, start));
    // Straight down: horizontal components below 10⁻¹³, V_z about −s (scenario P4 variant e).
    assert!(v[0].abs().lt(nd("1e-13")) && v[1].abs().lt(nd("1e-13")));
    assert!(v[2].lt(F64::ZERO));
    // Higher target: 16384, the same velocity as a level launch.
    let mut s = setup(&r, 5, 0);
    set_level(&mut s.w, cell_of(5, 0), 2);
    ground(&mut s.w, s.t);
    let f = fp(&s, 30);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let tp = pos(&s, s.t);
    let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, tp), 16384, start));
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, tp), 16383, start));
}

// spec: S-0306 Terms (pitch facings 16380 to 16386 give an exactly level velocity)
#[test]
fn terms_pitch_facings_that_fly_level() {
    for p in 16380..=16386 {
        let v = f1(9000, p, 77);
        assert!(v[2].eq_num(F64::ZERO), "pitch {p}");
        assert_eq!(v[0], f1(9000, 16383, 77)[0]);
    }
    assert!(!f1(9000, 16379, 77)[2].eq_num(F64::ZERO));
}

// spec: S-0306 claim 6 c and F13 (|Δz| ≤ 200: exactly level, pitch facing 16383)
#[test]
fn claim6c_level_within_200() {
    for dz in [-200, 0, 150, 200] {
        let mut s = setup(&base(), 5, 0);
        let f = fp(&s, -dz);
        let t = Target::Object(s.t);
        let id = fire(&mut s, t, f).unwrap();
        let tp = pos(&s, s.t);
        let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
        assert_eq!(pr(&s, id).vel, f1(yaw_to(f, tp), 16383, start), "Δz {dz}");
    }
}

// spec: S-0306 claim 6 c and F13 (|Δz| > 200: pitched by sgn(Z) × A(|Z| − 20, max(d′, 0.05)))
#[test]
fn claim6c_pitched_beyond_200() {
    for dz in [-450, -201, 201, 600] {
        let mut s = setup(&base(), 5, 0);
        let f = fp(&s, -dz);
        let t = Target::Object(s.t);
        let id = fire(&mut s, t, f).unwrap();
        let tp = pos(&s, s.t);
        let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
        let pitch = f13(dz, 20, d2(&s.w, f, tp));
        assert_ne!(pitch, 16383);
        assert_eq!(pr(&s, id).vel, f1(yaw_to(f, tp), pitch, start), "Δz {dz}");
    }
}

/// A straight shot at a tower (art height `hb`) from a fire point `h`
/// above level 0, the tower on level 0.
fn building_pitch(r: &mut Ruleset, hb: i32, h: i32) -> ([F64; 3], Pos, Pos) {
    r.sim.buildings.get_mut("TOWER").unwrap().art_height = Some(hb);
    let mut s = setup(r, 3, 0);
    let tw = testkit::spawn(&mut s.w, Kind::Building, "TOWER", 1, cell_of(5, 0), 0);
    s.w.obj_mut(s.a).unwrap().target = Some(Target::Object(tw));
    let f = fp(&s, h);
    let id = fire(&mut s, Target::Object(tw), f).unwrap();
    (pr(&s, id).vel, f, s.w.reference_point(tw))
}

// spec: S-0306 claim 6 c, F13 and Edge cases (quirks.building_pitch_term on: Z = 200 × H_b − the fire point's height, τ = 20 when |Z| ≥ 20)
#[test]
fn quirk_building_pitch_term_on() {
    let mut r = base();
    let (v, f, rp) = building_pitch(&mut r, 2, 500);
    let w = testkit::game_with(&r, N, 7);
    let dprime = d2(&w, f, rp);
    let start = arc(dprime).min(d3(&w, f, rp) / 2);
    let pitch = f13(400 - 500, 20, dprime);
    assert_ne!(pitch, f13(-500, 20, dprime));
    assert_eq!(v, f1(yaw_to(f, rp), pitch, start));
    // |Z| < 20: τ = 0 (Z = 400 − 410 = −10), and the 200 test still uses Δz.
    let mut r = base();
    let (v, f, rp) = building_pitch(&mut r, 2, 410);
    let pitch = f13(-10, 0, dprime);
    assert_eq!(v, f1(yaw_to(f, rp), pitch, start.min(d3(&w, f, rp) / 2)));
}

// spec: S-0306 Edge cases (quirks.building_pitch_term off: Z = Δz and τ = 20 for every target)
#[test]
fn quirk_building_pitch_term_off() {
    let mut r = base();
    r.globals.quirks.building_pitch_term = false;
    let (v, f, rp) = building_pitch(&mut r, 2, 500);
    let w = testkit::game_with(&r, N, 7);
    let dprime = d2(&w, f, rp);
    let start = arc(dprime).min(d3(&w, f, rp) / 2);
    assert_eq!(v, f1(yaw_to(f, rp), f13(-500, 20, dprime), start));
}

// spec: S-0306 claim 6 d (a vertical projectile takes case (c) by its flags, at start speed 1)
#[test]
fn claim6d_vertical_takes_the_straight_pitch() {
    let mut r = base();
    ball(&mut r).vertical = Some(true);
    let mut s = setup(&r, 5, 0);
    let f = fp(&s, 450);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let tp = pos(&s, s.t);
    let pitch = f13(tp.z - f.z, 20, d2(&s.w, f, tp));
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, tp), pitch, 1));
}

// spec: S-0306 claim 6 d and Edge cases (the arcing flag on a steering type: void at a level target, 45° at a higher one; scenario P5 run f)
#[test]
fn claim6d_arcing_flag_on_a_steering_type() {
    let mut r = base();
    ball(&mut r).arcing = Some(true);
    ball(&mut r).rot = Some(5);
    weapon(&mut r).speed = Some(25);
    let mut s = setup(&r, 5, 0);
    let f = fp(&s, 30);
    let t = Target::Object(s.t);
    assert!(fire(&mut s, t, f).is_none());
    let mut s = setup(&r, 5, 0);
    set_level(&mut s.w, cell_of(5, 0), 2);
    ground(&mut s.w, s.t);
    set_turret(&mut s.w, s.a, 16384);
    let f = fp(&s, 30);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).expect("45° launch");
    let v0 = f1(16384, 8191, 1);
    let v = pr(&s, id).vel;
    for i in 0..3 {
        assert!(close(v[i], v0[i]), "component {i}");
    }
}

// spec: S-0306 claim 7 and F1 (radial fire segments: shot k leaves the body facing turned by (π / n) k − π/2; b + 16384, 8192, 0, −8192 for n = 4; scenario P20)
#[test]
fn claim7_radial_fire_segments() {
    let mut r = base();
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .radial_fire_segments = Some(4);
    let mut s = setup(&r, 5, 0);
    let b = 6000;
    set_turret(&mut s.w, s.a, b as u16);
    let f = fp(&s, 0);
    let tp = pos(&s, s.t);
    let start = arc(d2(&s.w, f, tp)).min(d3(&s.w, f, tp) / 2);
    for (shot, k) in [0, 1, 2, 3, 0, 1].into_iter().enumerate() {
        let t = Target::Object(s.t);
        let id = fire(&mut s, t, f).unwrap();
        let y = radial_yaw(b, 4, k);
        let expected = (b + 16384 - 8192 * k).rem_euclid(65536);
        assert!(
            delta(y as u16, expected as u16).abs() <= 2,
            "shot {shot}: yaw {y}, expected about {expected}"
        );
        assert_eq!(pr(&s, id).vel, f1(y, 16383, start), "shot {shot}");
    }
}

// spec: S-0306 claim 7, F1 and Edge cases (quirks.zero_speed_launch on: a start speed of 0 launches at a horizontal speed of 100; scenario P28)
#[test]
fn quirk_zero_speed_launch_on() {
    let mut s = setup(&base(), 5, 0);
    let f = Pos::centre(pos(&s, s.a).cell(), 0);
    let c = Target::Cell(f.cell());
    let id = fire(&mut s, c, f).unwrap();
    let tp = pos(&s, s.t);
    let v = pr(&s, id).vel;
    assert_eq!(v, f1(yaw_to(f, tp), 16383, 0));
    let h = table_root_real(v[0].mul(v[0], RZ).add(v[1].mul(v[1], RZ), RZ), true, RZ);
    assert!(h.ge(nd("99.9")) && h.le(n(100)));
}

// spec: S-0306 Edge cases (quirks.zero_speed_launch off: the horizontal speed stays 0)
#[test]
fn quirk_zero_speed_launch_off() {
    let mut r = base();
    r.globals.quirks.zero_speed_launch = false;
    let mut s = setup(&r, 5, 0);
    let f = Pos::centre(pos(&s, s.a).cell(), 0);
    let c = Target::Cell(f.cell());
    let id = fire(&mut s, c, f).unwrap();
    let v = pr(&s, id).vel;
    assert!(v.iter().all(|c| c.eq_num(F64::ZERO)));
    assert_eq!(v, f1q(0, 16383, 0, false));
}

// spec: S-0306 claim 8 (the projectile is placed at the fire point and enters play; its first turn is in the same tick)
#[test]
fn claim8_first_turn_in_the_tick_of_placement() {
    let mut s = setup(&base(), 8, 0);
    let mut seen = None;
    for _ in 0..400 {
        glasnost_sim::tick::step(&mut s.w);
        if let Some(p) = s.w.projectiles.values().next() {
            seen = Some(p.clone());
            break;
        }
    }
    let p = seen.expect("the cart fires");
    assert!(s.w.update.contains(p.id));
    assert_eq!(p.placed, s.w.tick - 1);
    assert_eq!(p.turns, 1, "one turn taken in the tick of the shot");
    assert_ne!(p.pos, p.fire_point);
}

// spec: S-0306 claim 9 and Edge cases (quirks.stored_target_point on: K is the target's reference point at placement and does not move)
#[test]
fn quirk_stored_target_point_on() {
    let mut s = setup(&base(), 8, 0);
    let f = fp(&s, 100);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let k = pos(&s, s.t);
    assert_eq!(pr(&s, id).stored, k);
    let moved = Pos::new(k.x + 300, k.y, k.z);
    put(&mut s.w, s.t, moved);
    turn(&mut s.w, id);
    assert_eq!(pr(&s, id).stored, k);
}

// spec: S-0306 Edge cases (quirks.stored_target_point off: K is the target's reference point at the start of each turn)
#[test]
fn quirk_stored_target_point_off() {
    let mut r = base();
    r.globals.quirks.stored_target_point = false;
    let mut s = setup(&r, 8, 0);
    let f = fp(&s, 100);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let k = pos(&s, s.t);
    let moved = Pos::new(k.x + 300, k.y, k.z);
    put(&mut s.w, s.t, moved);
    turn(&mut s.w, id);
    assert_eq!(pr(&s, id).stored, moved);
}

// spec: S-0306 claim 9 (the arm time: the projectile type's arm field, or 0 when the target is an aircraft)
#[test]
fn claim9_arm_time() {
    let mut r = base();
    ball(&mut r).arm = Some(7);
    let mut s = setup(&r, 5, 0);
    let f = fp(&s, 30);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    assert_eq!(pr(&s, id).arm, 7);
    let plane = testkit::spawn(&mut s.w, Kind::Aircraft, "PLANE", 1, cell_of(4, 2), 0);
    let id = fire(&mut s, Target::Object(plane), f).unwrap();
    assert_eq!(pr(&s, id).arm, 0);
}

// spec: S-0306 claim 9 (an invisible projectile is moved at once to K and its velocity becomes 0)
#[test]
fn claim9_invisible_projectile_at_k() {
    let mut r = base();
    ball(&mut r).inviso = Some(true);
    let mut s = setup(&r, 5, 2);
    let f = fp(&s, 30);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    assert_eq!(pr(&s, id).pos, pos(&s, s.t));
    assert!(pr(&s, id).vel.iter().all(|c| c.eq_num(F64::ZERO)));
    // T1: Ω is the distance from the point it was moved to.
    assert_eq!(pr(&s, id).omega, 0);
}

// spec: S-0306 claim 9 (an invisible projectile stopped by the line-of-fire walk is placed at the first blocking cell's centre, at its terrain height; scenario P6 b)
#[test]
fn claim9_invisible_projectile_stopped_by_a_wall() {
    let mut r = base();
    ball(&mut r).inviso = Some(true);
    ball(&mut r).subject_to_walls = Some(true);
    overlays(&mut r, &[2]);
    let mut s = setup(&r, 5, 0);
    set_overlay(&mut s.w, cell_of(2, 0), 2);
    set_level(&mut s.w, cell_of(2, 0), 1);
    let f = fp(&s, 30);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let c = Pos::centre(cell_of(2, 0), 104);
    assert_eq!(pr(&s, id).pos, c);
}

// spec: S-0306 claim 9 (an invisible flak-scattering projectile: K moved by trunc(R(0, 2B) × D / R₀) in a random direction; scenario P3 variant d)
#[test]
fn claim9_invisible_flak_scatter_of_k() {
    let mut r = base();
    ball(&mut r).inviso = Some(true);
    ball(&mut r).flak_scatter = Some(true);
    r.sim.general.ballistic_scatter = Some(d("2"));
    let mut s = setup(&r, 6, 1);
    let f = fp(&s, 30);
    let mut c = s.w.rng.clone();
    let h = half(&s.w);
    let rr = c.ranged(0, 1024, h);
    let xi = c.ranged(0, 2147483646, h);
    let k0 = pos(&s, s.t);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    assert_eq!(s.w.rng, c, "radius draw, then direction draw");
    let sigma = rr * d3(&s.w, f, k0) / 2560;
    let k = f5((k0.x, k0.y), sigma, xi);
    let k = Pos::new(k.0, k.1, k0.z);
    assert_eq!(pr(&s, id).stored, k);
    assert_eq!(pr(&s, id).pos, k);
}

// spec: S-0306 claim 9 and F10 (a steering projectile's velocity is scaled to length 1 at placement, keeping its direction)
#[test]
fn claim9_steering_rescaled_to_one() {
    let mut r = base();
    ball(&mut r).rot = Some(5);
    weapon(&mut r).speed = Some(25);
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .radial_fire_segments = Some(1);
    let mut s = setup(&r, 5, 0);
    let f = fp(&s, 0);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let v = pr(&s, id).vel;
    let sp = speed(v);
    assert!(sp.ge(nd("0.999")) && sp.le(n(1)));
    // Shot k = 0 of one segment leaves a quarter turn clockwise: facing 16384, map east.
    assert!(v[0].gt(nd("0.99")) && v[1].abs().lt(nd("0.001")));
}

// ================================================================= flight

/// What a turn did.
#[derive(Debug, PartialEq, Eq)]
enum Out {
    /// Still flying, at this position.
    Flying(Pos),
    /// Detonated; the first detonation's point.
    Detonated(Pos),
    /// Removed without a detonation.
    Removed,
}

/// Launches `wp` at `shot` and then sets the projectile's position and
/// velocity (integers), its previous cell, and a large Ω.
fn place_with(s: &mut S, wp: u32, shot: Target, p: Pos, v: [i64; 3]) -> ObjectId {
    let f = fp(s, 300);
    let id = fire_with(s, wp, shot, f).expect("a launch");
    let x = s.w.projectiles.get_mut(&id).unwrap();
    x.pos = p;
    x.vel = v.map(n);
    x.prev_cell = p.cell();
    x.omega = 1_000_000;
    id
}

fn place(s: &mut S, shot: Target, p: Pos, v: [i64; 3]) -> ObjectId {
    let wp = s.wp;
    place_with(s, wp, shot, p, v)
}

fn step(s: &mut S, id: ObjectId) -> Out {
    s.w.effects.clear();
    turn(&mut s.w, id);
    if let Some(p) = s.w.projectiles.get(&id) {
        return Out::Flying(p.pos);
    }
    for e in &s.w.effects {
        if let glasnost_sim::combat::effects::Effect::Detonation { pos, .. } = e {
            return Out::Detonated(*pos);
        }
    }
    Out::Removed
}

/// A cell target far away (no reached row, no snapping nearby).
fn far() -> Target {
    Target::Cell(cell_of(12, 9))
}

fn c(dx: i32, dy: i32, z: i32) -> Pos {
    Pos::centre(cell_of(dx, dy), z)
}

// spec: S-0306 claim 10 (V_z falls by g each turn, then trunc(P + V) with the reduced velocity)
#[test]
fn claim10_straight_flight_falls_by_gravity() {
    let mut s = setup(&base(), 5, 5);
    let p = c(-3, 2, 400);
    let id = place(&mut s, far(), p, [37, -11, 4]);
    assert_eq!(
        step(&mut s, id),
        Out::Flying(Pos::new(p.x + 37, p.y - 11, p.z - 6))
    );
    assert!(pr(&s, id).vel[2].eq_num(n(-6)));
    assert_eq!(
        step(&mut s, id),
        Out::Flying(Pos::new(p.x + 74, p.y - 22, p.z - 22))
    );
}

// spec: S-0306 claim 10 and Terms (an arcing projectile shares the flight; a floater falls by g / 2)
#[test]
fn claim10_floater_falls_by_half_gravity() {
    let mut r = base();
    ball(&mut r).arcing = Some(true);
    ball(&mut r).floater = Some(true);
    let mut s = setup(&r, 5, 5);
    let p = c(-3, 2, 400);
    let id = place(&mut s, far(), p, [20, 0, 0]);
    assert_eq!(
        step(&mut s, id),
        Out::Flying(Pos::new(p.x + 20, p.y, p.z - 5))
    );
    assert_eq!(
        step(&mut s, id),
        Out::Flying(Pos::new(p.x + 40, p.y, p.z - 15))
    );
}

/// A vertical projectile flying level along map x at speed 1.
fn vertical(accel: i32, alt: i32, uncapped: bool) -> (S, ObjectId) {
    let mut r = base();
    ball(&mut r).vertical = Some(true);
    ball(&mut r).acceleration = Some(accel);
    ball(&mut r).detonation_altitude = Some(alt);
    weapon(&mut r).range = Some(d("6"));
    weapon(&mut r).speed = Some(20);
    r.globals.quirks.vertical_speed_uncapped = uncapped;
    let mut s = setup(&r, 5, 5);
    let id = place(&mut s, far(), c(-10, -8, 300), [1, 0, 0]);
    (s, id)
}

fn tspeed(s: &S, id: ObjectId) -> i32 {
    speed(pr(s, id).vel).to_i32_common()
}

// spec: S-0306 claim 11, F10 and Edge cases (quirks.vertical_speed_uncapped on: trunc(|V|) + α while below S_w, not capped; scenario P8 run a)
#[test]
fn quirk_vertical_speed_uncapped_on() {
    // S_w = v(1536, 10) = 135, α = 50.
    let (mut s, id) = vertical(50, 5000, true);
    assert_eq!(pr(&s, id).top_speed, 135);
    let mut seen = Vec::new();
    for _ in 0..6 {
        assert!(matches!(step(&mut s, id), Out::Flying(_)));
        seen.push(tspeed(&s, id));
        // Heading and pitch never change: level along map x, no gravity.
        let v = pr(&s, id).vel;
        assert!(v[1].eq_num(F64::ZERO) && v[2].eq_num(F64::ZERO));
    }
    // 1 → 51 → 101 → 151 (each possibly one short), then constant above 135.
    for (k, &sp) in seen.iter().take(3).enumerate() {
        let want = 1 + 50 * (k as i32 + 1);
        assert!(sp == want || sp == want - 1, "turn {k}: {sp}");
    }
    assert!(seen[2] > 135);
    assert_eq!(seen[3], seen[2]);
    assert_eq!(seen[5], seen[2]);
    assert_eq!(pr(&s, id).pos.z, 300);
}

// spec: S-0306 Edge cases (quirks.vertical_speed_uncapped off: min(trunc(|V|) + α, S_w))
#[test]
fn quirk_vertical_speed_uncapped_off() {
    let (mut s, id) = vertical(50, 5000, false);
    for _ in 0..5 {
        step(&mut s, id);
    }
    let sp = tspeed(&s, id);
    assert!(sp == 135 || sp == 134, "capped at S_w: {sp}");
}

// spec: S-0306 claim 11 and Edge cases (a detonation altitude of 0 detonates in the first turn whose world height is above 0)
#[test]
fn claim11_vertical_detonation_altitude() {
    let (mut s, id) = vertical(3, 0, true);
    assert!(matches!(step(&mut s, id), Out::Detonated(_)));
    // Above the altitude only once the new world height exceeds it.
    let (mut s, id) = vertical(3, 300, true);
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
    s.w.projectiles.get_mut(&id).unwrap().vel = [n(0), n(0), n(5)];
    assert!(matches!(step(&mut s, id), Out::Detonated(_)));
}

// spec: S-0306 claim 11 and T1 (a vertical projectile whose height above the surface at the start of the turn is negative detonates)
#[test]
fn claim11_vertical_negative_start_height() {
    let (mut s, id) = vertical(3, 5000, true);
    s.w.projectiles.get_mut(&id).unwrap().pos = c(-10, -8, -1);
    assert!(matches!(step(&mut s, id), Out::Detonated(_)));
}

/// A steering projectile `Ball` (ρ = `rot`, speed field 25: top speed 64)
/// with the given course lock duration and acceleration.
fn steering(rot: i32, lock: i32, accel: i32) -> Ruleset {
    let mut r = base();
    ball(&mut r).rot = Some(rot);
    ball(&mut r).course_lock_duration = Some(lock);
    ball(&mut r).acceleration = Some(accel);
    weapon(&mut r).speed = Some(25);
    r
}

fn steer_place(s: &mut S, p: Pos, v: [i64; 3]) -> ObjectId {
    let t = Target::Object(s.t);
    let id = place(s, t, p, v);
    let lock = s.w.rules().combat.projectile_of(s.wp).unwrap().course_lock;
    s.w.projectiles.get_mut(&id).unwrap().locked = lock != 1;
    id
}

// spec: S-0306 claim 12 and F10 (the speed grows by α towards the top speed: 1, 4, 7 …; scenario P9 a)
#[test]
fn claim12_steering_speed_grows_by_the_acceleration() {
    let r = steering(5, 1, 3);
    let mut s = setup(&r, 15, 0);
    let f = fp(&s, 300);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    for k in 1..=8 {
        assert!(matches!(step(&mut s, id), Out::Flying(_)));
        let sp = tspeed(&s, id);
        let want = 1 + 3 * k;
        assert!(sp == want || sp == want - 1, "turn {k}: {sp}");
    }
}

// spec: S-0306 claim 12 and F10 (above the top speed it falls by trunc(α / 2), not below 0)
#[test]
fn claim12_steering_speed_falls_above_the_top() {
    let r = steering(5, 1, 5);
    let mut s = setup(&r, 15, 0);
    let id = steer_place(&mut s, c(-12, 0, 300), [100, 0, 0]);
    step(&mut s, id);
    let sp = speed(pr(&s, id).vel);
    // 100 − 2, up to the rounding of the table root.
    assert!(sp.gt(nd("97.9")) && sp.le(n(98)), "{}", sp.to_i32_common());
}

// spec: S-0306 claim 12 (κ = 0 and S < 40: during the lock the acceleration is 1 in even ticks and 0 in odd ones; scenario P9 b)
#[test]
fn claim12_course_lock_zero_slow_missile() {
    let mut r = steering(5, 0, 3);
    weapon(&mut r).speed = Some(10); // top speed 25
    let mut s = setup(&r, 15, 0);
    let id = steer_place(&mut s, c(-12, 0, 300), [1, 0, 0]);
    let mut last = speed(pr(&s, id).vel);
    for _ in 0..6 {
        let even = s.w.tick % 2 == 0;
        step(&mut s, id);
        let sp = speed(pr(&s, id).vel);
        let gain = sp.sub(last, RZ);
        if even {
            assert!(gain.gt(nd("0.99")) && gain.lt(nd("1.01")), "even tick");
        } else {
            assert!(gain.abs().lt(nd("0.01")), "odd tick");
        }
        assert!(pr(&s, id).locked);
        last = sp;
        s.w.tick += 1;
    }
}

// spec: S-0306 claim 12 (κ = 0: the lock ends in the first turn with S ≥ 40 or S ≤ |V| + 0.5, a turn already unlocked)
#[test]
fn claim12_course_lock_zero_ends() {
    // S = 64 ≥ 40: unlocked in the first turn, with the normal acceleration.
    let r = steering(5, 0, 3);
    let mut s = setup(&r, 15, 0);
    let id = steer_place(&mut s, c(-12, 0, 300), [1, 0, 0]);
    step(&mut s, id);
    assert!(!pr(&s, id).locked);
    let sp = tspeed(&s, id);
    assert!(sp == 4 || sp == 3);
    // S = 25 and |V| = 24.6: 25 ≤ 24.6 + 0.5, unlocked in that turn.
    let mut r = steering(5, 0, 3);
    weapon(&mut r).speed = Some(10);
    let mut s = setup(&r, 15, 0);
    let id = steer_place(&mut s, c(-12, 0, 300), [24, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().vel = [nd("24.6"), n(0), n(0)];
    step(&mut s, id);
    assert!(!pr(&s, id).locked);
    // |V| = 24.4: still locked.
    let id = steer_place(&mut s, c(-12, 3, 300), [24, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().vel = [nd("24.4"), n(0), n(0)];
    step(&mut s, id);
    assert!(pr(&s, id).locked);
}

// spec: S-0306 claim 12 (κ ≥ 1: the lock covers the first κ − 1 turns, the κ-th is the first unlocked one)
#[test]
fn claim12_course_lock_duration() {
    let r = steering(5, 4, 3);
    let mut s = setup(&r, 15, 0);
    let f = fp(&s, 300);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    for turn_no in 1..=5 {
        step(&mut s, id);
        assert_eq!(pr(&s, id).locked, turn_no < 4, "turn {turn_no}");
    }
    // κ = 1: no locked turn.
    let r = steering(5, 1, 3);
    let mut s = setup(&r, 15, 0);
    let f = fp(&s, 300);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    assert!(!pr(&s, id).locked);
}

// spec: S-0306 claim 12 and Edge cases (κ < 0: the lock never ends, the acceleration is the normal one; scenario P9 p)
#[test]
fn claim12_negative_course_lock() {
    let r = steering(5, -1, 3);
    let mut s = setup(&r, 15, 0);
    let f = fp(&s, 300);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    for k in 1..=5 {
        step(&mut s, id);
        assert!(pr(&s, id).locked);
        let sp = tspeed(&s, id);
        assert!(sp == 1 + 3 * k || sp == 3 * k, "turn {k}: {sp}");
    }
    assert_eq!(pr(&s, id).unlocked, 0);
}

/// F7: the turn step for object number `id`, tick `tick`, turn rate ρ and
/// variation m, near or not.
fn f7(id: u32, tick: i32, rho: i32, m: F64, near: bool, wrap: bool) -> i32 {
    let j = (i64::from(id) + i64::from(tick)).rem_euclid(15);
    let k15 = nd("0.06666666666666667");
    let (c1, _) = direction_constants();
    let two_pi = c1.mul(n(4), RZ);
    let q = sin_t(n(j).mul(k15, RZ).mul(two_pi, RZ), RZ)
        .mul(m, RZ)
        .add(m.add(n(1), RZ), RZ)
        .mul(n(rho.into()), RZ)
        .to_i32_common();
    let qs = if near {
        n(q.into()).mul(nd("1.5"), RZ).to_i32_common()
    } else {
        q
    };
    if wrap {
        i32::from((256 * qs.rem_euclid(256)) as i16)
    } else {
        256 * qs.min(127)
    }
}

/// F7's turn towards a goal facing.
fn towards(cur: i32, goal: i32, t: i32) -> i32 {
    let dl = delta(cur as u16, goal as u16);
    if t.abs() >= dl.abs() {
        goal
    } else if dl >= 0 {
        i32::from((cur + t) as u16)
    } else {
        i32::from((cur - t) as u16)
    }
}

/// A level steering projectile (no pitch change with a target), the
/// turn-step test bed of claims 13 and 14.
fn level_missile(rot: i32, m: &str, wrap: bool) -> S {
    let mut r = steering(rot, 1, 3);
    ball(&mut r).level = Some(true);
    r.sim.general.missile_rot_var = Some(d(m));
    r.globals.quirks.missile_turn_wrap = wrap;
    let mut s = setup(&r, 15, 0);
    // Over water, so that the level row of T1 does not detonate it.
    water_role(&mut s.w, 0);
    s
}

/// One turn of the level missile flying east at its top speed 64 from `p`
/// with the target at `tp`; returns the facing F7 and F11 turn it to, the
/// velocity after the turn and the outcome.
fn level_turn(s: &mut S, p: Pos, tp: Pos, near: bool) -> (i32, [F64; 3], Out) {
    put(&mut s.w, s.t, tp);
    let id = steer_place(s, p, [64, 0, 0]);
    let rot = s.w.rules().combat.projectile_of(s.wp).unwrap().rot;
    let m = s.w.rules().combat.general.missile_rot_var;
    let wrap = s.w.rules().quirks.missile_turn_wrap;
    let t = f7(id.0, s.w.tick, rot, m, near, wrap);
    let pp = Pos::new(p.x + 64, p.y, p.z);
    let yg = yaw_to(pp, tp);
    let y = towards(16384, yg, t);
    let out = step(s, id);
    let v = s.w.projectiles.get(&id).map_or([F64::ZERO; 3], |x| x.vel);
    (y, v, out)
}

/// The F11 velocity of facing `y` at horizontal speed 64.
fn turned(y: i32) -> [F64; 3] {
    let th = theta(y, RZ);
    let h = table_root_real(n(4096), true, RZ);
    [
        cos_t(th, RZ).mul(h, RZ),
        sin_t(th, RZ).mul(h, RZ).negated(),
        F64::ZERO,
    ]
}

fn same_heading(v: [F64; 3], w: [F64; 3]) -> bool {
    close(v[0], w[0]) && close(v[1], w[1])
}

// spec: S-0306 claim 13 and F7 (t = 256 × (q mod 256) with q = trunc((1 + m + m sin_t(2πj/15)) × ρ); m = 0 gives q = ρ)
#[test]
fn claim13_turn_step_with_no_variation() {
    let mut s = level_missile(10, "0", true);
    // The target to the north-east, far: t = 2560.
    let p = c(-12, 0, 300);
    let tp = Pos::new(p.x + 3000, p.y - 3000, 300);
    let (y, v, out) = level_turn(&mut s, p, tp, false);
    assert!(matches!(out, Out::Flying(_)));
    assert_eq!(y, 16384 - 2560);
    assert!(same_heading(v, turned(y)));
}

// spec: S-0306 claim 13 and F7 (the 15-tick variation with m = 0.25: the step follows the phase j = (object number + tick) mod 15)
#[test]
fn claim13_turn_step_variation_phase() {
    for tick in 40..46 {
        let mut s = level_missile(20, "0.25", true);
        s.w.tick = tick;
        let p = c(-12, 0, 300);
        let tp = Pos::new(p.x + 300, p.y - 3000, 300);
        let (y, v, _) = level_turn(&mut s, p, tp, false);
        assert!(same_heading(v, turned(y)), "tick {tick}");
    }
}

// spec: S-0306 claim 13 and F7 (within 256 leptons of the target point the step uses q* = trunc(1.5 q))
#[test]
fn claim13_near_target_factor() {
    let mut s = level_missile(10, "0", true);
    let p = c(-12, 0, 300);
    let tp = Pos::new(p.x, p.y - 200, 300);
    let (y, v, out) = level_turn(&mut s, p, tp, true);
    assert!(matches!(out, Out::Flying(_)), "{out:?}");
    assert_eq!(y, 16384 - 3840);
    assert!(same_heading(v, turned(y)));
}

// spec: S-0306 claim 13 and Edge cases (quirks.missile_turn_wrap on: a residue from 128 to 255 gives a negative step, turning away; scenario P9 e)
#[test]
fn quirk_missile_turn_wrap_on() {
    let mut s = level_missile(150, "0", true);
    // t = 256 × 150 − 65536 = −27136; the target behind, on the north side.
    let p = c(-12, 0, 300);
    let tp = Pos::new(p.x - 6000, p.y - 600, 300);
    let (y, v, _) = level_turn(&mut s, p, tp, false);
    let yg = yaw_to(Pos::new(p.x + 64, p.y, p.z), tp);
    assert!(delta(16384, yg as u16).abs() > 27136);
    assert_eq!(y, i32::from((16384 + 27136) as u16));
    assert!(same_heading(v, turned(y)));
}

// spec: S-0306 Edge cases (quirks.missile_turn_wrap off: t = 256 × min(q, 127), always towards the goal)
#[test]
fn quirk_missile_turn_wrap_off() {
    let mut s = level_missile(150, "0", false);
    let p = c(-12, 0, 300);
    let tp = Pos::new(p.x - 6000, p.y - 600, 300);
    let (y, v, _) = level_turn(&mut s, p, tp, false);
    let yg = yaw_to(Pos::new(p.x + 64, p.y, p.z), tp);
    // |t| = 32512 is at least the facing difference: it snaps to the goal.
    assert_eq!(y, yg);
    assert!(same_heading(v, turned(y)));
}

// spec: S-0306 claim 14 (with a target the displacement is trunc(V) after the speed change, before the heading change)
#[test]
fn claim14_displacement_before_the_heading_change() {
    let mut s = level_missile(10, "0", true);
    let p = c(-12, 0, 300);
    let tp = Pos::new(p.x + 3000, p.y - 3000, 300);
    let (y, _, out) = level_turn(&mut s, p, tp, false);
    assert_ne!(y, 16384);
    assert_eq!(out, Out::Flying(Pos::new(p.x + 64, p.y, p.z)));
}

// spec: S-0306 claims 14 and 15 (without a target the heading is kept, the pitch climbs towards 8192 by t, and the displacement already includes the pitch change)
#[test]
fn claim14_no_target_displacement_includes_the_pitch_change() {
    let mut r = steering(10, 1, 3);
    r.sim.general.missile_rot_var = Some(d("0"));
    let mut s = setup(&r, 15, 0);
    let p = c(-12, 0, 300);
    let id = steer_place(&mut s, p, [64, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().target = None;
    let out = step(&mut s, id);
    let v = pr(&s, id).vel;
    // Heading kept (east), climbing by t = 2560.
    assert!(same(v, pitched(-2560)));
    let moved = Pos::new(
        p.x + v[0].to_i32_common(),
        p.y + v[1].to_i32_common(),
        p.z + v[2].to_i32_common(),
    );
    assert_eq!(out, Out::Flying(moved));
    assert!(moved.z > p.z);
}

/// The pitch facing of a velocity, F12's p(V).
fn pitch(v: [F64; 3]) -> i32 {
    let h = table_root_real(v[0].mul(v[0], RZ).add(v[1].mul(v[1], RZ), RZ), true, RZ);
    let (a, i) = table_angle_real(h, v[2], RZ);
    i32::from(glasnost_num::geom::facing_of_angle(a, i, true, RZ) as i16)
}

/// F12's rebuild of a velocity from pitch `pc` to `pn`, keeping length `m`.
fn rebuilt(v: [F64; 3], pc: i32, pn: i32, m: F64) -> [F64; 3] {
    let cpc = cos_t(theta(pc, RZ), RZ);
    let (hx, hy) = (v[0].div(cpc, RZ), v[1].div(cpc, RZ));
    let tn = theta(pn, RZ);
    [
        cos_t(tn, RZ).mul(hx, RZ),
        cos_t(tn, RZ).mul(hy, RZ),
        sin_t(tn, RZ).mul(m, RZ),
    ]
}

/// The velocity of a level missile flying east at 64 after a pitch turn by
/// `step` (signed) from level.
fn pitched(step: i32) -> [F64; 3] {
    let v = [n(64), F64::ZERO, F64::ZERO];
    rebuilt(v, 16383, 16383 + step, n(64))
}

fn same(v: [F64; 3], w: [F64; 3]) -> bool {
    (0..3).all(|i| close(v[i], w[i]))
}

// spec: S-0306 claim 15 (direct mode within 3 cells: the goal is the pitch of the vector to the target point, step trunc((t + 256) / 2))
#[test]
fn claim15_direct_mode_near_the_target() {
    let mut r = steering(10, 1, 3);
    r.sim.general.missile_rot_var = Some(d("0"));
    let mut s = setup(&r, 15, 0);
    let p = c(-12, 0, 300);
    // The target 600 ahead and 400 below: direct mode, the goal well below level.
    put(&mut s.w, s.t, Pos::new(p.x + 664, p.y, -100));
    let id = steer_place(&mut s, p, [64, 0, 0]);
    step(&mut s, id);
    // From level by (2560 + 256) / 2 = 1408 towards a goal below.
    assert!(same(pr(&s, id).vel, pitched(1408)));
}

// spec: S-0306 claim 15 (during course lock t = 0: a projectile with a target is in direct mode, step 128; scenario P9 d)
#[test]
fn claim15_pitch_turns_during_the_lock() {
    let mut r = steering(10, 10, 3);
    r.sim.general.missile_rot_var = Some(d("0"));
    let mut s = setup(&r, 15, 0);
    let p = c(-12, 0, 300);
    put(&mut s.w, s.t, Pos::new(p.x + 2500, p.y, 1500));
    let id = steer_place(&mut s, p, [64, 0, 0]);
    step(&mut s, id);
    assert!(pr(&s, id).locked);
    assert!(same(pr(&s, id).vel, pitched(-128)));
    // The heading did not change.
    assert!(pr(&s, id).vel[1].abs().lt(nd("0.0001")));
}

/// A cruising missile: ρ = 10, m = 0 (t = 2560, cruise step 1280), 10
/// cells from its target, flying level at height `z`.
fn cruise(z: i32, mods: impl Fn(&mut Ruleset)) -> (S, ObjectId, Pos) {
    let mut r = steering(10, 1, 3);
    r.sim.general.missile_rot_var = Some(d("0"));
    mods(&mut r);
    let mut s = setup(&r, 15, 0);
    let p = c(-12, 0, z);
    put(&mut s.w, s.t, Pos::new(p.x + 64 + 2560, p.y, 0));
    let id = steer_place(&mut s, p, [64, 0, 0]);
    (s, id, p)
}

// spec: S-0306 claim 15 (cruise mode: far below the cruising altitude E + 5 × 104 the goal is 8192 with step trunc(t / 2), and the projectile is lifted 18 leptons)
#[test]
fn claim15_cruise_climbs_and_lifts() {
    let (mut s, id, p) = cruise(100, |_| {});
    let out = step(&mut s, id);
    assert_eq!(out, Out::Flying(Pos::new(p.x + 64, p.y, p.z + 18)));
    assert!(same(pr(&s, id).vel, pitched(-1280)));
}

// spec: S-0306 claim 15 (cruise mode: above the cruising altitude the goal is 18432 and the projectile is lowered 18 leptons)
#[test]
fn claim15_cruise_descends_and_lowers() {
    let (mut s, id, p) = cruise(900, |_| {});
    let out = step(&mut s, id);
    assert_eq!(out, Out::Flying(Pos::new(p.x + 64, p.y, p.z - 18)));
    assert!(same(pr(&s, id).vel, pitched(1280)));
}

// spec: S-0306 claim 15 (within 20 leptons of the cruising altitude: no lift, goal 16384)
#[test]
fn claim15_cruise_dead_band() {
    let (mut s, id, p) = cruise(530, |_| {});
    let out = step(&mut s, id);
    assert_eq!(out, Out::Flying(Pos::new(p.x + 64, p.y, p.z)));
    let pc = pitch(pr(&s, id).vel);
    assert!((16383..=16384).contains(&pc), "pitch {pc}");
}

// spec: S-0306 claim 15 (between 20 and 52 below the cruising altitude: lifted 18, goal still 16384)
#[test]
fn claim15_cruise_lift_without_climb() {
    let (mut s, id, p) = cruise(490, |_| {});
    let out = step(&mut s, id);
    assert_eq!(out, Out::Flying(Pos::new(p.x + 64, p.y, p.z + 18)));
    let pc = pitch(pr(&s, id).vel);
    assert!((16383..=16384).contains(&pc), "pitch {pc}");
}

// spec: S-0306 claim 15 (very high: n = 10, and direct mode only within 6 cells; scenario P9 k)
#[test]
fn claim15_very_high_cruises_at_ten_levels() {
    // At 1040 the very high projectile holds its altitude.
    let (mut s, id, p) = cruise(1040, |r| ball(r).very_high = Some(true));
    assert_eq!(step(&mut s, id), Out::Flying(Pos::new(p.x + 64, p.y, p.z)));
    // Within 6 cells: direct mode (goal the vector to the target, below).
    let (mut s, id, p) = cruise(1040, |r| ball(r).very_high = Some(true));
    put(&mut s.w, s.t, Pos::new(p.x + 64 + 1300, p.y, 0));
    step(&mut s, id);
    assert!(same(pr(&s, id).vel, pitched(1408)));
}

// spec: S-0306 claim 15 (airburst: n = 10 and never direct mode by distance; scenario P9 l)
#[test]
fn claim15_airburst_never_direct_by_distance() {
    let (mut s, id, p) = cruise(1040, |r| ball(r).airburst = Some(true));
    // Within 3 cells, the target below.
    put(&mut s.w, s.t, Pos::new(p.x + 64 + 600, p.y, 0));
    assert_eq!(step(&mut s, id), Out::Flying(Pos::new(p.x + 64, p.y, p.z)));
    let pc = pitch(pr(&s, id).vel);
    assert!((16383..=16384).contains(&pc), "pitch {pc}");
}

// spec: S-0306 claim 15 (an aircraft target: direct mode at any distance; scenario P9 n)
#[test]
fn claim15_aircraft_target_direct_mode() {
    let (mut s, id, p) = cruise(100, |_| {});
    let plane = testkit::spawn(&mut s.w, Kind::Aircraft, "PLANE", 1, cell_of(8, 6), 0);
    put(&mut s.w, plane, Pos::new(p.x + 64 + 2560, p.y, 100));
    s.w.projectiles.get_mut(&id).unwrap().target = Some(Target::Object(plane));
    let out = step(&mut s, id);
    // No lift; the goal is level (the aircraft at the same height): the pitch stays level.
    assert_eq!(out, Out::Flying(Pos::new(p.x + 64, p.y, p.z)));
    let pc = pitch(pr(&s, id).vel);
    assert!((16383..=16384).contains(&pc), "pitch {pc}");
}

// spec: S-0306 claim 15 (a level projectile with a target keeps its pitch and gets no cruise lift; scenario P9 m)
#[test]
fn claim15_level_projectile_keeps_its_pitch() {
    let (mut s, id, p) = cruise(100, |r| ball(r).level = Some(true));
    water_role(&mut s.w, 0);
    assert_eq!(step(&mut s, id), Out::Flying(Pos::new(p.x + 64, p.y, p.z)));
    let pc = pitch(pr(&s, id).vel);
    assert!((16383..=16384).contains(&pc));
}

// spec: S-0306 claim 15 (the cruising altitude is measured at the cell of P′ + 6 × trunc(V))
#[test]
fn claim15_cruise_look_ahead_cell() {
    let (mut s, id, p) = cruise(530, |r| ball(r).acceleration = Some(0));
    // V = (42.7, 0, 0) keeps its length (α = 0 below the top speed). P′.x mod 256 = 2:
    // P′ + 6 × 42 stays in P′'s cell, P′ + trunc(6 × 42.7) = P′ + 256 reaches the next one.
    let start = Pos::new(p.x - 128 + 2 - 42, p.y, 530);
    {
        let x = s.w.projectiles.get_mut(&id).unwrap();
        x.pos = start;
        x.prev_cell = start.cell();
        x.vel = [nd("42.7"), n(0), n(0)];
    }
    let next = Pos::new(start.x + 42 + 256, start.y, 0).cell();
    assert_ne!(next, Pos::new(start.x + 42 + 252, start.y, 0).cell());
    set_deck(&mut s.w, next);
    let out = step(&mut s, id);
    // The look-ahead cell is P′'s own (no bridge): within the dead band, no lift, no climb.
    assert_eq!(out, Out::Flying(Pos::new(start.x + 42, start.y, 530)));
    assert!(pr(&s, id).vel[2].eq_num(F64::ZERO));
}

// spec: S-0306 claim 15 (with a high bridge in the look-ahead cell the cruising altitude is 416 higher; scenario P9 o)
#[test]
fn claim15_cruise_over_a_high_bridge() {
    let (mut s, id, p) = cruise(530, |_| {});
    let ahead = Pos::new(p.x + 64 + 384, p.y, 0).cell();
    set_deck(&mut s.w, ahead);
    let out = step(&mut s, id);
    assert_eq!(out, Out::Flying(Pos::new(p.x + 64, p.y, p.z + 18)));
    assert!(same(pr(&s, id).vel, pitched(-1280)));
}

/// The trailer test bed: a straight projectile with trailer `TRAIL` and
/// spawn delay `delay`, high above the map.
fn trailer(delay: i32) -> (S, ObjectId) {
    let mut r = base();
    ball(&mut r).trailer = Some("TRAIL".into());
    ball(&mut r).spawn_delay = Some(delay);
    let mut s = setup(&r, 5, 5);
    let id = place(&mut s, far(), c(-10, -8, 30000), [10, 0, 0]);
    (s, id)
}

fn trailers(s: &S) -> Vec<Pos> {
    s.w.effects
        .iter()
        .filter_map(|e| match e {
            glasnost_sim::combat::effects::Effect::Anim { name, pos, .. } if name == "TRAIL" => {
                Some(*pos)
            }
            _ => None,
        })
        .collect()
}

// spec: S-0306 claim 16 (a trailer animation at the start-of-turn position in every turn whose tick number is a multiple of the spawn delay; scenario P10)
#[test]
fn claim16_trailer_ticks_and_position() {
    let (mut s, id) = trailer(5);
    for tick in 100..112 {
        s.w.tick = tick;
        let start = pr(&s, id).pos;
        step(&mut s, id);
        let tr = trailers(&s);
        if tick % 5 == 0 {
            assert_eq!(tr, vec![start], "tick {tick}");
        } else {
            assert!(tr.is_empty(), "tick {tick}");
        }
    }
}

// spec: S-0306 claim 16 (a negative spawn delay: every tick number is a multiple of −1; scenario P10 variant e)
#[test]
fn claim16_negative_spawn_delay_every_turn() {
    let (mut s, id) = trailer(-1);
    for tick in 100..104 {
        s.w.tick = tick;
        step(&mut s, id);
        assert_eq!(trailers(&s).len(), 1, "tick {tick}");
    }
}

// spec: S-0306 Edge cases (effective spawn delay 0 with a trailer: no trailer animation, a departure)
#[test]
fn edge_spawn_delay_zero_no_trailer() {
    let (mut s, id) = trailer(0);
    for tick in 100..104 {
        s.w.tick = tick;
        step(&mut s, id);
        assert!(trailers(&s).is_empty());
    }
}

fn big_hit() -> glasnost_sim::combat::damage::Hit {
    glasnost_sim::combat::damage::Hit {
        amount: 10_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    }
}

// spec: S-0306 claim 17 (a target that leaves the game: its cell when it was not airborne, else nothing)
#[test]
fn claim17_lost_target() {
    let mut s = setup(&base(), 6, 0);
    let f = fp(&s, 300);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let cell = pos(&s, s.t).cell();
    let tt = s.t;
    glasnost_sim::combat::death::kill(&mut s.w, tt, &big_hit());
    assert_eq!(pr(&s, id).target, Some(Target::Cell(cell)));
    // An airborne target: nothing.
    let mut s = setup(&base(), 6, 0);
    let tp = pos(&s, s.t);
    put(&mut s.w, s.t, Pos::new(tp.x, tp.y, 400));
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    let tt = s.t;
    glasnost_sim::combat::death::kill(&mut s.w, tt, &big_hit());
    assert_eq!(pr(&s, id).target, None);
}

// spec: S-0306 claim 17 (a target that goes inside or is removed, leaving the map without being killed: its cell when it was not airborne, else nothing)
#[test]
fn claim17_target_taken_inside_or_removed() {
    for inside in [true, false] {
        let mut s = setup(&base(), 6, 0);
        let f = fp(&s, 300);
        let t = Target::Object(s.t);
        let id = fire(&mut s, t, f).unwrap();
        let cell = pos(&s, s.t).cell();
        let tt = s.t;
        if inside {
            glasnost_sim::objects::take_inside(&mut s.w, tt);
            assert!(s.w.obj(tt).is_some(), "the target still exists");
        } else {
            glasnost_sim::objects::remove(&mut s.w, tt);
        }
        assert_eq!(
            pr(&s, id).target,
            Some(Target::Cell(cell)),
            "inside {inside}"
        );
        // An airborne target: nothing.
        let mut s = setup(&base(), 6, 0);
        let tp = pos(&s, s.t);
        put(&mut s.w, s.t, Pos::new(tp.x, tp.y, 400));
        let t = Target::Object(s.t);
        let id = fire(&mut s, t, f).unwrap();
        let tt = s.t;
        if inside {
            glasnost_sim::objects::take_inside(&mut s.w, tt);
        } else {
            glasnost_sim::objects::remove(&mut s.w, tt);
        }
        assert_eq!(pr(&s, id).target, None, "airborne, inside {inside}");
    }
}

// spec: S-0306 claim 17 and T1 (a steering projectile without a target climbs and detonates once its height above the surface reaches the missile safety altitude; scenario P11 a)
#[test]
fn claim17_steering_without_target_climbs_to_the_safety_altitude() {
    let mut r = steering(20, 1, 3);
    r.sim.general.missile_safety_altitude = Some(500);
    let mut s = setup(&r, 15, 0);
    let p = c(-12, 0, 200);
    let id = steer_place(&mut s, p, [64, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().target = None;
    let mut last = 200;
    for _ in 0..200 {
        let h = pr(&s, id).pos.z;
        match step(&mut s, id) {
            Out::Flying(q) => {
                assert!(q.z >= last, "it climbs");
                assert!(h < 500);
                last = q.z;
            }
            Out::Detonated(_) => {
                assert!(h >= 500, "detonates once the start height reaches 500: {h}");
                return;
            }
            Out::Removed => panic!("removed"),
        }
        s.w.tick += 1;
    }
    panic!("never detonated");
}

// spec: S-0306 claims 17 and 21 (a projectile whose firer leaves the game keeps flying and detonates normally, with neither attacker nor attacking house; scenario P11 e)
#[test]
fn claim17_firer_killed_projectile_keeps_flying() {
    let mut s = setup(&base(), 6, 6);
    let p = c(-3, 2, 400);
    let id = place(&mut s, far(), p, [37, 0, 0]);
    let id2 = place(&mut s, far(), c(-3, 2, 20), [50, 0, -30]);
    let a = s.a;
    glasnost_sim::combat::death::kill(&mut s.w, a, &big_hit());
    assert_eq!(pr(&s, id).attacker, None);
    assert_eq!(pr(&s, id).house, None);
    assert_eq!(
        step(&mut s, id),
        Out::Flying(Pos::new(p.x + 37, p.y, p.z - 10))
    );
    assert!(matches!(step(&mut s, id2), Out::Detonated(_)));
}

// spec: S-0306 claim 18 (a degenerating projectile loses 1 point per turn without a detonation, not below 5)
#[test]
fn claim18_degeneration() {
    let mut r = base();
    ball(&mut r).degenerates = Some(true);
    let mut s = setup(&r, 5, 5);
    let id = place(&mut s, far(), c(-10, -8, 30000), [10, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().damage = 8;
    let mut seen = Vec::new();
    for _ in 0..5 {
        step(&mut s, id);
        seen.push(pr(&s, id).damage);
    }
    assert_eq!(seen, vec![7, 6, 5, 5, 5]);
}

// ================================================================= detonation (T1, T2, claims 19 to 21)

/// The projectile `id` as one made without a firer (S-0306 Terms: no
/// firer, no attacker, no attacking house).
fn no_firer(s: &mut S, id: ObjectId) {
    let x = s.w.projectiles.get_mut(&id).unwrap();
    x.firer = None;
    x.attacker = None;
    x.house = None;
}

// spec: S-0306 T1 row 1 and claim 20 (a) (below the terrain, a projectile with no firer: detonate at P′, raised to the surface)
#[test]
fn t1_row1_below_the_terrain() {
    let mut s = setup(&base(), 5, 5);
    let p = c(-3, 2, 20);
    let id = place(&mut s, far(), p, [50, 0, -30]);
    no_firer(&mut s, id);
    assert_eq!(step(&mut s, id), Out::Detonated(Pos::new(p.x + 50, p.y, 0)));
}

// spec: S-0306 T1 row 1 and Edge cases (quirks.owned_shell_ground_pass on: a projectile that has a firer is not detonated below the terrain; it flies on, its height not raised)
#[test]
fn t1_row1_owned_flies_on_below_the_terrain() {
    let mut s = setup(&base(), 5, 5);
    let p = c(-3, 2, 20);
    let id = place(&mut s, far(), p, [50, 0, -30]);
    assert!(glasnost_sim::combat::projectile::has_firer(
        &s.w,
        pr(&s, id)
    ));
    assert_eq!(
        step(&mut s, id),
        Out::Flying(Pos::new(p.x + 50, p.y, 20 - 40))
    );
}

// spec: S-0306 Edge cases (quirks.owned_shell_ground_pass off: the whole first row applies to a projectile that has a firer, so it detonates where it first passes below the terrain, raised to the surface)
#[test]
fn quirk_owned_shell_ground_pass_off_below_the_terrain() {
    let mut r = base();
    r.globals.quirks.owned_shell_ground_pass = false;
    let mut s = setup(&r, 5, 5);
    let p = c(-3, 2, 20);
    let id = place(&mut s, far(), p, [50, 0, -30]);
    assert_eq!(step(&mut s, id), Out::Detonated(Pos::new(p.x + 50, p.y, 0)));
}

// spec: S-0306 T1 (the deep-fall row h′ ≤ −416) with T1 row 1 and claim 20 (a) (quirks.owned_shell_ground_pass on; scenario P44 part b: a projectile with a firer flies on below the surface and detonates in the first turn in which it is 416 or more below it, at the surface there, farther along than where it passed below)
#[test]
fn p44b_owned_projectile_falls_to_the_deep_fall_row() {
    let mut s = setup(&base(), 5, 5);
    let p = c(-3, 2, 20);
    let id = place(&mut s, far(), p, [10, 0, -30]);
    // Level ground at height 0: V_z falls by 10 each turn (claim 10), so the
    // new heights are -20, -70, -130, -200, -280, -370 and then -470, the
    // first at 416 or more below the surface.
    let (mut z, mut vz) = (20, -30);
    for turn in 1..=7 {
        vz -= G;
        z += vz;
        let at = Pos::new(p.x + 10 * turn, p.y, z);
        if turn < 7 {
            assert_eq!(
                step(&mut s, id),
                Out::Flying(at),
                "turn {turn}: flies on below the surface"
            );
        } else {
            assert!(z <= -416);
            assert_eq!(
                step(&mut s, id),
                Out::Detonated(Pos::new(at.x, at.y, 0)),
                "the deep-fall row, at the surface"
            );
        }
        s.w.tick += 1;
    }
}

// spec: S-0306 T1 (the flak row: P′ below the target's reference height and h′ < 0, also for a projectile that has a firer) and claim 20 (a) (quirks.owned_shell_ground_pass on; scenario P45 part b: P44 part b with the flak-scattering flag and the weapon's range at the 6-cell distance of the attack detonates in the first turn with h′ < 0, at the surface there)
#[test]
fn p45b_owned_flak_projectile_detonates_in_its_first_turn_below_the_surface() {
    let mut r = base();
    ball(&mut r).flak_scatter = Some(true);
    weapon(&mut r).range = Some(d("6"));
    let mut s = setup(&r, 5, 5);
    let p = c(-3, 2, 20);
    let id = place(&mut s, far(), p, [10, 0, -30]);
    assert!(glasnost_sim::combat::projectile::has_firer(
        &s.w,
        pr(&s, id)
    ));
    // The first turn: V_z = -40, h′ = 20 - 40 = -20 < 0, below the far
    // cell's reference height 0 (level ground).
    assert_eq!(
        step(&mut s, id),
        Out::Detonated(Pos::new(p.x + 10, p.y, 0)),
        "the flak row in the first turn, at the surface"
    );
}

// spec: S-0306 T1 row 1 and Terms (a firer killed while the projectile flies: no firer, so the whole first row applies; scenario P44 part c: it detonates in the first turn below the terrain, at the surface there)
#[test]
fn p44c_firer_killed_detonates_below_the_terrain() {
    let mut s = setup(&base(), 5, 5);
    let p = c(-3, 2, 20);
    let id = place(&mut s, far(), p, [50, 0, -30]);
    let a = s.a;
    glasnost_sim::combat::death::kill(&mut s.w, a, &big_hit());
    assert!(!glasnost_sim::combat::projectile::has_firer(
        &s.w,
        pr(&s, id)
    ));
    assert_eq!(step(&mut s, id), Out::Detonated(Pos::new(p.x + 50, p.y, 0)));
}

// spec: S-0306 Terms (a firer removed from the game while the projectile flies: no firer; a firer gone inside, as into a closed transport, is still the firer) with T1 row 1
#[test]
fn terms_firer_removed_or_inside() {
    for removed in [true, false] {
        let mut s = setup(&base(), 5, 5);
        let p = c(-3, 2, 20);
        let id = place(&mut s, far(), p, [50, 0, -30]);
        let a = s.a;
        if removed {
            glasnost_sim::objects::remove(&mut s.w, a);
        } else {
            glasnost_sim::objects::take_inside(&mut s.w, a);
        }
        let owned = glasnost_sim::combat::projectile::has_firer(&s.w, pr(&s, id));
        assert_eq!(owned, !removed, "removed {removed}");
        let out = step(&mut s, id);
        if removed {
            assert_eq!(out, Out::Detonated(Pos::new(p.x + 50, p.y, 0)));
        } else {
            assert!(matches!(out, Out::Flying(_)), "inside: still the firer");
        }
    }
}

// spec: S-0306 T1 row 1 (crossing a bridge deck going up: z at the deck height)
#[test]
fn t1_row1_deck_crossing_up() {
    let mut s = setup(&base(), 5, 5);
    set_deck(&mut s.w, cell_of(-3, 2));
    let p = c(-3, 2, 400);
    let id = place(&mut s, far(), p, [10, 0, 40]);
    assert_eq!(
        step(&mut s, id),
        Out::Detonated(Pos::new(p.x + 10, p.y, 416))
    );
}

// spec: S-0306 T1 row 1 (crossing a bridge deck going down: z 20 below the deck)
#[test]
fn t1_row1_deck_crossing_down() {
    let mut s = setup(&base(), 5, 5);
    set_deck(&mut s.w, cell_of(-3, 2));
    let p = c(-3, 2, 430);
    let id = place(&mut s, far(), p, [10, 0, -10]);
    assert_eq!(
        step(&mut s, id),
        Out::Detonated(Pos::new(p.x + 10, p.y, 396))
    );
}

/// Who the projectile's firer is in the band tests.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Firer {
    /// The cart of house 0, still in play.
    Owned,
    /// None (Terms: the projectile has no firer).
    None,
}

/// A straight projectile entering cell (4, 0) at height `z` from the cell
/// before, with a building of `house` there; `tweak` changes the rules and
/// `prep` the building before the turn.
fn band_with(
    z: i32,
    house: u8,
    firer: Firer,
    tweak: impl FnOnce(&mut Ruleset),
    prep: impl FnOnce(&mut World, ObjectId),
) -> Out {
    let mut r = base();
    tweak(&mut r);
    let mut s = setup(&r, 6, 6);
    let b = testkit::spawn(&mut s.w, Kind::Building, "TOWER", house, cell_of(4, 0), 0);
    prep(&mut s.w, b);
    let p = c(3, 0, z);
    let id = place(&mut s, far(), p, [256, 0, 10]);
    if firer == Firer::None {
        no_firer(&mut s, id);
    }
    step(&mut s, id)
}

fn band(z: i32, house: u8, firer: Firer) -> Out {
    band_with(z, house, firer, |_| {}, |_, _| {})
}

// spec: S-0306 T1 row 1 (0 to 149 above the terrain in a cell holding an eligible building of any house, for a projectile with no firer: detonate, the point lowered to the terrain height; scenario P32 runs a, b and c)
#[test]
fn t1_row1_building_band() {
    // P′ lies in cell (4, 0) at 140 above the terrain; the point ends at the terrain height.
    let at = c(4, 0, 0);
    assert_eq!(band(140, 1, Firer::None), Out::Detonated(at));
    // 150 above: over the band (run b).
    assert!(matches!(band(150, 1, Firer::None), Out::Flying(_)));
    // A building allied with the house of a projectile without a firer stops it too (run c).
    assert_eq!(band(140, 0, Firer::None), Out::Detonated(at));
}

// spec: S-0306 T1 row 1 and Edge cases (quirks.owned_shell_ground_pass on: the building band does not detonate a projectile that has a firer; scenario P32 run g)
#[test]
fn t1_row1_building_band_owned_flies_over() {
    for house in [0, 1] {
        assert!(
            matches!(band(140, house, Firer::Owned), Out::Flying(_)),
            "house {house}"
        );
    }
}

// spec: S-0306 Edge cases (quirks.owned_shell_ground_pass off: the band detonates a projectile with a firer as one with no firer, at the terrain height, except at the firer itself and at a building allied with the firer's house)
#[test]
fn quirk_owned_shell_ground_pass_off_building_band() {
    let off = |r: &mut Ruleset| r.globals.quirks.owned_shell_ground_pass = false;
    assert_eq!(
        band_with(140, 1, Firer::Owned, off, |_, _| {}),
        Out::Detonated(c(4, 0, 0))
    );
    assert!(matches!(
        band_with(150, 1, Firer::Owned, off, |_, _| {}),
        Out::Flying(_)
    ));
    assert!(matches!(
        band_with(140, 0, Firer::Owned, off, |_, _| {}),
        Out::Flying(_)
    ));
}

// spec: S-0306 T1 row 1 (any house's building stops a projectile with no firer, an open gate included; scenario P32 run f)
#[test]
fn p32f_open_gate_stops_a_projectile_with_no_firer() {
    let mut seen = None;
    let out = band_with(
        140,
        1,
        Firer::None,
        |_| {},
        |w, b| {
            let o = w.obj_mut(b).unwrap();
            o.beh.current = Some(glasnost_sim::rules::Behaviour::Open);
            o.beh.gate.door = glasnost_sim::behaviour::Door::Open;
            seen = Some(glasnost_sim::process::gate_open(w, b));
        },
    );
    assert_eq!(seen, Some(true), "the gate is open");
    assert_eq!(out, Out::Detonated(c(4, 0, 0)));
}

// spec: S-0306 T1 row 1 and S-0211 Terms (an inactive laser-fence section is not an eligible building, and with a building present the wall overlays do not count; section states are not modelled, so every section is inactive; scenario P32 run i with its ordinary-building control)
#[test]
fn p32i_laser_fence_section_does_not_stop() {
    let fence = |r: &mut Ruleset| {
        r.sim.buildings.get_mut("TOWER").unwrap().laser_fence = Some(true);
    };
    assert!(matches!(
        band_with(140, 1, Firer::None, fence, |_, _| {}),
        Out::Flying(_)
    ));
    // The control: the same building without the flag detonates.
    assert_eq!(band(140, 1, Firer::None), Out::Detonated(c(4, 0, 0)));
    // A wall overlay in the fence's cell: a building is present, so the wall does not count.
    let fence_and_wall = |r: &mut Ruleset| {
        fence(r);
        overlays(r, &[2]);
    };
    let out = band_with(140, 1, Firer::None, fence_and_wall, |w, _| {
        set_overlay(w, cell_of(4, 0), 2)
    });
    assert!(matches!(out, Out::Flying(_)), "{out:?}");
}

// spec: S-0306 T1 row 1 and S-0214 Terms (a vehicle-like building, one that undeploys into a vehicle on a 1 × 1 foundation, is not an eligible building; scenario P32 run j with its ordinary-building control)
#[test]
fn p32j_vehicle_like_building_does_not_stop() {
    let like = |r: &mut Ruleset| {
        r.sim.buildings.get_mut("TOWER").unwrap().undeploys_into = Some("BUGGY".into());
    };
    assert!(matches!(
        band_with(140, 1, Firer::None, like, |_, _| {}),
        Out::Flying(_)
    ));
    assert_eq!(band(140, 1, Firer::None), Out::Detonated(c(4, 0, 0)));
}

// spec: S-0306 T1 row 1 (the firer's own building does not stop it: with a firer only the deck crossings apply, and with quirks.owned_shell_ground_pass off the firer itself is excepted; scenario P32 g)
#[test]
fn t1_row1_not_the_firer() {
    for quirk in [true, false] {
        let mut r = base();
        r.globals.quirks.owned_shell_ground_pass = quirk;
        let mut s = setup(&r, 6, 6);
        let tw = testkit::spawn(&mut s.w, Kind::Building, "TOWER", 1, cell_of(4, 0), 0);
        let p = c(3, 0, 140);
        let f = c(4, 0, 300);
        let id = launch(&mut s.w, tw, s.wp, far(), f, 50).unwrap();
        {
            let x = s.w.projectiles.get_mut(&id).unwrap();
            x.pos = p;
            x.vel = [n(256), n(0), n(10)];
            x.prev_cell = p.cell();
            x.omega = 1_000_000;
        }
        assert!(matches!(step(&mut s, id), Out::Flying(_)), "quirk {quirk}");
    }
}

// spec: S-0306 T1 row 1 and S-0307 claim 11 (0 to 149 above the terrain in a cell with no building and one of the wall overlays 2, 26 and 243: a projectile with no firer detonates at the terrain height; one with a firer only with quirks.owned_shell_ground_pass off; scenario P32 runs d and e)
#[test]
fn t1_row1_wall_overlays() {
    for (quirk, firer) in [
        (true, Firer::None),
        (true, Firer::Owned),
        (false, Firer::Owned),
    ] {
        for (o, walls) in [(2u8, true), (26, true), (243, true), (5, false)] {
            let mut r = base();
            r.globals.quirks.owned_shell_ground_pass = quirk;
            overlays(&mut r, &[2, 26, 243, 5]);
            let mut s = setup(&r, 6, 6);
            set_overlay(&mut s.w, cell_of(4, 0), o);
            let p = c(3, 0, 100);
            let id = place(&mut s, far(), p, [256, 0, 10]);
            if firer == Firer::None {
                no_firer(&mut s, id);
            }
            let stops = walls && (firer == Firer::None || !quirk);
            let out = step(&mut s, id);
            let what = format!("overlay {o}, {firer:?}, quirk {quirk}");
            if stops {
                assert_eq!(out, Out::Detonated(c(4, 0, 0)), "{what}");
            } else {
                assert!(matches!(out, Out::Flying(_)), "{what}");
            }
        }
    }
}

// spec: S-0306 T1 (reached: the cell of P′ is the cell of K and h < 208; claim 20 b snaps to the target's reference point)
#[test]
fn t1_reached_the_cell_of_k() {
    let mut s = setup(&base(), 6, 6);
    let k = cell_of(4, 0);
    let p = c(3, 0, 150);
    let id = place(&mut s, Target::Cell(k), p, [256, 0, 10]);
    assert_eq!(step(&mut s, id), Out::Detonated(c(4, 0, 0)));
    // h = 208 at the start of the turn: not reached.
    let id = place(&mut s, Target::Cell(k), c(3, 0, 208), [256, 0, 10]);
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

// spec: S-0306 T1 (reached: the cells of P′ and K hold the same building and h < 208; scenario P32 h)
#[test]
fn t1_reached_the_same_building() {
    let mut r = base();
    r.sim.buildings.insert(
        "BIG".into(),
        BuildingType {
            strength: Some(900),
            foundation: Some([2, 2]),
            ..Default::default()
        },
    );
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("BIG".into());
    let mut s = setup(&r, 6, 6);
    let big = testkit::spawn(&mut s.w, Kind::Building, "BIG", 1, cell_of(4, 0), 0);
    let rp = s.w.reference_point(big);
    assert_ne!(rp.cell(), cell_of(4, 0));
    s.w.obj_mut(s.a).unwrap().target = Some(Target::Object(big));
    // P′ in the top-left cell of the building, 180 above the terrain (over the band of row 1).
    let p = Pos::new(rp.x - 256 - 100, rp.y - 100, 180);
    let id = place(&mut s, Target::Object(big), p, [100, 0, 10]);
    assert_eq!(step(&mut s, id), Out::Detonated(rp));
}

// spec: S-0306 T1 and T2 (the proximity row: the nearest unit in the cell of P′ at d₃ < 128, not allied with the firer; the detonation is at its position)
#[test]
fn t1_proximity_row() {
    let mut s = setup(&base(), 6, 6);
    let v = testkit::spawn(&mut s.w, Kind::Vehicle, "BUGGY", 1, cell_of(4, 0), 0);
    let vp = pos(&s, v);
    let p = Pos::new(vp.x - 100, vp.y + 60, 100);
    let id = place(&mut s, far(), p, [50, 0, 10]);
    assert_eq!(step(&mut s, id), Out::Detonated(vp));
    // At 128 or more: no proximity.
    let id = place(
        &mut s,
        far(),
        Pos::new(vp.x - 100, vp.y + 60, 128),
        [50, 0, 10],
    );
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

// spec: S-0306 T1 and T2 (an allied unit does not count while the firer lives; with no firer any object counts; scenario P30)
#[test]
fn t1_proximity_allied_and_no_firer() {
    let mut s = setup(&base(), 6, 6);
    let v = testkit::spawn(&mut s.w, Kind::Vehicle, "BUGGY", 0, cell_of(4, 0), 0);
    let vp = pos(&s, v);
    let p = Pos::new(vp.x - 100, vp.y + 60, 100);
    let id = place(&mut s, far(), p, [50, 0, 10]);
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
    let id = place(&mut s, far(), p, [50, 0, 10]);
    s.w.projectiles.get_mut(&id).unwrap().firer = None;
    assert_eq!(step(&mut s, id), Out::Detonated(vp));
}

// spec: S-0306 T2 (the proximity row of an inaccurate projectile: the detonation is at P′)
#[test]
fn t2_proximity_inaccurate_at_p_prime() {
    let mut r = base();
    ball(&mut r).inaccurate = Some(true);
    let mut s = setup(&r, 6, 6);
    let v = testkit::spawn(&mut s.w, Kind::Vehicle, "BUGGY", 1, cell_of(4, 0), 0);
    let vp = pos(&s, v);
    let p = Pos::new(vp.x - 100, vp.y + 60, 100);
    let id = place(&mut s, far(), p, [50, 0, 10]);
    assert_eq!(
        step(&mut s, id),
        Out::Detonated(Pos::new(p.x + 50, p.y, 100))
    );
}

// spec: S-0306 T1 (the speed row: |V| < 10 after this turn's change and h < 10; scenario P36)
#[test]
fn t1_speed_row() {
    let mut r = base();
    r.sim.general.gravity = Some(0);
    let mut s = setup(&r, 6, 6);
    let p = c(-3, 2, 5);
    let id = place(&mut s, far(), p, [5, 0, 0]);
    assert_eq!(step(&mut s, id), Out::Detonated(Pos::new(p.x + 5, p.y, 5)));
    // Run b: 60 above the surface.
    let id = place(&mut s, far(), c(-3, 2, 60), [5, 0, 0]);
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
    // Run c: speed 25.
    let id = place(&mut s, far(), c(-3, 2, 5), [25, 0, 0]);
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

/// A steering projectile (ρ = 5, top speed 64, unlocked) at `p` with
/// velocity `v`, aimed at the buggy.
fn missile(s: &mut S, p: Pos, v: [i64; 3]) -> ObjectId {
    steer_place(s, p, v)
}

fn steering_s(dx: i32) -> S {
    let mut r = steering(5, 1, 3);
    r.sim.general.missile_rot_var = Some(d("0"));
    setup(&r, dx, 0)
}

// spec: S-0306 T1 and T2 (the steering-distance row: half the speed at least the steering distance; with h > 0 and a target, the detonation is at the target point)
#[test]
fn t1_steering_distance_row() {
    let mut s = steering_s(6);
    let tp = pos(&s, s.t);
    let id = missile(&mut s, Pos::new(tp.x - 130, tp.y, 40), [64, 0, 0]);
    // P′ is 66 leptons from the target point, with ζ = trunc(−40 / 4): r(66² + 10²) = 66 > 32.
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
    let id = missile(&mut s, Pos::new(tp.x - 90, tp.y, 40), [64, 0, 0]);
    // P′ is 26 away: 32 ≥ ⌊r(26² + 10²)⌋ = 27.
    assert_eq!(step(&mut s, id), Out::Detonated(tp));
}

// spec: S-0306 Terms (steering distance: the vertical difference counts at a quarter, ζ = trunc(Δz / 4))
#[test]
fn terms_steering_distance_vertical_quarter() {
    let mut s = steering_s(6);
    let tp = pos(&s, s.t);
    // P′ right above the target point, 120 up: ζ = 30 ≤ 32, detonation.
    let id = missile(&mut s, Pos::new(tp.x - 64, tp.y, 120), [64, 0, 0]);
    assert_eq!(step(&mut s, id), Out::Detonated(tp));
    // 140 up: ζ = 35 > 32.
    let id = missile(&mut s, Pos::new(tp.x - 64, tp.y, 140), [64, 0, 0]);
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

// spec: S-0306 T1 and T2 (steering: h ≤ 0 detonates, at P′ after the cruise lift of claim 15, here 18 leptons)
#[test]
fn t1_steering_ground_row() {
    let mut s = steering_s(10);
    let p = c(-5, 3, 0);
    let id = missile(&mut s, p, [64, 0, 0]);
    assert_eq!(
        step(&mut s, id),
        Out::Detonated(Pos::new(p.x + 64, p.y, 18))
    );
}

// spec: S-0306 T1 (steering: no target and h at least the missile safety altitude)
#[test]
fn t1_steering_safety_altitude_row() {
    let mut s = steering_s(10);
    let p = c(-5, 3, 500);
    let id = missile(&mut s, p, [64, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().target = None;
    assert!(matches!(step(&mut s, id), Out::Detonated(_)));
    // With a target it flies on.
    let id = missile(&mut s, p, [64, 0, 0]);
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

/// A steering projectile 60 unlocked turns into its flight, with closing
/// average `c60`, moving at right angles to its target (a closing step near
/// 0) 10 cells away, at its cruising altitude (no cruise lift, which the
/// closing step would see).
fn orbiting(mods: impl Fn(&mut Ruleset), c60: i64) -> Out {
    let mut r = steering(5, 1, 3);
    r.sim.general.missile_rot_var = Some(d("0"));
    mods(&mut r);
    let mut s = setup(&r, 10, 0);
    let tp = pos(&s, s.t);
    let id = missile(&mut s, Pos::new(tp.x - 2560, tp.y - 32, 520), [0, 64, 0]);
    {
        let x = s.w.projectiles.get_mut(&id).unwrap();
        x.unlocked = 60;
        x.closing = n(c60);
    }
    step(&mut s, id)
}

// spec: S-0306 T1 (closing average: after the first 60 unlocked turns, C in 0 … 59 detonates an orbiting missile; C_k = RZ₅₃(RZ₅₃(C_{k−1} × f₆₀) + c_k))
#[test]
fn t1_closing_average_row() {
    let f60 = F64::from_parts(false, 8857079267161975, -53, RZ);
    assert!(f60.lt(nd("0.98334")) && f60.gt(nd("0.98333")));
    // C_60 = 0, c_61 near 0: C_61 in [0, 60).
    assert!(matches!(orbiting(|_| {}, 0), Out::Detonated(_)));
    // C_60 = 100: C_61 = 98.33 + c_61 ≥ 60.
    assert!(matches!(orbiting(|_| {}, 100), Out::Flying(_)));
}

// spec: S-0306 T1 (the closing-average row does not apply to airburst or very high projectiles)
#[test]
fn t1_closing_average_not_for_airburst_or_very_high() {
    assert!(matches!(
        orbiting(|r| ball(r).very_high = Some(true), 0),
        Out::Flying(_)
    ));
    assert!(matches!(
        orbiting(|r| ball(r).airburst = Some(true), 0),
        Out::Flying(_)
    ));
}

// spec: S-0306 T1 and T2 (steering: strictly crossing a bridge deck height detonates with z at the deck height, going up or down)
#[test]
fn t1_steering_deck_crossing() {
    for (z0, vz) in [(400i64, 40i64), (440, -40)] {
        let mut s = steering_s(10);
        let p = c(-5, 3, z0 as i32);
        set_deck(&mut s.w, p.cell());
        set_deck(&mut s.w, cell_of(-4, 3));
        let id = missile(&mut s, p, [64, 0, vz]);
        s.w.projectiles.get_mut(&id).unwrap().target = None;
        s.w.projectiles.get_mut(&id).unwrap().locked = true;
        let out = step(&mut s, id);
        let Out::Detonated(q) = out else {
            panic!("no detonation: {out:?}")
        };
        assert_eq!(q.z, 416, "start {z0}, V_z {vz}");
    }
}

// spec: S-0306 T1 (steering: a deck height reached exactly is not a strict crossing)
#[test]
fn t1_steering_deck_crossing_is_strict() {
    let mut s = steering_s(10);
    let p = c(-5, 3, 416);
    set_deck(&mut s.w, p.cell());
    let id = missile(&mut s, p, [64, 0, -40]);
    s.w.projectiles.get_mut(&id).unwrap().locked = true;
    // From exactly the deck height downward: P is not strictly above it, no crossing.
    let out = step(&mut s, id);
    assert!(matches!(out, Out::Flying(_)), "{out:?}");
}

// spec: S-0306 T1 (arrival: past its arm time and η = trunc(d₃(P′, K) / 2) < 32; claim 20 b snaps to the target's reference point; scenario P22 a)
#[test]
fn t1_arrival_row() {
    let mut s = steering_s(6);
    let tp = pos(&s, s.t);
    // Speed 20, P′ 50 from K: η = 25; half the speed (10) is below the steering distance.
    let id = missile(&mut s, Pos::new(tp.x - 70, tp.y, 0), [20, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().pos.z = 1;
    let out = step(&mut s, id);
    assert_eq!(out, Out::Detonated(tp));
}

// spec: S-0306 claim 9 and T1 (before its arm time no arrival or passed-by check: scenario P22 b)
#[test]
fn t1_arrival_waits_for_the_arm_time() {
    let mut r = steering(5, 1, 3);
    ball(&mut r).arm = Some(1000);
    let mut s = setup(&r, 6, 0);
    let tp = pos(&s, s.t);
    let id = missile(&mut s, Pos::new(tp.x - 70, tp.y, 1), [20, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().omega = 77;
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
    // Ω keeps its value before the arm time.
    assert_eq!(pr(&s, id).omega, 77);
}

// spec: S-0306 T1 and claim 20 b (passed by: η < 256 and larger than Ω; the detonation stays at the projectile's own position, it is not an arrival; scenario P29 a)
#[test]
fn t1_passed_by_row_does_not_snap() {
    let mut s = steering_s(6);
    let tp = pos(&s, s.t);
    // Moving away from K (and the target), P′ 240 from it: η = 120 > Ω = 100.
    let p = Pos::new(tp.x + 176, tp.y, 300);
    let id = missile(&mut s, p, [64, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().omega = 100;
    assert_eq!(
        step(&mut s, id),
        Out::Detonated(Pos::new(p.x + 64, p.y, 300))
    );
}

// spec: S-0306 T1 (passed by counts as arrival when the firer's type is a jumpjet type: claim 20 b snaps; scenario P29 b)
#[test]
fn t1_passed_by_counts_as_arrival_for_a_jumpjet_firer() {
    let mut r = steering(5, 1, 3);
    r.sim.general.missile_rot_var = Some(d("0"));
    r.sim.vehicles.get_mut("CART").unwrap().jumpjet = Some(true);
    let mut s = setup(&r, 6, 0);
    let tp = pos(&s, s.t);
    let p = Pos::new(tp.x + 176, tp.y, 300);
    let id = missile(&mut s, p, [64, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().omega = 100;
    assert_eq!(step(&mut s, id), Out::Detonated(tp));
}

// spec: S-0306 T1 (each check past the arm time that detonates by neither arrival row replaces Ω by η, measured at P′ after the cruise lift of claim 15)
#[test]
fn t1_omega_is_replaced() {
    let mut s = steering_s(10);
    let tp = pos(&s, s.t);
    let p = Pos::new(tp.x - 1500, tp.y, 300);
    let id = missile(&mut s, p, [64, 0, 0]);
    step(&mut s, id);
    let pp = Pos::new(p.x + 64, p.y, p.z + 18);
    let k = pr(&s, id).stored;
    assert_eq!(pr(&s, id).omega, d3(&s.w, pp, k) / 2);
}

// spec: S-0306 T1 (Ω starts as the full, unhalved d₃ from the placement position to K; placement makes no check)
#[test]
fn t1_omega_at_placement() {
    let mut s = steering_s(6);
    let f = fp(&s, 300);
    let t = Target::Object(s.t);
    let id = fire(&mut s, t, f).unwrap();
    assert_eq!(pr(&s, id).omega, d3(&s.w, f, pos(&s, s.t)));
}

// spec: S-0306 Terms and T1 (a ranged projectile that is not steering is checked for arrival like a steering one; scenario P29 c)
#[test]
fn t1_ranged_straight_projectile_arrival() {
    let mut r = base();
    ball(&mut r).ranged = Some(true);
    r.sim.general.gravity = Some(0);
    let mut s = setup(&r, 6, 6);
    let k = c(3, -2, 400);
    let id = place(
        &mut s,
        Target::Cell(cell_of(3, -2)),
        Pos::new(k.x - 100, k.y, 400),
        [60, 0, 0],
    );
    s.w.projectiles.get_mut(&id).unwrap().stored = k;
    // η = 20 < 32: arrival. Without the ranged flag it flies on.
    assert!(matches!(step(&mut s, id), Out::Detonated(_)));
    let mut r2 = r.clone();
    ball(&mut r2).ranged = Some(false);
    let mut s = setup(&r2, 6, 6);
    let id = place(
        &mut s,
        Target::Cell(cell_of(3, -2)),
        Pos::new(k.x - 100, k.y, 400),
        [60, 0, 0],
    );
    s.w.projectiles.get_mut(&id).unwrap().stored = k;
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

// spec: S-0306 claim 22 and T1 (the line-of-fire test in flight for the single sample P′: a cliff blocks; scenario P25)
#[test]
fn t1_line_of_fire_in_flight() {
    for cliffs in [true, false] {
        let mut r = base();
        ball(&mut r).subject_to_cliffs = Some(cliffs);
        let mut s = setup(&r, 6, 6);
        set_level(&mut s.w, cell_of(3, 0), 4);
        let p = c(2, 0, 800);
        let id = place(&mut s, far(), p, [256, 0, 10]);
        let out = step(&mut s, id);
        assert_eq!(matches!(out, Out::Detonated(_)), cliffs, "cliffs {cliffs}");
    }
}

// spec: S-0306 T1 (deep fall: h′ ≤ −416 detonates, at the surface by claim 20 a; scenario P24)
#[test]
fn t1_deep_fall_row() {
    let mut s = steering_s(10);
    set_level(&mut s.w, cell_of(-4, 3), 6);
    let p = c(-5, 3, 50);
    let p = Pos::new(p.x + 100, p.y, p.z);
    let id = missile(&mut s, p, [64, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().locked = true;
    assert_eq!(
        step(&mut s, id),
        Out::Detonated(Pos::new(p.x + 64, p.y, 624))
    );
    // A plateau of 2 levels: h′ = −158, no deep fall; the h ≤ 0 row acts in the next turn.
    let mut s = steering_s(10);
    set_level(&mut s.w, cell_of(-4, 3), 2);
    let id = missile(&mut s, p, [64, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().locked = true;
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

// spec: S-0306 T1 (flak: P′ below the target's reference height and h′ < 0; scenario P34)
#[test]
fn t1_flak_row() {
    for flak in [true, false] {
        let mut r = steering(5, 1, 3);
        ball(&mut r).flak_scatter = Some(flak);
        let mut s = setup(&r, 10, 0);
        set_level(&mut s.w, cell_of(10, 0), 6);
        ground(&mut s.w, s.t);
        set_level(&mut s.w, cell_of(-4, 3), 2);
        let id = missile(&mut s, c(-5, 3, 50), [256, 0, 0]);
        s.w.projectiles.get_mut(&id).unwrap().locked = true;
        let out = step(&mut s, id);
        assert_eq!(matches!(out, Out::Detonated(_)), flak, "flak {flak}");
    }
}

// spec: S-0306 T1 (level: the cell of P′ is not one of the 14 water tiles starting at the theater's water role)
#[test]
fn t1_level_row() {
    for (first, detonates) in [(0, false), (1, true)] {
        let mut r = base();
        ball(&mut r).level = Some(true);
        let mut s = setup(&r, 6, 6);
        water_role(&mut s.w, first);
        let id = place(&mut s, far(), c(-3, 2, 3000), [20, 0, 0]);
        let out = step(&mut s, id);
        assert_eq!(
            matches!(out, Out::Detonated(_)),
            detonates,
            "water role {first}"
        );
    }
}

// spec: S-0306 T1 (anti-air with an airborne target: d₃ from P′ to its reference point below 128; claim 20 c moves the point onto the target; scenario P23)
#[test]
fn t1_anti_air_row() {
    let mut r = base();
    ball(&mut r).aa = Some(true);
    let mut s = setup(&r, 6, 0);
    let tp = pos(&s, s.t);
    let air = Pos::new(tp.x, tp.y, 400);
    put(&mut s.w, s.t, air);
    // P′ 120 from the hovering target, in another cell (no proximity row).
    let id = {
        let tt = Target::Object(s.t);
        place(&mut s, tt, Pos::new(air.x - 170, air.y, 400), [50, 0, 10])
    };
    assert_eq!(step(&mut s, id), Out::Detonated(air));
    // On the ground the row does not apply.
    let mut s = setup(&r, 6, 0);
    let tp = pos(&s, s.t);
    let id = {
        let tt = Target::Object(s.t);
        place(&mut s, tt, Pos::new(tp.x - 170, tp.y, 300), [50, 0, 10])
    };
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

// spec: S-0306 T1 and T2 (not steering: P′ outside the legal map, and no reached or proximity row: removed without a detonation)
#[test]
fn t2_outside_the_legal_map_removed() {
    let mut s = setup(&base(), 6, 6);
    let id = place(&mut s, far(), Pos::new(-2000, -2000, 3000), [10, 0, 0]);
    assert_eq!(step(&mut s, id), Out::Removed);
}

// spec: S-0306 T2 (a reached row wins over leaving the map: here the cell of K is outside the legal map)
#[test]
fn t2_reached_wins_over_leaving_the_map() {
    let mut s = setup(&base(), 6, 6);
    let k = CellXY::new(-8, -8);
    let p = Pos::new(-8 * 256 + 28, -8 * 256 + 128, 100);
    let id = place(&mut s, Target::Cell(k), p, [100, 0, 10]);
    assert!(matches!(step(&mut s, id), Out::Detonated(_)));
}

// spec: S-0306 claim 19 and T2 (no row of this spec removes a steering projectile for leaving the map)
#[test]
fn t2_steering_not_removed_outside_the_map() {
    let mut s = steering_s(6);
    let id = missile(&mut s, Pos::new(-3000, -3000, 3000), [64, 0, 0]);
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

// spec: S-0306 claim 20 c (no electromagnetic flag and D_S < 42: the target's target point, with the aim shift)
#[test]
fn claim20c_target_point_and_the_electromagnetic_flag() {
    for em in [false, true] {
        let mut r = base();
        r.sim
            .buildings
            .get_mut("TOWER")
            .unwrap()
            .target_coord_offset = Some([0, 0, 90]);
        r.sim.warheads.get_mut("Dud").unwrap().em_effect = Some(em);
        let mut s = setup(&r, 6, 6);
        let tw = testkit::spawn(&mut s.w, Kind::Building, "TOWER", 1, cell_of(4, 0), 0);
        let rp = s.w.reference_point(tw);
        // Below the terrain 20 leptons beside the reference point, in the next cell: row 1, raised to 0.
        let p = Pos::new(rp.x - 128 - 40, rp.y, 10);
        let id = place(&mut s, Target::Object(tw), p, [148, 0, -40]);
        let out = step(&mut s, id);
        let want = if em {
            rp
        } else {
            Pos::new(rp.x, rp.y, rp.z + 90)
        };
        assert_eq!(out, Out::Detonated(want), "electromagnetic {em}");
    }
}

// spec: S-0306 claim 20 c (with the electromagnetic flag and D_R ≥ 32 the point is unchanged; D_S is D_R less 64 per cell of width + height)
#[test]
fn claim20c_electromagnetic_far_point_unchanged() {
    let mut r = base();
    r.sim
        .buildings
        .get_mut("TOWER")
        .unwrap()
        .target_coord_offset = Some([0, 0, 90]);
    r.sim.warheads.get_mut("Dud").unwrap().em_effect = Some(true);
    let mut s = setup(&r, 6, 6);
    let tw = testkit::spawn(&mut s.w, Kind::Building, "TOWER", 1, cell_of(4, 0), 0);
    let rp = s.w.reference_point(tw);
    // 140 from the reference point (D_S = 140 − 128 = 12 < 42), in the cell before.
    let p = Pos::new(rp.x - 128 - 60, rp.y, 10);
    let id = place(&mut s, Target::Object(tw), p, [48, 0, -40]);
    // With no firer, so that T1's first row detonates it below the terrain.
    no_firer(&mut s, id);
    assert_eq!(
        step(&mut s, id),
        Out::Detonated(Pos::new(rp.x - 140, rp.y, 0))
    );
    // Without the flag the same detonation lands on the target point.
    let mut r2 = r.clone();
    r2.sim.warheads.get_mut("Dud").unwrap().em_effect = Some(false);
    let mut s = setup(&r2, 6, 6);
    let tw = testkit::spawn(&mut s.w, Kind::Building, "TOWER", 1, cell_of(4, 0), 0);
    let id = place(&mut s, Target::Object(tw), p, [48, 0, -40]);
    no_firer(&mut s, id);
    assert_eq!(
        step(&mut s, id),
        Out::Detonated(Pos::new(rp.x, rp.y, rp.z + 90))
    );
}

// spec: S-0306 claim 20 c (a vehicle target: D_S = D_R; at 42 or more and D_R ≥ 32 the point is unchanged)
#[test]
fn claim20c_vehicle_target_thresholds() {
    for (dist, snaps) in [(41, true), (42, false)] {
        let mut s = setup(&base(), 6, 0);
        // A target of the firer's own house, so that the proximity row does not apply.
        let own = testkit::spawn(&mut s.w, Kind::Vehicle, "BUGGY", 0, cell_of(6, 3), 0);
        let tp = pos(&s, own);
        // Deep below the surface: the deep-fall row detonates it (h′ ≤ −416), raised to the
        // surface by claim 20 a (a projectile with a firer is not detonated by the terrain).
        let p = Pos::new(tp.x - dist - 50, tp.y + 200, -380);
        let id = place(&mut s, Target::Object(own), p, [50, -200, -40]);
        // K elsewhere, so that no reached row (and its snap, claim 20 b) applies.
        s.w.projectiles.get_mut(&id).unwrap().stored = c(12, 9, 0);
        let want = if snaps {
            tp
        } else {
            Pos::new(tp.x - dist, tp.y, 0)
        };
        assert_eq!(step(&mut s, id), Out::Detonated(want), "distance {dist}");
    }
}

// spec: S-0306 claim 20 b and c (inaccurate projectiles keep their own position)
#[test]
fn claim20_inaccurate_keeps_its_position() {
    let mut r = base();
    ball(&mut r).inaccurate = Some(true);
    let mut s = setup(&r, 6, 0);
    let tp = pos(&s, s.t);
    let p = Pos::new(tp.x - 30, tp.y + 200, 10);
    let id = {
        let tt = Target::Object(s.t);
        place(&mut s, tt, p, [10, -200, -40])
    };
    assert_eq!(
        step(&mut s, id),
        Out::Detonated(Pos::new(tp.x - 20, tp.y, 0))
    );
}

// spec: S-0306 claim 21 and Edge cases (n_c detonations; after every one a draw R(256, 512) and one raw value for the nearby point N(Q, λ, snap off); cluster 0: none; scenario P14 variant d)
#[test]
fn claim21_cluster_detonations_and_draws() {
    for cluster in [0, 1, 2, 3] {
        let mut r = base();
        ball(&mut r).cluster = Some(cluster);
        let mut s = setup(&r, 18, 18);
        // Deep below the surface: the deep-fall row detonates it in its next turn (h′ ≤ −416), at
        // the surface (claim 20 a); a projectile with a firer is not detonated by the terrain.
        let p = c(-12, -12, -380);
        let id = place(&mut s, far(), p, [50, 0, -40]);
        let mut cl = s.w.rng.clone();
        let h = half(&s.w);
        let mut q = Pos::new(p.x + 50, p.y, 0);
        let mut want = Vec::new();
        for _ in 0..cluster {
            want.push(q);
            let lambda = cl.ranged(256, 512, h);
            let b = cl.raw();
            q = glasnost_sim::combat::death::nearby_point(&s.w, q, lambda, b, false);
        }
        s.w.effects.clear();
        turn(&mut s.w, id);
        let dets: Vec<Pos> =
            s.w.effects
                .iter()
                .filter_map(|e| match e {
                    glasnost_sim::combat::effects::Effect::Detonation { pos, .. } => Some(*pos),
                    _ => None,
                })
                .collect();
        assert_eq!(dets, want, "cluster {cluster}");
        assert_eq!(s.w.rng, cl, "cluster {cluster}: the draws");
        assert!(!s.w.projectiles.contains_key(&id));
        // The chain: each point 256 to 512 from the one before (horizontally, ±1).
        for w2 in dets.windows(2) {
            let dd = d2(&s.w, w2[0], w2[1]);
            assert!((255..=513).contains(&dd), "cluster {cluster}: step {dd}");
        }
    }
}

// spec: S-0306 claim 8 (revision 45: the projectiles a burst releases are raised as every placed projectile is: a child whose position is not above the terrain of its own x and y starts 1 lepton above it; the walk's X₁ keeps the unraised point; `unmeasured` in the spec)
#[test]
fn claim8_airburst_children_start_above_their_cells_terrain() {
    let mut r = base();
    ball(&mut r).airburst = Some(true);
    ball(&mut r).airburst_weapon = Some("Burst".into());
    r.sim.weapons.insert(
        "Burst".into(),
        WeaponType {
            damage: Some(9),
            range: Some(d("6")),
            speed: Some(77),
            projectile: Some("Pellet".into()),
            warhead: Some("Dud".into()),
            ..Default::default()
        },
    );
    r.sim
        .projectiles
        .insert("Pellet".into(), ProjectileType::default());
    let mut s = setup(&r, 18, 18);
    // Deep below the surface: the deep-fall row detonates it in its next turn (h′ ≤ −416), at
    // the surface (claim 20 a); a projectile with a firer is not detonated by the terrain.
    let p = c(-12, -12, -380);
    // The cell east of the detonation cell is raised two levels.
    let det = Pos::new(p.x + 50, p.y, 0).cell();
    let high = CellXY::new(det.x + 1, det.y);
    set_level(&mut s.w, high, 2);
    let id = place(&mut s, far(), p, [50, 0, -40]);
    let before: Vec<ObjectId> = s.w.projectiles.keys().copied().collect();
    assert!(matches!(step(&mut s, id), Out::Detonated(_)));
    let kids: Vec<Projectile> =
        s.w.projectiles
            .values()
            .filter(|x| !before.contains(&x.id))
            .cloned()
            .collect();
    assert_eq!(kids.len(), 9);
    let z0 = kids[0].fire_point.z;
    let mut raised_high = false;
    for k in &kids {
        assert_eq!(k.fire_point.z, z0, "X₁ at the burst's height, unraised");
        let t = s.w.surface_at(k.fire_point.x, k.fire_point.y);
        let want = if k.fire_point.z > t {
            k.fire_point.z
        } else {
            t + 1
        };
        assert_eq!(
            k.placed_at,
            Pos::new(k.fire_point.x, k.fire_point.y, want),
            "child in {:?}",
            k.fire_point.cell()
        );
        if k.fire_point.cell() == high {
            assert_eq!(k.placed_at.z, 2 * 104 + 1);
            raised_high = true;
        }
    }
    assert!(raised_high);
}

// spec: S-0306 claim 21 (an airburst detonates once and launches nine children at the eight neighbouring cells and its own, falling at trunc(S_w / 10), one draw R(0, 32) each)
#[test]
fn claim21_airburst_children() {
    let mut r = base();
    ball(&mut r).airburst = Some(true);
    ball(&mut r).airburst_weapon = Some("Burst".into());
    r.sim.weapons.insert(
        "Burst".into(),
        WeaponType {
            damage: Some(9),
            range: Some(d("6")),
            speed: Some(77),
            projectile: Some("Pellet".into()),
            warhead: Some("Dud".into()),
            ..Default::default()
        },
    );
    r.sim
        .projectiles
        .insert("Pellet".into(), ProjectileType::default());
    let mut s = setup(&r, 18, 18);
    // Deep below the surface: the deep-fall row detonates it in its next turn (h′ ≤ −416), at
    // the surface (claim 20 a); a projectile with a firer is not detonated by the terrain.
    let p = c(-12, -12, -380);
    let id = place(&mut s, far(), p, [50, 0, -40]);
    let mut cl = s.w.rng.clone();
    let h = half(&s.w);
    for _ in 0..9 {
        cl.ranged(0, 32, h);
    }
    let before: Vec<ObjectId> = s.w.projectiles.keys().copied().collect();
    let out = step(&mut s, id);
    assert!(matches!(out, Out::Detonated(_)));
    assert_eq!(s.w.rng, cl, "nine draws R(0, 32)");
    let kids: Vec<&Projectile> =
        s.w.projectiles
            .values()
            .filter(|x| !before.contains(&x.id))
            .collect();
    assert_eq!(kids.len(), 9);
    // S_w of a ρ = 0 weapon is v(R₀, g) = v(1536, 10) = 135: they fall at 13.
    let sw = arc(1536);
    let det = Pos::new(p.x + 50, p.y, 0).cell();
    let mut cells: Vec<CellXY> = kids.iter().map(|k| k.pos.cell()).collect();
    cells.sort_by_key(|c| (c.x, c.y));
    let mut want: Vec<CellXY> = (-1..=1)
        .flat_map(|dx| (-1..=1).map(move |dy| CellXY::new(det.x + dx, det.y + dy)))
        .collect();
    want.sort_by_key(|c| (c.x, c.y));
    assert_eq!(cells, want);
    for k in kids {
        assert!(k.vel[0].eq_num(F64::ZERO) && k.vel[1].eq_num(F64::ZERO));
        assert!(k.vel[2].eq_num(n(-i64::from(sw / 10))));
        assert_eq!(k.damage, 9);
    }
}

// spec: S-0306 claim 21 (every detonation passes the damage amount at that moment, the same for every cluster detonation, and the attacker while the firer has not been killed)
#[test]
fn claim21_detonation_carries_damage_and_attacker() {
    let mut r = base();
    ball(&mut r).cluster = Some(2);
    let mut s = setup(&r, 18, 18);
    // Deep below the surface: the deep-fall row detonates it in its next turn (h′ ≤ −416), at
    // the surface (claim 20 a); a projectile with a firer is not detonated by the terrain.
    let p = c(-12, -12, -380);
    let id = place(&mut s, far(), p, [50, 0, -40]);
    s.w.projectiles.get_mut(&id).unwrap().damage = 31;
    s.w.effects.clear();
    turn(&mut s.w, id);
    let amounts: Vec<i32> =
        s.w.effects
            .iter()
            .filter_map(|e| match e {
                glasnost_sim::combat::effects::Effect::Detonation { damage, .. } => Some(*damage),
                _ => None,
            })
            .collect();
    assert_eq!(amounts, vec![31, 31]);
}

// ================================================================= line of fire, eligibility, nuclear warhead

fn lof_rules(cliffs: bool, walls: bool) -> (S, glasnost_sim::combat::data::ProjectileRules) {
    let mut r = base();
    ball(&mut r).subject_to_cliffs = Some(cliffs);
    ball(&mut r).subject_to_walls = Some(walls);
    overlays(&mut r, &[7]);
    let s = setup(&r, 6, 6);
    let p = s.w.rules().combat.projectile_of(s.wp).unwrap().clone();
    (s, p)
}

// spec: S-0306 claim 22 (n = max(|Δcₓ|, |Δc_y|) samples X₁ + i × trunc((X₂ − X₁) / n); a cliff at least 4 above the previous sample's cell and above X₁'s; scenario P15)
#[test]
fn claim22_cliff_sample() {
    let (mut s, p) = lof_rules(true, false);
    let a = c(0, 0, 0);
    let b = c(5, 0, 0);
    set_level(&mut s.w, cell_of(2, 0), 4);
    assert_eq!(
        lof::walk(&s.w, &p, Some(glasnost_sim::world::HouseId(0)), a, b),
        Some(cell_of(2, 0))
    );
    // A ridge of 3 levels does not block (scenario P15 b).
    set_level(&mut s.w, cell_of(2, 0), 3);
    assert_eq!(lof::walk(&s.w, &p, None, a, b), None);
    // A step of 4 over the previous sample but not above X₁'s cell does not block.
    set_level(&mut s.w, cell_of(0, 0), 6);
    set_level(&mut s.w, cell_of(1, 0), 2);
    set_level(&mut s.w, cell_of(2, 0), 6);
    assert_eq!(lof::walk(&s.w, &p, None, a, b), None);
}

// spec: S-0306 claim 22 (the walk samples i = 0 … n − 1 only: the end cell itself is never sampled, and one cell gives no sample)
#[test]
fn claim22_samples_and_end_cell() {
    let (mut s, p) = lof_rules(true, false);
    let a = c(0, 0, 0);
    set_level(&mut s.w, cell_of(3, 0), 8);
    // The ridge cell is X₂'s cell: n = 3 samples at cells 0, 1, 2.
    assert_eq!(lof::walk(&s.w, &p, None, a, c(3, 0, 0)), None);
    // One cell further it is sampled.
    assert_eq!(
        lof::walk(&s.w, &p, None, a, c(4, 0, 0)),
        Some(cell_of(3, 0))
    );
    // Both points in one cell: nothing samples, nothing blocks.
    assert_eq!(
        lof::walk(
            &s.w,
            &p,
            None,
            Pos::new(a.x - 100, a.y, 0),
            Pos::new(a.x + 100, a.y, 0)
        ),
        None
    );
}

// spec: S-0306 claim 22 (a wall blocks unless it is X₂'s cell or X₁'s cell is higher than X₂'s; scenario P35 c and d)
#[test]
fn claim22_wall_sample() {
    let (mut s, p) = lof_rules(false, true);
    let a = c(0, 0, 0);
    set_overlay(&mut s.w, cell_of(2, 0), 7);
    assert_eq!(
        lof::walk(&s.w, &p, None, a, c(5, 0, 0)),
        Some(cell_of(2, 0))
    );
    // The wall in the end cell: the end cell is never a block (and is not sampled).
    assert_eq!(lof::walk(&s.w, &p, None, a, c(2, 0, 0)), None);
    // X₁'s cell 2 levels above X₂'s: not blocked (c1); 2 levels below: blocked (c2).
    set_level(&mut s.w, cell_of(0, 0), 2);
    assert_eq!(lof::walk(&s.w, &p, None, a, c(5, 0, 0)), None);
    set_level(&mut s.w, cell_of(0, 0), 0);
    set_level(&mut s.w, cell_of(5, 0), 2);
    assert_eq!(
        lof::walk(&s.w, &p, None, a, c(5, 0, 0)),
        Some(cell_of(2, 0))
    );
}

// spec: S-0306 claim 22 (an allied wall does not block while allied wall transparency is on; an enemy wall blocks either way; scenario P35)
#[test]
fn claim22_allied_wall_transparency() {
    use glasnost_sim::world::HouseId;
    for transparency in [false, true] {
        let mut r = base();
        ball(&mut r).subject_to_walls = Some(true);
        overlays(&mut r, &[7]);
        r.sim.general.allied_wall_transparency = Some(transparency);
        let mut s = setup(&r, 6, 6);
        let p = s.w.rules().combat.projectile_of(s.wp).unwrap().clone();
        set_overlay(&mut s.w, cell_of(2, 0), 7);
        let i = s.w.cells.idx(cell_of(2, 0)).unwrap();
        let (a, b) = (c(0, 0, 0), c(5, 0, 0));
        // Owned by the firer's own house (allied with itself).
        s.w.cells.wall_owner[i] = Some(HouseId(0));
        let own = lof::walk(&s.w, &p, Some(HouseId(0)), a, b);
        assert_eq!(
            own.is_none(),
            transparency,
            "own wall, transparency {transparency}"
        );
        s.w.cells.wall_owner[i] = Some(HouseId(1));
        let enemy = lof::walk(&s.w, &p, Some(HouseId(0)), a, b);
        assert_eq!(
            enemy,
            Some(cell_of(2, 0)),
            "enemy wall, transparency {transparency}"
        );
    }
}

// spec: S-0306 claim 22 and S-0307 Terms (the cliff test compares elevation levels: plus 4 for a cell with the elevation-deck mark)
#[test]
fn claim22_cliff_with_a_deck_cell() {
    let (mut s, p) = lof_rules(true, false);
    // A high-bridge cell without an overlay of its own has no elevation-deck mark: no cliff.
    set_deck(&mut s.w, cell_of(2, 0));
    assert_eq!(lof::walk(&s.w, &p, None, c(0, 0, 0), c(5, 0, 0)), None);
    set_bridge_overlay(&mut s.w, cell_of(2, 0));
    assert_eq!(
        lof::walk(&s.w, &p, None, c(0, 0, 0), c(5, 0, 0)),
        Some(cell_of(2, 0))
    );
}

// spec: S-0306 claim 23 (the firing check is illegal for an airborne target and a projectile that is not anti-air)
#[test]
fn claim23_airborne_target_needs_anti_air() {
    use glasnost_sim::combat::check::{Answer, check};
    for aa in [false, true] {
        let mut r = base();
        ball(&mut r).aa = Some(aa);
        let mut s = setup(&r, 4, 0);
        let tp = pos(&s, s.t);
        put(&mut s.w, s.t, Pos::new(tp.x, tp.y, 400));
        let a = check(&s.w, s.a, Some(Target::Object(s.t)), 0, true);
        assert_eq!(a == Answer::Illegal, !aa, "anti-air {aa}: {a:?}");
    }
}

// spec: S-0306 claim 23 (a cell target that is not airborne and a projectile that is not anti-ground: illegal; the anti-ground flag is not tested for a unit target; scenario P16 a and d)
#[test]
fn claim23_anti_ground() {
    use glasnost_sim::combat::check::{Answer, check};
    for ag in [false, true] {
        let mut r = base();
        ball(&mut r).ag = Some(ag);
        let s = setup(&r, 4, 0);
        let cell = check(&s.w, s.a, Some(Target::Cell(cell_of(3, 1))), 0, true);
        assert_eq!(
            cell == Answer::Illegal,
            !ag,
            "anti-ground {ag}: cell {cell:?}"
        );
        let unit = check(&s.w, s.a, Some(Target::Object(s.t)), 0, true);
        assert_ne!(unit, Answer::Illegal, "anti-ground {ag}: unit");
    }
}

// spec: S-0306 claim 25 (a warhead registered under the nuclear warhead name, compared without regard to case: with no animation type of the fireball name it detonates at once)
#[test]
fn claim25_nuclear_warhead_without_a_fireball_type() {
    let mut r = base();
    r.sim.warheads.insert("nuke".into(), WarheadType::default());
    weapon(&mut r).warhead = Some("nuke".into());
    let mut s = setup(&r, 6, 6);
    assert!(s.w.rules().combat.warhead_of(s.wp).unwrap().nuke);
    let p = c(-3, 2, 20);
    let id = place(&mut s, far(), p, [50, 0, -30]);
    no_firer(&mut s, id);
    assert_eq!(step(&mut s, id), Out::Detonated(Pos::new(p.x + 50, p.y, 0)));
    // Another name is not the nuclear warhead.
    let s2 = setup(&base(), 6, 6);
    assert!(!s2.w.rules().combat.warhead_of(s2.wp).unwrap().nuke);
}

// ================================================================= formulas and remaining edge cases

// spec: S-0306 F3 (reachability: disc ≥ 0 and (v² − gz + r(disc)) / ω ≥ 0; d = 0 counts as 0.001)
#[test]
fn f3_reachability() {
    let g = ballistics::gravity(G, false, RZ);
    let v = arc(1280);
    assert!(ballistics::reachable(v, 1280, 0, g, true, RZ));
    assert!(ballistics::reachable(v, 0, 0, g, true, RZ));
    assert!(ballistics::reachable(v, 0, -500, g, true, RZ));
    // Straight up far above: disc < 0.
    assert!(!ballistics::reachable(v, 0, 5000, g, true, RZ));
    assert!(!ballistics::reachable(v, 1200, 4000, g, true, RZ));
}

// spec: S-0306 F8 and Terms (acos_t(x) = π/2 − A_s[trunc(2048 (x + 1))]; a value that is not a number gives index 0)
#[test]
fn f8_table_arccosine() {
    let (c1, _) = direction_constants();
    // x = 0: index 2048, A_s = asin(0) = 0.
    assert_eq!(acos_t(F64::ZERO, RZ), c1);
    // x = 1: index 4096, asin(1) truncated to single precision.
    let one = acos_t(n(1), RZ);
    assert!(one.abs().lt(nd("0.0000001")));
    // x = −1 and not a number: index 0, θ = π/2 − A_s[0] = 3.1415925780918936.
    let nan = F64::ZERO.div(F64::ZERO, RZ);
    assert_eq!(acos_t(n(-1), RZ), acos_t(nan, RZ));
    assert_eq!(acos_t(nan, RZ), nd("3.1415925780918936"));
}

// spec: S-0306 F4 and Edge cases (zero planar distance, v = 0, the flatter shot: the failed solve for d + 1 launches at θ like the lob, a departure)
#[test]
fn f4_zero_distance_zero_speed_flatter_shot() {
    let v = zero_distance(&arcing_zero(0, 50, false), 300);
    let east = i32::from(direction(0, 0, true));
    assert_eq!(v, f1(east, -16383, 0));
}

// spec: S-0306 F9 (ν′ = trunc(ν / 3) with ζ = trunc((P_R.z + P.z) / 2) − P_R.z; a reached row snaps only when ν′ ≤ max(2|V|, 128))
#[test]
fn f9_reached_snap_distance() {
    let mut r = base();
    r.sim.buildings.insert(
        "HUGE".into(),
        BuildingType {
            strength: Some(900),
            foundation: Some([3, 3]),
            ..Default::default()
        },
    );
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("HUGE".into());
    r.sim.general.gravity = Some(0);
    for (corner, snaps) in [(false, true), (true, false)] {
        let mut s = setup(&r, 9, 9);
        let b = testkit::spawn(&mut s.w, Kind::Building, "HUGE", 1, cell_of(4, 0), 0);
        let rp = s.w.reference_point(b);
        // P′ near the reference point (ν′ small) or in the far corner cell of the building.
        let pp = if corner {
            Pos::new(rp.x - 370, rp.y - 370, 100)
        } else {
            Pos::new(rp.x - 30, rp.y - 30, 100)
        };
        let p = Pos::new(pp.x - 40, pp.y, 100);
        let id = place(&mut s, Target::Object(b), p, [40, 0, 0]);
        let zeta = (rp.z + pp.z) / 2 - rp.z;
        let (dx, dy) = (i64::from(pp.x - rp.x), i64::from(pp.y - rp.y));
        let nu = root_floor(
            (dx * dx + dy * dy + i64::from(zeta) * i64::from(zeta)) as u64,
            true,
            RZ,
        ) as i32;
        assert_eq!(nu / 3 <= 128, snaps);
        let out = step(&mut s, id);
        if snaps {
            assert_eq!(out, Out::Detonated(rp));
        } else {
            // Claim 20 c: D_S = D_R − 64 × (3 + 3) ≥ 42 and D_R ≥ 32: unchanged.
            assert_eq!(out, Out::Detonated(pp));
        }
    }
}

// spec: S-0306 T1 (a dropping projectile ignores the arrival and passed-by rows)
#[test]
fn t1_dropping_ignores_the_arrival_rows() {
    let mut r = base();
    ball(&mut r).dropping = Some(true);
    ball(&mut r).ranged = Some(true);
    r.sim.general.gravity = Some(0);
    let mut s = setup(&r, 6, 6);
    let k = c(3, -2, 400);
    let id = place(
        &mut s,
        Target::Cell(cell_of(3, -2)),
        Pos::new(k.x - 100, k.y, 400),
        [60, 0, 0],
    );
    s.w.projectiles.get_mut(&id).unwrap().stored = k;
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

// spec: S-0306 Edge cases and F11 (a steering projectile whose speed reaches 0: it does not move in that turn, and the heading change gives it a horizontal length of r(10000) = 100)
#[test]
fn edge_steering_speed_reaches_zero() {
    let mut r = steering(5, 1, 3);
    weapon(&mut r).speed = Some(0);
    // Within 3 cells of the target: direct mode, no cruise lift.
    let mut s = setup(&r, 3, 0);
    let p = c(1, 0, 300);
    let id = missile(&mut s, p, [1, 0, 0]);
    assert_eq!(step(&mut s, id), Out::Flying(p));
    let v = pr(&s, id).vel;
    let h = table_root_real(v[0].mul(v[0], RZ).add(v[1].mul(v[1], RZ), RZ), true, RZ);
    assert!(h.gt(nd("90")) && h.le(n(100)), "{}", h.to_i32_common());
    // It moves again in the next turn.
    assert!(matches!(step(&mut s, id), Out::Flying(q) if q != p));
}

// spec: S-0306 Edge cases (flak range 0: a scatter radius of 0, a departure; the draws are still taken)
#[test]
fn edge_flak_range_zero() {
    let mut r = base();
    ball(&mut r).flak_scatter = Some(true);
    weapon(&mut r).range = Some(d("0"));
    let (mut s, f, aim) = scatter_setup(&mut r, 0);
    let mut cl = s.w.rng.clone();
    let h = half(&s.w);
    cl.ranged(0, 512, h);
    cl.ranged(0, 2147483646, h);
    let id = fire(&mut s, Target::Cell(aim.cell()), f).unwrap();
    assert_eq!(s.w.rng, cl);
    let start = arc(d2(&s.w, f, aim)).min(d3(&s.w, f, aim) / 2);
    assert_eq!(pr(&s, id).vel, f1(yaw_to(f, aim), 8191, start));
}

/// A tower whose aim shift is minus its reference point: its target point
/// is the world origin.
fn origin_tower(r: &mut Ruleset) -> (S, ObjectId) {
    let w = testkit::game_with(r, N, 7);
    let rp = Pos::centre(cell_of(8, 0), 0);
    drop(w);
    r.sim
        .buildings
        .get_mut("TOWER")
        .unwrap()
        .target_coord_offset = Some([-rp.x, -rp.y, 0]);
    let mut s = setup(r, 10, 0);
    let tw = testkit::spawn(&mut s.w, Kind::Building, "TOWER", 1, cell_of(8, 0), 0);
    assert_eq!(s.w.reference_point(tw), rp);
    (s, tw)
}

// spec: S-0306 Terms and Edge cases (quirks.origin_as_missing_target_point on: a target whose target point is the world origin counts as no target: it climbs and the safety-altitude row applies; scenario P38)
#[test]
fn quirk_origin_as_missing_target_point_on_origin_target() {
    let mut r = steering(10, 1, 3);
    r.sim.general.missile_rot_var = Some(d("0"));
    let (mut s, tw) = origin_tower(&mut r);
    let p = c(-5, 3, 300);
    let id = place(&mut s, Target::Object(tw), p, [64, 0, 0]);
    step(&mut s, id);
    // Heading kept, climbing by t = 2560, as with no target.
    assert!(same(pr(&s, id).vel, pitched(-2560)));
    let id = place(&mut s, Target::Object(tw), c(-5, 3, 600), [64, 0, 0]);
    assert!(matches!(step(&mut s, id), Out::Detonated(_)));
}

// spec: S-0306 Edge cases (quirks.origin_as_missing_target_point off: a target counts as a target wherever its target point lies)
#[test]
fn quirk_origin_as_missing_target_point_off_origin_target() {
    let mut r = steering(10, 1, 3);
    r.sim.general.missile_rot_var = Some(d("0"));
    r.globals.quirks.origin_as_missing_target_point = false;
    let (mut s, tw) = origin_tower(&mut r);
    let id = place(&mut s, Target::Object(tw), c(-5, 3, 600), [64, 0, 0]);
    // No safety-altitude detonation: it has a target (far away, at the origin).
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

// spec: S-0306 claim 13 and Edge cases (quirks.origin_as_missing_target_point on: without a target the near-target test uses the world origin)
#[test]
fn quirk_origin_as_missing_target_point_on_near_origin() {
    let mut r = steering(10, 1, 3);
    r.sim.general.missile_rot_var = Some(d("0"));
    let mut s = setup(&r, 10, 0);
    let id = missile(&mut s, Pos::new(100, 100, 150), [64, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().target = None;
    step(&mut s, id);
    // Within 256 of the origin: q* = 15, the climb step is 3840.
    assert!(same(pr(&s, id).vel, pitched(-3840)));
}

// spec: S-0306 Edge cases (quirks.origin_as_missing_target_point off: a projectile without a target never counts as near)
#[test]
fn quirk_origin_as_missing_target_point_off_near_origin() {
    let mut r = steering(10, 1, 3);
    r.sim.general.missile_rot_var = Some(d("0"));
    r.globals.quirks.origin_as_missing_target_point = false;
    let mut s = setup(&r, 10, 0);
    let id = missile(&mut s, Pos::new(100, 100, 150), [64, 0, 0]);
    s.w.projectiles.get_mut(&id).unwrap().target = None;
    step(&mut s, id);
    assert!(same(pr(&s, id).vel, pitched(-2560)));
}

// spec: S-0306 T1 and Edge cases (quirks.origin_as_missing_target_point: the closing step of a projectile without a target is measured to the origin; off, the closing-average row does not apply)
#[test]
fn quirk_origin_as_missing_target_point_closing_average() {
    for on in [true, false] {
        let mut r = steering(5, 1, 3);
        r.sim.general.missile_rot_var = Some(d("0"));
        r.sim.general.missile_safety_altitude = Some(100_000);
        r.globals.quirks.origin_as_missing_target_point = on;
        let mut s = setup(&r, 10, 0);
        // Moving at right angles to the line to the origin: a closing step near 0.
        let p = c(-5, 3, 300);
        let id = missile(&mut s, p, [64, -64, 0]);
        {
            let x = s.w.projectiles.get_mut(&id).unwrap();
            x.target = None;
            x.unlocked = 60;
            x.closing = F64::ZERO;
        }
        let out = step(&mut s, id);
        assert_eq!(matches!(out, Out::Detonated(_)), on, "quirk {on}: {out:?}");
    }
}

// spec: S-0306 claim 16 (a scalable type fired by a vehicle: effective delay spawn delay + trunc(max(N − 5, 0) / 3), N its in-play count with the new one; 0 there means the field; removals do not lower it; scenario P10 variants b, d, e)
#[test]
fn claim16_scalable_raised_delay() {
    for (field, shots, want) in [(2, 5, 2), (2, 8, 3), (2, 11, 4), (-1, 8, -1), (-1, 11, 1)] {
        let mut r = base();
        ball(&mut r).trailer = Some("TRAIL".into());
        ball(&mut r).spawn_delay = Some(field);
        ball(&mut r).scalable = Some(true);
        let mut s = setup(&r, 5, 5);
        let from = s.w.obj(s.a).unwrap().pos;
        let mut ids = Vec::new();
        for _ in 0..shots {
            let id = glasnost_sim::combat::projectile::launch(
                &mut s.w,
                s.a,
                s.wp,
                Target::Object(s.t),
                from,
                10,
            );
            ids.push(id.expect("launched"));
        }
        let pid = s.w.rules().combat.weapon(s.wp).projectile.unwrap();
        assert_eq!(
            s.w.scaled_spawn_delay.get(&pid),
            Some(&want),
            "{field} {shots}"
        );
        // Removals do not lower it.
        for id in ids {
            glasnost_sim::combat::projectile::remove(&mut s.w, id);
        }
        assert_eq!(s.w.scaled_spawn_delay.get(&pid), Some(&want));
    }
}

// ---------------------------------------------------------------------------
// Revision 4: the vertical deck crossing and the lifted position.
// ---------------------------------------------------------------------------

// spec: S-0306 claim 11, T1 and T2 (a vertical projectile crossing the deck height of a high-bridge cell, up or down, detonates at its own new height: no move to the deck height, no 20-lepton drop; scenario P33 runs c and e)
#[test]
fn t1_vertical_deck_crossing_at_its_own_height() {
    // |V| = 40 is below S_w, so it is rescaled to 40 + α = 43 (the rescaled component falls just short of 43 in size and P + trunc(V) truncates it towards 0).
    for (z0, vz, want) in [(400, 40i64, 442), (450, -40, 408)] {
        let (mut s, id) = vertical(3, 5000, true);
        let p = c(-10, -8, z0);
        set_deck(&mut s.w, p.cell());
        let pj = s.w.projectiles.get_mut(&id).unwrap();
        pj.pos = p;
        pj.vel = [n(0), n(0), n(vz)];
        let out = step(&mut s, id);
        let Out::Detonated(q) = out else {
            panic!("no detonation: {out:?}")
        };
        assert_eq!(q.z, want, "start {z0}, V_z {vz}");
    }
}

// spec: S-0306 T1 (vertical: no detonation without a crossing of the deck height)
#[test]
fn t1_vertical_no_deck_crossing() {
    let (mut s, id) = vertical(3, 5000, true);
    let p = c(-10, -8, 300);
    set_deck(&mut s.w, p.cell());
    let pj = s.w.projectiles.get_mut(&id).unwrap();
    pj.pos = p;
    pj.vel = [n(0), n(0), n(40)];
    assert!(matches!(step(&mut s, id), Out::Flying(_)));
}

// spec: S-0306 claim 15 and T1 (every row tests the lifted P′: a cruise lift of 18 that takes P′ across the deck height detonates at the deck height; before the lift it would not have crossed)
#[test]
fn claim15_lifted_position_crosses_the_deck() {
    let (mut s, id, p) = cruise(400, |_| {});
    set_deck(&mut s.w, p.cell());
    set_deck(&mut s.w, cell_of(-11, 0));
    let out = step(&mut s, id);
    let Out::Detonated(q) = out else {
        panic!("no detonation: {out:?}")
    };
    assert_eq!(q.z, 416);
}

// spec: S-0306 claim 15 (without a bridge the lifted position is the position the projectile keeps)
#[test]
fn claim15_lifted_position_is_kept() {
    let (mut s, id, p) = cruise(400, |_| {});
    assert_eq!(step(&mut s, id), Out::Flying(Pos::new(p.x + 64, p.y, 418)));
}

// spec: S-0306 claim 20 c (threshold 128 for a target airborne in another render layer: an aircraft in flight; 42 otherwise)
#[test]
fn claim20c_airborne_target_in_another_layer_uses_128() {
    let mut r = base();
    ball(&mut r).aa = Some(true);
    // A flying aircraft: the flying locomotor (S-0501 Terms).
    r.sim.aircraft.get_mut("PLANE").unwrap().locomotor = Some(glasnost_rules::sim::Locomotor::Fly);
    let mut s = setup(&r, 6, 0);
    // Of the firer's own house, so that the proximity row does not apply.
    let plane = testkit::spawn(&mut s.w, Kind::Aircraft, "PLANE", 0, cell_of(6, 3), 0);
    let tp = pos(&s, plane);
    let air = Pos::new(tp.x, tp.y, tp.z + 400);
    put(&mut s.w, plane, air);
    let rp = s.w.reference_point(plane);
    // P′ 100 from the reference point (the anti-air row of T1 detonates; D_S = 100 ≥ 42 but below 128).
    let id = place(
        &mut s,
        Target::Object(plane),
        Pos::new(rp.x - 150, rp.y, rp.z),
        [50, 0, 0],
    );
    let Out::Detonated(p) = step(&mut s, id) else {
        panic!("the anti-air row detonates")
    };
    assert_eq!(
        (p.x, p.y),
        (rp.x, rp.y),
        "the point moves onto the target ({p:?}, {rp:?})"
    );
}

// spec: S-0306 F14 (ψ = f₃₂ of the drawn facing of (Vₓ, −V_y): east 8, north 0, south 16, west 24, up the screen 28; e = f₃₂ of the pitch: 8 level, 4 at 45° up, 0 straight up and 16 straight down; no horizontal part counts as east)
#[test]
fn f14_drawn_facings() {
    let s = setup(&base(), 3, 0);
    let f = |v: [i64; 3]| glasnost_sim::combat::projectile::drawn_facings(&s.w, v.map(n));
    assert_eq!(f([50, 0, 0]), (8, 8));
    assert_eq!(f([0, -50, 0]), (0, 8));
    assert_eq!(f([-50, 0, 0]), (24, 8));
    assert_eq!(f([0, 50, 0]), (16, 8));
    assert_eq!(f([-50, -50, 0]), (28, 8));
    assert_eq!(f([50, 0, 50]), (8, 4));
    assert_eq!(f([50, 0, -50]), (8, 12));
    assert_eq!(f([0, 0, 50]), (8, 0));
    assert_eq!(f([0, 0, -50]), (8, 16));
}

// spec: S-0306 claim 3 (the aim point of a target that is not a moving vehicle is its reference point, no aim shift) and S-0301 claim 24 e (the target line's end point)
#[test]
fn claim3_led_aim_point_of_a_still_target() {
    let s = setup(&base(), 3, 0);
    assert_eq!(
        glasnost_sim::combat::projectile::led_aim_point(&s.w, s.a, Target::Object(s.t)),
        s.w.reference_point(s.t)
    );
}

// ================================================================= revision 53: claims 3 (b) and 6 (a)

/// `base()` with the hover fields of [General], land rows that let a
/// hovering vehicle move, and SKIM: a fast hovering vehicle type (every
/// name and number invented).
fn hover_rules() -> Ruleset {
    use glasnost_rules::sim::{LandRow, LandType, LocomotionClass, Locomotor, MovementZone};
    let mut r = base();
    let row = || LandRow {
        foot: Some(d("1")),
        track: Some(d("1")),
        wheel: Some(d("1")),
        hover: Some(d("1")),
        float: Some(d("0")),
        amphibious: Some(d("1")),
        float_beach: Some(d("0")),
        buildable: None,
    };
    let s = &mut r.sim;
    s.land_types.insert(LandType::Clear, row());
    s.land_types.insert(LandType::Road, row());
    s.general.hover_height = Some(100);
    s.general.hover_bob = Some(d("0"));
    s.general.hover_boost = Some(d("1.5"));
    s.general.hover_acceleration = Some(d("0.2"));
    s.general.hover_brake = Some(d("0.01"));
    s.general.hover_dampen = Some(d("0.75"));
    s.vehicles.insert(
        "SKIM".into(),
        UnitType {
            strength: Some(300),
            speed: Some(40),
            rot: Some(10),
            locomotion_class: Some(LocomotionClass::Hover),
            locomotor: Some(Locomotor::Hover),
            movement_zone: Some(MovementZone::Amphibious),
            ..Default::default()
        },
    );
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("SKIM".into());
    r
}

/// A hovering vehicle of house 0 three cells from the firer, ordered to a
/// cell far away and run until it travels at speed; then switched off when
/// `off` (S-0503 claim 10, as by S-0405 claim 13's disabling). The firer's
/// target is set to it.
fn travelling_hover(quirk: bool, off: bool) -> (S, ObjectId) {
    use glasnost_sim::orders::{self, Order};
    use glasnost_sim::rules::Behaviour;
    let mut r = hover_rules();
    r.globals.quirks.switched_off_hover_lead = quirk;
    let mut s = setup(&r, 6, 6);
    while orders::blocked(&s.w) {
        glasnost_sim::tick::step(&mut s.w);
    }
    let h = testkit::spawn(&mut s.w, Kind::Vehicle, "SKIM", 0, cell_of(3, -6), 0);
    assert!(orders::issue(
        &mut s.w,
        glasnost_sim::world::HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(cell_of(3, 12))),
        },
    ));
    let mut ticks = 0;
    while !glasnost_sim::hover::moving_now(&s.w, h) || ticks < 30 {
        glasnost_sim::tick::step(&mut s.w);
        s.w.effects.clear();
        ticks += 1;
        assert!(ticks < 200, "the hover travels");
    }
    if off {
        glasnost_sim::hover::switched(&mut s.w, h, true);
        assert!(glasnost_sim::hover::switched_off(&s.w, h));
    }
    s.w.obj_mut(s.a).unwrap().target = Some(Target::Object(h));
    (s, h)
}

// spec: S-0306 claim 3 (b) and Edge cases (quirks.switched_off_hover_lead on: a switched-off hovering vehicle that S-0503 counts as moving is led by the ordinary rule; off: a switched-off vehicle is not led, and one switched on is led in both positions; the lead moves only the launch, the stored point is the reference point; scenario P44 part a)
#[test]
fn claim3b_switched_off_hover_lead() {
    let mut launches = Vec::new();
    for quirk in [true, false] {
        // Switched on and travelling: led in both positions.
        let (s, h) = travelling_hover(quirk, false);
        let rp = s.w.reference_point(h);
        let led = glasnost_sim::combat::projectile::led_aim_point(&s.w, s.a, Target::Object(h));
        assert_ne!(led, rp, "a travelling hover is led (quirk {quirk})");
        // Switched off while it still moves in S-0503's sense.
        let (mut s, h) = travelling_hover(quirk, true);
        assert!(
            glasnost_sim::hover::moving(&s.w, h),
            "S-0503 counts it as moving"
        );
        let rp = s.w.reference_point(h);
        let led = glasnost_sim::combat::projectile::led_aim_point(&s.w, s.a, Target::Object(h));
        assert_eq!(led != rp, quirk, "led only with the quirk on");
        let f = fp(&s, 300);
        let id = fire(&mut s, Target::Object(h), f).expect("a launch");
        assert_eq!(
            pr(&s, id).stored,
            rp,
            "the stored point is the reference point"
        );
        launches.push(pr(&s, id).vel);
    }
    // The two worlds differ only in the switch: the led launch points elsewhere.
    assert_ne!(launches[0], launches[1]);
}

// spec: S-0306 claim 3 (b) (r62: a switched-off hovering vehicle that is led is led by the amount that its held speed fraction ρ gives, so none when ρ = 0; quirks.switched_off_hover_lead on, the default)
#[test]
fn claim3b_a_switched_off_hover_holding_rho_zero_is_not_moved() {
    let (mut s, h) = travelling_hover(true, true);
    assert!(glasnost_sim::hover::moving(&s.w, h));
    let rp = s.w.reference_point(h);
    let led = glasnost_sim::combat::projectile::led_aim_point(&s.w, s.a, Target::Object(h));
    assert_ne!(led, rp, "ρ > 0: led");
    s.w.obj_mut(h).unwrap().hover.as_mut().unwrap().rho = F64::ZERO;
    assert!(glasnost_sim::hover::moving(&s.w, h), "still moving");
    let led = glasnost_sim::combat::projectile::led_aim_point(&s.w, s.a, Target::Object(h));
    assert_eq!(led, rp, "ρ = 0: no lead");
}

// spec: S-0306 claim 3 (b) (r62: a switched-off hovering vehicle keeps its heading point, S-0503 claims 16 and 19, and is led while it has a destination or a heading point; one that arrives at its heading point with no destination and no stored step loses the point, S-0503 claim 15, and is no longer led; scenario P45 part a)
#[test]
fn claim3b_a_switched_off_hover_that_arrives_is_no_longer_led() {
    let (mut s, h) = travelling_hover(true, true);
    let point = s.w.obj(h).unwrap().hover.as_ref().unwrap().point;
    assert!(
        point.is_some(),
        "it keeps its heading point at the switch-off"
    );
    // Led while it keeps its heading point (its destination was cleared by
    // the stop request, S-0503 claim 10).
    let rp = s.w.reference_point(h);
    let led = glasnost_sim::combat::projectile::led_aim_point(&s.w, s.a, Target::Object(h));
    assert_ne!(led, rp, "led while it has a heading point");
    // In our engine the switched-off vehicle slows and stops short of its
    // point (S-0503 claims 2 and 3), so the arrival is set up directly: its
    // heading point where it now stands on the map plane, which S-0503 F5's
    // test counts as reached at any step length, and no stored step left.
    let here = s.w.obj(h).unwrap().pos;
    {
        let o = s.w.obj_mut(h).unwrap();
        o.path.steps.clear();
        o.hover.as_mut().unwrap().point = Some(glasnost_sim::world::Pos::new(here.x, here.y, 0));
    }
    assert!(glasnost_sim::hover::moving(&s.w, h));
    glasnost_sim::tick::step(&mut s.w);
    s.w.effects.clear();
    assert!(!glasnost_sim::hover::moving(&s.w, h), "no longer moving");
    assert_eq!(
        s.w.obj(h).unwrap().hover.as_ref().unwrap().point,
        None,
        "the arrival cleared the heading point"
    );
    let rp = s.w.reference_point(h);
    let led = glasnost_sim::combat::projectile::led_aim_point(&s.w, s.a, Target::Object(h));
    assert_eq!(led, rp, "not moving: not led");
}

/// The cart ordered to attack the buggy 5 cells away with `Shot` on an
/// arcing projectile of turn rate `rot` and weapon speed 10, then run
/// through the ordinary tick for `ticks`; gives the shots made and the
/// cart's rearm countdown before and after.
fn attack_with_arc(rot: i32, ticks: i32) -> (usize, i32, i32, bool) {
    use glasnost_sim::orders::{self, Order};
    use glasnost_sim::rules::Behaviour;
    let mut r = base();
    ball(&mut r).arcing = Some(true);
    ball(&mut r).rot = Some(rot);
    weapon(&mut r).speed = Some(10);
    let mut s = setup(&r, 5, 0);
    while orders::blocked(&s.w) {
        glasnost_sim::tick::step(&mut s.w);
    }
    let (a, t) = (s.a, s.t);
    assert!(orders::issue(
        &mut s.w,
        glasnost_sim::world::HouseId(0),
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(t)),
            dest: None,
        },
    ));
    let before = s.w.obj(a).unwrap().combat.rearm_until;
    let mut shots = 0;
    for _ in 0..ticks {
        s.w.effects.clear();
        glasnost_sim::tick::step(&mut s.w);
        shots += s
            .w
            .effects
            .iter()
            .filter(|e| matches!(e, glasnost_sim::combat::effects::Effect::Shot { firer, .. } if *firer == a))
            .count();
    }
    let o = s.w.obj(a).unwrap();
    let still = o.target == Some(Target::Object(t));
    (shots, before, o.combat.rearm_until, still)
}

// spec: S-0306 claim 6 (a) (revision 53: in an ordinary vehicle firing step after the rearm countdown has run out, a void launch leaves that countdown unchanged and every later attempt at the same unscattered impact point is void too; S-0301 claim 15) with a control that launches
#[test]
fn claim6a_void_launch_in_the_firing_step_keeps_the_countdown() {
    // A negative turn rate with weapon speed 10: F4 has no solution 5 cells away.
    let (shots, before, after, still) = attack_with_arc(-1, 120);
    assert_eq!(shots, 0, "no projectile at any attempt");
    assert_eq!(after, before, "the countdown is unchanged");
    assert!(still, "the firer keeps its target and goes on attempting");
    // The control: ρ = 0 takes its speed from the distance, so F4 is solved and the shot leaves.
    let (shots, before, after, _) = attack_with_arc(0, 120);
    assert!(shots > 0, "the control fires");
    assert_ne!(after, before, "and starts its countdown");
}
