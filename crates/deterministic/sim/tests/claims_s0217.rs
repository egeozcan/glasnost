//! S-0217 (Q-2951 answered with S-0508 r39): the house-wide hunt order
//! (claim 1), the periodic checks of a test-only computer-controlled player
//! house (claim 2), the sale and order without a factory and a recent hit
//! (claim 3) and the order for a house with no building (claim 4).
//! Synthetic data on the garrison fixtures of `structures_common`: houses
//! Alpha (0) and Beta (1), the neutral civilian-side house Civ (2); a test
//! marks Alpha as a test-only computer-controlled player house where it
//! needs the checks (S-0215 Terms). Every name and number is invented.

mod structures_common;

use glasnost_map::triggers::{Action, Event, Triggers};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, UnitType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::computer;
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::garrison::{self, Evict};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};
use structures_common::*;

/// The garrison fixtures' ruleset with YARD (a building factory: its
/// factory kind makes buildings, with a build-up sprite), MCV (a vehicle
/// that deploys into YARD, a construction vehicle) and SELF (a building
/// whose type damages itself).
fn rules() -> Ruleset {
    let mut r = ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .extend(["YARD".to_string(), "SELF".to_string()]);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("MCV".to_string());
    let mut yard = BuildingType {
        strength: Some(400),
        foundation: Some([2, 2]),
        factory: Some("building".into()),
        ..Default::default()
    };
    yard.econ.buildup = Some(true);
    yard.econ.buildup_frames = Some(4);
    r.sim.buildings.insert("YARD".into(), yard);
    let mut selfb = BuildingType {
        strength: Some(400),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    selfb.combat.damage_self = Some(true);
    r.sim.buildings.insert("SELF".into(), selfb);
    let mcv = UnitType {
        deploys_into: Some("YARD".into()),
        ..r.sim.vehicles.get("CART").unwrap().clone()
    };
    r.sim.vehicles.insert("MCV".into(), mcv);
    r
}

fn world() -> World {
    game_on(&rules(), Triggers::default())
}

/// `w` with Alpha marked as a test-only computer-controlled player house
/// (S-0215 Terms) and its next check due at `tick`, with the world at that
/// tick.
fn computer_at(w: &mut World, tick: i32) {
    w.houses[0].human = false;
    w.tick = tick;
    w.houses[0].checks.next = tick;
}

fn beh(w: &World, id: ObjectId) -> (Option<Behaviour>, Option<Behaviour>) {
    let o = w.obj(id).unwrap();
    (o.beh.current, o.beh.queued)
}

/// A hit of `amount` with no warhead, ignoring defences, by `attacker`.
fn hit(amount: i32, attacker: Option<ObjectId>) -> Hit {
    Hit {
        amount,
        warhead: None,
        distance: 0,
        attacker,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

/// A HUT (civilian garrison building) at `(dx, dy)` with `n` GARRI of
/// house `h` inside, owned by `h` after the building's turn (S-0508 claim
/// 7).
fn occupied_hut(w: &mut World, h: u8, n: usize, dx: i32, dy: i32) -> (ObjectId, Vec<ObjectId>) {
    let b = put(w, Kind::Building, "HUT", 2, dx, dy);
    let us: Vec<ObjectId> = (0..n)
        .map(|i| {
            let u = put(w, Kind::Infantry, "GARRI", h, dx - 3, dy + i as i32);
            assert!(garrison::attempt(w, u, b), "went inside");
            u
        })
        .collect();
    garrison::turn(w, b);
    assert_eq!(owner(w, b), Some(HouseId(h)), "S-0508 claim 7");
    (b, us)
}

// ---------------------------------------------------------------------------
// Claim 1

// spec: S-0217 claim 1 (each mobile actor of the house on the map has hunt queued without an immediate start, S-0215 claim 7; the other houses' actors are not reached)
#[test]
fn claim_1_mobile_actors_have_hunt_queued_without_a_start() {
    let mut w = world();
    let inf = put(&mut w, Kind::Infantry, "GARRI", 0, -4, 0);
    let cart = put(&mut w, Kind::Vehicle, "CART", 0, -4, 3);
    let other = put(&mut w, Kind::Infantry, "GARRI", 1, 4, 0);
    computer::hunt_order(&mut w, A);
    assert_eq!(
        beh(&w, inf),
        (Some(Behaviour::Guard), Some(Behaviour::Hunt))
    );
    assert_eq!(
        beh(&w, cart),
        (Some(Behaviour::Guard), Some(Behaviour::Hunt))
    );
    assert_eq!(beh(&w, other), (Some(Behaviour::Guard), None));
}

// spec: S-0217 claim 1 (a building with occupants has them evicted with hunting and without inside fallback, S-0508 claims 11 and 13; occupants are reached only through a building the house owns at that moment)
#[test]
fn claim_1_occupants_come_out_hunting_only_from_an_owned_building() {
    let mut w = world();
    let (b, us) = occupied_hut(&mut w, 0, 2, 0, 0);
    // A second hut whose occupant's house does not own it yet: its turn has not come.
    let c = put(&mut w, Kind::Building, "HUT", 2, 6, 6);
    let late = put(&mut w, Kind::Infantry, "GARRI", 0, 3, 6);
    assert!(garrison::attempt(&mut w, late, c));
    assert_eq!(owner(&w, c), Some(CIV));
    computer::hunt_order(&mut w, A);
    for &u in &us {
        assert!(on_map(&w, u), "put out");
        assert_eq!(w.obj(u).unwrap().beh.queued, Some(Behaviour::Hunt));
    }
    assert!(occ(&w, b).is_empty());
    assert!(!on_map(&w, late), "a building Civ owns keeps its occupant");
    assert_eq!(occ(&w, c), vec![late]);
}

// spec: S-0217 claim 1 (the house's buildings with occupants are reached in reverse creation order: the occupants of the one created later are put out first and take the earlier draws)
#[test]
fn claim_1_the_building_created_later_is_evicted_first() {
    let mut w = world();
    let (early, _) = occupied_hut(&mut w, 0, 2, -4, -4);
    let (late, _) = occupied_hut(&mut w, 0, 2, 5, 5);
    assert!(early < late);
    let mut ordered = w.clone();
    garrison::evict(&mut ordered, late, Evict::HUNT);
    garrison::evict(&mut ordered, early, Evict::HUNT);
    let mut wrong = w.clone();
    garrison::evict(&mut wrong, early, Evict::HUNT);
    garrison::evict(&mut wrong, late, Evict::HUNT);
    computer::hunt_order(&mut w, A);
    let hash = glasnost_sim::save::state_hash;
    assert_eq!(hash(&w), hash(&ordered));
    assert_ne!(hash(&w), hash(&wrong), "the order shows in the draws");
}

// spec: S-0217 claim 1 and S-0011 claim 23 (map action code 6 gives the house-wide hunt order to the house its number slot resolves to: units outside garrisons have hunt queued too, S-0508 claim 13 r39)
#[test]
fn claim_1_map_action_6_reaches_units_outside_garrisons() {
    let ev = Event {
        code: 1,
        kind: 0,
        value: -1,
        type_name: None,
    };
    // House value 0: the first house playing country 0 (Alpha).
    let t = trigger(
        "HUNT",
        ev,
        vec![Action {
            code: 6,
            number: 0,
            ..Default::default()
        }],
    );
    let tr = Triggers {
        tags: vec![tag_def(0, &t, 2)],
        triggers: vec![t],
        ..Default::default()
    };
    let mut w = game_on(&rules(), tr);
    let out = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 6);
    let enemy = put(&mut w, Kind::Infantry, "GARRI", 1, 6, 6);
    let b = put(&mut w, Kind::Building, "HUT", 2, 0, 0);
    tag(&mut w, b, "HUNT");
    // An entry raises the tag.
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -3, 0);
    garrison::attempt(&mut w, u, b);
    assert_eq!(
        beh(&w, out),
        (Some(Behaviour::Guard), Some(Behaviour::Hunt))
    );
    assert_eq!(beh(&w, enemy), (Some(Behaviour::Guard), None));
}

// ---------------------------------------------------------------------------
// Claim 2

// spec: S-0217 claim 2 and Formulas (a check sets the next check to t + 105 + R, R one R(1, 7) draw on the scenario stream, the check's only draw of its own when its order evicts no occupant)
#[test]
fn claim_2_one_draw_sets_the_next_check() {
    let mut w = world();
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -4, 0);
    computer_at(&mut w, 300);
    let mut replay: Rng = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let r = replay.ranged(1, 7, half);
    computer::house_update(&mut w, A);
    assert_eq!(w.rng, replay, "one R(1, 7) draw and no other");
    assert_eq!(w.houses[0].checks.next, 300 + 105 + r);
    assert_eq!(
        w.obj(u).unwrap().beh.queued,
        Some(Behaviour::Hunt),
        "claim 4"
    );
    // Before the delay runs out: no check, no draw.
    let before = w.rng.clone();
    w.tick = 301;
    w.obj_mut(u).unwrap().beh.queued = None;
    computer::house_update(&mut w, A);
    assert_eq!(w.rng, before);
    assert_eq!(w.obj(u).unwrap().beh.queued, None);
}

// spec: S-0217 claim 2 (a check in the house update of tick 0, then every 106 to 112 ticks)
#[test]
fn claim_2_checks_from_tick_0_every_106_to_112_ticks() {
    let mut w = {
        let mut map = testkit::empty_map(SIZE);
        map.triggers = Triggers::default();
        let mut s = testkit::setup(11);
        s.houses[0].human = false;
        let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(
            &rules(),
            "synthetic-test",
        ));
        glasnost_sim::load::new_game(rules, &map, testkit::catalogue(), &s).unwrap()
    };
    assert_eq!(w.houses[0].checks.next, 0);
    let mut checks = Vec::new();
    let mut next = w.houses[0].checks.next;
    while w.tick < 1000 {
        let t = w.tick;
        testkit::run(&mut w, 1);
        if w.houses[0].checks.next != next {
            checks.push(t);
            next = w.houses[0].checks.next;
        }
    }
    assert_eq!(checks.first(), Some(&0));
    assert!(checks.len() >= 8);
    for pair in checks.windows(2) {
        assert!((106..=112).contains(&(pair[1] - pair[0])), "{checks:?}");
    }
    // Beta is human-controlled: no checks.
    assert_eq!(w.houses[1].checks.next, 0);
}

// spec: S-0217 claim 2 (human-controlled houses, our AI houses among them, and the neutral house make no checks)
#[test]
fn claim_2_no_check_for_other_houses() {
    let mut w = world();
    assert!(!computer::makes_checks(&w, A));
    assert!(!computer::makes_checks(&w, B));
    assert!(!computer::makes_checks(&w, CIV));
    let u = put(&mut w, Kind::Infantry, "GARRI", 2, 4, 4);
    let before = w.rng.clone();
    computer::house_update(&mut w, A);
    computer::house_update(&mut w, B);
    computer::house_update(&mut w, CIV);
    assert_eq!(w.rng, before);
    assert_eq!(w.obj(u).unwrap().beh.queued, None);
    w.houses[0].human = false;
    assert!(computer::makes_checks(&w, A));
}

// ---------------------------------------------------------------------------
// Claim 3

// spec: S-0217 claim 3 (L becomes the current tick when a building of the house is hit by damage that has an attacker, whatever the attacker's house) and Q-2986 (interim: a hit that lowers the building's health)
#[test]
fn claim_3_a_hit_with_an_attacker_sets_the_last_base_hit_tick() {
    let mut w = world();
    let b = put(&mut w, Kind::Building, "TARGET", 0, 0, 0);
    let enemy = put(&mut w, Kind::Infantry, "GARRI", 1, 5, 5);
    let own = put(&mut w, Kind::Infantry, "GARRI", 0, -5, 5);
    w.tick = 50;
    damage::apply(&mut w, b, hit(10, None));
    assert_eq!(w.houses[0].checks.last_base_hit, 0, "no attacker");
    damage::apply(&mut w, b, hit(10, Some(enemy)));
    assert_eq!(w.houses[0].checks.last_base_hit, 50);
    w.tick = 70;
    damage::apply(&mut w, b, hit(10, Some(own)));
    assert_eq!(w.houses[0].checks.last_base_hit, 70, "the house's own unit");
    w.tick = 90;
    damage::apply(&mut w, b, hit(-10, Some(enemy)));
    assert_eq!(
        w.houses[0].checks.last_base_hit, 70,
        "Q-2986 interim: a healing hit"
    );
    assert_eq!(w.houses[1].checks.last_base_hit, 0);
}

// spec: S-0217 claim 3 and Import mapping (a building whose type undeploys into a vehicle and covers one cell does not change L; the building's own damage changes it only when its type damages itself)
#[test]
fn claim_3_vehicle_like_and_self_damage() {
    let mut w = world();
    let v = put(&mut w, Kind::Building, "VLIKE", 0, 0, 0);
    let plain = put(&mut w, Kind::Building, "TARGET", 0, 4, 4);
    let selfb = put(&mut w, Kind::Building, "SELF", 0, -4, -4);
    let enemy = put(&mut w, Kind::Infantry, "GARRI", 1, 6, 0);
    w.tick = 40;
    damage::apply(&mut w, v, hit(10, Some(enemy)));
    assert_eq!(w.houses[0].checks.last_base_hit, 0, "vehicle-like");
    damage::apply(&mut w, plain, hit(10, Some(plain)));
    assert_eq!(
        w.houses[0].checks.last_base_hit, 0,
        "own damage, no self-damage mark"
    );
    damage::apply(&mut w, selfb, hit(10, Some(selfb)));
    assert_eq!(w.houses[0].checks.last_base_hit, 40, "self-damage mark");
}

// spec: S-0217 claim 3 (not held and no factory building: every own building on the map with health left receives a sell order, S-0408 claim 4, then the house-wide hunt order, in the same house update; whatever construction vehicles it owns)
#[test]
fn claim_3_sale_then_order_when_not_held() {
    let mut w = world();
    let b = put(&mut w, Kind::Building, "TARGET", 0, 0, 0);
    let mcv = put(&mut w, Kind::Vehicle, "MCV", 0, -6, 0);
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 4);
    computer_at(&mut w, 901);
    computer::house_update(&mut w, A);
    assert_eq!(w.obj(b).unwrap().beh.current, Some(Behaviour::Selling));
    assert_eq!(w.obj(u).unwrap().beh.queued, Some(Behaviour::Hunt));
    assert_eq!(w.obj(mcv).unwrap().beh.queued, Some(Behaviour::Hunt));
}

// spec: S-0217 claim 3 and Formulas (held at t <= L + 900: no sale; a house that owns a building other than an occupied civilian one gets no order from claim 4 either)
#[test]
fn claim_3_held_until_l_plus_900() {
    let mut w = world();
    let b = put(&mut w, Kind::Building, "TARGET", 0, 0, 0);
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 4);
    w.houses[0].checks.last_base_hit = 400;
    computer_at(&mut w, 1300);
    computer::house_update(&mut w, A);
    assert_ne!(w.obj(b).unwrap().beh.current, Some(Behaviour::Selling));
    assert_eq!(w.obj(u).unwrap().beh.queued, None);
    computer_at(&mut w, 1301);
    computer::house_update(&mut w, A);
    assert_eq!(w.obj(b).unwrap().beh.current, Some(Behaviour::Selling));
    assert_eq!(w.obj(u).unwrap().beh.queued, Some(Behaviour::Hunt));
}

// spec: S-0217 claim 3 (a house none of whose buildings has been hit is held at every check up to tick 900)
#[test]
fn claim_3_held_up_to_tick_900() {
    let mut w = world();
    let b = put(&mut w, Kind::Building, "TARGET", 0, 0, 0);
    computer_at(&mut w, 900);
    computer::house_update(&mut w, A);
    assert_ne!(w.obj(b).unwrap().beh.current, Some(Behaviour::Selling));
}

// spec: S-0217 claim 3 (a factory building of any kind on the map stops the sale and the order; the factory kind need not make buildings)
#[test]
fn claim_3_a_factory_building_stops_both() {
    let mut w = world();
    let b = put(&mut w, Kind::Building, "TARGET", 0, 0, 0);
    let f = put(&mut w, Kind::Building, "WARF", 0, 5, -5);
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 4);
    computer_at(&mut w, 2000);
    computer::house_update(&mut w, A);
    assert_ne!(w.obj(b).unwrap().beh.current, Some(Behaviour::Selling));
    assert_ne!(w.obj(f).unwrap().beh.current, Some(Behaviour::Selling));
    assert_eq!(w.obj(u).unwrap().beh.queued, None);
}

// ---------------------------------------------------------------------------
// Claim 4

// spec: S-0217 claim 4 (a house with no building and no construction vehicle receives the house-wide hunt order at every check, held or not)
#[test]
fn claim_4_no_building_gives_the_order_while_held() {
    let mut w = world();
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 4);
    computer_at(&mut w, 120);
    computer::house_update(&mut w, A);
    assert_eq!(w.obj(u).unwrap().beh.queued, Some(Behaviour::Hunt));
}

// spec: S-0217 claim 4 (a house that owns a construction vehicle does not get claim 4's order; held, claim 3 gives none either)
#[test]
fn claim_4_a_construction_vehicle_stops_the_order_while_held() {
    let mut w = world();
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 4);
    put(&mut w, Kind::Vehicle, "MCV", 0, -6, 0);
    computer_at(&mut w, 120);
    computer::house_update(&mut w, A);
    assert_eq!(w.obj(u).unwrap().beh.queued, None);
}

// spec: S-0217 claim 4 (occupied civilian buildings do not count: the occupiers come out with hunt at a held check, and the building receives no sell order while held)
#[test]
fn claim_4_occupiers_come_out_at_a_held_check() {
    let mut w = world();
    let (b, us) = occupied_hut(&mut w, 0, 2, 0, 0);
    computer_at(&mut w, 200);
    computer::house_update(&mut w, A);
    for &u in &us {
        assert!(on_map(&w, u));
        assert_eq!(w.obj(u).unwrap().beh.queued, Some(Behaviour::Hunt));
    }
    assert_ne!(w.obj(b).unwrap().beh.current, Some(Behaviour::Selling));
}

// spec: S-0217 claim 4 and claim 3 (not held and no factory: the occupied civilian building receives the sell order first, then the order puts its occupiers out with hunt)
#[test]
fn claim_4_occupied_civilian_building_sold_when_not_held() {
    let mut w = world();
    let (b, us) = occupied_hut(&mut w, 0, 2, 0, 0);
    computer_at(&mut w, 1000);
    computer::house_update(&mut w, A);
    assert_eq!(w.obj(b).unwrap().beh.current, Some(Behaviour::Selling));
    for &u in &us {
        assert!(on_map(&w, u));
        assert_eq!(w.obj(u).unwrap().beh.queued, Some(Behaviour::Hunt));
    }
}

// spec: S-0217 claim 4 and S-0508 claim 4 (r39: a refused unit of a house with no building other than occupied civilian buildings and no construction vehicle has hunt queued at its house's next check)
#[test]
fn claim_4_a_refused_unit_has_hunt_queued_at_the_next_check() {
    let mut w = world();
    let (b, _) = occupied_hut(&mut w, 0, 3, 0, 0);
    let refused = put(&mut w, Kind::Infantry, "GARRI", 0, -3, 4);
    assert!(!garrison::attempt(&mut w, refused, b), "full");
    assert!(on_map(&w, refused));
    w.obj_mut(refused).unwrap().beh.queued = None;
    computer_at(&mut w, 300);
    computer::house_update(&mut w, A);
    assert_eq!(w.obj(refused).unwrap().beh.queued, Some(Behaviour::Hunt));
}
