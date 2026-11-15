//! S-0306 claim 21 and S-0302 claim 1: a later cluster detonation carries
//! no firer that an earlier detonation of the same projectile killed
//! (issue 318). Synthetic data only.

use glasnost_num::{F64, Rounding};
use glasnost_rules::sim::{BuildingType, Decimal, ProjectileType, WarheadType, WeaponType};
use glasnost_sim::combat::projectile::{launch, turn};
use glasnost_sim::econ::stats;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Pos, Target};

const N: i32 = 40;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

// spec: S-0306 claim 21 and S-0302 claim 1 (each cluster detonation carries the firer unless it was killed between the shot and that detonation)
#[test]
fn issue_318_later_cluster_detonation_after_the_firer_is_killed() {
    let mut r = testkit::combat_ruleset();
    r.sim.general.gravity = Some(10);
    // Off, so that the later hits on the building in its countdown credit nothing by themselves.
    r.globals.quirks.delay_kill_credited_again = false;
    r.sim.weapons.insert(
        "Shot".into(),
        WeaponType {
            damage: Some(60),
            rof: Some(100),
            range: Some(d("10")),
            speed: Some(50),
            projectile: Some("Ball".into()),
            warhead: Some("Slow".into()),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "Ball".into(),
        ProjectileType {
            cluster: Some(3),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "Slow".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            cell_spread: Some(d("8")),
            percent_at_max: Some(d("1")),
            causes_delay_kill: Some(true),
            delay_kill_frames: Some(1000),
            delay_kill_at_max: Some(d("1")),
            ..Default::default()
        },
    );
    // The firer: a building that its own detonations hit and that a
    // delayed kill keeps in play.
    let mut post = BuildingType {
        strength: Some(10),
        foundation: Some([1, 1]),
        eligible_for_delay_kill: Some(true),
        ..Default::default()
    };
    post.combat.damage_self = Some(true);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("POST".into());
    r.sim.buildings.insert("POST".into(), post);
    // The victim: an enemy vehicle that survives one detonation and dies at the second.
    let mut box_ = r.sim.vehicles["BUGGY"].clone();
    box_.strength = Some(100);
    box_.primary = None;
    box_.combat.armor = None;
    r.registries.get_mut("vehicles").unwrap().push("BOX".into());
    r.sim.vehicles.insert("BOX".into(), box_);

    let mut w = testkit::game_with(&r, N, 7);
    let firer = testkit::spawn(&mut w, Kind::Building, "POST", 0, testkit::mid(N, 0, 0), 0);
    let victim = testkit::spawn(&mut w, Kind::Vehicle, "BOX", 1, testkit::mid(N, 2, 0), 0);
    let wp = w.rules().combat.weapon_index["Shot"];
    let fp = w.obj(firer).unwrap().pos;
    let far = Target::Cell(testkit::mid(N, 12, 9));
    let id = launch(&mut w, firer, wp, far, Pos::new(fp.x, fp.y, fp.z + 300), 60).expect("a launch");
    {
        // Deep below the surface next to the firer: the deep-fall row detonates it in its next turn.
        let p = Pos::centre(testkit::mid(N, 1, 0), -380);
        let x = w.projectiles.get_mut(&id).unwrap();
        x.pos = p;
        x.vel = [50, 0, -40].map(|v| F64::from_i64(v, Rounding::TowardZero));
        x.prev_cell = p.cell();
        x.omega = 1_000_000;
        assert_eq!((x.attacker, x.house), (Some(firer), Some(HouseId(0))));
    }
    let kills = stats::of(&w, HouseId(0)).kills;
    turn(&mut w, id);
    assert!(!w.projectiles.contains_key(&id), "the projectile detonated");
    let f = w.obj(firer).expect("the firer stays in play");
    assert!(f.combat.killed, "the first detonation killed the firer");
    assert!(
        w.obj(victim).is_none_or(|o| o.health <= 0),
        "a later detonation destroyed the victim"
    );
    // The first detonation's killing hit on the firer carries the firer: one kill for its house.
    // The victim was destroyed by the second or third detonation, after the
    // firer was killed: that detonation carries no attacker, so nobody is credited.
    assert_eq!(
        stats::of(&w, HouseId(0)).kills,
        kills + 1,
        "a detonation after the firer's kill credited the firer's house"
    );
}
