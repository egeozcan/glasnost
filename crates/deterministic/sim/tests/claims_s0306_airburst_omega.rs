//! S-0306 T1 and claim 8: an airburst child's passed-by record is its
//! placement distance (issue 315). Synthetic data only.

use glasnost_num::{F64, Rounding};
use glasnost_rules::sim::{Decimal, ProjectileType, WarheadType, WeaponType};
use glasnost_sim::combat::projectile::{d3, launch, turn};
use glasnost_sim::combat::state::Projectile;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target};

// spec: S-0306 T1 (Ω is set at placement to the full d₃ from the position at placement to K) and claim 8 (the projectiles a burst releases are placed as every projectile is)
#[test]
fn issue_315_airburst_child_omega_is_its_placement_distance() {
    let n = 40;
    let mut r = testkit::combat_ruleset();
    r.sim.general.gravity = Some(10);
    r.sim.warheads.insert("Dud".into(), WarheadType::default());
    // The parent: a vertical airburst projectile (it detonates above its detonation altitude, 0).
    r.sim.projectiles.insert(
        "Ball".into(),
        ProjectileType {
            vertical: Some(true),
            airburst: Some(true),
            airburst_weapon: Some("Burst".into()),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Shot".into(),
        WeaponType {
            damage: Some(50),
            rof: Some(100),
            range: Some(Decimal("10".into())),
            speed: Some(50),
            projectile: Some("Ball".into()),
            warhead: Some("Dud".into()),
            ..Default::default()
        },
    );
    // The children: a straight projectile with the ranged flag.
    r.sim.projectiles.insert(
        "Pellet".into(),
        ProjectileType {
            ranged: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Burst".into(),
        WeaponType {
            damage: Some(9),
            range: Some(Decimal("6".into())),
            speed: Some(77),
            projectile: Some("Pellet".into()),
            warhead: Some("Dud".into()),
            ..Default::default()
        },
    );
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("Shot".into());
    let mut w = testkit::game_with(&r, n, 7);
    let a = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, testkit::mid(n, 0, 0), 0);
    let wp = w.rules().combat.weapon_index["Shot"];
    let ap = w.obj(a).unwrap().pos;
    let far = Target::Cell(testkit::mid(n, 12, 9));
    let id = launch(&mut w, a, wp, far, Pos::new(ap.x, ap.y, ap.z + 300), 50).expect("a launch");
    // The parent 300 leptons above an empty cell, sinking slowly.
    let p = Pos::centre(testkit::mid(n, -12, -12), 300);
    {
        let x = w.projectiles.get_mut(&id).unwrap();
        x.pos = p;
        x.vel = [0i64, 0, -5].map(|v| F64::from_i64(v, Rounding::TowardZero));
        x.prev_cell = p.cell();
        x.omega = 1_000_000;
    }
    let before: Vec<ObjectId> = w.projectiles.keys().copied().collect();
    turn(&mut w, id);
    assert!(w.projectiles.get(&id).is_none(), "the parent detonated");
    let kids: Vec<Projectile> = w
        .projectiles
        .values()
        .filter(|x| !before.contains(&x.id))
        .cloned()
        .collect();
    assert_eq!(kids.len(), 9);
    let k = kids
        .iter()
        .find(|k| k.pos.cell() == p.cell())
        .expect("the child over the burst's own cell")
        .clone();
    let h = k.placed_at.z - w.surface_at(k.placed_at.x, k.placed_at.y);
    let want = d3(&w, k.placed_at, k.stored);
    assert_eq!(k.omega, want, "Ω at placement");
    // Its first turn: it is still more than 208 above the ground at the start of the turn (no
    // reached row), and η = trunc(d₃(P′, K) / 2) is below the placement distance, so the turn
    // only records η.
    turn(&mut w, k.id);
    assert!(
        w.projectiles.contains_key(&k.id),
        "the child detonated in its first turn, {h} leptons up, by the passed-by row"
    );
}
