//! Claim tests of S-0208 claim 27 (r82, Q-3317) for the walking and
//! driving locomotors: the point a building supplies to an infantry unit's
//! or a vehicle's locomotion at an assignment, on an empty synthetic map
//! with invented types. Every name and number here is made up; no game
//! data. House Alpha (0) of the testkit's sandbox game.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::bunker;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use glasnost_sim::{drive, econ, path, save, testkit, tick, walk};

const SIZE: i32 = 30;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut t = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        number_impassable_rows: Some(0),
        ..Default::default()
    };
    f(&mut t);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), t);
}

/// The foundations of the centre cases.
const FOUNDATIONS: [(i32, i32); 6] = [(1, 1), (2, 2), (3, 2), (3, 3), (4, 4), (5, 3)];

fn plain_name(w: i32, h: i32) -> String {
    format!("P{w}{h}")
}

/// The combat ruleset with:
/// - P11, P22, P32, P33, P44, P53: buildings of those foundations with none
///   of the helipad, unit-repair, unit-reload and vehicle-bunker flags;
/// - RLD: a 3 × 3 building with the unit-reload flag alone and one docking
///   place 256 leptons east of the centre;
/// - REP: a 3 × 3 building with the unit-repair flag and one docking place
///   at (+512, −256) from the centre;
/// - HPAD: a 2 × 2 building with the helipad flag and two docking places;
/// - PIT: a 2 × 2 vehicle bunker;
/// - CAPB: a capturable 3 × 2 building with no flags;
/// - ENGW: a walking engineer made from WALKER.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    for (fw, fh) in FOUNDATIONS {
        building(&mut r, &plain_name(fw, fh), |t| {
            t.foundation = Some([fw, fh]);
        });
    }
    building(&mut r, "RLD", |t| {
        t.foundation = Some([3, 3]);
        t.econ.unit_reload = Some(true);
        t.econ.number_of_docks = Some(1);
        t.econ.docking_offsets = Some(vec![[256, 0, 0]]);
    });
    building(&mut r, "REP", |t| {
        t.foundation = Some([3, 3]);
        t.unit_repair = Some(true);
        t.econ.number_of_docks = Some(1);
        t.econ.docking_offsets = Some(vec![[512, -256, 0]]);
    });
    building(&mut r, "HPAD", |t| {
        t.helipad = Some(true);
        t.econ.number_of_docks = Some(2);
        t.econ.docking_offsets = Some(vec![[-256, 0, 0], [256, 0, 0]]);
    });
    building(&mut r, "PIT", |t| {
        t.bunker = Some(true);
    });
    building(&mut r, "CAPB", |t| {
        t.foundation = Some([3, 2]);
        t.capturable = Some(true);
    });
    let mut eng = r.sim.infantry["WALKER"].clone();
    eng.primary = None;
    eng.engineer = Some(true);
    register(&mut r, "infantry", "ENGW");
    r.sim.infantry.insert("ENGW".into(), eng);
    r
}

fn game() -> World {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    w
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, 0, at(dx, dy), 0)
}

/// Building `name` with its position cell at (0, 0), and a unit of type
/// `unit` at (`dx`, `dy`), settled.
fn scene(name: &str, kind: Kind, unit: &str, dx: i32, dy: i32) -> (World, ObjectId, ObjectId) {
    let mut w = game();
    let b = put(&mut w, Kind::Building, name, 0, 0);
    testkit::run(&mut w, 3);
    let u = put(&mut w, kind, unit, dx, dy);
    testkit::run(&mut w, 2);
    (w, b, u)
}

/// Four starting cells, eight cells from a W × H foundation at (0, 0) on
/// each side: west, east, north, south.
fn sides(fw: i32, fh: i32) -> [(i32, i32); 4] {
    [
        (-8, fh / 2),
        (fw - 1 + 8, fh / 2),
        (fw / 2, -8),
        (fw / 2, fh - 1 + 8),
    ]
}

/// The centre claim 27 gives (S-0204 claim 4): the position moved by
/// 128 × (W − 1) along x and 128 × (H − 1) along y.
fn centre(w: &World, b: ObjectId, fw: i32, fh: i32) -> (i32, i32) {
    let p = w.obj(b).unwrap().pos;
    (p.x + 128 * (fw - 1), p.y + 128 * (fh - 1))
}

fn xy(p: Pos) -> (i32, i32) {
    (p.x, p.y)
}

fn cell_of((x, y): (i32, i32)) -> CellXY {
    CellXY::new(x.div_euclid(256), y.div_euclid(256))
}

/// A unit's three points: its stored point, its searches' goal cell and its
/// heading point.
type Points = (Option<(i32, i32)>, Option<CellXY>, Option<(i32, i32)>);

/// The walker's points after the assignment: its stored object point (the
/// locomotor destination, S-0208 Terms), its searches' goal cell (claim
/// 25) and its heading point (S-0212 Terms).
fn walker_points(w: &World, u: ObjectId) -> Points {
    (
        walk::object_locomotor_point(w, u).map(xy),
        path::search_goal(w, u),
        path::heading_point(w, u).map(xy),
    )
}

/// The driver's points after the assignment: its installed goal (S-0207
/// claim 25), its searches' goal cell and its heading point.
fn driver_points(w: &World, v: ObjectId) -> Points {
    (
        drive::goal(w, v).map(xy),
        path::goal_cell(w, v),
        path::heading_point(w, v).map(xy),
    )
}

fn assign(w: &mut World, u: ObjectId, b: ObjectId) {
    econ::assign_destination(w, u, Some(Target::Object(b)));
    assert_eq!(w.obj(u).unwrap().dest, Some(Target::Object(b)), "assigned");
}

// spec: S-0208 claim 27 (r82, Q-3317: a building with none of the helipad, unit-repair and vehicle-bunker flags gives an infantry unit's locomotion its centre, the position moved by 128 × (W − 1) and 128 × (H − 1); its cell is the foundation cell (⌊W / 2⌋, ⌊H / 2⌋) on from the position cell) and claim 25 (r68: the walker's searches look for that point's cell)
#[test]
fn claim_27_a_walker_is_given_the_centre() {
    for (fw, fh) in FOUNDATIONS {
        for (dx, dy) in sides(fw, fh) {
            let what = format!("{fw}x{fh} from ({dx}, {dy})");
            let (mut w, b, u) = scene(&plain_name(fw, fh), Kind::Infantry, "WALKER", dx, dy);
            assign(&mut w, u, b);
            let c = centre(&w, b, fw, fh);
            assert_eq!(cell_of(c), at(fw / 2, fh / 2), "{what}: the centre's cell");
            assert_eq!(
                walker_points(&w, u),
                (Some(c), Some(at(fw / 2, fh / 2)), Some(c)),
                "{what}: point, search goal and heading point"
            );
        }
    }
}

// spec: S-0208 claim 27 (r82, Q-3317: the same centre for a vehicle's locomotion, S-0207 claim 25's point installed at the assignment; the point does not depend on whether the unit is an infantry unit or a vehicle, nor on its locomotor)
#[test]
fn claim_27_a_driver_is_given_the_centre() {
    for (fw, fh) in FOUNDATIONS {
        for (dx, dy) in sides(fw, fh) {
            let what = format!("{fw}x{fh} from ({dx}, {dy})");
            let (mut w, b, v) = scene(&plain_name(fw, fh), Kind::Vehicle, "BUGGY", dx, dy);
            assign(&mut w, v, b);
            let c = centre(&w, b, fw, fh);
            assert_eq!(
                driver_points(&w, v),
                (Some(c), Some(at(fw / 2, fh / 2)), Some(c)),
                "{what}: installed goal, goal cell and heading point"
            );
        }
    }
}

// spec: S-0208 claim 27 (r82, Q-3317, W22 part f's case: a building whose type has the unit-reload flag but none of the helipad, unit-repair and vehicle-bunker flags gives its centre, not a docking point, to an infantry unit or a vehicle)
#[test]
fn claim_27_the_unit_reload_flag_alone_gives_the_centre() {
    for (kind, name) in [(Kind::Infantry, "WALKER"), (Kind::Vehicle, "BUGGY")] {
        let (mut w, b, u) = scene("RLD", kind, name, -8, 1);
        assign(&mut w, u, b);
        let c = centre(&w, b, 3, 3);
        let got = if kind == Kind::Infantry {
            walker_points(&w, u)
        } else {
            driver_points(&w, u)
        };
        assert_eq!(got, (Some(c), Some(at(1, 1)), Some(c)), "{name}");
    }
}

// spec: S-0208 claim 27 (r82, Q-3317: a type with the unit-repair flag and none of the obsolete-unloading, resource-unloading and vehicle-bunker flags gives S-0607 claim 7's docking-place rule, S-0314 claim 18 and S-0404 claim 15: with one docking place, the centre plus its displacement)
#[test]
fn claim_27_a_unit_repair_type_gives_its_docking_place() {
    for (kind, name) in [(Kind::Infantry, "WALKER"), (Kind::Vehicle, "BUGGY")] {
        let (mut w, b, u) = scene("REP", kind, name, -8, 1);
        assign(&mut w, u, b);
        let (cx, cy) = centre(&w, b, 3, 3);
        let d = (cx + 512, cy - 256);
        let got = if kind == Kind::Infantry {
            walker_points(&w, u)
        } else {
            driver_points(&w, u)
        };
        assert_eq!(got, (Some(d), Some(cell_of(d)), Some(d)), "{name}");
    }
}

// spec: S-0208 claim 27 (r82, Q-3317: a type with the helipad flag gives S-0607 claim 7's docking-place rule: with a count greater than 1 and no place held through a contact with that building, the foundation centre)
#[test]
fn claim_27_a_helipad_with_two_places_and_no_contact_gives_the_centre() {
    for (kind, name) in [(Kind::Infantry, "WALKER"), (Kind::Vehicle, "BUGGY")] {
        let (mut w, b, u) = scene("HPAD", kind, name, -8, 1);
        assign(&mut w, u, b);
        let c = centre(&w, b, 2, 2);
        let got = if kind == Kind::Infantry {
            walker_points(&w, u)
        } else {
            driver_points(&w, u)
        };
        assert_eq!(got, (Some(c), Some(at(1, 1)), Some(c)), "{name}");
    }
}

// spec: S-0208 claim 27 (r82, Q-3317: a type with the vehicle-bunker flag and neither unloading flag gives S-0606 claim 4's quadrant point, taken from the unit's position at the assignment, the centre of one foundation cell of a 2 × 2 foundation; for a unit with no bunker contact too) and S-0606 claim 4
#[test]
fn claim_27_a_vehicle_bunker_gives_the_quadrant_point_from_the_position() {
    let mut cells = Vec::new();
    for (kind, name) in [(Kind::Infantry, "WALKER"), (Kind::Vehicle, "CRUSHER")] {
        for (dx, dy) in sides(2, 2) {
            let what = format!("{name} from ({dx}, {dy})");
            let (mut w, b, u) = scene("PIT", kind, name, dx, dy);
            assert_eq!(w.obj(u).unwrap().contact, None, "{what}: no contact");
            let q = xy(bunker::docking_point(&w, b, u));
            assign(&mut w, u, b);
            assert_ne!(q, centre(&w, b, 2, 2), "{what}: not the centre");
            assert_eq!(
                (q.0.rem_euclid(256), q.1.rem_euclid(256)),
                (128, 128),
                "{what}: a cell centre"
            );
            let got = if kind == Kind::Infantry {
                walker_points(&w, u)
            } else {
                driver_points(&w, u)
            };
            assert_eq!(got, (Some(q), Some(cell_of(q)), Some(q)), "{what}");
            cells.push(cell_of(q));
        }
    }
    cells.sort_by_key(|c| (c.x, c.y));
    cells.dedup();
    assert!(cells.len() > 1, "the sides give different cells: {cells:?}");
}

// spec: S-0208 claim 27 (r82, Q-3317: the point does not depend on the behaviour that gives the destination, capture included) and S-0208 Terms (r67: the locomotor destination is the point at the assignment; it does not follow the object)
#[test]
fn claim_27_a_capturing_walker_is_given_the_centre_and_keeps_it() {
    let mut w = game();
    let b = testkit::spawn(&mut w, Kind::Building, "CAPB", 1, at(0, 0), 0);
    testkit::run(&mut w, 3);
    let u = put(&mut w, Kind::Infantry, "ENGW", -8, 1);
    testkit::run(&mut w, 2);
    let house = w.obj(u).unwrap().owner.unwrap();
    assert!(orders::issue(
        &mut w,
        house,
        Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Capture,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    ));
    let mut seen = false;
    for _ in 0..4 {
        tick::step(&mut w);
        if w.obj(u).unwrap().dest == Some(Target::Object(b)) {
            seen = true;
            break;
        }
    }
    assert!(seen, "the capture gives the building as the destination");
    let c = centre(&w, b, 3, 2);
    assert_eq!(cell_of(c), at(1, 1));
    assert_eq!(
        walk::object_locomotor_point(&w, u).map(xy),
        Some(c),
        "the centre"
    );
    for _ in 0..10 {
        tick::step(&mut w);
    }
    assert_eq!(
        walk::object_locomotor_point(&w, u).map(xy),
        Some(c),
        "kept while it walks"
    );
    assert_eq!(path::search_goal(&w, u), Some(at(1, 1)));
    let bytes = save::save(&w);
    let l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
}
