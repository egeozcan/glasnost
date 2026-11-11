//! S-0508 r19 (Q-2926 and Q-2927 answered): claim 1's units on their way
//! and the meaning of "may occupy" in S-0215 claims 23 and 27 and S-0309
//! claim 25 part (e); claim 4's unchanged target after a refusal and the
//! *hunt* a refused unit does or does not get back (S-0215 claims 9, 10
//! and 28). Synthetic data: invented types, no stock values. House Alpha
//! (0) is a player house marked as a test-only computer-controlled player
//! house unless a test leaves it human-controlled (S-0215 Terms, S-0309
//! Terms *Test-only exception*); house Beta (1) is a human player house;
//! house Gamma (2) is the neutral house, of a multiplayer-passive country.

use std::sync::Arc;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Country, Decimal, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::structures::garrison;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, ObjectId, Target, World};

const SIZE: i32 = 30;
/// HUT's invented occupant capacity.
const CAP: i32 = 3;

/// The testkit's combat ruleset with HUT (garrisonable, capacity
/// [`CAP`]), PLAIN (not garrisonable), the armed infantry types GARRI (an
/// occupier), ASLT (an assaulter that is not an occupier) and SPYI (an
/// infiltrator with neither mark), each with a guard range of 6 cells, and
/// the country Gammaland (passive).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .extend(["HUT".to_string(), "PLAIN".to_string()]);
    r.registries.get_mut("infantry").unwrap().extend([
        "GARRI".to_string(),
        "ASLT".to_string(),
        "SPYI".to_string(),
    ]);
    r.registries
        .get_mut("countries")
        .unwrap()
        .push("Gammaland".to_string());
    let s = &mut r.sim;
    for (name, occupiable) in [("HUT", true), ("PLAIN", false)] {
        s.buildings.insert(
            name.into(),
            BuildingType {
                strength: Some(400),
                foundation: Some([1, 1]),
                can_be_occupied: Some(occupiable),
                max_number_occupants: Some(CAP),
                ..Default::default()
            },
        );
    }
    let base = UnitType {
        strength: Some(100),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        primary: Some("Rifle".into()),
        guard_range: Some(Decimal("6".into())),
        ..Default::default()
    };
    s.infantry.insert(
        "GARRI".into(),
        UnitType {
            occupier: Some(true),
            ..base.clone()
        },
    );
    s.infantry.insert(
        "ASLT".into(),
        UnitType {
            assaulter: Some(true),
            ..base.clone()
        },
    );
    s.infantry.insert(
        "SPYI".into(),
        UnitType {
            infiltrate: Some(true),
            ..base
        },
    );
    s.countries.insert(
        "Gammaland".into(),
        Country {
            multiplay_passive: Some(true),
            ..Default::default()
        },
    );
    r
}

/// A game whose house 0 is human-controlled when `human` is set and a
/// marked test-only computer-controlled player house otherwise.
fn game(human: bool) -> World {
    let mut s = testkit::setup(7);
    s.houses[0].human = human;
    s.houses.push(HouseSetup {
        name: "Gamma".into(),
        country: "Gammaland".into(),
        human: false,
        start: None,
        role: HouseRole::Neutral,
        ai: false,
    });
    let rules = Arc::new(SimRules::new(&rules(), "synthetic-test"));
    let mut w =
        glasnost_sim::load::new_game(rules, &testkit::empty_map(SIZE), testkit::catalogue(), &s)
            .expect("synthetic test game loads");
    testkit::run(&mut w, 4);
    assert_eq!(w.houses[0].human, human);
    assert!(glasnost_sim::econ::setup::passive(&w, HouseId(2)));
    assert!(!w.allied(Some(HouseId(1)), Some(HouseId(0))));
    w
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

/// Gives `a` behaviour `b` and target `t`. The objects placed so far take
/// their first turns first.
fn order(w: &mut World, a: ObjectId, b: Behaviour, t: ObjectId) {
    testkit::run(w, 4);
    let tick = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.beh.force(b, tick, true);
    o.target = Some(Target::Object(t));
    o.targeting_timer = tick + 1000;
}

/// Sends `u` into building `b` (capture with `b` as destination) and runs
/// until it is inside.
fn go_inside(w: &mut World, u: ObjectId, b: ObjectId) {
    let tick = w.tick;
    w.obj_mut(u)
        .unwrap()
        .beh
        .force(Behaviour::Capture, tick, true);
    glasnost_sim::econ::assign_destination(w, u, Some(Target::Object(b)));
    testkit::run(w, 300);
    assert!(!w.obj(u).unwrap().on_map, "inside");
}

// ---------------------------------------------------------------------------
// Claim 1: units on their way count for nothing.

// spec: S-0508 claim 1 (r19, Q-2926: units on their way count for nothing; the count compared is that of the occupants inside, so any number of units may hold the same building at once, and those that arrive after it is full are refused, claim 4) and S-0215 claim 27 (the computer-controlled occupier in hunt goes for the building)
#[test]
fn units_on_their_way_hold_no_place() {
    let mut w = game(false);
    // spec: S-0217 claims 2 and 4 (the test holds house 0's next check past its ticks: owning only the occupied hut, its check would put the occupants out with hunt; claims_s0217 tests that)
    w.houses[0].checks.next = i32::MAX;
    let hut = put(&mut w, Kind::Building, "HUT", 2, 5, 0);
    let n = CAP + 2;
    let units: Vec<ObjectId> = (0..n)
        .map(|i| put(&mut w, Kind::Infantry, "GARRI", 0, -3, i - 2))
        .collect();
    for &u in &units {
        order(&mut w, u, Behaviour::Hunt, hut);
    }
    glasnost_sim::tick::step(&mut w);
    for &u in &units {
        let o = w.obj(u).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Capture), "every one goes");
        assert_eq!(o.dest, Some(Target::Object(hut)));
    }
    // spec: S-0508 claim 1 (r19: no place is held for the units bound for it, which here outnumber its free places) and S-0309 claim 25 part e (a further computer occupier still finds it)
    assert_eq!(garrison::count(&w, hut), 0);
    let late = put(&mut w, Kind::Infantry, "GARRI", 0, 1, 0);
    assert!(garrison::may_enter(&w, late, hut));
    assert_eq!(
        acquire::search(&mut w, late, SearchKind::Range, SearchContext::PerTick),
        Some(hut)
    );
    testkit::run(&mut w, 300);
    // spec: S-0508 claims 1 and 4 (refused only once the count inside is the capacity: the rest stay on the map)
    assert_eq!(garrison::count(&w, hut), CAP);
    let outside = units
        .iter()
        .filter(|&&u| w.obj(u).is_some_and(|o| o.on_map))
        .count();
    assert_eq!(
        outside,
        (n - CAP) as usize,
        "the late arrivals were refused"
    );
}

// ---------------------------------------------------------------------------
// Claim 1: "may occupy" in S-0215 claims 23 and 27 and S-0309 claim 25 (e).

// spec: S-0508 claim 1 (r19, Q-2927: for an assaulter type that is not an occupier type, "may occupy" in S-0215 claims 23 and 27 is claim 14's assault eligibility) and S-0215 claim 27 (the computer-controlled assaulter in hunt whose target is a hostile garrisoned building takes it as its destination and is forced into capture)
#[test]
fn hunt_computer_assaulter_goes_for_an_eligible_building() {
    let mut w = game(false);
    let hut = put(&mut w, Kind::Building, "HUT", 1, 5, 0);
    let held = put(&mut w, Kind::Infantry, "GARRI", 1, 5, 1);
    go_inside(&mut w, held, hut);
    let a = put(&mut w, Kind::Infantry, "ASLT", 0, -3, 0);
    assert!(!garrison::may_enter(&w, a, hut), "not an occupier");
    assert!(garrison::assault_eligible(&w, a, hut));
    assert!(garrison::may_occupy(&w, a, hut));
    order(&mut w, a, Behaviour::Hunt, hut);
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Capture));
    assert_eq!(o.dest, Some(Target::Object(hut)));
}

// spec: S-0508 claim 1 (r19, Q-2927: claim 14's assault eligibility needs an occupant; an empty passive building fails it) and S-0215 claim 23 (the computer-controlled assaulter in attack keeps attacking)
#[test]
fn attack_computer_assaulter_keeps_attacking_an_empty_building() {
    let mut w = game(false);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 4, 0);
    let a = put(&mut w, Kind::Infantry, "ASLT", 0, 0, 0);
    assert!(!garrison::may_occupy(&w, a, hut));
    order(&mut w, a, Behaviour::Attack, hut);
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
    assert_ne!(o.dest, Some(Target::Object(hut)));
}

// spec: S-0508 claim 1 (r19, Q-2927: S-0309 claim 25 part e asks the test only of a searcher of an occupier type, so an assaulter type that is not an occupier type never passes that alternative)
#[test]
fn computer_assaulter_never_finds_a_passive_building() {
    let mut w = game(false);
    let a = put(&mut w, Kind::Infantry, "ASLT", 0, 0, 0);
    put(&mut w, Kind::Building, "HUT", 2, 3, 0);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Range, SearchContext::PerTick),
        None
    );
}

// spec: S-0508 claim 1 (r19, Q-2927: an infiltrator type takes the target as destination and is forced into capture whatever the test gives) and S-0215 claim 23 (the infiltrator alternative of the second infantry row)
#[test]
fn attack_computer_infiltrator_goes_whatever_the_test() {
    let mut w = game(false);
    let plain = put(&mut w, Kind::Building, "PLAIN", 1, 4, 0);
    let s = put(&mut w, Kind::Infantry, "SPYI", 0, 0, 0);
    assert!(!garrison::may_occupy(&w, s, plain), "neither mark");
    order(&mut w, s, Behaviour::Attack, plain);
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(s).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Capture));
    assert_eq!(o.dest, Some(Target::Object(plain)));
}

// spec: S-0508 claim 1 (r19, Q-2926: part e has no behaviour condition; in area guard the search that picks the building is area guard's own, S-0215 claim 26)
#[test]
fn area_guard_search_picks_the_passive_building() {
    let mut w = game(false);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 3, 0);
    let a = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    testkit::run(&mut w, 4);
    let tick = w.tick;
    w.obj_mut(a)
        .unwrap()
        .beh
        .force(Behaviour::AreaGuard, tick, true);
    testkit::run(&mut w, 60);
    let o = w.obj(a).unwrap();
    let went = o.target == Some(Target::Object(hut)) || !o.on_map;
    assert!(
        went,
        "area guard's search took the building: {:?} {:?}",
        o.beh.current, o.target
    );
}

// ---------------------------------------------------------------------------
// Claim 4: the refusal.

// spec: S-0508 claim 4 (r19, Q-2926: the refusal clears the destination and sends the scatter request, the unit stays on the map, and the refusal does not change its target)
#[test]
fn refusal_keeps_the_target() {
    let mut w = game(false);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 5, 0);
    for i in 0..CAP {
        let u = put(&mut w, Kind::Infantry, "GARRI", 2, 5, 1 + i);
        go_inside(&mut w, u, hut);
    }
    assert_eq!(garrison::count(&w, hut), CAP);
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, 4, 0);
    let tick = w.tick;
    let o = w.obj_mut(u).unwrap();
    o.beh.force(Behaviour::Capture, tick, true);
    o.target = Some(Target::Object(hut));
    o.dest = Some(Target::Object(hut));
    assert!(!garrison::attempt(&mut w, u, hut), "refused");
    let o = w.obj(u).unwrap();
    assert!(o.on_map, "it stays on the map");
    assert_ne!(o.dest, Some(Target::Object(hut)), "the destination cleared");
    assert_eq!(o.target, Some(Target::Object(hut)), "the target unchanged");
    assert_eq!(garrison::count(&w, hut), CAP);
}

// spec: S-0508 claim 4 (r19, Q-2926: the force into capture removes a current or queued hunt, S-0215 claim 9) and S-0215 claim 23 (the computer occupier in attack with hunt queued)
#[test]
fn forced_capture_removes_a_queued_hunt() {
    let mut w = game(false);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 4, 0);
    let a = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    order(&mut w, a, Behaviour::Attack, hut);
    w.obj_mut(a).unwrap().beh.queued = Some(Behaviour::Hunt);
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Capture));
    assert_eq!(o.beh.queued, None, "the queued hunt is gone");
}

// spec: S-0508 claim 4 (r19, Q-2926: the force does not touch a suspended hunt, S-0215 claims 9 and 10, so a later revert can make it current again; quirks.stale_suspended_revert)
#[test]
fn forced_capture_keeps_a_suspended_hunt() {
    let mut w = game(false);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 4, 0);
    let a = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    testkit::run(&mut w, 4);
    let tick = w.tick;
    w.obj_mut(a).unwrap().beh.force(Behaviour::Hunt, tick, true);
    // An override (a retaliation, S-0215 claim 18) to attack the building suspends the hunt.
    assert!(acquire::override_attack(&mut w, a, hut));
    let o = w.obj_mut(a).unwrap();
    assert_eq!(o.beh.suspended, Some(Behaviour::Hunt));
    o.beh.expired = true;
    glasnost_sim::tick::step(&mut w);
    let o = w.obj_mut(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Capture), "capture forced");
    assert_eq!(
        o.beh.suspended,
        Some(Behaviour::Hunt),
        "suspended hunt kept"
    );
    assert!(o.beh.revert().is_some());
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Hunt),
        "a revert makes it current"
    );
}

// spec: S-0508 claim 4 (r19, Q-2926: capture's hunt rule, S-0215 claim 28, leaves out every unit of an occupier type by its type alone; an assaulter type that is not an occupier type is not left out) and S-0215 claim 28 row 4 (a destination and no target), for a test-only computer-controlled player house
#[test]
fn capture_hunt_rule_leaves_out_occupiers_only() {
    for (name, hunts) in [("GARRI", false), ("ASLT", true)] {
        let mut w = game(false);
        let hut = put(&mut w, Kind::Building, "HUT", 2, 6, 0);
        let u = put(&mut w, Kind::Infantry, name, 0, 0, 0);
        testkit::run(&mut w, 4);
        let tick = w.tick;
        let o = w.obj_mut(u).unwrap();
        o.beh.force(Behaviour::Capture, tick, true);
        o.beh.expired = true;
        o.target = None;
        o.dest = Some(Target::Object(hut));
        glasnost_sim::tick::step(&mut w);
        let o = w.obj(u).unwrap();
        assert!(o.beh.last.is_some(), "{name}: processed");
        // spec: S-0215 claim 11 (a ready actor's queued behaviour may start late in the same tick)
        let hunt = o.beh.current == Some(Behaviour::Hunt) || o.beh.queued == Some(Behaviour::Hunt);
        if hunts {
            assert!(hunt, "{name}: hunt queued");
            assert_eq!(o.dest, None, "{name}: destination cleared");
        } else {
            assert!(!hunt, "{name}: no hunt");
            assert_eq!(o.beh.queued, None, "{name}: nothing queued");
            assert_eq!(o.dest, Some(Target::Object(hut)), "{name}: no change");
        }
    }
}

// spec: S-0215 claim 28 (the hunt rule is original only: an actor of a human-controlled owner, every player house in our engine, is left as the other effects leave it)
#[test]
fn capture_hunt_rule_not_for_a_human_owner() {
    let mut w = game(true);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 6, 0);
    let u = put(&mut w, Kind::Infantry, "ASLT", 0, 0, 0);
    testkit::run(&mut w, 4);
    let tick = w.tick;
    let o = w.obj_mut(u).unwrap();
    o.beh.force(Behaviour::Capture, tick, true);
    o.beh.expired = true;
    o.target = None;
    o.dest = Some(Target::Object(hut));
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(u).unwrap();
    assert_eq!(o.beh.queued, None);
    assert_eq!(o.dest, Some(Target::Object(hut)));
}
