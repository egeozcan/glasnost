//! Claim tests for S-0306 revision 45 (claim 8, the start point raised
//! above the ground; scenario P43) with S-0313 revision 37 (claim 1a, an
//! absent muzzle position), on invented rules and a flat synthetic map.
//! Shots are made through the real fire path (`combat::fire::shot`, which
//! takes the fire point of `combat::position::fire_point`) and flown with
//! `combat::projectile::turn`. See
//! `docs/m8-claims/intake51-S-0306-S-0313.md`.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ProjectileType, WarheadType, WeaponType};
use glasnost_sim::combat::fire;
use glasnost_sim::combat::position::fire_point;
use glasnost_sim::combat::projectile::{launch, start_point, turn};
use glasnost_sim::combat::range::target_point;
use glasnost_sim::combat::state::Projectile;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

const N: i32 = 40;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// The combat ruleset with a turretless CART (no turret displacement, no
/// muzzle-position keys) carrying `Sam` (a steering projectile `Missile`:
/// ρ 5, acceleration 3, speed field 25) as its primary and `Gun` (a
/// straight projectile `Ball`) as its secondary.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.insert(
        "Sam".into(),
        WeaponType {
            damage: Some(60),
            rof: Some(100),
            range: Some(d("12")),
            speed: Some(25),
            projectile: Some("Missile".into()),
            warhead: Some("Dud".into()),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Gun".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(100),
            range: Some(d("12")),
            speed: Some(50),
            projectile: Some("Ball".into()),
            warhead: Some("Dud".into()),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "Missile".into(),
        ProjectileType {
            rot: Some(5),
            aa: Some(true),
            ag: Some(true),
            acceleration: Some(3),
            course_lock_duration: Some(1),
            ..Default::default()
        },
    );
    r.sim
        .projectiles
        .insert("Ball".into(), ProjectileType::default());
    r.sim.warheads.insert(
        "Dud".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.turret = Some(false);
    cart.primary = Some("Sam".into());
    cart.secondary = Some("Gun".into());
    cart.combat.primary_fire_flh = None;
    cart.combat.secondary_fire_flh = None;
    cart.combat.turret_offset = None;
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(30000);
    r
}

struct S {
    w: World,
    /// The firer, a CART of house 0 at the middle cell, facing east.
    a: ObjectId,
    /// The target, a BUGGY of house 1, `dx` cells east.
    t: ObjectId,
}

fn setup(r: &Ruleset, dx: i32) -> S {
    let mut w = testkit::game_with(r, N, 7);
    let a = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        0,
        testkit::mid(N, 0, 0),
        16384,
    );
    let t = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, dx, 0),
        49152,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(t));
    S { w, a, t }
}

fn pos(s: &S, id: ObjectId) -> Pos {
    s.w.obj(id).unwrap().pos
}

/// Holds the target `dz` leptons above its surface (a hovering target).
fn hover(s: &mut S, dz: i32) {
    let p = pos(s, s.t);
    s.w.obj_mut(s.t).unwrap().pos = Pos::new(p.x, p.y, p.z + dz);
}

/// Fires slot `slot` at the target through the real fire path and returns
/// the projectile it placed.
fn shoot(s: &mut S, slot: usize) -> Projectile {
    let before: Vec<ObjectId> = s.w.projectiles.keys().copied().collect();
    let t = Target::Object(s.t);
    assert!(fire::shot(&mut s.w, s.a, slot, t), "the shot is made");
    let made: Vec<&Projectile> =
        s.w.projectiles
            .values()
            .filter(|p| !before.contains(&p.id))
            .collect();
    assert_eq!(made.len(), 1, "one projectile");
    made[0].clone()
}

fn surface(w: &World, p: Pos) -> i32 {
    w.surface_at(p.x, p.y)
}

// ---------------------------------------------------------------------------
// S-0313 claim 1a with S-0306 claim 8: P43 a, the steering projectile.

// spec: S-0306 claim 8 and scenario P43 a (revision 45: a steering projectile fired from a fire point at the surface starts 1 lepton above it, at the same x and y, and is not detonated in its first turn by T1's h ≤ 0 row) and S-0313 claim 1a (no muzzle-position key: 0, 0, 0, so the fire point is the vehicle's own position)
#[test]
fn p43a_steering_projectile_from_the_surface_survives_its_first_turn() {
    let mut s = setup(&rules(), 6);
    hover(&mut s, 300);
    let own = pos(&s, s.a);
    assert_eq!(own.z, surface(&s.w, own), "the firer stands on the surface");
    // S-0313 claim 1a: no muzzle-position key, the fire point is the base point.
    assert_eq!(fire_point(&s.w, s.a, 0, 0), own);
    let pr = shoot(&mut s, 0);
    assert_eq!(pr.pos, Pos::new(own.x, own.y, own.z + 1), "start point");
    assert_eq!(pr.placed_at, pr.pos);
    assert_eq!(pr.fire_point, own, "the walk's X₁ is not raised");
    let id = pr.id;
    turn(&mut s.w, id);
    let after =
        s.w.projectiles
            .get(&id)
            .expect("still in flight after turn 1");
    assert!(after.pos.z > surface(&s.w, after.pos), "above the ground");
    // Under revision 38's placement (the unraised fire point) the same
    // projectile detonated in its first turn, at the launcher.
    let mut s2 = setup(&rules(), 6);
    hover(&mut s2, 300);
    let pr2 = shoot(&mut s2, 0);
    s2.w.projectiles.get_mut(&pr2.id).unwrap().pos = own;
    turn(&mut s2.w, pr2.id);
    assert!(
        !s2.w.projectiles.contains_key(&pr2.id),
        "at the surface, the h ≤ 0 row detonates it"
    );
}

// spec: S-0306 claim 8 and scenario P43 a (revision 45: the raised missile flies on and detonates by the hovering target, not at the launcher; the damage it does is left open, S-0306 Open questions, Q-2111 question 3)
#[test]
fn p43a_the_raised_missile_reaches_the_hovering_target() {
    let mut s = setup(&rules(), 6);
    hover(&mut s, 300);
    let own = pos(&s, s.a);
    let tp = target_point(&s.w, Target::Object(s.t));
    let pr = shoot(&mut s, 0);
    let mut last = pr.pos;
    for _ in 0..200 {
        let Some(p) = s.w.projectiles.get(&pr.id) else {
            break;
        };
        last = p.pos;
        turn(&mut s.w, pr.id);
    }
    assert!(!s.w.projectiles.contains_key(&pr.id), "it detonated");
    let away = (last.x - own.x).abs().max((last.y - own.y).abs());
    assert!(away > 3 * 256, "away from the launcher: {last:?}");
    let near = (last.x - tp.x).abs().max((last.y - tp.y).abs());
    assert!(near < 256, "next to the target: {last:?}, {tp:?}");
}

// ---------------------------------------------------------------------------
// P43 b and c: a straight projectile, muzzle at the surface or above it.

// spec: S-0306 claim 8 and scenario P43 b (revision 45: a straight projectile with a muzzle position of 0, 0, 0 starts at height 1, at the vehicle's x and y; claims 5 to 7 use the unraised fire point)
#[test]
fn p43b_straight_projectile_from_a_zero_muzzle() {
    let mut r = rules();
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .secondary_fire_flh = Some([0, 0, 0]);
    let mut s = setup(&r, 2);
    let own = pos(&s, s.a);
    // The target's reference point 201 above the fire point: F13 gives the
    // flat pitch for |Δz| ≤ 200, so the raised point (Δz = 200) and the
    // unraised one (Δz = 201) launch differently.
    let tp = target_point(&s.w, Target::Object(s.t));
    hover(&mut s, own.z + 201 - tp.z);
    assert_eq!(target_point(&s.w, Target::Object(s.t)).z, own.z + 201);
    let fresh = s.w.clone();
    let pr = shoot(&mut s, 1);
    assert_eq!(pr.pos, Pos::new(own.x, own.y, own.z + 1));
    assert_eq!(pr.fire_point, own);
    // The velocity is the one launched from the unraised fire point, and it
    // differs from one launched from the raised start point.
    let gun = s.w.rules().combat.weapon_index["Gun"];
    let t = Target::Object(s.t);
    let mut w1 = fresh.clone();
    let v_unraised = launch(&mut w1, s.a, gun, t, own, 40).map(|id| w1.projectiles[&id].vel);
    let mut w2 = fresh;
    let raised = Pos::new(own.x, own.y, own.z + 1);
    let v_raised = launch(&mut w2, s.a, gun, t, raised, 40).map(|id| w2.projectiles[&id].vel);
    assert_eq!(Some(pr.vel), v_unraised);
    assert_ne!(
        v_unraised, v_raised,
        "the geometry tells the two points apart"
    );
}

// spec: S-0306 claim 8 and scenario P43 c (revision 45: a fire point above the terrain is used as it is: a muzzle height of 40 starts at height 40)
#[test]
fn p43c_fire_point_above_the_ground_is_not_raised() {
    let mut r = rules();
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .primary_fire_flh = Some([0, 0, 40]);
    let mut s = setup(&r, 6);
    hover(&mut s, 300);
    let own = pos(&s, s.a);
    let pr = shoot(&mut s, 0);
    assert_eq!(pr.pos, Pos::new(own.x, own.y, own.z + 40));
    assert_eq!(pr.fire_point, pr.pos);
}

// spec: S-0306 claim 8 (revision 45: a fire point below the terrain starts 1 lepton above the terrain at its x and y, a bridge deck there ignored; a position 1 above is kept)
#[test]
fn claim8_start_point_below_at_and_above_the_terrain() {
    let s = setup(&rules(), 6);
    let own = pos(&s, s.a);
    let z = surface(&s.w, own);
    for (dz, want) in [(-300, 1), (-1, 1), (0, 1), (1, 1), (2, 2), (500, 500)] {
        let p = Pos::new(own.x, own.y, z + dz);
        assert_eq!(
            start_point(&s.w, p),
            Pos::new(own.x, own.y, z + want),
            "dz {dz}"
        );
    }
    // A raised cell: its own terrain height decides, not level 0.
    let mut w = s.w.clone();
    let c = testkit::mid(N, 3, 3);
    w.cells.grid.get_mut(c).unwrap().level = 4;
    let p = Pos::centre(c, 0);
    let t = w.surface_at(p.x, p.y);
    assert_eq!(t, 4 * 104);
    assert_eq!(start_point(&w, p), Pos::new(p.x, p.y, t + 1));
    // A bridge deck in the cell is ignored: the terrain decides.
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
    assert_eq!(start_point(&w, p), Pos::new(p.x, p.y, t + 1));
    assert_eq!(
        start_point(&w, Pos::new(p.x, p.y, t + 100)),
        Pos::new(p.x, p.y, t + 100),
        "under the deck, above the terrain: kept"
    );
}

// spec: S-0306 claims 8 and 9 (revision 45: an invisible projectile is placed at its start point and then moved to K; that later move is not raised)
#[test]
fn claim8_invisible_projectile_moved_to_k_is_not_raised() {
    let mut r = rules();
    r.sim.projectiles.get_mut("Ball").unwrap().inviso = Some(true);
    let mut s = setup(&r, 3);
    let k = pos(&s, s.t);
    let pr = shoot(&mut s, 1);
    assert_eq!(
        pr.pos, k,
        "at K, the target's reference point on the ground"
    );
    assert_eq!(pr.pos.z, surface(&s.w, k));
}

// ---------------------------------------------------------------------------
// S-0313 claim 1a: absent muzzle positions in the real fire path.

// spec: S-0313 claim 1a and scenario P1 part e (an absent secondary muzzle position is 0, 0, 0: the shot starts at the vehicle's own x and y, not at the primary's F1 nor at an alternate muzzle position; its height is S-0306 claim 8's for a fire point at the surface)
#[test]
fn claim1a_absent_secondary_muzzle_is_the_zero_triple() {
    let mut r = rules();
    {
        let c = &mut r.sim.vehicles.get_mut("CART").unwrap().combat;
        c.primary_fire_flh = Some([100, 40, 50]);
        c.alternate_flh = Some(vec![
            [10, 20, 30],
            [11, 21, 31],
            [12, 22, 32],
            [13, 23, 33],
            [14, 24, 34],
        ]);
    }
    for facing in [0u16, 16384] {
        let mut s = setup(&r, 2);
        {
            let o = s.w.obj_mut(s.a).unwrap();
            o.body = glasnost_sim::turning::Turning::at(facing);
            o.turret = glasnost_sim::turning::Turning::at(facing);
        }
        let own = pos(&s, s.a);
        assert_eq!(fire_point(&s.w, s.a, 1, 0), own, "facing {facing}");
        assert_ne!(fire_point(&s.w, s.a, 0, 0), own, "the primary's own muzzle");
        let pr = shoot(&mut s, 1);
        assert_eq!(pr.pos, Pos::new(own.x, own.y, own.z + 1), "facing {facing}");
    }
}
