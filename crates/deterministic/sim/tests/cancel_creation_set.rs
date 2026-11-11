//! A cancelled production item takes its whole creation set out of the
//! game (#156): the slaves its master got at production start and its
//! mind-control record. Synthetic data only: the `econ_common` test
//! ruleset with an invented slave type and takeover weapon.

mod econ_common;

use econ_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ProjectileType, WarheadType, WeaponType};
use glasnost_sim::econ::{slaves, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, World};

const SLAVES: i32 = 3;

/// The econ ruleset whose TANK names an invented slave type SLAVE with
/// three slaves.
fn slave_rules() -> Ruleset {
    let mut r = ruleset();
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("SLAVE".into());
    r.sim.infantry.insert("SLAVE".into(), foot(100));
    let tank = r.sim.vehicles.get_mut("TANK").unwrap();
    tank.econ.enslaves = Some("SLAVE".into());
    tank.econ.slaves_number = Some(SLAVES);
    tank.econ.slave_regen_rate = Some(50);
    tank.econ.slave_reload_rate = Some(40);
    r
}

/// The econ ruleset whose TANK's primary weapon has an invented takeover
/// warhead.
fn controller_rules() -> Ruleset {
    let mut r = ruleset();
    r.sim
        .projectiles
        .insert("Beam".into(), ProjectileType::default());
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            verses: Some(vec![Decimal("1".into()); 11]),
            mind_control: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Ray".into(),
        WeaponType {
            damage: Some(2),
            rof: Some(30),
            range: Some(Decimal("5".into())),
            speed: Some(100),
            projectile: Some("Beam".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    r.sim.vehicles.get_mut("TANK").unwrap().primary = Some("Ray".into());
    r
}

fn factory_game(r: &Ruleset) -> World {
    let mut w = game(r);
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    w
}

// spec: S-0402 claim 19 (a produced master gets its slaves at production start) and S-0404 claim 6 (a cancel deletes the item's object)
#[test]
fn a_cancelled_slave_master_takes_its_slaves_with_it() {
    let mut w = factory_game(&slave_rules());
    let tank = t(&w, Kind::Vehicle, "TANK");
    let slave = t(&w, Kind::Infantry, "SLAVE");
    let objects_before = w.objects.len();
    let credits_before = credits(&w, 0);

    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let master = item(&w, 0, tank).expect("an item").object;
    assert_eq!(slaves::slaves_of(&w, master).len(), SLAVES as usize);
    assert_eq!(tech::owned_count(&w, HouseId(0), slave), SLAVES);

    assert!(issue(&mut w, 0, Order::Cancel { what: tank }));
    step(&mut w);
    assert!(item(&w, 0, tank).is_none());
    assert!(w.obj(master).is_none());
    // spec: S-0404 claim 6 (the full refund)
    assert_eq!(credits(&w, 0), credits_before);
    assert_eq!(
        tech::owned_count(&w, HouseId(0), slave),
        0,
        "the slaves of the cancelled master are still in the world"
    );

    // Ten more Build and Cancel pairs, one pair per tick.
    for _ in 0..10 {
        assert!(issue(&mut w, 0, Order::Build { what: tank }));
        assert!(issue(&mut w, 0, Order::Cancel { what: tank }));
        step(&mut w);
    }
    for _ in 0..20 {
        step(&mut w);
    }
    assert_eq!(tech::owned_count(&w, HouseId(0), slave), 0);
    assert_eq!(w.objects.len(), objects_before);
    assert_eq!(credits(&w, 0), credits_before);
}

// spec: S-0601 claim 1 (a controller gets its record when it is created) and S-0404 claim 6 (a cancel deletes the item's object)
#[test]
fn a_cancelled_controller_leaves_no_record() {
    let mut w = factory_game(&controller_rules());
    let tank = t(&w, Kind::Vehicle, "TANK");
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    step(&mut w);
    let master = item(&w, 0, tank).expect("an item").object;
    assert!(
        glasnost_sim::mind::controller(&w, master).is_some(),
        "the controller gets its record at production start"
    );
    assert!(issue(&mut w, 0, Order::Cancel { what: tank }));
    step(&mut w);
    assert!(w.obj(master).is_none());
    assert!(
        glasnost_sim::mind::controller(&w, master).is_none(),
        "the cancelled controller's record is still there"
    );
}
