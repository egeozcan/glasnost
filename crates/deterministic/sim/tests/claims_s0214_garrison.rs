//! S-0214 table I row I8 with S-0508 claim 1's occupation test, on
//! synthetic data: invented types, no stock values. Houses Alpha (0) and
//! Beta (1) of the testkit's sandbox game, not allied, and a third house
//! (2) of a passive country.

use std::sync::Arc;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Country, Decimal, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, ObjectId, World};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn hut(capacity: Option<i32>, occupiable: bool) -> BuildingType {
    BuildingType {
        strength: Some(400),
        foundation: Some([1, 1]),
        can_be_occupied: Some(occupiable),
        max_number_occupants: capacity,
        ..Default::default()
    }
}

/// HUT (garrisonable, capacity 3, strength 400), HUT0 (capacity 0), HUTNEG
/// (capacity −1), PLAIN (not garrisonable); infantry GARRI (occupier) and
/// the testkit's WALKER (not an occupier); red ratio 0.5. The country
/// Gammaland is passive.
fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries.insert(
        "buildings".into(),
        ["HUT", "HUT0", "HUTNEG", "PLAIN"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("GARRI".to_string());
    r.registries
        .get_mut("countries")
        .unwrap()
        .push("Gammaland".to_string());
    let s = &mut r.sim;
    s.buildings.insert("HUT".into(), hut(Some(3), true));
    s.buildings.insert("HUT0".into(), hut(Some(0), true));
    s.buildings.insert("HUTNEG".into(), hut(Some(-1), true));
    s.buildings.insert("PLAIN".into(), hut(Some(3), false));
    s.infantry.insert(
        "GARRI".into(),
        UnitType {
            strength: Some(100),
            speed: Some(4),
            locomotion_class: Some(LocomotionClass::Foot),
            locomotor: Some(Locomotor::Walk),
            occupier: Some(true),
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
    s.general.condition_red = Some(Decimal("0.5".into()));
    r
}

fn game() -> World {
    let r = rules();
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let mut s = testkit::setup(7);
    s.houses.push(HouseSetup {
        name: "Gamma".into(),
        country: "Gammaland".into(),
        human: false,
        start: None,
        role: HouseRole::Neutral,
        ai: false,
    });
    // The testkit's empty map with its local area cut by 2 cells on each
    // side, so that some legal cells lie outside the usable area.
    let mut m = testkit::empty_map(SIZE);
    let fp = glasnost_map::Footprint::new(SIZE, SIZE, [2, 2, SIZE - 4, SIZE - 4]);
    let mut cells = glasnost_map::CellGrid::new(fp);
    let all: Vec<glasnost_map::CellXY> = cells.iter().map(|(c, _)| c).collect();
    for c in all {
        if let (Some(dst), Some(src)) = (cells.get_mut(c), m.cells.get(c)) {
            *dst = *src;
        }
    }
    m.footprint = fp;
    m.cells = cells;
    let mut w = glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &s)
        .expect("synthetic test game loads");
    testkit::run(&mut w, 4);
    w
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn act(w: &World, actor: ObjectId, t: ObjectId, force_fire: bool) -> Action {
    let house = w.obj(actor).unwrap().owner.unwrap();
    let mods = Modifiers {
        force_fire,
        ..Default::default()
    };
    targeting::action(w, house, actor, ClickOn::Object(t), mods)
}

fn garrisons(w: &World, actor: ObjectId, t: ObjectId) -> bool {
    act(w, actor, t, false) == Action::Capture
}

// spec: S-0214 table I row I8 and S-0508 claim 1 (own house and passive house: capture (garrison); force-fire held: not that action)
#[test]
fn i8_own_and_passive_buildings() {
    let mut w = game();
    assert!(glasnost_sim::econ::setup::passive(&w, HouseId(2)));
    assert!(!glasnost_sim::econ::setup::passive(&w, A));
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let own = put(&mut w, Kind::Building, "HUT", 0, 3, 0);
    let pas = put(&mut w, Kind::Building, "HUT", 2, -3, 0);
    assert!(garrisons(&w, g, own));
    assert!(garrisons(&w, g, pas));
    // spec: S-0214 table I row I8 (force-fire not held)
    assert_ne!(act(&w, g, own, true), Action::Capture);
    assert_ne!(act(&w, g, pas, true), Action::Capture);
}

// spec: S-0214 table I row I8 and S-0508 claim 1 (alliance plays no part: neither an enemy's nor an ally's building)
#[test]
fn i8_enemy_and_allied_buildings_refused() {
    let mut w = game();
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let other = put(&mut w, Kind::Building, "HUT", 1, 3, 0);
    assert!(!garrisons(&w, g, other), "enemy");
    w.houses[0].allies = vec![A, B];
    w.houses[1].allies = vec![A, B];
    assert!(!garrisons(&w, g, other), "allied");
}

// spec: S-0214 table I row I8 and S-0508 claim 1 (the building's type garrisonable, the unit's type an occupier type)
#[test]
fn i8_type_flags() {
    let mut w = game();
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let walker = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 2);
    let plain = put(&mut w, Kind::Building, "PLAIN", 0, 3, 0);
    let h = put(&mut w, Kind::Building, "HUT", 0, -3, 0);
    assert!(!garrisons(&w, g, plain));
    assert!(!garrisons(&w, walker, h));
}

// spec: S-0214 table I row I8 and S-0508 claim 1 (not being built up and not being sold, S-0408)
#[test]
fn i8_construction_and_selling_refused() {
    let mut w = game();
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let h = put(&mut w, Kind::Building, "HUT", 0, 3, 0);
    assert!(garrisons(&w, g, h));
    let t = w.tick;
    w.obj_mut(h).unwrap().beh.force(Behaviour::Selling, t, true);
    assert!(!garrisons(&w, g, h), "being sold");
    w.obj_mut(h)
        .unwrap()
        .beh
        .force(Behaviour::Construction, t, true);
    assert!(!garrisons(&w, g, h), "being built up");
    w.obj_mut(h).unwrap().beh.force(Behaviour::Guard, t, true);
    assert!(garrisons(&w, g, h));
}

// spec: S-0214 table I row I8 and S-0508 claim 1 (occupant count not equal to capacity: equality, so capacity 0 lets no unit in and a negative capacity does not stop entries)
#[test]
fn i8_capacity_equality() {
    let mut w = game();
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let zero = put(&mut w, Kind::Building, "HUT0", 0, 3, 0);
    let neg = put(&mut w, Kind::Building, "HUTNEG", 0, -3, 0);
    // The occupant count is 0 (no occupation exists yet, `power::occupants`).
    assert!(!garrisons(&w, g, zero));
    assert!(garrisons(&w, g, neg));
}

// spec: S-0214 table I row I8 and S-0508 claim 1 (not both positive health and badly damaged, h ≤ the red ratio, S-0108 Terms and F3)
#[test]
fn i8_badly_damaged_refused() {
    let mut w = game();
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let h = put(&mut w, Kind::Building, "HUT", 0, 3, 0);
    w.obj_mut(h).unwrap().health = 201;
    assert!(garrisons(&w, g, h), "above the red ratio");
    w.obj_mut(h).unwrap().health = 200;
    assert!(!garrisons(&w, g, h), "at the red ratio: badly damaged");
    w.obj_mut(h).unwrap().health = 1;
    assert!(!garrisons(&w, g, h));
    // Zero health adds no positive-health requirement.
    w.obj_mut(h).unwrap().health = 0;
    assert!(garrisons(&w, g, h));
}

/// The first legal cell outside the usable area, in row order.
fn outside_cell(w: &World) -> Option<glasnost_map::CellXY> {
    (0..4 * SIZE)
        .flat_map(|y| (0..4 * SIZE).map(move |x| glasnost_map::CellXY::new(x, y)))
        .find(|&c| w.cells.idx(c).is_some() && !w.cells.usable(c))
}

// spec: S-0214 table I row I8 and S-0508 claim 1 (the building's centre lies in the usable area)
#[test]
fn i8_centre_outside_usable_area_refused() {
    let mut w = game();
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let c = outside_cell(&w).expect("the test map has legal cells outside the usable area");
    let h = testkit::spawn(&mut w, Kind::Building, "HUT", 0, c, 0);
    assert!(!w.cells.usable(w.reference_point(h).cell()));
    assert!(!garrisons(&w, g, h));
}

// spec: S-0214 table I row I8 (a refused garrison falls through to the later rows: the same action as over a building that is not garrisonable)
#[test]
fn i8_refusal_falls_through() {
    let mut w = game();
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 0);
    let enemy_hut = put(&mut w, Kind::Building, "HUT", 1, 3, 0);
    let enemy_plain = put(&mut w, Kind::Building, "PLAIN", 1, -3, 0);
    assert_eq!(act(&w, g, enemy_hut, false), act(&w, g, enemy_plain, false));
}
