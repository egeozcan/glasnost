//! A long chain of death weapons, each object's killing the next, resolves
//! whole on a small thread stack (#218; S-0312 r74 claim 33, Q-3000
//! answered). The chain's order is S-0312 claim 1's and S-0303 claim 1's:
//! each death weapon inside the killing hit, depth first, with no depth
//! limit. Synthetic data only: an invented exploding vehicle BARREL on the
//! testkit's combat rules.
//!
//! A stack overflow aborts the whole test process, so this file holds only
//! this test.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LocomotionClass, Locomotor, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::ObjectId;

const SIZE: i32 = 160;
const HALF: i32 = 60;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("BARREL".into());
    let mut v = UnitType {
        strength: Some(200),
        speed: Some(5),
        rot: Some(5),
        cost: Some(600),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    };
    v.combat.explodes = Some(true);
    v.combat.death_weapon = Some("Chain".into());
    r.sim.vehicles.insert("BARREL".into(), v);
    r.sim.weapons.insert(
        "Chain".into(),
        WeaponType {
            damage: Some(500),
            rof: Some(30),
            range: dec("2"),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("LINK".into()),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "LINK".into(),
        WarheadType {
            // A radius of one cell: the four edge neighbours only.
            cell_spread: dec("1"),
            percent_at_max: dec("1"),
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            ..Default::default()
        },
    );
    r
}

/// A serpentine of `n` cells, each an edge neighbour of the one before and
/// of no other cell of the path.
fn path(n: usize) -> Vec<CellXY> {
    let mut out = Vec::new();
    let mut y = -HALF;
    let mut right = true;
    'rows: while y <= HALF {
        let xs: Vec<i32> = if right {
            (-HALF..=HALF).collect()
        } else {
            (-HALF..=HALF).rev().collect()
        };
        let end = *xs.last().unwrap();
        for x in xs {
            out.push(testkit::mid(SIZE, x, y));
            if out.len() == n {
                break 'rows;
            }
        }
        for link in 1..=2 {
            out.push(testkit::mid(SIZE, end, y + link));
            if out.len() == n {
                break 'rows;
            }
        }
        y += 3;
        right = !right;
    }
    assert_eq!(out.len(), n);
    out
}

/// Kills the first of a chain of `n` barrels; returns how many died.
fn run_chain(n: usize) -> usize {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 11);
    let ids: Vec<ObjectId> = path(n)
        .into_iter()
        .map(|c| testkit::spawn(&mut w, Kind::Vehicle, "BARREL", 0, c, 0))
        .collect();
    let lethal = Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    };
    damage::apply(&mut w, ids[0], lethal);
    ids.iter()
        .filter(|&&id| w.obj(id).is_none_or(|o| o.health <= 0))
        .count()
}

// spec: S-0312 claims 1 and 6 (the death weapon is set off inside the killing hit), claim 33 (no depth limit, the same on every machine) and S-0303 claim 1 (a nested detonation's effects come before the outer detonation's later hits): a chain of any length resolves whole, the same on every stack
#[test]
// The thread belongs to the test harness, not to the simulation, which runs
// on it alone; a stack overflow aborts the process.
#[allow(clippy::disallowed_types)]
fn a_long_death_weapon_chain_resolves_on_a_small_stack() {
    const N: usize = 400;
    let dead = std::thread::Builder::new()
        .stack_size(1 << 20)
        .spawn(|| run_chain(N))
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(dead, N, "the whole chain dies inside the one hit");
}
