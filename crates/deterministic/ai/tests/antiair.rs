//! The own AI against drainers in the air (M7b AI round 3, PLAN §4.7): a
//! house that sees an enemy drainer in the air and has nothing that can
//! hit it builds the cheapest unit or defence whose weapon can (an
//! anti-air projectile, S-0306 claim 23; a versus value that is not zero,
//! S-0304 claim 1), one per drainer; a ruleset with no such type leaves
//! the brain as it was. The tests use the synthetic M7b skirmish (Alpha's
//! drainer DISC) and, for Beta, an invented anti-air vehicle FLAKT.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::fixture_m7b as kit;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{ProjectileType, WeaponType};
use glasnost_sim::combat::weapon;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const BETA: HouseId = HouseId(1);

/// The M7b rules with FLAKT: Beta's war-factory vehicle with an anti-air
/// gun (test data of our own).
fn flak_rules() -> Arc<SimRules> {
    let mut r = kit::ruleset();
    r.sim.projectiles.insert(
        "FLAKP".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(true),
            ..Default::default()
        },
    );
    let gun = r.sim.weapons["Gun"].clone();
    r.sim.weapons.insert(
        "FLAKW".into(),
        WeaponType {
            projectile: Some("FLAKP".into()),
            ..gun
        },
    );
    let mut flak = r.sim.vehicles["GUNR"].clone();
    flak.primary = Some("FLAKW".into());
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("FLAKT".into());
    r.sim.vehicles.insert("FLAKT".into(), flak);
    Arc::new(SimRules::new(&r, kit::MAP_NAME))
}

fn game(rules: Arc<SimRules>) -> Game {
    let mut g = Game::new(
        rules,
        &kit::map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(5),
    )
    .expect("the M7b skirmish loads");
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
    g
}

fn home(w: &World, h: HouseId) -> CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(h) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap()
}

fn spawn(w: &mut World, name: &str, kind: Kind, house: u8, dx: i32, dy: i32) -> ObjectId {
    let h = home(w, BETA);
    testkit::spawn(w, kind, name, house, CellXY::new(h.x + dx, h.y + dy), 0)
}

/// Beta with a war factory and an Alpha disc in the air over its base;
/// returns the world and the disc.
fn drained(rules: Arc<SimRules>) -> (World, ObjectId) {
    let g = game(rules);
    let mut w = g.world.clone();
    spawn(&mut w, "FACT", Kind::Building, 1, -6, 6);
    let disc = spawn(&mut w, "DISC", Kind::Vehicle, 0, 3, -3);
    let mut n = 0;
    while !weapon::airborne_by_height(&w, disc) && n < 600 {
        testkit::run(&mut w, 1);
        n += 1;
    }
    assert!(
        weapon::airborne_by_height(&w, disc),
        "the disc is in the air"
    );
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    (w, disc)
}

/// Beta's orders at this think and its anti-air builds.
fn think(w: &World) -> (Vec<Order>, u32) {
    let mut br = Brain::new(w, &kit::map(), BETA);
    let out = br.think(w);
    (out, br.special_uses().anti_air_builds)
}

fn builds(w: &World, out: &[Order], name: &str) -> bool {
    let i = w.rules().index_of(Kind::Vehicle, name).unwrap();
    out.iter().any(
        |o| matches!(o, Order::Build { what } if what.kind == Kind::Vehicle && what.index == i),
    )
}

// S-0306 claim 23 and S-0304 claim 1: a house that sees an enemy drainer in the air and has nothing that can hit it builds a unit that can.
#[test]
fn builds_an_anti_air_unit_against_a_drainer_in_the_air() {
    let (w, _) = drained(flak_rules());
    let (out, n) = think(&w);
    assert!(builds(&w, &out, "FLAKT"), "{out:?}");
    assert_eq!(n, 1);
}

// PLAN §4.7 (tuning of our own opponent): one able unit per drainer; with one on the map, none more.
#[test]
fn builds_none_with_one_able_unit_per_drainer() {
    let (mut w, _) = drained(flak_rules());
    spawn(&mut w, "FLAKT", Kind::Vehicle, 1, -3, 3);
    let (out, n) = think(&w);
    assert!(!builds(&w, &out, "FLAKT"), "{out:?}");
    assert_eq!(n, 0);
}

// S-0306 claim 23: without a type whose projectile is anti-air the brain builds nothing for it.
#[test]
fn builds_nothing_without_an_anti_air_type() {
    let (w, _) = drained(kit::rules());
    let (_, n) = think(&w);
    assert_eq!(n, 0);
}

// S-0306 claim 23: the disc in the air is a type the anti-air test finds hittable by FLAKT and by no other Beta type.
#[test]
fn only_the_anti_air_type_hits_the_disc() {
    let (w, disc) = drained(flak_rules());
    let armour = glasnost_sim::combat::veterancy::type_combat(&w, disc)
        .unwrap()
        .armour;
    let t = |n: &str| glasnost_sim::econ::TypeRef {
        kind: Kind::Vehicle,
        index: w.rules().index_of(Kind::Vehicle, n).unwrap(),
    };
    assert!(glasnost_ai::antiair::hits_air(&w, t("FLAKT"), armour));
    assert!(!glasnost_ai::antiair::hits_air(&w, t("GUNR"), armour));
}
