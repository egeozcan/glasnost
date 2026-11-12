//! S-0312 r74 claim 33 (Q-3000 answered), scenario D25: death weapons
//! inside death weapons nest depth first with no depth limit, the same on
//! every machine. A row of exploding vehicles, each within reach of the
//! next one's death weapon only, dies inside the one killing hit of the
//! first, and a vehicle S next to the first, which the first one's
//! detonation hits after the second, dies after the last of the row.
//! Synthetic data only: an invented exploding vehicle BARREL and death
//! weapon on the testkit's combat rules.
//!
//! The runs use a small thread stack: a stack overflow aborts the whole
//! test process.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LocomotionClass, Locomotor, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::combat::effects::Effect;
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
            // D25: a reach of 320 leptons (S-0303 claim 3), so a straight
            // neighbour (256) and no diagonal one (about 362).
            cell_spread: dec("1.25"),
            percent_at_max: dec("1"),
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            ..Default::default()
        },
    );
    r
}

/// D25's path of `n` cells: rows along the x axis two cells apart, the
/// first running from the first cell toward decreasing x, each pair of rows
/// joined at alternate ends by one cell between them.
fn path(n: usize) -> Vec<CellXY> {
    let mut out = Vec::new();
    let mut y = -HALF;
    let mut left = true;
    'rows: loop {
        let xs: Vec<i32> = if left {
            (-HALF..=HALF).rev().collect()
        } else {
            (-HALF..=HALF).collect()
        };
        let end = *xs.last().unwrap();
        for x in xs {
            out.push(testkit::mid(SIZE, x, y));
            if out.len() == n {
                break 'rows;
            }
        }
        out.push(testkit::mid(SIZE, end, y + 1));
        if out.len() == n {
            break;
        }
        y += 2;
        left = !left;
        assert!(y <= HALF, "the map is too small for {n}");
    }
    out
}

/// Kills the first of a path of `n` barrels with S beside it; returns how
/// many of the path died, whether S died, and the order of the last
/// barrel's and S's killing hits.
fn run_d25(n: usize) -> (usize, bool, Option<(usize, usize)>) {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 11);
    let cells = path(n);
    let first = cells[0];
    let ids: Vec<ObjectId> = cells
        .into_iter()
        .map(|c| testkit::spawn(&mut w, Kind::Vehicle, "BARREL", 0, c, 0))
        .collect();
    let s_cell = CellXY::new(first.x + 1, first.y);
    let s = testkit::spawn(&mut w, Kind::Vehicle, "BARREL", 0, s_cell, 0);
    w.effects.clear();
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
    let dead = |id: ObjectId| w.obj(id).is_none_or(|o| o.health <= 0);
    let path_dead = ids.iter().filter(|&&id| dead(id)).count();
    let killed_at = |who: ObjectId| {
        w.effects.iter().position(|e| {
            matches!(e, Effect::Hit { id, result: HitResult::NowDead, .. } if *id == who)
        })
    };
    let last = *ids.last().unwrap();
    let order = killed_at(last).zip(killed_at(s));
    (path_dead, dead(s), order)
}

// The thread belongs to the test harness, not to the simulation, which runs
// on it alone.
#[allow(clippy::disallowed_types)]
fn on_small_stack(n: usize) -> (usize, bool, Option<(usize, usize)>) {
    std::thread::Builder::new()
        .stack_size(1 << 20)
        .spawn(move || run_d25(n))
        .unwrap()
        .join()
        .unwrap()
}

fn check(n: usize) {
    let (path_dead, s_dead, order) = on_small_stack(n);
    assert_eq!(path_dead, n, "the whole row dies inside the one hit");
    assert!(s_dead, "S dies inside the one hit");
    let (last, s) = order.expect("both killing hits");
    assert!(
        last < s,
        "the last of the row dies before S (killing hits at {last} and {s})"
    );
}

// spec: S-0312 r74 claim 33 and scenario D25 part a (N = 50: every death nested before a later hit of an outer detonation completes before it, at every depth)
#[test]
fn d25_a_row_of_50_nests_depth_first() {
    check(50);
}

// spec: S-0312 r74 claim 33 and scenario D25 part b (N = 600: no depth limit, the same on every machine)
#[test]
fn d25_b_row_of_600_nests_depth_first() {
    check(600);
}
