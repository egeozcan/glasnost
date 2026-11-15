//! Alliance changes and the shroud (S-0409 Edge cases, *Ally reveal and
//! alliance changes*, scenario V15; #336, #337). Synthetic data only:
//! vehicle EYE1 (sight 1), buildings TOWER (sight 4), GAP (gap generator,
//! radius 3) and SAT (spy satellite), all 1 × 1, on a flat map of 40.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::econ::{shroud, victory};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, ObjectId, World};

const SIZE: i32 = 40;
const H: HouseId = HouseId(0);
const B: HouseId = HouseId(1);
const P: HouseId = HouseId(2);
const Q: HouseId = HouseId(3);

fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("EYE1".into());
    r.sim.vehicles.insert(
        "EYE1".into(),
        UnitType {
            strength: Some(100),
            speed: Some(8),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            sight: Some(1),
            ..Default::default()
        },
    );
    r.registries.insert(
        "buildings".into(),
        ["TOWER", "GAP", "SAT"].iter().map(|s| s.to_string()).collect(),
    );
    let b = |sight: i32| {
        let mut b = BuildingType {
            strength: Some(500),
            foundation: Some([1, 1]),
            ..Default::default()
        };
        b.econ.sight = Some(sight);
        b
    };
    r.sim.buildings.insert("TOWER".into(), b(4));
    let mut gap = b(0);
    gap.gap_generator = Some(true);
    gap.econ.gap_radius_in_cells = Some(3);
    r.sim.buildings.insert("GAP".into(), gap);
    let mut sat = b(0);
    sat.econ.spy_sat = Some(true);
    r.sim.buildings.insert("SAT".into(), sat);
    // spec: S-0414 claim 14 (both alliance-order flags on); ally reveal stays on (absent: yes, S-0409 Import mapping)
    r.sim.general.economy.allies_allowed = Some(true);
    r.sim.general.economy.ally_change_allowed = Some(true);
    r
}

/// The testkit's game with two more player houses P and Q (houses 2 and
/// 3), none allied. P and Q are V15's other enemy bystanders: they keep an
/// alliance order of H from being a last-enemy refusal (S-0414 claim 12).
fn four_houses() -> World {
    let mut s = testkit::setup(3);
    for name in ["Pi", "Qu"] {
        s.houses.push(HouseSetup {
            name: name.into(),
            country: "Betaland".into(),
            human: true,
            start: None,
            role: HouseRole::Player,
            ai: false,
        });
    }
    let rules = std::sync::Arc::new(SimRules::new(&rules(), "synthetic-test"));
    glasnost_sim::load::new_game(rules, &testkit::empty_map(SIZE), testkit::catalogue(), &s)
        .expect("synthetic test game loads")
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn spawn(w: &mut World, kind: Kind, name: &str, h: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, h, c(dx, dy), 0)
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

/// S-0409 scenario V15 part A: H and B allied both ways, B's tower sees
/// cells of its generator's field; H breaks the alliance.
fn v15_part_a(generator_first: bool) -> (World, CellXY) {
    let mut w = four_houses();
    w.houses[0].allies = vec![H, B];
    w.houses[1].allies = vec![H, B];
    let order: [(&str, i32); 2] = if generator_first {
        [("GAP", 0), ("TOWER", 2)]
    } else {
        [("TOWER", 2), ("GAP", 0)]
    };
    for (name, dx) in order {
        spawn(&mut w, Kind::Building, name, 1, dx, 0);
    }
    for _ in 0..3 {
        step(&mut w);
    }
    let cell = c(1, 0);
    assert!(shroud::explored(&w, H, cell), "explored before the break");
    assert_eq!(shroud::counts(&w, H, cell), Some((-1, 0)));
    victory::alliance(&mut w, H, B);
    assert!(!w.allied(Some(B), Some(H)), "the alliance ended both ways");
    (w, cell)
}

// spec: S-0409 Edge cases (ally reveal and alliance changes) and scenario V15 part A
#[test]
fn issue_336_building_first_cells_unexplored_on_the_break_tick() {
    let (w, cell) = v15_part_a(false);
    assert!(!shroud::explored(&w, H, cell));
    assert!(!shroud::pending(&w, H, cell));
}

// spec: S-0409 Edge cases (ally reveal and alliance changes: every sight and field taken back, S1 rows 2 and 5, before any is applied again, rows 1 and 3) and scenario V15 part A
#[test]
fn issue_336_generator_first_cells_unexplored_on_the_break_tick() {
    let (w, cell) = v15_part_a(true);
    assert_eq!(
        (
            shroud::explored(&w, H, cell),
            shroud::pending(&w, H, cell),
            shroud::counts(&w, H, cell)
        ),
        (false, false, Some((1, 1))),
        "V15 part A: unexplored on the break tick, without waiting for a sweep"
    );
}
