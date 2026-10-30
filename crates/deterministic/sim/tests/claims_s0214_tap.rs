//! Claim tests for S-0214 claim 32's type-select tap candidates and F9's
//! group centre point, on synthetic data: the testkit's sandbox game with
//! invented types MCV (deploys into the building YARD), DRONE and CTRL (a
//! building that powers DRONE). No stock values.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{group_centre, tap_candidate, tap_types};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);

fn ruleset() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries.insert(
        "buildings".into(),
        ["YARD", "CTRL"].iter().map(|s| s.to_string()).collect(),
    );
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .extend(["MCV".to_string(), "DRONE".to_string()]);
    let s = &mut r.sim;
    let building = || BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    s.buildings.insert("YARD".into(), building());
    let mut ctrl = building();
    ctrl.econ.powers_unit = Some("DRONE".into());
    s.buildings.insert("CTRL".into(), ctrl);
    let vehicle = |into: Option<&str>| UnitType {
        strength: Some(100),
        speed: Some(6),
        rot: Some(8),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        deploys_into: into.map(Into::into),
        ..Default::default()
    };
    s.vehicles.insert("MCV".into(), vehicle(Some("YARD")));
    s.vehicles.insert("DRONE".into(), vehicle(None));
    r
}

fn game() -> World {
    let mut w = testkit::game_with(&ruleset(), SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, 0), 0)
}

fn ty(w: &World, kind: Kind, name: &str) -> (Kind, u32) {
    (kind, w.rules().index_of(kind, name).unwrap())
}

/// spec: S-0214 claim 32 and Import mapping (the tap's types: each selected object's type, with the building type it deploys into)
#[test]
fn claim_32_the_tap_types_take_deploys_into() {
    let mut w = game();
    let mcv = spawn(&mut w, Kind::Vehicle, "MCV", 0, 0);
    let drone = spawn(&mut w, Kind::Vehicle, "DRONE", 0, 1);
    assert_eq!(
        tap_types(&w, &[mcv, drone]),
        vec![
            ty(&w, Kind::Vehicle, "MCV"),
            ty(&w, Kind::Building, "YARD"),
            ty(&w, Kind::Vehicle, "DRONE"),
        ]
    );
    assert_eq!(
        tap_types(&w, &[drone]),
        vec![ty(&w, Kind::Vehicle, "DRONE")]
    );
}

/// spec: S-0214 claim 32 (a candidate: own, of a tap type, passing the band box's tests other than the screen or a building whose type powers a unit, can be selected and not selected) and Import mapping (PowersUnit)
#[test]
fn claim_32_candidates() {
    let mut w = game();
    let mcv = spawn(&mut w, Kind::Vehicle, "MCV", 0, 0);
    let other_mcv = spawn(&mut w, Kind::Vehicle, "MCV", 0, 2);
    let their_mcv = spawn(&mut w, Kind::Vehicle, "MCV", 1, 4);
    let yard = spawn(&mut w, Kind::Building, "YARD", 0, -3);
    let ctrl = spawn(&mut w, Kind::Building, "CTRL", 0, -5);
    let their_ctrl = spawn(&mut w, Kind::Building, "CTRL", 1, -7);
    let sel = [mcv];
    let mut types = tap_types(&w, &sel);
    types.push(ty(&w, Kind::Building, "CTRL"));
    let cand = |id| tap_candidate(&w, A, &sel, &types, id);
    assert!(cand(other_mcv), "own, of the type");
    assert!(!cand(mcv), "selected already");
    assert!(!cand(their_mcv), "not own");
    assert!(
        !cand(yard),
        "an ordinary building fails the band box's kind test"
    );
    assert!(cand(ctrl), "a building that powers a unit");
    assert!(!cand(their_ctrl), "not own");
    let types = tap_types(&w, &sel);
    assert!(
        !tap_candidate(&w, A, &sel, &types, ctrl),
        "not of a tap type"
    );
}

fn at(w: &mut World, id: ObjectId, x: i32, y: i32, z: i32) {
    w.obj_mut(id).unwrap().pos = Pos::new(x, y, z);
}

/// spec: S-0214 F9 (the mean, truncated toward zero, for one or two objects; for three or more, the mean of all but the first farthest one; the mean again when every distance is 0; none for no object)
#[test]
fn f9_the_group_centre_point() {
    let mut w = game();
    let ids: Vec<ObjectId> = (0..4)
        .map(|i| spawn(&mut w, Kind::Vehicle, "DRONE", 0, i))
        .collect();
    assert_eq!(group_centre(&w, &[]), None);
    at(&mut w, ids[0], 1000, 2000, 0);
    assert_eq!(group_centre(&w, &ids[..1]), Some((1000, 2000, 0)));
    at(&mut w, ids[1], 1001, 2003, 0);
    assert_eq!(group_centre(&w, &ids[..2]), Some((1000, 2001, 0)));
    // Three in a row: the ends tie for farthest and the first is left out.
    at(&mut w, ids[0], 1000, 0, 0);
    at(&mut w, ids[1], 2000, 0, 0);
    at(&mut w, ids[2], 3000, 0, 0);
    assert_eq!(group_centre(&w, &ids[..3]), Some((2500, 0, 0)));
    // A far one is left out.
    at(&mut w, ids[0], 2000, 0, 0);
    at(&mut w, ids[1], 2100, 0, 0);
    at(&mut w, ids[2], 2200, 0, 0);
    at(&mut w, ids[3], 9000, 0, 0);
    assert_eq!(group_centre(&w, &ids), Some((2100, 0, 0)));
    // Every distance 0: the mean.
    for &id in &ids {
        at(&mut w, id, 500, 600, 7);
    }
    assert_eq!(group_centre(&w, &ids), Some((500, 600, 7)));
}
