//! S-0205 claim 1 (sim batch S20, #381): an integer result outside the
//! range wraps. The delayed-kill radius (S-0302 F7), the price sums
//! (S-0404 claim 2) and the far cells of the flying-object search
//! (S-0303 F7) state no clamp, so every build profile gives the wrapped
//! result: these tests run in the dev profile, which checks overflow.
//! Synthetic data with extreme invented values.

mod common;
mod econ_common;

use econ_common::*;
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::sim::{Decimal, WarheadType};
use glasnost_sim::combat::{damage, flying, veterancy};
use glasnost_sim::econ::production;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::walker::{self, Walker};
use glasnost_sim::world::{HouseId, Pos};

// spec: S-0302 F7 (R′ = 256 × trunc(s)) with S-0205 claim 1 (an integer result outside the range wraps)
#[test]
fn delay_kill_radius_wraps_for_a_huge_spread() {
    let mut r = testkit::combat_ruleset();
    r.sim.warheads.insert(
        "WIDE".into(),
        WarheadType {
            causes_delay_kill: Some(true),
            delay_kill_frames: Some(100),
            delay_kill_at_max: Some(Decimal("0.5".into())),
            cell_spread: Some(Decimal("9000000".into())),
            ..Default::default()
        },
    );
    let w = testkit::game_with(&r, 24, 1);
    let h = w
        .rules()
        .combat
        .warheads
        .iter()
        .find(|h| h.name == "WIDE")
        .expect("the warhead loads, no range check")
        .clone();
    assert_eq!(h.spread.to_i32_common(), 9_000_000);
    // At r = 0 the distance term is 0 for the wrapped R′ = −1990967296, so K = F.
    let k = damage::delay_kill_countdown(&w, h.delay_kill_frames, h.delay_kill_at_max, h.spread, 0);
    assert_eq!(k, 100);
}

// spec: S-0404 Formulas (credit amounts are signed 32-bit integers, arithmetic wraps: S-0205 claim 1)
#[test]
fn pad_term_sum_wraps() {
    let mut r = ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("DOCK".into());
    r.sim.buildings.insert("DOCK".into(), plain(3000));
    for n in ["PADA", "PADB"] {
        r.registries.get_mut("aircraft").unwrap().push(n.into());
        let mut a = r.sim.aircraft["PLANE"].clone();
        a.cost = Some(1_000_000_000);
        a.econ.dock = Some(vec!["DOCK".into()]);
        r.sim.aircraft.insert(n.into(), a);
    }
    r.sim.general.economy.pad_aircraft = Some(vec!["PADA".into(), "PADB".into()]);
    r.sim
        .countries
        .get_mut("Alphaland")
        .unwrap()
        .econ
        .cost_aircraft_mult = dec("1.5");
    let w = game(&r);
    let dock = t(&w, Kind::Building, "DOCK");
    // C' = 3000 - (10^9 + 10^9) / 2 = -999997000; P1 = P2 = 1500000000;
    // P1 + P2 = 3000000000 wraps to -1294967296, halved -647483648.
    assert_eq!(
        production::price(&w, HouseId(0), dock),
        -999_997_000 + -647_483_648
    );
}

// spec: S-0404 Formulas, claim 2 (own part + free unit price, 32-bit wrap, then limited below by 0)
#[test]
fn free_unit_sum_wraps() {
    let mut r = ruleset();
    let mut b = plain(2_000_000_000);
    b.econ.free_unit = Some("FREEV".into());
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("GIFTED".into());
    r.sim.buildings.insert("GIFTED".into(), b);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("FREEV".into());
    r.sim.vehicles.insert("FREEV".into(), wheel(1_000_000_000));
    r.sim
        .countries
        .get_mut("Alphaland")
        .unwrap()
        .econ
        .cost_units_mult = dec("1.5");
    let w = game(&r);
    // C' = 10^9, own part 10^9, free unit 1500000000; the sum 2500000000 wraps
    // to -1794967296, limited below by 0.
    assert_eq!(
        production::price(&w, HouseId(0), t(&w, Kind::Building, "GIFTED")),
        0
    );
}

// spec: S-0615 F1 (frame = R × (7 − d) + (⌊t / 3⌋ mod R)) with S-0205 claim 1 (an integer result outside the range wraps)
#[test]
fn walker_frame_wraps_for_huge_running_frames() {
    let at = |x, y| Pos::centre(CellXY::new(x, y), 0);
    let big = i32::MAX / 2;
    let mut checked = 0;
    for (dx, dy) in [
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
    ] {
        let wk = Walker {
            anim: 0,
            pos: at(10, 10),
            dest: Some(at(10 + dx, 10 + dy)),
            retries: 0,
            expiring: false,
            stage: 0,
            falling: false,
            rate: 0,
            acc: F64::ZERO,
        };
        let table = true;
        // With R = 1 and t = 0 the frame is 7 − d.
        let k = walker::frame(&wk, 1, 0, table);
        assert_eq!(walker::frame(&wk, big, 0, table), big.wrapping_mul(k));
        if big.checked_mul(k).is_none() {
            checked += 1;
        }
    }
    assert!(checked > 0, "some direction overflows");
}

// spec: S-0311 F2 (V = m × A_v) with S-0205 claim 1 (an integer result outside the range wraps)
#[test]
fn kill_value_wraps_for_a_huge_cost() {
    let mut s = common::Scene::new();
    let mut tank = common::unit_type(common::TRACK, 4, 5);
    tank.cost = Some(1_500_000_000);
    s.vehicle("TANK", tank);
    let mut w = s.build();
    let a = testkit::spawn(&mut w, Kind::Vehicle, "TANK", 0, common::cell(0, 0), 0);
    let b = testkit::spawn(&mut w, Kind::Vehicle, "TANK", 1, common::cell(4, 0), 0);
    // nocite: 1.0 as a single, a veteran experience (S-0311 claim 1)
    w.obj_mut(b).unwrap().combat.experience = glasnost_num::F32(0x3F80_0000);
    let before = w.houses[0].combat.score;
    veterancy::destruction_credited(&mut w, b, Some(a), Some(HouseId(0)));
    // V = 2 × 1500000000 = 3000000000 wraps to −1294967296.
    assert_eq!(
        w.houses[0].combat.score.wrapping_sub(before),
        -1_294_967_296
    );
}

// spec: S-0303 F7 (the far sector of the east run is the sector of the cell ρ cells east) with S-0205 claim 1 (an integer result outside the range wraps)
#[test]
fn flying_search_far_cell_wraps_for_a_huge_radius() {
    let w = testkit::game_with(&testkit::combat_ruleset(), 24, 1);
    // B = 47, sector size 2: the centre cell (20, 20) is in sector (10, 10).
    let c = CellXY::new(20, 20);
    let out = flying::sector_set(&w, c, i32::MAX - 10);
    assert_eq!(out[0], (10, 10));
    // East and south: 20 + ρ wraps below 0, its sector is 0, the runs are empty.
    assert!(!out.contains(&(11, 10)), "no east run: {out:?}");
    assert!(!out.contains(&(10, 11)), "no south run: {out:?}");
    // North and west: 20 − ρ does not wrap, its sector is 0, the runs reach it.
    assert!(out.contains(&(0, 10)), "the west run: {out:?}");
    assert!(out.contains(&(10, 0)), "the north run: {out:?}");
}
