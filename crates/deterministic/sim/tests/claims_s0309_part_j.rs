//! S-0309 claim 25 part j (r133): the two anger rules of a test-only
//! computer-controlled player house, at the start of each of its checks
//! (S-0217 claim 2), and `quirks.anger_to_first_house` in both positions.
//! Synthetic data: houses H (human, house 0), Q (house 1) and P (house 2),
//! none allied, as in S-0309 scenario A9 part f; Q and P are test-only
//! computer-controlled player houses. Every name and number is invented.

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Country;
use glasnost_sim::computer;
use glasnost_sim::rules::SimRules;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, World};

const SIZE: i32 = 40;
const H: HouseId = HouseId(0);
const Q: HouseId = HouseId(1);
const P: HouseId = HouseId(2);

/// The testkit's rules with Gammaland (not passive) and Calmland
/// (multiplayer-passive).
fn ruleset() -> Ruleset {
    let mut r = testkit::ruleset();
    for (name, passive) in [("Gammaland", false), ("Calmland", true)] {
        r.registries
            .entry("countries".into())
            .or_default()
            .push(name.to_string());
        r.sim.countries.insert(
            name.into(),
            Country {
                multiplay_passive: Some(passive),
                ..Default::default()
            },
        );
    }
    r
}

/// A sandbox game of H, Q and P on an empty map with `r`. P's country is
/// `p_country`; Q and P are test-only computer-controlled player houses.
/// Base records: P at (10, 10), Q next to it at (12, 10), H far away at
/// (30, 30).
fn game(r: &Ruleset, p_country: &str) -> World {
    let mut s = testkit::setup(5);
    s.houses[1].human = false;
    s.houses.push(HouseSetup {
        name: "P".into(),
        country: p_country.into(),
        human: false,
        start: None,
        role: HouseRole::Player,
        ai: false,
    });
    let rules = Arc::new(SimRules::new(r, "synthetic-test"));
    let mut w =
        glasnost_sim::load::new_game(rules, &testkit::empty_map(SIZE), testkit::catalogue(), &s)
            .expect("synthetic test game loads");
    for (h, c) in [(P, (10, 10)), (Q, (12, 10)), (H, (30, 30))] {
        w.houses[usize::from(h.0)].base.start = Some(CellXY::new(c.0, c.1));
    }
    w
}

fn standard() -> World {
    game(&ruleset(), "Gammaland")
}

fn quirk_off() -> World {
    let mut r = ruleset();
    r.globals.quirks.anger_to_first_house = false;
    game(&r, "Gammaland")
}

/// Runs P's check at tick `t` (its delay run out).
fn check(w: &mut World, t: i32) {
    w.tick = t;
    w.houses[usize::from(P.0)].checks.next = t;
    computer::house_update(w, P);
}

fn anger(w: &World, of: HouseId, to: HouseId) -> i32 {
    w.houses[usize::from(of.0)].combat.anger[usize::from(to.0)]
}

fn enemy(w: &World, of: HouseId) -> Option<HouseId> {
    w.houses[usize::from(of.0)].combat.enemy
}

// spec: S-0309 claim 25 part j (anger towards the first house: with no enemy house and a base record, the anger towards the first house in house order that is not itself, not passive and not defeated grows by 1 at the start of each check, then claim 16's recomputation; how far away it is plays no part; quirks.anger_to_first_house on)
#[test]
fn part_j_anger_goes_to_the_first_house() {
    let mut w = standard();
    assert!(computer::makes_checks(&w, P));
    assert_eq!(enemy(&w, P), None);
    check(&mut w, 0);
    assert_eq!(anger(&w, P, H), 1);
    assert_eq!(anger(&w, P, Q), 0);
    assert_eq!(enemy(&w, P), Some(H), "claim 16's recomputation");
    // Q's first house is H too.
    w.houses[usize::from(Q.0)].checks.next = 0;
    computer::house_update(&mut w, Q);
    assert_eq!(anger(&w, Q, H), 1);
    assert_eq!(enemy(&w, Q), Some(H));
    // With an enemy house the rule adds nothing at the next check.
    check(&mut w, 120);
    assert_eq!(anger(&w, P, H), 1);
    // H makes no checks and gains no anger.
    computer::house_update(&mut w, H);
    assert_eq!(w.houses[0].combat.anger, vec![0, 0, 0]);
}

// spec: S-0309 Edge cases (quirks.anger_to_first_house off: the nearest such house by the straight-line horizontal distance between the cells of the base records, the base centre before the base-start cell)
#[test]
fn part_j_quirk_off_anger_goes_to_the_nearest_house() {
    let mut w = quirk_off();
    check(&mut w, 0);
    assert_eq!(anger(&w, P, Q), 1);
    assert_eq!(anger(&w, P, H), 0);
    assert_eq!(enemy(&w, P), Some(Q));
    // A recorded base centre comes before the base-start cell.
    let mut w = quirk_off();
    w.houses[0].base.centre = Some(CellXY::new(11, 11));
    check(&mut w, 0);
    assert_eq!((anger(&w, P, H), anger(&w, P, Q)), (1, 0));
}

// spec: S-0309 Edge cases (quirks.anger_to_first_house off: ties go to the earlier house in house order; a house with neither record is left out, and with every house left out no anger is added)
#[test]
fn part_j_quirk_off_ties_and_houses_without_records() {
    let mut w = quirk_off();
    w.houses[0].base.start = Some(CellXY::new(8, 10));
    check(&mut w, 0);
    assert_eq!((anger(&w, P, H), anger(&w, P, Q)), (1, 0), "a tie");
    let mut w = quirk_off();
    w.houses[1].base.start = None;
    check(&mut w, 0);
    assert_eq!((anger(&w, P, H), anger(&w, P, Q)), (1, 0), "Q left out");
    let mut w = quirk_off();
    w.houses[0].base.start = None;
    w.houses[1].base.start = None;
    check(&mut w, 0);
    assert_eq!((anger(&w, P, H), anger(&w, P, Q)), (0, 0));
    assert_eq!(enemy(&w, P), None);
}

// spec: S-0309 claim 25 part j (the house's own conditions: its country not multiplayer-passive, no enemy house, a recorded base centre or base-start cell)
#[test]
fn part_j_the_house_must_be_active_without_an_enemy_and_with_a_base_record() {
    let mut w = game(&ruleset(), "Calmland");
    check(&mut w, 0);
    assert_eq!(w.houses[2].combat.anger, vec![0, 0, 0], "a passive country");
    let mut w = standard();
    w.houses[2].base.start = None;
    check(&mut w, 0);
    assert_eq!(w.houses[2].combat.anger, vec![0, 0, 0], "no base record");
    let mut w = standard();
    w.houses[2].base.start = None;
    w.houses[2].base.centre = Some(CellXY::new(10, 10));
    check(&mut w, 0);
    assert_eq!(anger(&w, P, H), 1, "a base centre is a record");
    let mut w = standard();
    w.houses[2].combat.anger[1] = 5;
    w.houses[2].combat.enemy = Some(Q);
    check(&mut w, 0);
    assert_eq!(anger(&w, P, H), 0, "an enemy house");
}

// spec: S-0309 claim 25 part j (the first house that is not itself, not passive and not defeated; S-0009 claim 28: a house the cleanup removed is no longer in the game)
#[test]
fn part_j_passive_defeated_and_removed_houses_are_skipped() {
    let mut w = game(&ruleset(), "Gammaland");
    w.houses[0].econ.result.defeated = true;
    check(&mut w, 0);
    assert_eq!((anger(&w, P, H), anger(&w, P, Q)), (0, 1), "H defeated");
    let mut w = standard();
    w.houses[0].removed = true;
    check(&mut w, 0);
    assert_eq!((anger(&w, P, H), anger(&w, P, Q)), (0, 1), "H removed");
    let mut r = ruleset();
    r.sim
        .countries
        .get_mut("Alphaland")
        .expect("the testkit's country")
        .multiplay_passive = Some(true);
    let mut w = game(&r, "Gammaland");
    check(&mut w, 0);
    assert_eq!((anger(&w, P, H), anger(&w, P, Q)), (0, 1), "H passive");
}

// spec: S-0309 claim 25 part j and claim 16 (an allied first house gains the anger but does not become the enemy house, so the house gains anger towards it at every check)
#[test]
fn part_j_an_allied_first_house_gains_anger_and_is_never_the_enemy() {
    let mut w = standard();
    w.houses[2].allies = vec![H];
    check(&mut w, 0);
    check(&mut w, 120);
    check(&mut w, 240);
    assert_eq!(anger(&w, P, H), 3);
    assert_eq!(enemy(&w, P), None);
}

// spec: S-0309 claim 25 part j (anger towards the first house: a wrapping addition of 1 point, F8)
#[test]
fn part_j_the_addition_wraps() {
    let mut w = standard();
    w.houses[2].allies = vec![H];
    w.houses[2].combat.anger[0] = i32::MAX;
    check(&mut w, 0);
    assert_eq!(anger(&w, P, H), i32::MIN);
}

// spec: S-0309 claim 25 part j (defeated enemy house: its anger towards it is removed and it has no enemy house, even with an anger above 0 towards another house, until an anger value next changes; at the next check the first rule makes such a change; the first rule comes first, so a house whose enemy is defeated gains no anger at that check)
#[test]
fn part_j_a_defeated_enemy_house_is_cleared() {
    let mut w = standard();
    w.houses[2].combat.anger[1] = 7;
    w.houses[2].combat.anger[0] = 3;
    w.houses[2].combat.enemy = Some(Q);
    w.houses[1].econ.result.defeated = true;
    check(&mut w, 0);
    assert_eq!(anger(&w, P, Q), 0, "removed");
    assert_eq!(anger(&w, P, H), 3, "no anger at this check");
    assert_eq!(
        enemy(&w, P),
        None,
        "H's anger above 0 does not make it the enemy"
    );
    check(&mut w, 120);
    assert_eq!(anger(&w, P, H), 4);
    assert_eq!(enemy(&w, P), Some(H));
}

// spec: S-0309 claim 25 part j (the defeated-enemy rule holds whatever the country and the base records)
#[test]
fn part_j_the_defeated_enemy_rule_needs_no_base_record() {
    let mut w = game(&ruleset(), "Calmland");
    w.houses[2].base.start = None;
    w.houses[2].combat.anger[1] = 7;
    w.houses[2].combat.enemy = Some(Q);
    w.houses[1].econ.result.defeated = true;
    check(&mut w, 0);
    assert_eq!(anger(&w, P, Q), 0);
    assert_eq!(enemy(&w, P), None);
}

// spec: S-0309 claim 25 part j (neither rule takes a draw: the check's only draw is S-0217 claim 2's R(1, 7)) and S-0217 claim 2 (only at a check)
#[test]
fn part_j_no_draw_and_only_at_a_check() {
    let mut w = standard();
    let mut replay = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    replay.ranged(1, 7, half);
    check(&mut w, 0);
    assert_eq!(w.rng, replay);
    let mut w = standard();
    w.houses[2].checks.next = 500;
    w.tick = 499;
    computer::house_update(&mut w, P);
    assert_eq!(anger(&w, P, H), 0, "no check before the delay runs out");
}
