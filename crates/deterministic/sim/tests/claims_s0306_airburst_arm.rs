//! S-0306 claims 9 and 21 (revision 93, answers Q-3130), sim issues S21
//! (#315): an airburst child's arm time is its own projectile type's arm
//! field, whatever the bursting projectile's arm time. Synthetic data only.
//! Ledger: `docs/m8-claims/sim-issues.md`, *Batch S21*.

use glasnost_num::{F64, Rounding};
use glasnost_rules::sim::{Decimal, ProjectileType, WarheadType, WeaponType};
use glasnost_sim::combat::projectile::{launch, turn};
use glasnost_sim::combat::state::Projectile;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target};

/// The arm times of the nine children of a burst, the bursting type's arm
/// field `parent` and the children's `child`.
fn children_arm(parent: i32, child: i32) -> Vec<i32> {
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
            arm: Some(parent),
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
            arm: Some(child),
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
        x.arm = 0;
    }
    let before: Vec<ObjectId> = w.projectiles.keys().copied().collect();
    turn(&mut w, id);
    assert!(!w.projectiles.contains_key(&id), "the parent detonated");
    let kids: Vec<Projectile> = w
        .projectiles
        .values()
        .filter(|x| !before.contains(&x.id))
        .cloned()
        .collect();
    assert_eq!(kids.len(), 9);
    kids.iter().map(|k| k.arm).collect()
}

// spec: S-0306 claims 9 and 21 (revision 93, Q-3130: each child is placed as claim 9 says, so its arm time is its own projectile type's arm field; its target is a cell, never an aircraft)
#[test]
fn issue_315_an_airburst_child_takes_its_own_type_arm_field() {
    assert_eq!(children_arm(0, 1000), vec![1000; 9]);
    assert_eq!(children_arm(500, 0), vec![0; 9]);
}
