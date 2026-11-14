//! S-0205 claim 1 (sim batch S13, #258): an integer result outside the
//! range wraps; the power and sale topics state no clamp for the blackout
//! end tick (S-0405 claim 6), a building's output and drain sums (S-0405
//! F1, F2) or the adjusted cost (S-0408 F2). Every build profile gives the
//! wrapped result: these tests run in the dev profile, which checks
//! overflow. Synthetic data with extreme invented values.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::{power, sell};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const SIZE: i32 = 24;

fn building(power: i32, cost: i32) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(400),
        cost: Some(cost),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    b.econ.power = Some(power);
    b
}

/// GEN (output 2147483647) with its upgrade UPGEN (output 5); SINK
/// (drain 2147483647) with its upgrade UPSINK (drain 5); FREE (cost
/// −2000000000, free unit CART of cost 2000000000).
fn rules(one_short: bool) -> Ruleset {
    let mut r = testkit::ruleset();
    r.globals.quirks.blackout_one_short = one_short;
    r.registries.insert(
        "buildings".into(),
        ["GEN", "UPGEN", "SINK", "UPSINK", "FREE"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    let s = &mut r.sim;
    s.buildings.insert("GEN".into(), building(i32::MAX, 100));
    let mut up = building(5, 100);
    up.econ.powers_up_building = Some("GEN".into());
    s.buildings.insert("UPGEN".into(), up);
    s.buildings.insert("SINK".into(), building(-i32::MAX, 100));
    let mut up = building(-5, 100);
    up.econ.powers_up_building = Some("SINK".into());
    s.buildings.insert("UPSINK".into(), up);
    let mut free = building(0, -2_000_000_000);
    free.econ.free_unit = Some("CART".into());
    s.buildings.insert("FREE".into(), free);
    s.vehicles.get_mut("CART").unwrap().cost = Some(2_000_000_000);
    r
}

fn world(one_short: bool) -> World {
    let r = rules(one_short);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

/// A building of type `ty` holding one upgrade of type `up`.
fn upgraded(w: &mut World, ty: &str, up: &str) -> ObjectId {
    let id = testkit::spawn(w, Kind::Building, ty, 0, testkit::mid(SIZE, 0, 0), 0);
    let u = w.rules().index_of(Kind::Building, up).unwrap();
    let b = &mut w.obj_mut(id).unwrap().econ.building;
    b.upgrades = [Some(u), None, None];
    b.upgrade_count = 1;
    id
}

// spec: S-0205 claim 1 (an integer result outside the range wraps); S-0405 claim 6 (quirks.blackout_one_short: the end tick t₀ + d − 1)
#[test]
fn the_blackout_end_wraps_with_the_switch_on() {
    let mut w = world(true);
    let t0 = w.tick;
    assert!(t0 >= 2, "t₀ + d − 1 overflows");
    power::start_blackout(&mut w, HouseId(0), i32::MAX);
    let end = t0.wrapping_add(i32::MAX).wrapping_sub(1);
    assert_eq!(power::blackout_running(&w, HouseId(0)), w.tick <= end);
}

// spec: S-0205 claim 1 (an integer result outside the range wraps); S-0405 Edge cases (quirks.blackout_one_short off: d house updates from the first one after the start)
#[test]
fn the_blackout_end_wraps_with_the_switch_off() {
    let mut w = world(false);
    let t0 = w.tick;
    power::start_blackout(&mut w, HouseId(0), i32::MAX);
    // Whether the first house update after the start is t₀ or t₀ + 1, the
    // end tick wraps to a negative number, so no blackout runs.
    for first in [t0, t0.wrapping_add(1)] {
        assert!(first.wrapping_add(i32::MAX) < 0);
    }
    assert!(!power::blackout_running(&w, HouseId(0)));
}

// spec: S-0205 claim 1 (an integer result outside the range wraps); S-0405 F1 (P = base output + Σ base output of the held upgrades; at full health the output is P)
#[test]
fn a_building_output_sum_wraps() {
    let mut w = world(true);
    let id = upgraded(&mut w, "GEN", "UPGEN");
    assert_eq!(power::building_output(&w, id), i32::MAX.wrapping_add(5));
}

// spec: S-0205 claim 1 (an integer result outside the range wraps); S-0405 F2 (base drain + Σ base drain of the held upgrades)
#[test]
fn a_building_drain_sum_wraps() {
    let mut w = world(true);
    let id = upgraded(&mut w, "SINK", "UPSINK");
    assert_eq!(power::building_drain(&w, id), i32::MAX.wrapping_add(5));
}

// spec: S-0205 claim 1 (an integer result outside the range wraps); S-0405 claim 7 (the house's output O sums its counted buildings' outputs)
#[test]
fn the_house_output_sum_wraps() {
    let mut w = world(true);
    let a = testkit::spawn(
        &mut w,
        Kind::Building,
        "GEN",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    let b = testkit::spawn(
        &mut w,
        Kind::Building,
        "GEN",
        0,
        testkit::mid(SIZE, 3, 0),
        0,
    );
    let expect = power::building_output(&w, a).wrapping_add(power::building_output(&w, b));
    testkit::run(&mut w, 2);
    assert_eq!(w.houses[0].econ.power.output, expect);
}

// spec: S-0205 claim 1 (an integer result outside the range wraps); S-0408 F2 (C = max(0, K − d − F) with a free unit)
#[test]
fn the_adjusted_cost_wraps() {
    let w = world(true);
    let b = w.rules().index_of(Kind::Building, "FREE").unwrap();
    let expect = (-2_000_000_000i32).wrapping_sub(2_000_000_000).max(0);
    assert_eq!(sell::adjusted_cost(&w, b), expect);
}
