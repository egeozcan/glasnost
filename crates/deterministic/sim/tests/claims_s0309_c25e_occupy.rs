//! S-0309 claim 25 part (e)'s occupation alternative and the infantry rule
//! of a computer-controlled owner in S-0215 claims 23 (second row of the
//! infantry table) and 27 (first infantry rule), on synthetic data:
//! invented types, no stock values. House Alpha (0) is a player house that
//! each test marks as a test-only computer-controlled player house or
//! leaves human-controlled (S-0215 Terms, S-0309 Terms *Test-only
//! exception*); house Beta (1) is a human player house; house Gamma (2) is
//! the neutral house, of a multiplayer-passive country.

use std::sync::Arc;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Country, Decimal, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::structures::garrison;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, ObjectId, Target, World};

const SIZE: i32 = 30;

/// The testkit's combat ruleset with HUT (garrisonable, capacity 3),
/// PLAIN (not garrisonable), infantry GARRI (an armed occupier with a
/// guard range of 6 cells) and the country Gammaland (passive).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .extend(["HUT".to_string(), "PLAIN".to_string()]);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("GARRI".to_string());
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
                max_number_occupants: Some(3),
                ..Default::default()
            },
        );
    }
    s.infantry.insert(
        "GARRI".into(),
        UnitType {
            strength: Some(100),
            speed: Some(4),
            locomotion_class: Some(LocomotionClass::Foot),
            locomotor: Some(Locomotor::Walk),
            occupier: Some(true),
            primary: Some("Rifle".into()),
            guard_range: Some(Decimal("6".into())),
            ..Default::default()
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
    assert_eq!(w.houses[0].role, HouseRole::Player);
    assert_eq!(w.houses[0].human, human);
    assert!(glasnost_sim::econ::setup::passive(&w, HouseId(2)));
    w
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn range_search(w: &mut World, a: ObjectId) -> Option<ObjectId> {
    acquire::search(w, a, SearchKind::Range, SearchContext::PerTick)
}

/// Gives `a` behaviour `b` and target `t`, with a targeting timer far in
/// the future so that a search would show as a timer restart. The objects
/// placed so far take their first turns first.
fn order(w: &mut World, a: ObjectId, b: Behaviour, t: ObjectId) {
    testkit::run(w, 4);
    let tick = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.beh.force(b, tick, true);
    o.target = Some(Target::Object(t));
    o.targeting_timer = tick + 1000;
}

// ---------------------------------------------------------------------------
// S-0309 claim 25 part (e): the occupation alternative.

// spec: S-0309 claim 25 part e (a searcher of a marked test house: the passive-house row lets a building through when A is an infantry unit that may occupy it, S-0508 claim 1's test)
#[test]
fn computer_occupier_finds_a_passive_garrisonable_building() {
    let mut w = game(false);
    let a = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 3, 0);
    assert!(garrison::may_enter(&w, a, hut));
    assert_eq!(range_search(&mut w, a), Some(hut));
}

// spec: S-0309 claim 25 part e and table C row passive house (a player searcher is rejected by the row whatever it may occupy)
#[test]
fn human_occupier_does_not_find_a_passive_building() {
    let mut w = game(true);
    let a = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 3, 0);
    assert!(garrison::may_enter(&w, a, hut));
    assert_eq!(range_search(&mut w, a), None);
}

// spec: S-0309 claim 25 part e (the occupation alternative needs A to be able to occupy T: a building that cannot be occupied, or a searcher that is not an occupier, is still rejected by the passive-house row)
#[test]
fn computer_searcher_rejected_without_occupation() {
    let mut w = game(false);
    let a = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    put(&mut w, Kind::Building, "PLAIN", 2, 3, 0);
    assert_eq!(range_search(&mut w, a), None, "not garrisonable");
    // WALKER has no guard range: its search reaches as far as its weapon
    // (S-0309 F1), 4 cells.
    let mut w = game(false);
    let walker = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 3, 0);
    assert!(!garrison::may_enter(&w, walker, hut));
    assert_eq!(range_search(&mut w, walker), None, "not an occupier");
}

// ---------------------------------------------------------------------------
// S-0215 claim 27: hunt, first infantry rule of a computer-controlled owner.

// spec: S-0215 claim 27 (infantry of a computer-controlled owner, original only, judged on the target held when the processing begins: an occupier whose target is a building it may occupy takes it as its destination and capture is forced; 1 tick, no search, no targeting-timer restart; S-0215 Terms, Test-only computer-controlled player house)
#[test]
fn hunt_computer_occupier_goes_for_the_building() {
    let mut w = game(false);
    let a = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 4, 0);
    order(&mut w, a, Behaviour::Hunt, hut);
    let timer = w.obj(a).unwrap().targeting_timer;
    let t = w.tick;
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Object(hut)));
    assert_eq!(o.beh.current, Some(Behaviour::Capture));
    assert_eq!(o.beh.last, Some((t, 1)));
    assert_eq!(o.targeting_timer, timer, "no search, no timer restart");
    // spec: S-0508 claim 1 (the capture then ends in the occupation)
    testkit::run(&mut w, 300);
    assert_eq!(garrison::count(&w, hut), 1);
    assert!(!w.obj(a).unwrap().on_map);
}

// spec: S-0215 claim 27 (the rule is the computer-controlled owner's only: a human player's occupier in hunt does not go for the building)
#[test]
fn hunt_human_occupier_keeps_hunting() {
    let mut w = game(true);
    let a = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 4, 0);
    order(&mut w, a, Behaviour::Hunt, hut);
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Hunt));
    assert_ne!(o.dest, Some(Target::Object(hut)));
}

// spec: S-0215 claim 27 (the type must occupy or assault buildings and may occupy this one: a non-occupier, or a building it may not occupy, takes the other rules)
#[test]
fn hunt_computer_rule_needs_an_occupation() {
    let mut w = game(false);
    let walker = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 4, 0);
    order(&mut w, walker, Behaviour::Hunt, hut);
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 3);
    let plain = put(&mut w, Kind::Building, "PLAIN", 2, 4, 3);
    order(&mut w, g, Behaviour::Hunt, plain);
    glasnost_sim::tick::step(&mut w);
    for a in [walker, g] {
        assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Hunt));
    }
}

// ---------------------------------------------------------------------------
// S-0215 claim 23: attack, second row of the infantry table.

// spec: S-0215 claim 23 (Infantry, second row, original only: a computer-controlled owner's occupier whose target is a building it may occupy takes it as its destination and capture is forced; 1 tick)
#[test]
fn attack_computer_occupier_goes_for_the_building() {
    let mut w = game(false);
    let a = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 4, 0);
    order(&mut w, a, Behaviour::Attack, hut);
    let t = w.tick;
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Object(hut)));
    assert_eq!(o.beh.current, Some(Behaviour::Capture));
    assert_eq!(o.beh.last, Some((t, 1)));
}

// spec: S-0215 claim 23 (the second row is the computer-controlled owner's only: a human player's occupier attacks)
#[test]
fn attack_human_occupier_attacks() {
    let mut w = game(true);
    let a = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let hut = put(&mut w, Kind::Building, "HUT", 2, 4, 0);
    order(&mut w, a, Behaviour::Attack, hut);
    glasnost_sim::tick::step(&mut w);
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Attack));
}
