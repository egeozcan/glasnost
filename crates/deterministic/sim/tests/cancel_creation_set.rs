//! A cancelled production item takes its whole creation set out of the
//! game (#156): the slaves its master got at production start and its
//! mind-control record. The slaves are destroyed as S-0312 claim 31 gives
//! for a cancelled production master (Q-2973): a loss each, no kill, no
//! score, no message of their own. Synthetic data only: the `econ_common`
//! test ruleset with an invented slave type and takeover weapon.

mod econ_common;

use econ_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ProjectileType, WarheadType, WeaponType};
use glasnost_sim::econ::{TypeRef, slaves, stats, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, World};

const SLAVES: i32 = 3;

/// The econ ruleset whose TANK names an invented slave type SLAVE with
/// three slaves.
fn slave_rules() -> Ruleset {
    slave_rules_scored(true)
}

/// [`slave_rules`], with the slave type's not-scored mark when `scored` is
/// false.
fn slave_rules_scored(scored: bool) -> Ruleset {
    let mut r = ruleset();
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("SLAVE".into());
    let mut slave = foot(100);
    if !scored {
        slave.combat.dont_score = Some(true);
    }
    r.sim.infantry.insert("SLAVE".into(), slave);
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

/// Every house's (kills, losses, score).
fn tallies(w: &World) -> Vec<(i32, i32, i32)> {
    (0..w.houses.len())
        .map(|h| {
            let s = stats::of(w, HouseId(h as u8));
            (s.kills, s.losses, s.score)
        })
        .collect()
}

/// Each house's (kills, losses, score).
type Tallies = Vec<(i32, i32, i32)>;

/// Starts building `what` in house 0, then ends the item with `end`.
/// Returns the tallies before and after the end, the effects of the tick
/// that ended it, and the house's slave count after it.
fn build_then(
    r: &Ruleset,
    what: &str,
    end: impl FnOnce(&mut World, TypeRef),
) -> (Tallies, Tallies, Vec<String>, i32) {
    let mut w = factory_game(r);
    let ty = t(&w, Kind::Vehicle, what);
    let slave = t(&w, Kind::Infantry, "SLAVE");
    assert!(issue(&mut w, 0, Order::Build { what: ty }));
    step(&mut w);
    assert!(item(&w, 0, ty).is_some(), "the item is being built");
    let before = tallies(&w);
    end(&mut w, ty);
    let effects = w.effects.iter().map(|e| format!("{e:?}")).collect();
    assert!(item(&w, 0, ty).is_none(), "the item ended");
    (
        before,
        tallies(&w),
        effects,
        tech::owned_count(&w, HouseId(0), slave),
    )
}

fn cancel(w: &mut World, ty: TypeRef) {
    assert!(issue(w, 0, Order::Cancel { what: ty }));
    step(w);
}

// spec: S-0312 claim 31 (a cancelled production master: every slave is destroyed and removed at the cancellation, a loss each to its house, no kill and no score for any house; the master gives no loss) and S-0414 claim 21
#[test]
fn a_cancelled_masters_slaves_are_losses_without_kills_or_score() {
    let (before, after, _, left) = build_then(&slave_rules(), "TANK", cancel);
    assert_eq!(left, 0, "no slave is left");
    for (h, (b, a)) in before.iter().zip(&after).enumerate() {
        let losses = if h == 0 { SLAVES } else { 0 };
        assert_eq!(a.0, b.0, "house {h}: no kill");
        assert_eq!(a.1 - b.1, losses, "house {h}: a loss per slave");
        assert_eq!(a.2, b.2, "house {h}: no score");
    }
}

// spec: S-0312 claim 31 (subject to S-0414 claim 21's victim exclusions: a not-scored slave gives no loss)
#[test]
fn a_cancelled_masters_not_scored_slaves_give_no_loss() {
    let (before, after, _, left) = build_then(&slave_rules_scored(false), "TANK", cancel);
    assert_eq!(left, 0);
    assert_eq!(before, after);
}

// spec: S-0312 claim 31 (the slaves' removal adds no sound or message of its own to those of the cancellation; scenario D24 part e is the control)
#[test]
fn a_cancelled_masters_slaves_add_no_effect() {
    let (_, _, with, _) = build_then(&slave_rules(), "TANK", cancel);
    let (_, _, without, _) = build_then(&slave_rules(), "JEEP", cancel);
    assert_eq!(with, without);
}

// spec: S-0312 claim 31 (a cancellation by S-0404 claim 8's check, when no factory building of the house could make the item any longer, is the same removal)
#[test]
fn a_master_cancelled_by_its_factorys_removal_loses_its_slaves() {
    let (before, after, _, left) = build_then(&slave_rules(), "TANK", |w, _| {
        let warfact = t(w, Kind::Building, "WARFACT").index;
        let factory = w
            .objects
            .iter()
            .find(|(_, o)| o.kind == Kind::Building && o.type_index == warfact)
            .map(|(&id, _)| id)
            .expect("the factory");
        glasnost_sim::objects::remove(w, factory);
        step(w);
    });
    assert_eq!(left, 0);
    assert_eq!(after[0].1 - before[0].1, SLAVES);
    assert_eq!(after[0].0, before[0].0);
    assert_eq!(after[0].2, before[0].2);
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
